//! Checks for undo, soft deletes, backups before AI writes and the Ask AI mismatch flag.

use crate::model::*;
use crate::selftest::T;
use crate::store::{Store, WorkspaceMeta};
use serde_json::json;

pub fn run(t: &mut T, meta: WorkspaceMeta, port: u16) {
    println!("\n-- Undo, soft delete, AI safety --");
    let mut s = Store::open(meta.clone());

    // a task with a subtask, deleted then undone
    let mut parent = Task::new("Undo me");
    let pid = parent.id.clone();
    let mut child = Task::new("Undo my child");
    child.parent_id = Some(pid.clone());
    let cid = child.id.clone();
    parent.updated -= 10;
    s.data.tasks.push(parent);
    s.data.tasks.push(child);
    let cs = s.data.record_change("Deleted task", "delete", |d| d.delete_task(&pid));
    t.check("deleting a task records a change set (soft delete)", cs.is_some() && s.data.task(&pid).is_none() && s.data.recently_deleted().iter().any(|c| Some(&c.id) == cs.as_ref()), "no change set");
    let r = s.data.undo_change(cs.as_deref().unwrap_or(""));
    t.check("undo restores the task and its subtask", r.is_ok() && s.data.task(&pid).is_some() && s.data.task(&cid).is_some(), format!("{r:?}"));
    let mut other = s.data.clone();
    other.tombstones.push(Tombstone { id: pid.clone(), deleted: now_ms() - 5000 });
    s.data.merge(&other);
    t.check("a restored item survives merging with a copy that still has the old deletion", s.data.task(&pid).is_some(), "lost on merge");
    t.check("an undone delete leaves Recently deleted", !s.data.recently_deleted().iter().any(|c| Some(&c.id) == cs.as_ref()), "still listed");

    // undo leaves later edits alone
    let cs2 = s.data.record_change("Completed", "complete", |d| {
        d.task_mut(&cid).unwrap().complete();
    });
    std::thread::sleep(std::time::Duration::from_millis(5));
    if let Some(x) = s.data.task_mut(&cid) {
        x.title = "Edited after".into();
        x.updated = now_ms() + 1;
    }
    let r = s.data.undo_change(cs2.as_deref().unwrap_or(""));
    t.check("undo skips items edited after the change", r.as_ref().map_or(false, |(_, skipped)| *skipped == 1) && s.data.task(&cid).map_or(false, |x| x.title == "Edited after"), format!("{r:?}"));
    let _ = s.save();

    // AI delete over MCP: soft, restorable, backed up first
    let backups_before = s.list_backups().len();
    crate::mcp::begin_ai_batch(&meta.id);
    let made = crate::mcp::call(&mut s, "create_task", &json!({"title": "AI-made task"}));
    let tid = made.ok().and_then(|x| serde_json::from_str::<serde_json::Value>(&x).ok()).and_then(|v| v["task"]["id"].as_str().map(|s| s.to_string())).unwrap_or_default();
    let del = crate::mcp::call(&mut s, "delete_item", &json!({"id": tid}));
    let backups_after = s.list_backups().len();
    t.check("an AI write batch takes one backup first", backups_after == backups_before + 1, format!("{backups_before} → {backups_after}"));
    let listed = s.data.recently_deleted().into_iter().find(|c| c.label.starts_with("AI: delete item")).map(|c| c.id.clone());
    t.check("an AI/MCP delete is soft and shows in Recently deleted", del.is_ok() && s.data.task(&tid).is_none() && listed.is_some(), format!("{del:?}"));
    let r = s.data.undo_change(listed.as_deref().unwrap_or(""));
    t.check("restoring an AI delete brings the item back", r.is_ok() && s.data.task(&tid).is_some(), format!("{r:?}"));
    let ai_sets = s.data.history.iter().filter(|c| c.label.starts_with("AI:")).count();
    t.check("every AI write is recorded so it can be undone", ai_sets >= 2, ai_sets);
    let _ = s.save();

    // Ask AI: only tool-confirmed results; mismatches flagged
    let rep = crate::ai::confirmed_report("I created a new task for you.", &["✗ create task: Call the supplier".to_string()]);
    t.check("a claimed-but-failed change is flagged as a mismatch", rep.starts_with("⚠ Mismatch") && rep.contains("Not done") && !rep.contains("Confirmed in your workspace"), &rep);
    let rep = crate::ai::confirmed_report("Added it.", &["✓ create task: Call the supplier".to_string()]);
    t.check("confirmed changes are listed without a warning", !rep.contains("Mismatch") && rep.contains("Confirmed in your workspace"), &rep);
    let rep = crate::ai::confirmed_report("I've added the task to your list.", &[]);
    t.check("a claim with no tool call at all is flagged", rep.starts_with("⚠ Mismatch"), &rep);

    // the agent loop end to end: the stand-in model's tool call fails, its prose claims success
    if let Ok(fake) = tiny_http::Server::http(("127.0.0.1", port)) {
        let th = std::thread::spawn(move || {
            for (i, mut req) in fake.incoming_requests().take(2).enumerate() {
                let mut body = String::new();
                let _ = std::io::Read::read_to_string(req.as_reader(), &mut body);
                let reply = if i == 0 {
                    json!({"choices": [{"message": {"role": "assistant", "content": null, "tool_calls": [{"id": "c1", "type": "function", "function": {"name": "update_task", "arguments": "{\"id\":\"no-such-task\",\"title\":\"x\"}"}}]}}]})
                } else {
                    json!({"choices": [{"message": {"role": "assistant", "content": "I created a new task with the details you provided."}}]})
                };
                let _ = req.respond(tiny_http::Response::from_string(reply.to_string()).with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()));
            }
        });
        let cfg = crate::ai::AiConfig { provider: "openai".into(), url: format!("http://127.0.0.1:{port}"), model: "stand-in".into(), key: String::new() };
        let out = crate::ai::run_agent(&cfg, crate::ai::AGENT_SYSTEM, "Add a task", meta.clone(), true);
        let _ = th.join();
        t.check("Ask AI flags a model that claims a change its tool call didn't make (the Ollama case)", out.as_ref().map_or(false, |o| o.starts_with("⚠ Mismatch") && o.contains("✗ update task")), format!("{out:?}"));
    }
}
