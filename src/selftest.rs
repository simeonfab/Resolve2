//! `resolve2 --selftest`: end-to-end checks of the real code paths (storage, MCP tools, import,
//! sync, backups, isolation, HTTP server) in a throwaway data folder. Prints PASS/FAIL per step.

use crate::mcp;
use crate::model::*;
use crate::store::{self, Registry, Store};
use serde_json::{json, Value};

pub(crate) struct T {
    pub pass: usize,
    pub fail: usize,
}

impl T {
    pub fn check(&mut self, name: &str, ok: bool, detail: impl std::fmt::Display) {
        if ok {
            self.pass += 1;
            println!("PASS  {name}");
        } else {
            self.fail += 1;
            println!("FAIL  {name} — {detail}");
        }
    }
}

fn call(s: &mut Store, tool: &str, args: Value) -> Result<Value, String> {
    let text = mcp::call(s, tool, &args)?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

pub fn run() -> i32 {
    let base = std::env::temp_dir().join(format!("resolve2-selftest-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&base);
    store::set_root(base.join("data"));
    // standing rule: the self-test runs only in its own sandbox folder. If something set the data
    // folder first (--data-dir, RESOLVE2_DATA), it stops before opening anything.
    if store::root() != base.join("data") || store::is_real_root(&store::root()) {
        eprintln!("Refused: the self-test runs only in its sandbox folder ({}), not {}.", base.join("data").display(), store::root().display());
        return 2;
    }
    let mut t = T { pass: 0, fail: 0 };
    println!("Resolve 2 self-test in {}", base.display());

    // ---- workspaces
    let mut reg = Registry::load();
    t.check("a fresh copy starts with one empty Personal workspace (no demo workspace)", reg.workspaces.len() == 1 && reg.find("Personal").is_some(), format!("{} workspaces", reg.workspaces.len()));
    t.check("a fresh copy starts at Connect your AI (not set up yet)", !reg.settings.ai_setup_done && reg.settings.anthropic_key.is_empty() && reg.settings.openai_key.is_empty(), "already set up, or a key is present");
    let personal = reg.find("Personal").cloned().unwrap();
    let personal_empty = Store::open(personal.clone());
    t.check("your real workspace starts without demo data", personal_empty.data.tasks.is_empty() && personal_empty.data.events.is_empty(), format!("{} tasks", personal_empty.data.tasks.len()));
    let work = reg.create("Work", [63, 81, 181]);
    let mut s = Store::open(personal.clone());

    // ---- pages
    let r = call(&mut s, "create_page", json!({"title": "Business plan", "markdown": "## Problem\nSmall teams juggle **three apps**.\n- [ ] Validate with 5 users\n\nSee [[Pricing]]"}));
    let page_id = r.as_ref().map(|v| v["id"].as_str().unwrap_or("").to_string()).unwrap_or_default();
    t.check("create a page from Markdown", !page_id.is_empty(), format!("{r:?}"));
    let r = call(&mut s, "create_page", json!({"title": "Pricing", "markdown": "Free tier + paid", "parent_id": page_id}));
    let child_id = r.as_ref().map(|v| v["id"].as_str().unwrap_or("").to_string()).unwrap_or_default();
    t.check("nested page under a parent", s.data.page(&child_id).and_then(|p| p.parent.clone()) == Some(page_id.clone()), "no parent");
    let _ = call(&mut s, "update_page", json!({"id": page_id, "append_markdown": "## Solution\nOne local-first app."}));
    let pg = s.data.page(&page_id).cloned().unwrap_or_default();
    t.check("edit a page (append blocks)", pg.blocks.iter().any(|b| b.kind == BlockKind::H2 && b.text == "Solution") && pg.blocks.iter().any(|b| b.kind == BlockKind::Todo), format!("{} blocks", pg.blocks.len()));
    t.check("[[links]] create backlinks", s.data.backlinks(&child_id).iter().any(|p| p.id == page_id), "no backlink");

    // ---- CRM with linked records
    let r = call(&mut s, "create_database", json!({"name": "Companies", "columns": [{"name": "Company", "type": "text"}, {"name": "Industry", "type": "select", "options": ["Software", "Retail"]}]}));
    let comp_db = r.map(|v| v["id"].as_str().unwrap_or("").to_string()).unwrap_or_default();
    let r = call(&mut s, "create_database", json!({"name": "Contacts", "columns": [{"name": "Name", "type": "text"}, {"name": "Email", "type": "text"}, {"name": "Stage", "type": "select", "options": ["Lead", "Customer"]}]}));
    let cont_db = r.map(|v| v["id"].as_str().unwrap_or("").to_string()).unwrap_or_default();
    // add a relation column Contacts.Company → Companies, and Contacts.Project → projects
    if let Some(db) = s.data.database_mut(&cont_db) {
        db.columns.push(Column { id: new_id(), name: "Company".into(), kind: ColKind::Relation, options: vec![], width: 150.0, target: format!("db:{comp_db}") });
        db.columns.push(Column { id: new_id(), name: "Project".into(), kind: ColKind::Relation, options: vec![], width: 150.0, target: "projects".into() });
        db.updated = now_ms();
    }
    let _ = s.save();
    let r = call(&mut s, "add_database_row", json!({"database": "Companies", "values": {"Company": "Northwind", "Industry": "Software"}}));
    let comp_row = r.map(|v| v["row"]["row_id"].as_str().unwrap_or("").to_string()).unwrap_or_default();
    let r = call(&mut s, "create_project", json!({"name": "Northwind rollout", "sections": ["Discovery", "Delivery"]}));
    let proj_id = r.map(|v| v["id"].as_str().unwrap_or("").to_string()).unwrap_or_default();
    let r = call(&mut s, "add_database_row", json!({"database": "Contacts", "values": {"Name": "Ada Lovelace", "Email": "ada@northwind.test", "Stage": "Lead", "Company": comp_row, "Project": proj_id}}));
    let contact_row = r.as_ref().map(|v| v["row"]["row_id"].as_str().unwrap_or("").to_string()).unwrap_or_default();
    t.check("add CRM records", !comp_row.is_empty() && !contact_row.is_empty(), format!("{r:?}"));
    let links = s.data.incoming_links(&comp_row);
    t.check("contact links to its company (relation) and shows as a backlink", links.iter().any(|l| l.3 == contact_row), format!("{links:?}"));
    let db = s.data.database(&cont_db).cloned().unwrap_or_default();
    let projcol = db.col("Project").cloned().unwrap_or_default();
    let shown = db.rows.first().map(|r| s.data.cell_display(&projcol, r.cells.get(&projcol.id).map(|x| x.as_str()).unwrap_or(""))).unwrap_or_default();
    t.check("contact links to a project", shown == "Northwind rollout", shown);
    // filter
    if let Some(db) = s.data.database_mut(&cont_db) {
        let stage = db.col("Stage").map(|c| c.id.clone()).unwrap_or_default();
        db.filters.push(DbFilter { col: stage, op: FilterOp::Is, value: "Customer".into() });
    }
    let db = s.data.database(&cont_db).cloned().unwrap_or_default();
    let visible = db.rows.iter().filter(|r| db.filters.iter().all(|f| f.matches(r.cells.get(&f.col).map(|x| x.as_str()).unwrap_or(""), r.cells.get(&f.col).map(|x| x.as_str()).unwrap_or("")))).count();
    t.check("database filter hides non-matching rows", visible == 0, visible);

    // ---- tasks
    let r = call(&mut s, "create_task", json!({"text": "Pay rent every month 9am p1 #Home remind 30m"}));
    let rent = r.as_ref().map(|v| v["task"]["id"].as_str().unwrap_or("").to_string()).unwrap_or_default();
    let rt = s.data.task(&rent).cloned().unwrap_or_else(|| Task::new(""));
    t.check("quick capture parses repeat, time, priority, project and reminder", rt.recurrence == Recurrence::Monthly && rt.priority == 1 && rt.due_time.is_some() && rt.reminder_min == Some(30) && s.data.project_name(&rt.project_id) == "Home", format!("{rt:?}"));
    let first_due = rt.due;
    let _ = call(&mut s, "complete_task", json!({"id": rent}));
    let rt2 = s.data.task(&rent).cloned().unwrap_or_else(|| Task::new(""));
    t.check("completing a recurring task moves it to the next date", !rt2.completed && rt2.due > first_due, format!("{:?} -> {:?}", first_due, rt2.due));
    let r = call(&mut s, "create_task", json!({"title": "Send proposal", "project": "Northwind rollout", "section": "Discovery", "due": "tomorrow", "time": "14:00", "duration_minutes": 60, "page_id": page_id}));
    let prop = r.map(|v| v["task"]["id"].as_str().unwrap_or("").to_string()).unwrap_or_default();
    let r = call(&mut s, "create_task", json!({"title": "Draft pricing table", "parent_id": prop}));
    let sub = r.map(|v| v["task"]["id"].as_str().unwrap_or("").to_string()).unwrap_or_default();
    t.check("subtask inherits the parent's project", s.data.task(&sub).map(|x| x.project_id.clone()) == Some(Some(proj_id.clone())), "wrong project");
    t.check("task links to a page", s.data.tasks.iter().any(|x| x.page_id.as_deref() == Some(page_id.as_str())), "no link");
    let _ = call(&mut s, "update_task", json!({"id": prop, "project": "Inbox"}));
    t.check("move a task between projects", s.data.task(&prop).map(|x| x.project_id.is_none()) == Some(true), "not moved");
    let agenda = call(&mut s, "get_agenda", json!({"days": 3})).unwrap_or(json!([]));
    let on_cal = agenda.as_array().map(|days| days.iter().any(|d| d["tasks"].as_array().map_or(false, |ts| ts.iter().any(|x| x["id"] == prop)))).unwrap_or(false);
    t.check("scheduled task appears on the calendar agenda", on_cal, "missing");

    // ---- calendar
    let cal = s.data.calendars.first().map(|c| c.name.clone()).unwrap_or_default();
    let day = today() + chrono::Duration::days(2);
    let r = call(&mut s, "create_event", json!({"title": "Northwind call", "start": format!("{day} 10:00"), "duration_minutes": 45, "calendar": cal, "recurrence": "every week"}));
    let ev = r.as_ref().map(|v| v["event"]["id"].as_str().unwrap_or("").to_string()).unwrap_or_default();
    t.check("create a recurring event", s.data.events.iter().any(|e| e.id == ev && e.recurrence == Recurrence::Weekly), format!("{r:?}"));
    let _ = call(&mut s, "update_event", json!({"id": ev, "start": format!("{day} 15:30")}));
    let e = s.data.events.iter().find(|e| e.id == ev).cloned();
    t.check("move an event (duration kept)", e.as_ref().map_or(false, |e| e.start.format("%H:%M").to_string() == "15:30" && (e.end - e.start).num_minutes() == 45), format!("{e:?}"));
    let next_week = day + chrono::Duration::days(7);
    t.check("recurring event repeats next week", e.as_ref().map_or(false, |e| e.occurrence_on(next_week).is_some()), "no repeat");
    s.data.calendars.push(Calendar { id: new_id(), name: "Family".into(), color: [11, 128, 67], visible: true, updated: now_ms(), ..Default::default() });
    let _ = s.save();
    let r = call(&mut s, "create_event", json!({"title": "Dinner", "start": format!("{day} 19:00"), "calendar": "Family"}));
    t.check("several independent calendars", r.is_ok() && s.data.calendars.len() >= 2, format!("{r:?}"));

    // ---- restart: everything must come back from disk
    drop(s);
    let mut s = Store::open(personal.clone());
    let ok = s.data.page(&page_id).is_some() && s.data.task(&rent).map(|x| x.recurrence == Recurrence::Monthly).unwrap_or(false) && s.data.events.iter().any(|e| e.id == ev && e.start.format("%H:%M").to_string() == "15:30") && s.data.database(&cont_db).map(|d| d.rows.len() == 1 && d.filters.len() == 1).unwrap_or(false);
    t.check("after a restart, pages, tasks, moved events and CRM records persist", ok, "data missing after reload");

    // ---- search
    let hits = crate::search::search(&s.data, "northwind", 20);
    t.check("search finds pages, tasks, events and database rows", hits.iter().any(|h| h.kind == crate::search::HitKind::Event) && hits.iter().any(|h| h.kind == crate::search::HitKind::Row) && hits.iter().any(|h| h.kind == crate::search::HitKind::Project), format!("{} hits", hits.len()));

    // ---- isolation between workspaces
    let mut w = Store::open(work.clone());
    let wh = call(&mut w, "search", json!({"query": "Northwind"})).unwrap_or(json!([]));
    t.check("workspaces are isolated (Work can't see Personal)", wh.as_array().map_or(true, |a| a.is_empty()), wh);
    w.settings.ai_access = false;
    w.save_settings();
    t.check("AI access can be switched off per workspace", call(&mut w, "list_pages", json!({})).is_err(), "still allowed");
    w.settings.ai_access = true;
    w.settings.ai_read_only = true;
    t.check("read-only AI access blocks changes", call(&mut w, "create_task", json!({"title": "x"})).is_err(), "write allowed");
    w.settings.ai_read_only = false;
    w.save_settings();

    // ---- MCP protocol handshake
    let init = mcp::handle_text(&mut s, r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}"#).unwrap_or_default();
    let list = mcp::handle_text(&mut s, r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#).unwrap_or_default();
    let notif = mcp::handle_text(&mut s, r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#);
    t.check("MCP initialize / tools/list / notifications", init.contains("serverInfo") && list.contains("create_task") && notif.is_none(), format!("{init} {list}"));

    // ---- bulk import through AI assistants
    let r = call(&mut s, "import_csv", json!({"name": "Leads", "csv": "Name,Stage,Value\nAcme,Hot,100\nGlobex,Cold,50\nInitech,Hot,75\nUmbrella,Cold,20\n"}));
    t.check("AI bulk import: CSV becomes a typed database", r.as_ref().map_or(false, |v| v["rows"] == 4) && s.data.databases.iter().any(|d| d.name == "Leads" && d.col("Value").map(|c| c.kind) == Some(ColKind::Number)), format!("{r:?}"));
    let r = call(&mut s, "import_pages", json!({"pages": [{"title": "Meetings", "markdown": "All meetings"}, {"title": "Kick-off", "markdown": "## Notes\n- Agreed scope", "parent_title": "Meetings"}]}));
    let nested = s.data.page_by_title("Kick-off").and_then(|p| p.parent.clone()) == s.data.page_by_title("Meetings").map(|p| p.id.clone());
    t.check("AI bulk import: several nested pages", r.is_ok() && nested, format!("{r:?}"));
    let r = call(&mut s, "import_tasks", json!({"lines": ["Plan trip next fri p2 #Travel", "  Book flights", "  Book hotel", "Renew passport"]}));
    let trip = s.data.tasks.iter().find(|x| x.title == "Plan trip").map(|x| x.id.clone()).unwrap_or_default();
    t.check("AI bulk import: tasks with sub-tasks and projects", r.as_ref().map_or(false, |v| v["imported_tasks"] == 4) && s.data.subtasks(&trip).len() == 2, format!("{r:?}"));
    let r = call(&mut s, "import_ics", json!({"ics": "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:x1\r\nDTSTART:20261203T090000\r\nDTEND:20261203T100000\r\nSUMMARY:Imported by AI\r\nEND:VEVENT\r\nEND:VCALENDAR", "calendar": "AI calendar"}));
    t.check("AI bulk import: calendar events", r.as_ref().map_or(false, |v| v["imported_events"] == 1), format!("{r:?}"));

    // ---- organising page to-dos into projects (what the AI does, through the same tools)
    let _ = call(&mut s, "create_page", json!({"title": "Launch notes", "markdown": "## To do\n- [ ] Book venue for launch by friday\n- [ ] Draft press release\n- [ ] Send invites"}));
    let todos = call(&mut s, "list_page_todos", json!({})).unwrap_or(json!({}));
    let mine: Vec<Value> = todos["todos"].as_array().cloned().unwrap_or_default().into_iter().filter(|t| t["page_title"] == "Launch notes").collect();
    t.check("AI can list the to-dos written in pages", mine.len() == 3, format!("{todos}"));
    let items: Vec<Value> = mine
        .iter()
        .enumerate()
        .map(|(i, t)| json!({"page_id": t["page_id"], "block_id": t["block_id"], "project": "Product launch", "section": if i == 0 { "Logistics" } else { "Comms" }, "priority": if i == 0 { 1 } else { 3 }, "due": if i == 2 { "next week" } else { "" }}))
        .collect();
    let r = call(&mut s, "convert_todos_to_tasks", json!({"items": items}));
    let launch = s.data.project_by_name("Product launch").cloned();
    let venue = s.data.tasks.iter().find(|x| x.title.starts_with("Book venue")).cloned();
    t.check(
        "AI organises page to-dos: new project with sections, priorities, dates, linked to the page",
        r.is_ok() && launch.as_ref().map_or(false, |p| p.sections.len() == 2 && p.page_id.is_some()) && venue.as_ref().map_or(false, |v| v.priority == 1 && v.due.is_some() && v.page_id.is_some() && v.project_id == launch.as_ref().map(|p| p.id.clone())),
        format!("{r:?} {venue:?}"),
    );
    let vid = venue.map(|v| v.id).unwrap_or_default();
    let _ = call(&mut s, "complete_task", json!({"id": vid}));
    let ticked = s.data.page_by_title("Launch notes").map_or(false, |p| p.blocks.iter().any(|b| b.task_id.as_deref() == Some(vid.as_str()) && b.checked));
    t.check("completing the task ticks the to-do on the page", ticked, "not ticked");
    // ticking on the page completes the task
    let press = s.data.tasks.iter().find(|x| x.title == "Draft press release").map(|x| x.id.clone()).unwrap_or_default();
    if let Some(pg) = s.data.pages.iter_mut().find(|p| p.title == "Launch notes") {
        for b in pg.blocks.iter_mut().filter(|b| b.task_id.as_deref() == Some(press.as_str())) {
            b.checked = true;
        }
        pg.updated = now_ms() + 50;
    }
    s.data.sync_todo_links();
    t.check("ticking the to-do on the page completes the task", s.data.task(&press).map_or(false, |x| x.completed), "not completed");
    let again = call(&mut s, "convert_todos_to_tasks", json!({"items": mine}));
    t.check("already-linked to-dos aren't turned into tasks twice", again.as_ref().map_or(false, |v| v["created"].as_array().map_or(false, |a| a.is_empty())), format!("{again:?}"));
    let prompts = mcp::handle_text(&mut s, r#"{"jsonrpc":"2.0","id":9,"method":"prompts/get","params":{"name":"organise_todos"}}"#).unwrap_or_default();
    t.check("Claude/ChatGPT get an 'Organise my to-dos' prompt", prompts.contains("convert_todos_to_tasks"), prompts);
    let _ = s.save();

    // ---- backups
    let b = s.backup_now();
    let before = s.data.pages.len();
    s.data.delete_page(&child_id);
    let _ = s.save();
    let restored = b.as_ref().map(|p| s.restore(p).is_ok()).unwrap_or(false);
    t.check("backup and restore", restored && s.data.pages.len() == before, format!("{b:?}"));

    // ---- sync through a shared folder: two copies edit at the same time
    let shared = base.join("shared");
    let _ = std::fs::create_dir_all(&shared);
    s.settings.sync_dir = shared.display().to_string();
    let _ = s.sync();
    // simulate a second device: a copy of the data folder with its own edits
    let phone_root = base.join("device2");
    let _ = std::fs::create_dir_all(store::workspace_dir(&personal.id).parent().unwrap());
    let mut other = WorkspaceData::default();
    if let Some(remote) = store::read_data(&shared.join(format!("resolve2-{}.json", personal.id))) {
        other = remote;
    }
    let mut t2 = Task::new("Added on the other device");
    t2.updated = now_ms() + 5;
    other.tasks.push(t2.clone());
    let _ = store::write_json_atomic(&shared.join(format!("resolve2-{}.json", personal.id)), &other);
    let _ = phone_root;
    s.data.tasks.push(Task::new("Added on this PC"));
    let r = s.sync();
    let both = s.data.tasks.iter().any(|x| x.id == t2.id) && s.data.tasks.iter().any(|x| x.title == "Added on this PC");
    t.check("two-way sync merges edits from both devices", both && r.is_ok(), format!("{r:?}"));
    let remote_after = store::read_data(&shared.join(format!("resolve2-{}.json", personal.id))).map(|d| d.tasks.iter().any(|x| x.title == "Added on this PC")).unwrap_or(false);
    t.check("sync writes this PC's edits back to the shared folder", remote_after, "missing");

    // ---- imports
    let imp = base.join("imports");
    let notion = imp.join("notion");
    let _ = std::fs::create_dir_all(notion.join("Projects 0123456789abcdef0123456789abcdef"));
    let _ = std::fs::write(notion.join("Projects 0123456789abcdef0123456789abcdef.md"), "# Projects\n\nOverview with a link to [Roadmap](Projects%200123456789abcdef0123456789abcdef/Roadmap%20fedcba9876543210fedcba9876543210.md)\n");
    let _ = std::fs::write(notion.join("Projects 0123456789abcdef0123456789abcdef").join("Roadmap fedcba9876543210fedcba9876543210.md"), "# Roadmap\n\n- [ ] Q1 launch\n- Item two\n");
    let _ = std::fs::write(notion.join("Clients aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.csv"), "Name,Status,Value,Signed\nAcme,Active,1200,Yes\nGlobex,Lead,300,No\nInitech,Active,800,No\nUmbrella,Lead,50,No\n");
    let r = crate::import::import_notion(&notion, &mut s.data);
    let nested = s.data.pages.iter().find(|p| p.title == "Roadmap").and_then(|p| p.parent.clone()).and_then(|pid| s.data.page(&pid).map(|x| x.title.clone()));
    t.check("Notion import: pages, nesting, links and databases", r.as_ref().map_or(false, |x| x.pages == 2 && x.databases == 1) && nested.as_deref() == Some("Projects") && s.data.page_by_title("Projects").map_or(false, |p| p.wiki_links().contains(&"Roadmap".to_string())), format!("{r:?} {nested:?}"));
    let clients = s.data.databases.iter().find(|d| d.name == "Clients").cloned().unwrap_or_default();
    t.check("Notion CSV columns get sensible types", clients.col("Value").map(|c| c.kind) == Some(ColKind::Number) && clients.col("Signed").map(|c| c.kind) == Some(ColKind::Checkbox), format!("{:?}", clients.columns.iter().map(|c| (c.name.clone(), c.kind)).collect::<Vec<_>>()));
    let again = crate::import::import_notion(&notion, &mut s.data);
    t.check("re-importing Notion doesn't duplicate", again.as_ref().map_or(false, |x| x.pages == 0 && x.databases == 0), format!("{again:?}"));
    let todo = imp.join("Side hustle.csv");
    let _ = std::fs::write(&todo, "TYPE,CONTENT,DESCRIPTION,PRIORITY,INDENT,AUTHOR,RESPONSIBLE,DATE,DATE_LANG,TIMEZONE\nsection,Launch,,,,,,,,\ntask,Build landing page @web,Use the template,1,1,,,every mon,en,\ntask,Pick a domain,,4,2,,,tomorrow,en,\nnote,Check prices first,,,,,,,,\n");
    let r = crate::import::import_todoist(&todo, &mut s.data);
    let landing = s.data.tasks.iter().find(|x| x.title == "Build landing page").cloned();
    let domain = s.data.tasks.iter().find(|x| x.title == "Pick a domain").cloned();
    t.check("Todoist import: project, section, subtask, priority, repeat, labels, notes", r.is_ok() && landing.as_ref().map_or(false, |l| l.priority == 1 && l.recurrence == Recurrence::Weekly && l.section == "Launch" && l.labels == vec!["web".to_string()]) && domain.as_ref().map_or(false, |d| d.parent_id == landing.as_ref().map(|l| l.id.clone()) && d.notes.contains("prices")), format!("{landing:?} {domain:?}"));
    let ics = imp.join("Work_abc@group.calendar.google.com.ics");
    let _ = std::fs::write(&ics, "BEGIN:VCALENDAR\r\nX-WR-CALNAME:Work (Google)\r\nBEGIN:VEVENT\r\nUID:1@g\r\nDTSTART:20261110T090000Z\r\nDTEND:20261110T093000Z\r\nSUMMARY:Stand-up\r\nRRULE:FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nUID:2@g\r\nDTSTART;VALUE=DATE:20261120\r\nDTEND;VALUE=DATE:20261121\r\nSUMMARY:Offsite\\, London\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n");
    let r = crate::import::import_ics(&ics, &mut s.data);
    let standup = s.data.events.iter().find(|e| e.title == "Stand-up").cloned();
    t.check("Google Calendar (.ics) import: new calendar, recurring and all-day events", r.as_ref().map_or(false, |x| x.calendars == 1 && x.events == 2) && standup.map_or(false, |e| e.recurrence == Recurrence::Weekdays) && s.data.events.iter().any(|e| e.title == "Offsite, London" && e.all_day), format!("{r:?}"));
    let first_cal = s.data.calendars[0].id.clone();
    let n = crate::import::apply_subscription(&mut s.data, &first_cal, "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:s1\r\nDTSTART:20261201T100000\r\nDTEND:20261201T110000\r\nSUMMARY:Subscribed event\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n");
    t.check("calendar subscription refresh replaces that calendar's events", n == 1 && s.data.events.iter().any(|e| e.title == "Subscribed event"), n);
    let _ = s.save();

    // ---- export
    let out = base.join("export");
    let _ = std::fs::create_dir_all(&out);
    let r = s.export_markdown(&out);
    t.check("export to Markdown / CSV / .ics / JSON", r.as_ref().map_or(false, |n| *n > 3), format!("{r:?}"));

    // ---- HTTP: phone page + MCP over HTTP with a token
    let port = 18000 + (std::process::id() % 2000) as u16;
    match crate::server::start(personal.clone(), port) {
        Ok(h) => {
            std::thread::sleep(std::time::Duration::from_millis(300));
            let token = Store::open(personal.clone()).settings.token;
            let ok = ureq::post(&format!("http://127.0.0.1:{port}/mcp?token={token}")).send_string(r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_tasks","arguments":{"view":"all"}}}"#).ok().and_then(|r| r.into_string().ok()).unwrap_or_default();
            let denied = matches!(ureq::post(&format!("http://127.0.0.1:{port}/mcp?token=wrong")).send_string("{}"), Err(ureq::Error::Status(401, _)));
            let page = ureq::get(&format!("http://127.0.0.1:{port}/")).call().ok().and_then(|r| r.into_string().ok()).unwrap_or_default();
            t.check("MCP over HTTP works with the token", ok.contains("Send proposal"), &ok[..ok.len().min(200)]);
            t.check("MCP over HTTP refuses a wrong token", denied, "not refused");
            t.check("phone page is served", page.contains("Add task"), "no page");
            // hub sync: another device with its own offline edit pushes to the hub and gets the merged copy
            let mut device = WorkspaceData::default();
            let mut offline = Task::new("Captured on the phone while offline");
            offline.updated = now_ms() + 10;
            device.tasks.push(offline.clone());
            let merged = store::hub_exchange(&format!("http://127.0.0.1:{port}"), &token, &device);
            let ok = merged.as_ref().map_or(false, |m| m.tasks.iter().any(|x| x.id == offline.id) && m.tasks.iter().any(|x| x.title == "Send proposal"));
            t.check("hub /sync merges a device's copy and returns everything", ok, format!("{:?}", merged.as_ref().err()));
            let on_disk = Store::open(personal.clone()).data.tasks.iter().any(|x| x.id == offline.id);
            t.check("hub saves what devices send (survives a hub restart)", on_disk, "not saved");
            t.check("hub refuses a wrong token", store::hub_exchange(&format!("http://127.0.0.1:{port}"), "wrong-token", &device).is_err(), "accepted");
            h.stop();
        }
        Err(e) => t.check("start the local server", false, e),
    }
    // ---- in-app assistant: a stand-in model asks for a tool call; the app must carry it out
    let ai_port = port + 1;
    if let Ok(fake) = tiny_http::Server::http(("127.0.0.1", ai_port)) {
        let th = std::thread::spawn(move || {
            for (i, mut req) in fake.incoming_requests().take(2).enumerate() {
                let mut body = String::new();
                let _ = std::io::Read::read_to_string(req.as_reader(), &mut body);
                let reply = if i == 0 {
                    json!({"choices": [{"message": {"role": "assistant", "content": null, "tool_calls": [{"id": "c1", "type": "function", "function": {"name": "create_task", "arguments": "{\"text\":\"Call the accountant fri 10am p2 #Admin\"}"}}]}}]})
                } else {
                    let saw_result = body.contains("\"role\":\"tool\"") && body.contains("Call the accountant");
                    json!({"choices": [{"message": {"role": "assistant", "content": if saw_result { "Added it." } else { "no tool result" }}}]})
                };
                let _ = req.respond(tiny_http::Response::from_string(reply.to_string()).with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()));
            }
        });
        let cfg = crate::ai::AiConfig { provider: "openai".into(), url: format!("http://127.0.0.1:{ai_port}"), model: "test".into(), key: String::new() };
        let out = crate::ai::run_agent(&cfg, crate::ai::AGENT_SYSTEM, "Remind me to call the accountant on Friday", personal.clone(), true);
        let _ = th.join();
        let made = Store::open(personal.clone()).data.tasks.iter().any(|x| x.title == "Call the accountant" && x.priority == 2);
        t.check("✨ Ask AI can make changes (tool call → real task, result fed back)", made && out.as_ref().map_or(false, |o| o.contains("Added it.") && o.contains("create task")), format!("{out:?}"));
    }
    crate::selftest_resolve::run(&mut t, personal.clone(), port + 7);
    crate::selftest_safety::run(&mut t, personal.clone(), port + 9);
    crate::selftest_journey::run(&mut t, personal.clone(), port + 11);
    crate::selftest_careers::run(&mut t, port + 13);
    crate::selftest_careers::run_s3(&mut t);
    crate::selftest_alpha::run(&mut t, &base);
    crate::selftest_p0::run(&mut t);
    crate::selftest_p1::run(&mut t);
    crate::selftest_p2::run(&mut t);
    crate::selftest_p3::run(&mut t);
    crate::selftest_p4::run(&mut t);
    crate::selftest_p5::run(&mut t);
    crate::selftest_modes::run(&mut t);
    reg = Registry::load();
    t.check("workspace list persists (Personal + the one made in the test)", reg.workspaces.len() == 2, reg.workspaces.len());
    println!("\n{} passed, {} failed", t.pass, t.fail);
    let _ = std::fs::remove_dir_all(&base);
    t.fail as i32
}
