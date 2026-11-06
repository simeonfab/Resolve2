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

    navigation(t, &mut s);
}

/// Part 4: Undo / Redo at the top of the sidebar, the navigation config, AI Team conversations
/// and the cut list.
fn navigation(t: &mut T, s: &mut Store) {
    println!("\n-- Navigation (Part 4) --");
    let tid = {
        let task = Task::new("Undo/redo me");
        let id = task.id.clone();
        s.data.tasks.push(task);
        id
    };
    let tid2 = tid.clone();
    let cs = s.data.record_change("Deleted task \"Undo/redo me\"", "delete", |d| d.delete_task(&tid2));
    t.check("Undo targets the newest change", s.data.undo_target().map(|c| Some(&c.id) == cs.as_ref()).unwrap_or(false), "wrong target");
    let r = s.data.undo_recorded(cs.as_deref().unwrap_or(""));
    t.check("Undo (recorded) brings the task back", r.is_ok() && s.data.task(&tid).is_some(), format!("{r:?}"));
    t.check("after an undo, Redo is offered", s.data.redo_target().is_some(), "no redo");
    let r = s.data.redo_last();
    t.check("Redo deletes it again", r.is_ok() && s.data.task(&tid).is_none(), format!("{r:?}"));
    t.check("Redo can itself be undone", s.data.undo_target().map_or(false, |c| c.kind == "redo"), "redo not undoable");
    let r = s.data.undo_recorded(&s.data.undo_target().map(|c| c.id.clone()).unwrap_or_default());
    t.check("undoing the redo restores the task", r.is_ok() && s.data.task(&tid).is_some(), format!("{r:?}"));
    // a new change clears Redo
    let tid3 = tid.clone();
    s.data.record_change("Completed", "complete", |d| {
        if let Some(x) = d.task_mut(&tid3) {
            x.complete();
        }
    });
    t.check("a new change clears Redo", s.data.redo_target().is_none(), "redo still offered");

    // the sidebar comes from one config, in the brief's order, and nothing appears twice
    use crate::ui::nav::{Section, Target, NAV, UTILITIES, FRAMEWORKS};
    let labels: Vec<&str> = NAV.iter().flat_map(|g| g.items.iter().map(|i| i.label)).collect();
    t.check("sidebar groups: Workspace, Product management (with picker), Collaboration", NAV.iter().map(|g| g.label).collect::<Vec<_>>() == ["Workspace", "Product management", "Collaboration"] && NAV[1].picker, NAV.iter().map(|g| g.label).collect::<Vec<_>>().join(", "));
    t.check("sidebar items in the brief's order", labels == ["Home", "Projects", "Calendar", "Overview", "Roadmap", "Tasks", "Frameworks", "Knowledge", "AI Team", "Handoffs"], labels.join(", "));
    let utils: Vec<&str> = UTILITIES.iter().map(|i| i.label).collect();
    t.check("bottom: Search, Settings, Help", utils.len() == 3 && utils[0] == "Search" && utils[1].starts_with("Settings") && utils[2].starts_with("Help"), utils.join(", "));
    let mut targets: Vec<Target> = NAV.iter().flat_map(|g| g.items.iter().map(|i| i.target)).chain(UTILITIES.iter().map(|i| i.target)).collect();
    let n = targets.len();
    targets.dedup();
    t.check("no destination appears twice in the sidebar", targets.len() == n, n);
    t.check("Frameworks registry starts with Hypothesis Zero", FRAMEWORKS.first().map_or(false, |f| f.name == "Hypothesis Zero"), "");
    let ask = |s: Section| NAV.iter().flat_map(|g| g.items.iter()).find(|i| i.target == Target::Sec(s)).map(|i| i.ask.placeholder).unwrap_or("");
    t.check("Ask Resolve wording changes per section (Roadmap vs Overview)", ask(Section::Roadmap) == "Ask about your roadmap" && ask(Section::Overview) != ask(Section::Roadmap), ask(Section::Roadmap));

    // AI Team conversations live in the workspace data and merge like everything else
    let mut a = WorkspaceData::default();
    let mut c = Conversation { id: new_id(), agent: "general".into(), updated: 1, ..Default::default() };
    a.conversations.push(c.clone());
    let mut b = a.clone();
    c.messages.push(ChatMessage { role: "user".into(), text: "Hello".into(), at: 2, ..Default::default() });
    c.updated = 2;
    b.conversations[0] = c;
    a.merge(&b);
    t.check("AI Team conversations merge (newest wins)", a.conversations.len() == 1 && a.conversations[0].messages.len() == 1, a.conversations.len());
    t.check("the Ask AI conversation is the General assistant's", crate::resolve::team::specialist(crate::resolve::team::GENERAL).name == "General assistant" && crate::resolve::team::TEAM.len() == 5, "");

    // a theme left off the roadmap during the intake lands on the project's cut list
    let mut d = WorkspaceData::default();
    let mut i = crate::resolve::Intake::default();
    i.id = new_id();
    i.proposal = Some(crate::resolve::journey::Proposal { name: "Cut test".into(), cut: vec!["Redesign the logo".into()], ..Default::default() });
    let iid = i.id.clone();
    d.intakes.push(i);
    let pid = crate::resolve::journey::adopt_all(&mut d, &iid).unwrap_or_default();
    let cut = d.context_for(&pid).map(|c| c.recommendation.do_not_yet.iter().map(|n| n.item.clone()).collect::<Vec<_>>()).unwrap_or_default();
    t.check("themes left off the roadmap are kept on the cut list", cut == ["Redesign the logo"], format!("{cut:?}"));
    let _ = s.save();
}
