//! Local HTTP server: MCP over HTTP (for ChatGPT connectors and other remote clients)
//! and a phone-friendly web page for Android on the same Wi-Fi. Every request needs the
//! workspace's private token, and each server only ever opens its own workspace.

use crate::store::{Store, WorkspaceMeta};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use tiny_http::{Header, Method, Response, Server};

pub struct ServerHandle {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    pub port: u16,
    pub workspace_id: String,
}

impl ServerHandle {
    pub fn stop(mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

pub fn lan_ip() -> Option<String> {
    // no packet is sent: connecting a UDP socket only picks the outgoing interface
    let s = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect("8.8.8.8:80").ok()?;
    Some(s.local_addr().ok()?.ip().to_string())
}

fn header(k: &str, v: &str) -> Header {
    Header::from_bytes(k.as_bytes(), v.as_bytes()).unwrap()
}

fn token_ok(req: &tiny_http::Request, token: &str) -> bool {
    let url = req.url();
    if let Some(q) = url.split_once('?').map(|x| x.1) {
        for part in q.split('&') {
            if let Some(v) = part.strip_prefix("token=") {
                if v == token {
                    return true;
                }
            }
        }
    }
    req.headers().iter().any(|h| {
        h.field.equiv("Authorization") && {
            let v = h.value.as_str();
            v.strip_prefix("Bearer ").map_or(false, |t| t.trim() == token)
        }
    })
}

pub fn start(meta: WorkspaceMeta, port: u16) -> Result<ServerHandle, String> {
    let server = Server::http(("0.0.0.0", port)).map_err(|e| format!("Couldn't open port {port}: {e}"))?;
    let stop = Arc::new(AtomicBool::new(false));
    let stop2 = stop.clone();
    let wid = meta.id.clone();
    let thread = std::thread::spawn(move || {
        let mut store = Store::open(meta);
        while !stop2.load(Ordering::SeqCst) {
            let req = match server.recv_timeout(std::time::Duration::from_millis(300)) {
                Ok(Some(r)) => r,
                Ok(None) => continue,
                Err(_) => break,
            };
            serve_one(&mut store, req);
        }
    });
    Ok(ServerHandle { stop, thread: Some(thread), port, workspace_id: wid })
}

/// Blocking server for `resolve2 --serve`.
/// Run this on an always-on machine (this PC, or a cloud server later) to make it the hub.
pub fn run_forever(meta: WorkspaceMeta, port: u16, token: Option<String>) -> Result<(), String> {
    let server = Server::http(("0.0.0.0", port)).map_err(|e| format!("Couldn't open port {port}: {e}"))?;
    let mut store = Store::open(meta);
    if let Some(t) = token.filter(|t| t.len() >= 16) {
        store.settings.token = t;
        store.save_settings();
    }
    eprintln!("Resolve 2 hub serving '{}' on port {port} (data in {})", store.meta.name, store.dir.display());
    eprintln!("  phone page: http://{}:{port}/?token={}", lan_ip().unwrap_or_else(|| "localhost".into()), store.settings.token);
    eprintln!("  MCP:        http://localhost:{port}/mcp?token={}", store.settings.token);
    eprintln!("  hub sync:   http://<this-machine>:{port}/sync  (same token)");
    for req in server.incoming_requests() {
        serve_one(&mut store, req);
    }
    Ok(())
}

fn serve_one(store: &mut Store, mut req: tiny_http::Request) {
    // pick up the latest token / access settings changed in the app
    if let Ok(s) = std::fs::read_to_string(store.dir.join("settings.json")) {
        if let Ok(v) = serde_json::from_str(&s) {
            store.settings = v;
        }
    }
    let path = req.url().split('?').next().unwrap_or("/").to_string();
    let cors = [
        header("Access-Control-Allow-Origin", "*"),
        header("Access-Control-Allow-Headers", "Content-Type, Authorization, Mcp-Session-Id, Mcp-Protocol-Version"),
        header("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
    ];
    let respond = |req: tiny_http::Request, status: u16, ctype: &str, body: String| {
        let mut r = Response::from_string(body).with_status_code(status).with_header(header("Content-Type", ctype));
        for h in cors.iter() {
            r.add_header(h.clone());
        }
        let _ = req.respond(r);
    };
    if req.method() == &Method::Options {
        respond(req, 204, "text/plain", String::new());
        return;
    }
    match (req.method().clone(), path.as_str()) {
        (Method::Get, "/") | (Method::Get, "/phone") => respond(req, 200, "text/html; charset=utf-8", PHONE_HTML.replace("{{NAME}}", &html_escape(&store.meta.name))),
        (Method::Get, "/manifest.json") => respond(req, 200, "application/json", format!(r##"{{"name":"Resolve 2: {}","short_name":"Resolve 2","start_url":"/","display":"standalone","background_color":"#1e1e1e","theme_color":"#1e1e1e"}}"##, store.meta.name.replace('"', ""))),
        (Method::Get, "/health") => respond(req, 200, "text/plain", "ok".into()),
        (Method::Post, "/mcp") => {
            if !token_ok(&req, &store.settings.token) {
                respond(req, 401, "application/json", r#"{"error":"missing or wrong token"}"#.into());
                return;
            }
            let mut body = String::new();
            let _ = std::io::Read::read_to_string(req.as_reader(), &mut body);
            match crate::mcp::handle_text(store, &body) {
                Some(out) => respond(req, 200, "application/json", out),
                None => respond(req, 202, "application/json", String::new()),
            }
        }
        // Hub sync: a device posts its whole copy, the hub merges (newest edit per item wins,
        // deletions respected) and answers with the merged workspace. GET just downloads it.
        (Method::Post, "/sync") | (Method::Get, "/sync") => {
            if !token_ok(&req, &store.settings.token) {
                respond(req, 401, "application/json", r#"{"error":"missing or wrong token"}"#.into());
                return;
            }
            store.reload_if_changed();
            if req.method() == &Method::Post {
                let mut body = String::new();
                let _ = std::io::Read::read_to_string(req.as_reader(), &mut body);
                match serde_json::from_str::<crate::model::WorkspaceData>(&body) {
                    Ok(incoming) => {
                        store.data.merge(&incoming);
                        if let Err(e) = store.save() {
                            respond(req, 500, "application/json", serde_json::json!({"error": e}).to_string());
                            return;
                        }
                    }
                    Err(e) => {
                        respond(req, 400, "application/json", serde_json::json!({"error": format!("not a workspace: {e}")}).to_string());
                        return;
                    }
                }
            }
            respond(req, 200, "application/json", serde_json::to_string(&store.data).unwrap_or_default());
        }
        (Method::Get, "/mcp") => respond(req, 405, "text/plain", "Use POST for MCP requests".into()),
        _ => respond(req, 404, "text/plain", "Not found".into()),
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

const PHONE_HTML: &str = r##"<!doctype html>
<html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<link rel="manifest" href="/manifest.json"><meta name="theme-color" content="#1e1e1e">
<title>Resolve 2: {{NAME}}</title>
<style>
:root{--bg:#1e1e1e;--card:#282828;--line:#333;--text:#eee;--dim:#999;--red:#dc4c3e;--accent:#dc4c3e}
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--text);font:15px/1.4 system-ui,-apple-system,Segoe UI,Roboto,sans-serif}
header{position:sticky;top:0;background:var(--bg);padding:14px 16px 8px;border-bottom:1px solid var(--line);z-index:2}
h1{font-size:22px;margin:0 0 10px}.tabs{display:flex;gap:6px;overflow:auto}
.tabs button{background:none;border:1px solid var(--line);color:var(--dim);padding:6px 12px;border-radius:16px;font-size:14px}
.tabs button.on{background:var(--accent);border-color:var(--accent);color:#fff}
main{padding:8px 16px 90px}.row{display:flex;gap:12px;padding:12px 0;border-bottom:1px solid var(--line);align-items:flex-start}
.ck{width:20px;height:20px;border-radius:50%;border:2px solid #888;flex:none;margin-top:1px;background:none}
.p1{border-color:#d1453b;background:#d1453b22}.p2{border-color:#eb8909;background:#eb890922}.p3{border-color:#246fe0;background:#246fe022}
.meta{font-size:12px;color:var(--dim);margin-top:2px}.od{color:var(--red)}.day{font-weight:600;margin:18px 0 4px;font-size:14px}
.ev{border-left:4px solid #3f51b5;padding:6px 10px;background:var(--card);border-radius:6px;margin:6px 0}
form{position:fixed;bottom:0;left:0;right:0;display:flex;gap:8px;padding:10px 12px;background:var(--card);border-top:1px solid var(--line)}
input{flex:1;background:var(--bg);border:1px solid var(--line);color:var(--text);border-radius:8px;padding:10px;font-size:15px}
form button{background:var(--accent);color:#fff;border:0;border-radius:8px;padding:0 16px;font-weight:600}
.page{white-space:pre-wrap;background:var(--card);padding:12px;border-radius:8px;font-size:14px}.err{color:var(--red);padding:20px 0}
a{color:#7aa7ff}
</style></head><body>
<header><h1>{{NAME}}</h1><div class="tabs">
<button data-v="today" class="on">Today</button><button data-v="upcoming">Upcoming</button><button data-v="inbox">Inbox</button><button data-v="agenda">Calendar</button><button data-v="search">Search</button></div></header>
<main id="m"></main>
<form id="f"><input id="q" placeholder="Add task: Call mum tomorrow 6pm p1 #Home" autocomplete="off"><button>Add</button></form>
<script>
const qs=new URLSearchParams(location.search);if(qs.get('token'))localStorage.setItem('resolve2_token',qs.get('token'));
const token=localStorage.getItem('resolve2_token')||'';let view='today',n=0;const m=document.getElementById('m');
async function rpc(name,args){const r=await fetch('/mcp?token='+encodeURIComponent(token),{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({jsonrpc:'2.0',id:++n,method:'tools/call',params:{name,arguments:args||{}}})});
 if(r.status==401)throw new Error('This page needs the link with the token from Resolve 2 → Settings → Phone & sync.');const j=await r.json();const t=j.result.content[0].text;if(j.result.isError)throw new Error(t);return JSON.parse(t)}
const esc=s=>(s||'').replace(/[&<>]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;'}[c]));
function taskRow(t){return `<div class="row"><button class="ck ${t.priority}" onclick="done('${t.id}')"></button><div><div>${esc(t.title)}</div><div class="meta ${t.overdue?'od':''}">${t.due?('📅 '+t.due+(t.time?' '+t.time:'')):''} ${t.recurrence?'🔁':''} · ${esc(t.project)}</div></div></div>`}
async function done(id){await rpc('complete_task',{id});load()}
async function load(){try{
 if(view=='search'){m.innerHTML='<p class="meta">Type in the box below and press Search.</p>';document.getElementById('q').placeholder='Search everything';return}
 document.getElementById('q').placeholder='Add task: Call mum tomorrow 6pm p1 #Home';
 if(view=='agenda'){const a=await rpc('get_agenda',{days:7});m.innerHTML=a.map(d=>`<div class="day">${d.weekday} ${d.date}</div>`+d.events.map(e=>`<div class="ev">${e.all_day?'All day':e.start.slice(11)} · ${esc(e.title)}<div class="meta">${esc(e.calendar||'')} ${esc(e.location)}</div></div>`).join('')+d.tasks.map(taskRow).join('')).join('');return}
 const ts=await rpc('list_tasks',{view});m.innerHTML=ts.length?ts.filter(t=>!t.parent_id).map(taskRow).join(''):'<p class="meta">Nothing here. Enjoy your day!</p>';
}catch(e){m.innerHTML='<div class="err">'+esc(e.message)+'</div>'}}
document.querySelectorAll('.tabs button').forEach(b=>b.onclick=()=>{document.querySelectorAll('.tabs button').forEach(x=>x.classList.remove('on'));b.classList.add('on');view=b.dataset.v;load()});
document.getElementById('f').onsubmit=async e=>{e.preventDefault();const q=document.getElementById('q');const v=q.value.trim();if(!v)return;
 try{if(view=='search'){const h=await rpc('search',{query:v});m.innerHTML=h.map(x=>`<div class="row"><div><div>${esc(x.title)}</div><div class="meta">${x.kind} · ${esc(x.snippet)}</div>${x.kind=='Page'?`<a href="#" onclick="openPage('${x.id}');return false">Open</a>`:''}</div></div>`).join('')||'<p class="meta">No results</p>';return}
 await rpc('create_task',{text:v});q.value='';load()}catch(err){m.innerHTML='<div class="err">'+esc(err.message)+'</div>'}};
async function openPage(id){const p=await rpc('read_page',{id});m.innerHTML='<div class="page">'+esc(p.markdown)+'</div>'}
load();setInterval(()=>{if(view!='search')load()},30000);
</script></body></html>"##;
