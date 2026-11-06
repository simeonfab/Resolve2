//! Local AI (no subscription): talks to Ollama or any OpenAI-compatible local server
//! (LM Studio, llama.cpp server, Jan...) on this computer.

use serde_json::{json, Value};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::time::Duration;

pub struct AiJob {
    rx: Receiver<Result<String, String>>,
    pub label: String,
}

impl AiJob {
    /// Some(result) once finished.
    pub fn poll(&self) -> Option<Result<String, String>> {
        match self.rx.try_recv() {
            Ok(r) => Some(r),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err("The AI request stopped unexpectedly".into())),
        }
    }
}

fn base(url: &str) -> String {
    url.trim().trim_end_matches('/').to_string()
}

fn chat(provider: &str, url: &str, model: &str, system: &str, prompt: &str) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new().timeout_connect(Duration::from_secs(5)).timeout(Duration::from_secs(600)).build();
    let messages = json!([{"role": "system", "content": system}, {"role": "user", "content": prompt}]);
    if provider == "openai" {
        let resp: Value = agent
            .post(&format!("{}/v1/chat/completions", base(url)))
            .send_json(json!({"model": model, "messages": messages, "stream": false}))
            .map_err(|e| explain(e, url))?
            .into_json()
            .map_err(|e| e.to_string())?;
        resp["choices"][0]["message"]["content"].as_str().map(|s| s.to_string()).ok_or_else(|| format!("Unexpected reply: {resp}"))
    } else {
        let resp: Value = agent
            .post(&format!("{}/api/chat", base(url)))
            .send_json(json!({"model": model, "messages": messages, "stream": false}))
            .map_err(|e| explain(e, url))?
            .into_json()
            .map_err(|e| e.to_string())?;
        resp["message"]["content"].as_str().map(|s| s.to_string()).ok_or_else(|| format!("Unexpected reply: {resp}"))
    }
}

fn explain(e: ureq::Error, url: &str) -> String {
    match e {
        ureq::Error::Status(404, r) => {
            let body = r.into_string().unwrap_or_default();
            if body.contains("model") {
                format!("The model isn't installed. In a terminal run: ollama pull <model>  ({body})")
            } else {
                format!("Not found at {url}: {body}")
            }
        }
        ureq::Error::Status(code, r) => format!("The AI server answered {code}: {}", r.into_string().unwrap_or_default()),
        ureq::Error::Transport(_) => format!("No local AI is running at {url}. Install Ollama (free, ollama.com) and run `ollama pull llama3.2`, or start LM Studio's server."),
    }
}

pub fn ask(provider: &str, url: &str, model: &str, system: &str, prompt: &str, label: &str) -> AiJob {
    let (tx, rx) = channel();
    let (p, u, m, s, q) = (provider.to_string(), url.to_string(), model.to_string(), system.to_string(), prompt.to_string());
    std::thread::spawn(move || {
        let _ = tx.send(chat(&p, &u, &m, &s, &q));
    });
    AiJob { rx, label: label.to_string() }
}

pub fn list_models(provider: &str, url: &str) -> Result<Vec<String>, String> {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(5)).build();
    if provider == "openai" {
        let v: Value = agent.get(&format!("{}/v1/models", base(url))).call().map_err(|e| explain(e, url))?.into_json().map_err(|e| e.to_string())?;
        Ok(v["data"].as_array().map(|a| a.iter().filter_map(|m| m["id"].as_str().map(|s| s.to_string())).collect()).unwrap_or_default())
    } else {
        let v: Value = agent.get(&format!("{}/api/tags", base(url))).call().map_err(|e| explain(e, url))?.into_json().map_err(|e| e.to_string())?;
        Ok(v["models"].as_array().map(|a| a.iter().filter_map(|m| m["name"].as_str().map(|s| s.to_string())).collect()).unwrap_or_default())
    }
}

pub fn list_models_async(provider: &str, url: &str) -> Receiver<Result<Vec<String>, String>> {
    let (tx, rx) = channel();
    let (p, u) = (provider.to_string(), url.to_string());
    std::thread::spawn(move || {
        let _ = tx.send(list_models(&p, &u));
    });
    rx
}

// ---------------------------------------------------------------- the in-app assistant that can act

#[derive(Clone)]
pub struct AiConfig {
    /// "anthropic" (Claude API), "openai-cloud" (ChatGPT API), "ollama", "openai" (local OpenAI-compatible)
    pub provider: String,
    pub url: String,
    pub model: String,
    pub key: String,
}

pub const CLAUDE_MODEL: &str = "claude-opus-5-5";

// ---------------------------------------------------------------- usage & cost tracking
//
// Every model call made through an API or local server appends one line to
// <data root>/ai-usage.jsonl: {ts, provider, model, input, output, purpose}.
// Costs are estimated from the per-million-token prices the user enters in Settings
// (no built-in price list: prices change and differ per contract).
// CLI providers (Claude Code / Codex subscriptions) don't report tokens, so they are logged with zeros.

thread_local! {
    static PURPOSE: std::cell::RefCell<String> = std::cell::RefCell::new("assistant".into());
}

/// Tag the model calls made on this thread (e.g. "intake", "guide:planner").
pub fn set_purpose(p: &str) {
    PURPOSE.with(|x| *x.borrow_mut() = p.to_string());
}

pub fn usage_path() -> std::path::PathBuf {
    crate::store::root().join("ai-usage.jsonl")
}

pub fn record_usage(provider: &str, model: &str, input: u64, output: u64) {
    use std::io::Write;
    let purpose = PURPOSE.with(|x| x.borrow().clone());
    let line = json!({"ts": crate::model::now_ms(), "provider": provider, "model": model, "input": input, "output": output, "purpose": purpose});
    let _ = std::fs::create_dir_all(crate::store::root());
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(usage_path()) {
        let _ = writeln!(f, "{line}");
    }
}

fn record_from_response(provider: &str, model: &str, resp: &Value) {
    let u = &resp["usage"];
    let input = u["input_tokens"].as_u64().or(u["prompt_tokens"].as_u64()).or(resp["prompt_eval_count"].as_u64()).unwrap_or(0);
    let output = u["output_tokens"].as_u64().or(u["completion_tokens"].as_u64()).or(resp["eval_count"].as_u64()).unwrap_or(0);
    record_usage(provider, model, input, output);
}

#[derive(Default, Clone, Debug)]
pub struct UsageTotals {
    pub calls: u64,
    pub input: u64,
    pub output: u64,
    /// (provider/model, calls, input, output)
    pub by_model: Vec<(String, u64, u64, u64)>,
}

/// Totals since `since_ms` (e.g. the first day of this month).
pub fn usage_since(since_ms: i64) -> UsageTotals {
    let mut t = UsageTotals::default();
    let text = std::fs::read_to_string(usage_path()).unwrap_or_default();
    let mut map: std::collections::BTreeMap<String, (u64, u64, u64)> = Default::default();
    for l in text.lines() {
        let Ok(v) = serde_json::from_str::<Value>(l) else { continue };
        if v["ts"].as_i64().unwrap_or(0) < since_ms {
            continue;
        }
        let (i, o) = (v["input"].as_u64().unwrap_or(0), v["output"].as_u64().unwrap_or(0));
        t.calls += 1;
        t.input += i;
        t.output += o;
        let e = map.entry(format!("{} / {}", v["provider"].as_str().unwrap_or("?"), v["model"].as_str().unwrap_or("?"))).or_default();
        e.0 += 1;
        e.1 += i;
        e.2 += o;
    }
    t.by_model = map.into_iter().map(|(k, (c, i, o))| (k, c, i, o)).collect();
    t
}

/// POST JSON over HTTPS with Windows' curl.exe (the app has no TLS library of its own).
fn post_https(url: &str, headers: &[(&str, String)], body: &Value) -> Result<Value, String> {
    let tmp = std::env::temp_dir().join(format!("resolve2-ai-{}-{}.json", std::process::id(), crate::model::new_id()));
    std::fs::write(&tmp, serde_json::to_vec(body).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let mut cmd = std::process::Command::new("curl.exe");
    cmd.args(["-sS", "--max-time", "600", "-X", "POST", "-H", "content-type: application/json"]);
    for (k, v) in headers {
        cmd.arg("-H").arg(format!("{k}: {v}"));
    }
    cmd.arg("--data-binary").arg(format!("@{}", tmp.display())).arg(url);
    let out = cmd.output().map_err(|e| format!("Couldn't run curl: {e}"));
    let _ = std::fs::remove_file(&tmp);
    let out = out?;
    if !out.status.success() {
        return Err(format!("Couldn't reach the AI service: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    let v: Value = serde_json::from_slice(&out.stdout).map_err(|e| format!("Unexpected reply from the AI service: {e}"))?;
    if let Some(err) = v.get("error") {
        let msg = err.get("message").and_then(|m| m.as_str()).map(|s| s.to_string()).unwrap_or_else(|| err.to_string());
        return Err(format!("The AI service said: {msg}"));
    }
    Ok(v)
}

fn post_http(url: &str, headers: &[(&str, String)], body: &Value) -> Result<Value, String> {
    let agent = ureq::AgentBuilder::new().timeout_connect(Duration::from_secs(5)).timeout(Duration::from_secs(600)).build();
    let mut req = agent.post(url);
    for (k, v) in headers {
        req = req.set(k, v);
    }
    match req.send_json(body.clone()) {
        Ok(r) => r.into_json().map_err(|e| e.to_string()),
        Err(ureq::Error::Status(code, r)) => Err(format!("The AI server answered {code}: {}", r.into_string().unwrap_or_default())),
        Err(e) => Err(explain(e, url)),
    }
}

fn post(url: &str, headers: &[(&str, String)], body: &Value) -> Result<Value, String> {
    if url.starts_with("https://") {
        post_https(url, headers, body)
    } else {
        post_http(url, headers, body)
    }
}

/// Run an assistant turn that may use the workspace tools (the same ones MCP exposes).
/// Tool calls run against the workspace on disk; the app picks the changes up within seconds.
/// Find a command-line tool installed by npm (claude.cmd / codex.cmd) or as an .exe.
fn find_cli(name: &str) -> Option<std::path::PathBuf> {
    let mut dirs: Vec<std::path::PathBuf> = std::env::var_os("PATH").map(|p| std::env::split_paths(&p).collect()).unwrap_or_default();
    if let Ok(a) = std::env::var("APPDATA") {
        dirs.push(std::path::PathBuf::from(a).join("npm"));
    }
    if let Ok(h) = std::env::var("USERPROFILE") {
        dirs.push(std::path::PathBuf::from(&h).join(".local").join("bin"));
    }
    for d in dirs {
        for ext in ["exe", "cmd"] {
            let p = d.join(format!("{name}.{ext}"));
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

pub fn cli_status(provider: &str) -> Result<String, String> {
    let name = if provider == "codex-cli" { "codex" } else { "claude" };
    let path = find_cli(name).ok_or_else(|| {
        if name == "claude" {
            "Claude Code isn't installed. Install it from claude.com/claude-code, then run `claude` once and sign in with your Claude account.".to_string()
        } else {
            "Codex isn't installed. Install it with `npm install -g @openai/codex`, then run `codex` once and choose Sign in with ChatGPT.".to_string()
        }
    })?;
    let out = std::process::Command::new(&path).arg("--version").output().map_err(|e| e.to_string())?;
    Ok(format!("Found {} {}", name, String::from_utf8_lossy(&out.stdout).trim()))
}

fn counts(d: &crate::model::WorkspaceData) -> [usize; 5] {
    [d.tasks.len(), d.tasks.iter().filter(|t| t.completed).count(), d.pages.len(), d.events.len(), d.databases.iter().map(|x| x.rows.len()).sum()]
}

/// Use the person's own Claude (Pro/Max) or ChatGPT subscription through the Claude Code or
/// Codex command-line apps: no API key, no per-use cost. They reach the workspace through MCP.
fn run_cli(provider: &str, system: &str, prompt: &str, meta: crate::store::WorkspaceMeta, allow_write: bool) -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?.display().to_string();
    let before = crate::store::Store::open(meta.clone());
    let read_only = !allow_write || before.settings.ai_read_only;
    let before_counts = counts(&before.data);
    let work = std::env::temp_dir().join(format!("resolve-2-ai-{}", std::process::id()));
    std::fs::create_dir_all(&work).map_err(|e| e.to_string())?;
    let read_tools = ["workspace_overview", "search", "get_agenda", "list_pages", "read_page", "list_tasks", "list_projects", "list_events", "list_calendars", "list_databases", "query_database", "list_page_todos", "resolve_projects", "resolve_project", "list_records"];
    let output = if provider == "codex-cli" {
        let codex = find_cli("codex").ok_or("Codex isn't installed. Install it with `npm install -g @openai/codex`, then run `codex` and sign in with ChatGPT.")?;
        let full = format!("{system}\n\n{prompt}{}", if read_only { "\n\n(Read-only: don't change anything.)" } else { "" });
        let mut child = std::process::Command::new(codex)
            .current_dir(&work)
            .args(["exec", "--skip-git-repo-check", "-c", "approval_policy=\"never\""])
            .arg("-c")
            .arg(format!("mcp_servers.resolve2.command={}", serde_json::to_string(&exe).unwrap()))
            .arg("-c")
            .arg(format!("mcp_servers.resolve2.args={}", serde_json::to_string(&vec!["--data-dir".to_string(), crate::store::root().display().to_string(), "--mcp".into(), "--workspace".into(), meta.id.clone()]).unwrap()))
            .arg("-")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Couldn't start Codex: {e}"))?;
        use std::io::Write;
        child.stdin.take().unwrap().write_all(full.as_bytes()).map_err(|e| e.to_string())?;
        child.wait_with_output().map_err(|e| e.to_string())?
    } else {
        let claude = find_cli("claude").ok_or("Claude Code isn't installed. Install it from claude.com/claude-code, run `claude` once and sign in with your Claude account.")?;
        let cfg = work.join("mcp.json");
        let root = crate::store::root().display().to_string();
        std::fs::write(&cfg, serde_json::to_string(&json!({"mcpServers": {"cc": {"command": exe, "args": ["--data-dir", root, "--mcp", "--workspace", meta.id]}}})).unwrap()).map_err(|e| e.to_string())?;
        let sys = work.join("system.txt");
        std::fs::write(&sys, system).map_err(|e| e.to_string())?;
        let allowed = if read_only { read_tools.iter().map(|t| format!("mcp__cc__{t}")).collect::<Vec<_>>().join(",") } else { "mcp__cc".to_string() };
        let mut child = std::process::Command::new(claude)
            .current_dir(&work)
            .arg("-p")
            .arg("--mcp-config")
            .arg(&cfg)
            .arg("--strict-mcp-config")
            .arg("--allowedTools")
            .arg(&allowed)
            .arg("--append-system-prompt-file")
            .arg(&sys)
            .args(["--output-format", "text"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Couldn't start Claude Code: {e}"))?;
        use std::io::Write;
        child.stdin.take().unwrap().write_all(prompt.as_bytes()).map_err(|e| e.to_string())?;
        child.wait_with_output().map_err(|e| e.to_string())?
    };
    let _ = std::fs::remove_dir_all(&work);
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !output.status.success() && text.is_empty() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let hint = if err.to_lowercase().contains("login") || err.to_lowercase().contains("auth") || err.to_lowercase().contains("sign") {
            "\nOpen a terminal, run the app's command once (`claude` or `codex`) and sign in with your subscription."
        } else {
            ""
        };
        return Err(format!("{} stopped: {}{hint}", if provider == "codex-cli" { "Codex" } else { "Claude Code" }, err.chars().take(400).collect::<String>()));
    }
    let after = counts(&crate::store::Store::open(meta).data);
    let names = ["tasks", "tasks completed", "pages", "events", "database rows"];
    let mut changes = vec![];
    for i in 0..5 {
        if after[i] != before_counts[i] {
            let diff = after[i] as i64 - before_counts[i] as i64;
            changes.push(format!("{}{diff} {}", if diff > 0 { "+" } else { "" }, names[i]));
        }
    }
    let claims = claims_changes(&text);
    Ok(if changes.is_empty() {
        if claims {
            format!("⚠ Mismatch: the reply says changes were made, but nothing in your workspace changed. Treat its claims as not done.\n\n{text}")
        } else {
            text
        }
    } else {
        format!("{text}\n\nConfirmed in your workspace: {}", changes.join(", "))
    })
}

/// Does the model's prose claim it changed something?
pub fn claims_changes(text: &str) -> bool {
    let l = text.to_lowercase();
    ["i created", "i've created", "i have created", "i added", "i've added", "i have added", "created a new", "created the", "added a new", "added the", "i updated", "i've updated", "i scheduled", "i've scheduled", "i deleted", "i've deleted", "i moved", "i completed", "i've completed", "i marked", "has been created", "has been added", "has been updated", "have been added", "was created", "was added", "is now on your", "done! ", "all set"]
        .iter()
        .any(|p| l.contains(p))
}

/// The assistant's answer, showing only what the tools confirmed. If the prose claims changes the
/// tools did not complete, say so on screen.
pub fn confirmed_report(text: &str, actions: &[String]) -> String {
    let ok: Vec<&String> = actions.iter().filter(|a| a.starts_with('✓')).collect();
    let failed: Vec<&String> = actions.iter().filter(|a| a.starts_with('✗')).collect();
    let mismatch = claims_changes(text) && (ok.is_empty() || !failed.is_empty());
    let mut s = String::new();
    if mismatch {
        s.push_str(if ok.is_empty() {
            "⚠ Mismatch: the reply says changes were made, but the tools completed none. Nothing below the line happened unless it is listed as confirmed.\n\n"
        } else {
            "⚠ Mismatch: the reply claims more than the tools completed. Only the confirmed changes below happened.\n\n"
        });
    }
    s.push_str(text.trim());
    if !ok.is_empty() {
        s.push_str(&format!("\n\nConfirmed in your workspace:\n{}", ok.iter().map(|a| format!("- {a}")).collect::<Vec<_>>().join("\n")));
    }
    if !failed.is_empty() {
        s.push_str(&format!("\n\nNot done (the tool failed):\n{}", failed.iter().map(|a| format!("- {a}")).collect::<Vec<_>>().join("\n")));
    }
    s
}
pub fn claude_code_installed() -> bool {
    find_cli("claude").is_some()
}

pub(crate) fn run_agent(cfg: &AiConfig, system: &str, prompt: &str, meta: crate::store::WorkspaceMeta, allow_write: bool) -> Result<String, String> {
    if cfg.provider == "claude-code" || cfg.provider == "codex-cli" {
        return run_cli(&cfg.provider, system, prompt, meta, allow_write);
    }
    let mut store = crate::store::Store::open(meta);
    crate::mcp::begin_ai_batch(&store.meta.id);
    let read_only = !allow_write || store.settings.ai_read_only;
    let tools = crate::mcp::tools(read_only);
    let tool_list = tools.as_array().cloned().unwrap_or_default();
    let mut actions: Vec<String> = vec![];
    let mut run_tool = |name: &str, input: &Value, actions: &mut Vec<String>| -> (String, bool) {
        if read_only && !matches!(name, "workspace_overview" | "search" | "get_agenda" | "list_pages" | "read_page" | "list_tasks" | "list_projects" | "list_events" | "list_calendars" | "list_databases" | "query_database" | "resolve_projects" | "resolve_project" | "list_records") {
            return ("Changes are turned off for this conversation.".into(), true);
        }
        let r = crate::mcp::call(&mut store, name, input);
        let ok = r.is_ok();
        if !name.starts_with("list_") && !matches!(name, "search" | "read_page" | "get_agenda" | "workspace_overview" | "query_database" | "resolve_projects" | "resolve_project") {
            let what = input.get("title").or(input.get("text")).or(input.get("name")).or(input.get("id")).map(|v| v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string())).unwrap_or_default();
            actions.push(format!("{} {}{}", if ok { "✓" } else { "✗" }, name.replace('_', " "), if what.is_empty() { String::new() } else { format!(": {what}") }));
        }
        match r {
            Ok(t) => (t, false),
            Err(e) => (e, true),
        }
    };
    let finish = |text: String, actions: &Vec<String>| -> String { confirmed_report(&text, actions) };
    const MAX_STEPS: usize = 16;
    if cfg.provider == "anthropic" {
        if cfg.key.trim().is_empty() {
            return Err("Add your Anthropic API key in Settings → AI & MCP first.".into());
        }
        let tools: Vec<Value> = tool_list.iter().map(|t| json!({"name": t["name"], "description": t["description"], "input_schema": t["inputSchema"]})).collect();
        let mut messages = vec![json!({"role": "user", "content": prompt})];
        let headers = [
            ("x-api-key", cfg.key.trim().to_string()),
            ("anthropic-version", "2026-10-21".to_string()),
            ("anthropic-beta", "server-side-fallback-2026-07-01".to_string()),
        ];
        let model = if cfg.model.trim().is_empty() { CLAUDE_MODEL.to_string() } else { cfg.model.trim().to_string() };
        for _ in 0..MAX_STEPS {
            let body = json!({"model": model, "max_tokens": 16000, "system": system, "messages": messages, "tools": tools, "fallbacks": "default"});
            let resp = post("https://api.anthropic.com/v1/messages", &headers, &body)?;
            record_from_response("anthropic", &model, &resp);
            let content = resp["content"].as_array().cloned().unwrap_or_default();
            let stop = resp["stop_reason"].as_str().unwrap_or("");
            if stop == "refusal" {
                return Ok(finish("Claude declined this request.".into(), &actions));
            }
            let text: String = content.iter().filter(|b| b["type"] == "text").filter_map(|b| b["text"].as_str()).collect::<Vec<_>>().join("\n");
            if stop != "tool_use" {
                return Ok(finish(text, &actions));
            }
            // keep the whole assistant turn (thinking blocks included) unchanged
            messages.push(json!({"role": "assistant", "content": content}));
            let mut results = vec![];
            for b in content.iter().filter(|b| b["type"] == "tool_use") {
                let (out, is_err) = run_tool(b["name"].as_str().unwrap_or(""), &b["input"], &mut actions);
                results.push(json!({"type": "tool_result", "tool_use_id": b["id"], "content": out, "is_error": is_err}));
            }
            messages.push(json!({"role": "user", "content": results}));
        }
        return Ok(finish("I stopped after many steps; tell me to continue if there's more to do.".into(), &actions));
    }
    // OpenAI-style chat completions: ChatGPT API, Ollama and local OpenAI-compatible servers
    let (endpoint, headers): (String, Vec<(&str, String)>) = match cfg.provider.as_str() {
        "openai-cloud" => {
            if cfg.key.trim().is_empty() {
                return Err("Add your OpenAI API key in Settings → AI & MCP first.".into());
            }
            ("https://api.openai.com/v1/chat/completions".into(), vec![("Authorization", format!("Bearer {}", cfg.key.trim()))])
        }
        _ => (format!("{}/v1/chat/completions", base(&cfg.url)), vec![]),
    };
    let tools: Vec<Value> = tool_list.iter().map(|t| json!({"type": "function", "function": {"name": t["name"], "description": t["description"], "parameters": t["inputSchema"]}})).collect();
    let mut messages = vec![json!({"role": "system", "content": system}), json!({"role": "user", "content": prompt})];
    let mut use_tools = true;
    for _ in 0..MAX_STEPS {
        let mut body = json!({"model": cfg.model, "messages": messages, "stream": false});
        if use_tools {
            body["tools"] = json!(tools);
        }
        let resp = match post(&endpoint, &headers, &body) {
            Ok(r) => r,
            Err(e) if use_tools && e.to_lowercase().contains("tool") => {
                // small local models often can't call tools: answer without them
                use_tools = false;
                continue;
            }
            Err(e) => return Err(e),
        };
        record_from_response(&cfg.provider, &cfg.model, &resp);
        let msg = resp["choices"][0]["message"].clone();
        let calls = msg["tool_calls"].as_array().cloned().unwrap_or_default();
        if calls.is_empty() {
            let text = msg["content"].as_str().unwrap_or("").to_string();
            let note = if use_tools { String::new() } else { "\n\n(This model can't make changes in the app; pick a model with tool support to let it act.)".to_string() };
            return Ok(finish(format!("{text}{note}"), &actions));
        }
        messages.push(msg.clone());
        for c in calls {
            let name = c["function"]["name"].as_str().unwrap_or("").to_string();
            let args: Value = match &c["function"]["arguments"] {
                Value::String(s) => serde_json::from_str(s).unwrap_or(json!({})),
                v => v.clone(),
            };
            let (out, _) = run_tool(&name, &args, &mut actions);
            messages.push(json!({"role": "tool", "tool_call_id": c["id"], "content": out}));
        }
    }
    Ok(finish("I stopped after many steps; tell me to continue if there's more to do.".into(), &actions))
}

// ---------------------------------------------------------------- call trace (Test Intake inspector)

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Trace {
    pub at: i64,
    pub purpose: String,
    pub provider: String,
    pub model: String,
    /// The full prompt sent (system prompt; the user turn is a fixed "Return the JSON now.").
    pub request: String,
    /// The model's raw text, or the error.
    pub response: String,
    pub ok: bool,
    /// How long the call took.
    #[serde(default)]
    pub ms: u64,
}

thread_local! {
    static TRACE: std::cell::RefCell<Vec<Trace>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn trace_push(cfg: &AiConfig, system: &str, raw: &Result<String, String>, ms: u64) {
    let purpose = PURPOSE.with(|x| x.borrow().clone());
    let t = Trace {
        at: crate::model::now_ms(),
        purpose,
        provider: cfg.provider.clone(),
        model: if cfg.model.is_empty() { "(default)".into() } else { cfg.model.clone() },
        request: system.to_string(),
        response: match raw { Ok(s) => s.clone(), Err(e) => format!("ERROR: {e}") },
        ok: raw.is_ok(),
        ms,
    };
    TRACE.with(|x| x.borrow_mut().push(t));
}

/// Take the calls recorded on this thread since the last take.
pub fn take_trace() -> Vec<Trace> {
    TRACE.with(|x| std::mem::take(&mut *x.borrow_mut()))
}
// ---------------------------------------------------------------- structured (JSON) completions
//
// Used by Resolve Intake and ResolveOS roles. One system prompt (with the user's text fenced
// inside it, as in the original intake), JSON back, validated by the caller; on a parse/shape
// failure the call is retried ONCE with a correction note, like the original `callWithValidation`.

/// Extract the first JSON object from model text (handles ```json fences and chatter).
pub fn parse_json_object(text: &str) -> Result<Value, String> {
    let t = text.trim();
    if let Ok(v) = serde_json::from_str::<Value>(t) {
        if v.is_object() {
            return Ok(v);
        }
    }
    let start = t.find('{').ok_or("The model didn't return JSON")?;
    let end = t.rfind('}').ok_or("The model didn't return complete JSON")?;
    serde_json::from_str::<Value>(&t[start..=end]).map_err(|e| format!("The model's JSON couldn't be read: {e}"))
}

/// One completion that should return a JSON object. `temperature` None = provider default.
/// One raw call; returns the model's text. Every call (success or failure) is added to the trace.
pub fn complete_json_once(cfg: &AiConfig, system: &str, temperature: Option<f64>) -> Result<Value, String> {
    let t0 = std::time::Instant::now();
    let raw = complete_text_once(cfg, system, temperature);
    trace_push(cfg, system, &raw, t0.elapsed().as_millis() as u64);
    raw.and_then(|t| parse_json_object(&t))
}

fn complete_text_once(cfg: &AiConfig, system: &str, temperature: Option<f64>) -> Result<String, String> {
    let user = "Return the JSON now.";
    match cfg.provider.as_str() {
        "anthropic" => {
            if cfg.key.trim().is_empty() {
                return Err("Add your Anthropic API key in Settings → AI & MCP first.".into());
            }
            let model = if cfg.model.trim().is_empty() { CLAUDE_MODEL.to_string() } else { cfg.model.trim().to_string() };
            let headers = [("x-api-key", cfg.key.trim().to_string()), ("anthropic-version", "2026-10-21".to_string())];
            let mut body = json!({"model": model, "max_tokens": 8000, "system": system, "messages": [{"role": "user", "content": user}]});
            if let Some(t) = temperature {
                body["temperature"] = json!(t);
            }
            let resp = post("https://api.anthropic.com/v1/messages", &headers, &body)?;
            record_from_response("anthropic", &model, &resp);
            let text: String = resp["content"].as_array().cloned().unwrap_or_default().iter().filter(|b| b["type"] == "text").filter_map(|b| b["text"].as_str().map(|s| s.to_string())).collect::<Vec<_>>().join("\n");
            Ok(text)
        }
        "claude-code" | "codex-cli" => {
            // the intake's waits use faster models on the Claude Code CLI: sorting an answer is a
            // small judgement (haiku); the next question must arrive inside the 15 s limit (sonnet)
            let purpose = PURPOSE.with(|x| x.borrow().clone());
            let model = match (cfg.model.trim().is_empty() && cfg.provider == "claude-code", purpose.as_str()) {
                (true, "intake:classify") => "haiku".to_string(),
                (true, "intake:next-question") => "sonnet".to_string(),
                _ => cfg.model.trim().to_string(),
            };
            let text = run_cli_plain_model(&cfg.provider, system, user, &model)?;
            record_usage(&cfg.provider, "subscription", 0, 0);
            Ok(text)
        }
        _ => {
            let (endpoint, headers): (String, Vec<(&str, String)>) = match cfg.provider.as_str() {
                "openai-cloud" => {
                    if cfg.key.trim().is_empty() {
                        return Err("Add your OpenAI API key in Settings → AI & MCP first.".into());
                    }
                    ("https://api.openai.com/v1/chat/completions".into(), vec![("Authorization", format!("Bearer {}", cfg.key.trim()))])
                }
                _ => (format!("{}/v1/chat/completions", base(&cfg.url)), vec![]),
            };
            let mut body = json!({"model": cfg.model, "messages": [{"role": "system", "content": system}, {"role": "user", "content": user}], "stream": false, "response_format": {"type": "json_object"}});
            if let Some(t) = temperature {
                body["temperature"] = json!(t);
            }
            let resp = post(&endpoint, &headers, &body)?;
            record_from_response(&cfg.provider, &cfg.model, &resp);
            let text = resp["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
            Ok(text)
        }
    }
}

/// `build(is_retry)` returns the system prompt; `validate` checks/normalises the shape.
pub fn complete_json<T>(cfg: &AiConfig, temperature: Option<f64>, build: impl Fn(bool) -> String, validate: impl Fn(&Value) -> Result<T, String>) -> Result<T, String> {
    check_budget()?;
    let first = complete_json_once(cfg, &build(false), temperature).and_then(|v| validate(&v));
    match first {
        Ok(x) => Ok(x),
        Err(e) if e.starts_with("Add your") || e.contains("Couldn't reach") || e.contains("No local AI") || e.contains("isn't installed") || e.contains("budget") => Err(e),
        Err(_) => complete_json_once(cfg, &build(true), temperature).and_then(|v| validate(&v)).map_err(|e| format!("The AI's answer didn't come back in the right shape, even after one retry ({e}). Try again, or pick a stronger model.")),
    }
}

/// Stop AI calls once this month's estimated spend reaches the user's budget (if set).
pub fn check_budget() -> Result<(), String> {
    let s = crate::store::Registry::load().settings;
    if s.monthly_budget <= 0.0 {
        return Ok(());
    }
    let now = chrono::Local::now();
    let since = chrono::NaiveDate::from_ymd_opt(chrono::Datelike::year(&now), chrono::Datelike::month(&now), 1)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .and_then(|d| d.and_local_timezone(chrono::Local).single())
        .map(|d| d.timestamp_millis())
        .unwrap_or(0);
    let u = usage_since(since);
    let cost = (u.input as f64 * s.price_in_per_m + u.output as f64 * s.price_out_per_m) / 1_000_000.0;
    if cost >= s.monthly_budget {
        return Err(format!("This month's AI budget (${:.2}) has been reached (≈ ${cost:.2}). Raise it in Settings → AI & MCP, or continue manually.", s.monthly_budget));
    }
    Ok(())
}

/// The configured provider as an AiConfig (keys come from this PC's settings only).
pub fn current_config() -> AiConfig {
    let s = crate::store::Registry::load().settings;
    let key = match s.ai_provider.as_str() {
        "anthropic" => s.anthropic_key.clone(),
        "openai-cloud" => s.openai_key.clone(),
        _ => String::new(),
    };
    AiConfig { provider: s.ai_provider, url: s.ai_url, model: s.ai_model, key }
}

/// A plain (no tools) prompt through the Claude Code / Codex CLI.
#[allow(dead_code)]
fn run_cli_plain(provider: &str, system: &str, prompt: &str) -> Result<String, String> {
    run_cli_plain_model(provider, system, prompt, "")
}

fn run_cli_plain_model(provider: &str, system: &str, prompt: &str, model: &str) -> Result<String, String> {
    use std::io::Write;
    let work = std::env::temp_dir().join(format!("resolve-2-plain-{}-{}", std::process::id(), crate::model::new_id()));
    std::fs::create_dir_all(&work).map_err(|e| e.to_string())?;
    let output = if provider == "codex-cli" {
        let codex = find_cli("codex").ok_or("Codex isn't installed.")?;
        let mut child = std::process::Command::new(codex)
            .current_dir(&work)
            .args(["exec", "--skip-git-repo-check", "-"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Couldn't start Codex: {e}"))?;
        child.stdin.take().unwrap().write_all(format!("{system}\n\n{prompt}").as_bytes()).map_err(|e| e.to_string())?;
        child.wait_with_output().map_err(|e| e.to_string())?
    } else {
        let claude = find_cli("claude").ok_or("Claude Code isn't installed.")?;
        let sys = work.join("system.txt");
        std::fs::write(&sys, system).map_err(|e| e.to_string())?;
        let mut cmd = std::process::Command::new(claude);
        cmd.current_dir(&work).args(["-p", "--strict-mcp-config", "--output-format", "text"]);
        if !model.is_empty() {
            cmd.arg("--model").arg(model);
        }
        let mut child = cmd
            .arg("--system-prompt-file")
            .arg(&sys)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Couldn't start Claude Code: {e}"))?;
        child.stdin.take().unwrap().write_all(prompt.as_bytes()).map_err(|e| e.to_string())?;
        child.wait_with_output().map_err(|e| e.to_string())?
    };
    let _ = std::fs::remove_dir_all(&work);
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        return Err(format!("The CLI returned nothing: {}", String::from_utf8_lossy(&output.stderr).chars().take(300).collect::<String>()));
    }
    Ok(text)
}

pub fn ask_agent(cfg: AiConfig, system: &str, prompt: &str, meta: crate::store::WorkspaceMeta, allow_write: bool, label: &str) -> AiJob {
    let (tx, rx) = channel();
    let (s, q) = (system.to_string(), prompt.to_string());
    std::thread::spawn(move || {
        let _ = tx.send(run_agent(&cfg, &s, &q, meta, allow_write));
    });
    AiJob { rx, label: label.to_string() }
}

/// Used by the "Organise my to-dos" button and offered to Claude/ChatGPT as an MCP prompt.
pub const ORGANISE_PROMPT: &str = "Organise the to-dos I've written inside my pages into my task projects.
1. Call list_page_todos to get every unticked to-do (with page titles), and list_projects for my existing projects.
2. Read any page whose to-dos need more context (read_page).
3. Decide for each to-do: which project it belongs to (prefer existing projects; create a new project only when several to-dos clearly share a theme — name it well, and add sections when a project has distinct phases); a clean task title; a due date and time if the text or page implies one (\"by Friday\", \"before the launch on the 12th\", meeting dates); a priority (p1 only for genuinely urgent or blocking items); repeats if it recurs; labels like @phone or @errand when obvious; and subtasks (parent_block_id) where one to-do is clearly part of another.
4. Call convert_todos_to_tasks ONCE with all the items. Skip to-dos that are already linked. Don't invent work that isn't written down.
5. Reply with a short summary: new projects, then per project the tasks created with their dates, and any to-dos you left alone and why.";

pub const AGENT_SYSTEM: &str = "You are the assistant built into Resolve 2, the user's personal workspace of pages (documents), databases (tables/boards, e.g. a CRM), projects and tasks (priorities p1-p4, due dates, repeats, reminders) and calendars. You can read and change the workspace with the tools provided - use them to actually do what the user asks (create or edit pages, add tasks or events, update database rows) rather than describing how. Look things up with search or read_page before editing them. Ask before deleting anything. Dates are local; today is given in the request. Keep replies short and use Markdown; when you write content for a page, write it into the page.";

pub const SYSTEM: &str = "You are a helpful assistant inside Resolve 2, a personal workspace of pages, tasks and calendars. Be concise. Use Markdown: short paragraphs, '- ' bullets, '- [ ] ' for action items.";
