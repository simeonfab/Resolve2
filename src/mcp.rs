//! Model Context Protocol server: lets Claude, ChatGPT, Codex (or any MCP client, including
//! local models) read, search, create and update the workspace.
//!   stdio:  resolve2.exe --mcp [--workspace NAME]
//!   HTTP:   POST http://<pc>:<port>/mcp?token=...   (started from Settings or with --serve)

use crate::model::*;
use crate::store::Store;
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use serde_json::{json, Map, Value};
use std::io::{BufRead, Write};

pub const PROTOCOL: &str = "2025-06-18";

fn tool(name: &str, desc: &str, props: Value, required: &[&str]) -> Value {
    json!({
        "name": name,
        "description": desc,
        "inputSchema": { "type": "object", "properties": props, "required": required }
    })
}

const WRITE_TOOLS: [&str; 22] = [
    "add_record",
    "update_record",
    "update_project",
    "create_calendar",
    "add_database_column",
    "convert_todos_to_tasks",
    "import_pages",
    "import_tasks",
    "import_csv",
    "import_ics",
    "create_page",
    "update_page",
    "create_task",
    "update_task",
    "complete_task",
    "create_project",
    "create_event",
    "update_event",
    "add_database_row",
    "update_database_row",
    "delete_item",
    "create_database",
];

pub fn tools(read_only: bool) -> Value {
    let s = |d: &str| json!({"type": "string", "description": d});
    let n = |d: &str| json!({"type": "number", "description": d});
    let b = |d: &str| json!({"type": "boolean", "description": d});
    let mut list = vec![
        tool("workspace_overview", "Summary of the workspace: counts, projects, calendars, databases, favourite pages and what's due today. Call this first.", json!({}), &[]),
        tool("search", "Full-text search across pages, tasks, events, projects and database rows. Prefix with page:, task:, event: or db: to narrow.", json!({"query": s("Words to look for"), "limit": n("Max results, default 20")}), &["query"]),
        tool("get_agenda", "Tasks and events day by day (overdue tasks are listed under today).", json!({"date": s("Start date YYYY-MM-DD, default today"), "days": n("Number of days, default 7")}), &[]),
        tool("list_pages", "All pages with id, title, parent and last edit time.", json!({}), &[]),
        tool("read_page", "A page as Markdown, plus its sub-pages, backlinks and linked tasks.", json!({"id": s("Page id"), "title": s("Or the exact page title")}), &[]),
        tool("list_tasks", "Tasks filtered by view. view: today | overdue | upcoming | inbox | all | completed. Optional project name and text query.", json!({"view": s("today, overdue, upcoming, inbox, all or completed (default all open)"), "project": s("Project name"), "query": s("Text filter"), "limit": n("Max results, default 100")}), &[]),
        tool("list_projects", "Projects with sections, open task counts and linked document.", json!({}), &[]),
        tool("list_events", "Calendar events (recurring events expanded) between two dates.", json!({"from": s("YYYY-MM-DD, default today"), "to": s("YYYY-MM-DD inclusive, default from + 7 days"), "calendar": s("Calendar name filter")}), &[]),
        tool("list_calendars", "Calendars with their colours and visibility.", json!({}), &[]),
        tool("list_databases", "Databases (tables / boards) with their columns.", json!({}), &[]),
        tool("list_page_todos", "Every unticked to-do written inside pages (page, block id, text, and whether it is already linked to a task). Use with convert_todos_to_tasks.", json!({}), &[]),
        tool("resolve_projects", "Resolve projects (projects with an intake/ResolveOS context): outcome, whether it is confirmed, progress and the next move.", json!({}), &[]),
        tool("resolve_project", "Everything Resolve knows about one project: intake understanding, outcome, recommendation, plan, the deterministic 'what matters now' assessment, and all its records (decisions, hypotheses, experiments, evidence, risks…).", json!({"project": s("Project id or name")}), &["project"]),
        tool("list_records", "Project records. kind: hypothesis, experiment, evidence, decision, risk, assumption, requirement, dependency, question, blocker, milestone, update, guidance.", json!({"project": s("Project id or name"), "kind": s("Optional kind"), "open_only": b("Only records still to act on")}), &[]),
        tool("query_database", "Rows of a database as objects keyed by column name.", json!({"database": s("Database id or name"), "query": s("Optional text filter"), "limit": n("Max rows, default 200")}), &["database"]),
    ];
    if !read_only {
        list.extend([
            tool("add_record", "Add a Resolve record to a project. Hypotheses can nest (parent_id = another hypothesis); experiments hang under the hypothesis they test. Fields by kind — hypothesis: negation, alternatives, type, confidence, uncertainty, decision, revisit; experiment: decision, prediction, test_type, method, baseline, support_if, refute_if, inconclusive_if, limits, owner, date, sources, observations, limitations, result, update, next; evidence: nature (fact/assumption/inference/evidence), direction (supports/contradicts/neutral), quality, source, date; decision: options, rationale, uncertainty, revisit, date, owner.", json!({"project": s("Project id or name"), "kind": s("Record kind"), "title": s("Title / claim"), "body": s("Notes"), "status": s("Optional status valid for the kind"), "parent_id": s("Parent record id"), "fields": {"type": "object"}, "links": {"type": "array", "items": {"type": "string"}, "description": "Linked task/page/record ids"}}), &["project", "kind", "title"]),
            tool("update_record", "Change a record's title, body, status, fields or links. An experiment's prediction and thresholds are fixed once it is Running (Hypothesis Zero: no retroactive tuning).", json!({"id": s("Record id"), "title": s(""), "body": s(""), "status": s(""), "fields": {"type": "object"}, "links": {"type": "array", "items": {"type": "string"}}}), &["id"]),
            tool("create_page", "Create a page from Markdown (headings, lists, - [ ] to-dos, > quotes, ``` code, [[Page links]]).", json!({"title": s("Page title"), "markdown": s("Body as Markdown"), "parent_id": s("Optional parent page id"), "icon": s("Optional emoji")}), &["title"]),
            tool("update_page", "Change a page: title, body (replace or append), move it under another page, icon, favourite.", json!({"id": s("Page id"), "title": s("New title"), "markdown": s("Replace the whole body with this Markdown"), "append_markdown": s("Markdown to add at the end"), "parent_id": s("Move under this page id (empty = top level)"), "icon": s("Emoji"), "favorite": b("Pin to Favorites")}), &["id"]),
            tool("create_task", "Create a task. `text` accepts quick-capture syntax like 'Pay rent every month 9am p1 #Home @bills remind 30m'. Other fields override it.", json!({
                "text": s("Quick-capture text"), "title": s("Plain title"), "notes": s("Description"), "project": s("Project name (created if missing)"), "section": s("Section within the project"),
                "priority": n("1 urgent .. 4 none"), "due": s("YYYY-MM-DD or words like 'tomorrow', 'fri'"), "time": s("HH:MM"), "recurrence": s("e.g. 'every day', 'every weekday', 'every 2 weeks', 'every month'"),
                "labels": {"type": "array", "items": {"type": "string"}}, "parent_id": s("Make it a subtask of this task id"), "page_id": s("Link to this page id"), "page_title": s("Or link to the page with this title"),
                "reminder_minutes": n("Remind this many minutes before the due time"), "duration_minutes": n("How long it takes (shows as a calendar block)")
            }), &[]),
            tool("update_task", "Change any task field; set completed true/false. Empty string clears due/time/project/page.", json!({
                "id": s("Task id"), "title": s(""), "notes": s(""), "project": s("Project name, '' for Inbox"), "section": s(""), "priority": n("1..4"), "due": s("YYYY-MM-DD or words, '' clears"),
                "time": s("HH:MM, '' clears"), "recurrence": s("'none' to stop repeating"), "labels": {"type": "array", "items": {"type": "string"}}, "page_id": s("'' unlinks"), "completed": b("Done or not"),
                "reminder_minutes": n("-1 clears"), "duration_minutes": n("")
            }), &["id"]),
            tool("complete_task", "Complete a task. Recurring tasks move to their next date.", json!({"id": s("Task id")}), &["id"]),
            tool("create_project", "Create a project.", json!({"name": s("Name"), "sections": {"type": "array", "items": {"type": "string"}}}), &["name"]),
            tool("create_event", "Create a calendar event.", json!({
                "title": s("Title"), "start": s("YYYY-MM-DD HH:MM (or YYYY-MM-DD for all-day)"), "end": s("YYYY-MM-DD HH:MM"), "duration_minutes": n("Used when end is missing, default 60"),
                "all_day": b(""), "calendar": s("Calendar name (default first)"), "location": s(""), "notes": s(""), "recurrence": s("e.g. 'every week'"), "page_id": s("Linked page id"), "reminder_minutes": n("")
            }), &["title", "start"]),
            tool("update_event", "Change an event (any field from create_event).", json!({"id": s("Event id"), "title": s(""), "start": s(""), "end": s(""), "all_day": b(""), "calendar": s(""), "location": s(""), "notes": s(""), "recurrence": s("")}), &["id"]),
            tool("create_database", "Create a database with columns. Column types: text, number, select, checkbox, date, url.", json!({"name": s("Name"), "columns": {"type": "array", "items": {"type": "object", "properties": {"name": {"type": "string"}, "type": {"type": "string"}, "options": {"type": "array", "items": {"type": "string"}}}}}}), &["name", "columns"]),
            tool("add_database_row", "Add a row. values maps column names to values (checkbox: true/false, date: YYYY-MM-DD).", json!({"database": s("Database id or name"), "values": {"type": "object"}}), &["database", "values"]),
            tool("update_database_row", "Change cells of a row.", json!({"database": s("Database id or name"), "row_id": s("Row id"), "values": {"type": "object"}}), &["database", "row_id", "values"]),
            tool("delete_item", "Delete a page, task, event, project or database by id.", json!({"id": s("Item id")}), &["id"]),
            tool("import_pages", "Bulk import: many pages at once, each from Markdown, optionally nested under a parent (by title — earlier pages in the same call count). Use this to move notes or a whole Notion/Docs structure in.", json!({"pages": {"type": "array", "items": {"type": "object", "properties": {"title": {"type": "string"}, "markdown": {"type": "string"}, "parent_title": {"type": "string"}, "icon": {"type": "string"}}, "required": ["title"]}}}), &["pages"]),
            tool("import_tasks", "Bulk import: many tasks at once, one quick-capture line each (e.g. 'Call bank tomorrow 10am p2 #Admin every month'). Indented lines (two spaces or a tab) under a task become its subtasks.", json!({"lines": {"type": "array", "items": {"type": "string"}}, "project": s("Default project for lines without #project")}), &["lines"]),
            tool("import_csv", "Bulk import: a CSV table (first row = column names) becomes a new database with sensible column types — e.g. a CRM contact list or spreadsheet.", json!({"name": s("Database name"), "csv": s("CSV text")}), &["name", "csv"]),
            tool("import_ics", "Bulk import: iCalendar (.ics) text into a calendar (created if missing).", json!({"ics": s(".ics text"), "calendar": s("Calendar name")}), &["ics"]),
            tool("convert_todos_to_tasks", "Turn to-dos written in pages (see list_page_todos) into real tasks, in one call. Each item names its page_id and block_id plus where the task goes. Projects and sections are created if missing. The page to-do stays and is linked to the task: ticking either ticks both.", json!({
                "items": {"type": "array", "items": {"type": "object", "properties": {
                    "page_id": {"type": "string"}, "block_id": {"type": "string"}, "title": {"type": "string", "description": "Clean task title (default: the to-do text)"},
                    "project": {"type": "string", "description": "Project name; created if missing; 'Inbox' for none"}, "section": {"type": "string"},
                    "due": {"type": "string", "description": "YYYY-MM-DD or words like 'fri'"}, "time": {"type": "string", "description": "HH:MM"},
                    "priority": {"type": "number", "description": "1 urgent .. 4 none"}, "recurrence": {"type": "string"}, "labels": {"type": "array", "items": {"type": "string"}},
                    "notes": {"type": "string"}, "duration_minutes": {"type": "number"}, "parent_block_id": {"type": "string", "description": "Make it a subtask of the task made from this other to-do block"}
                }, "required": ["page_id", "block_id"]}},
                "link_projects_to_pages": b("When a new project is created from a page, make that page the project's document (default true)")
            }), &["items"]),
            tool("update_project", "Rename, recolour, add sections to, archive or attach a document to a project.", json!({"id": s("Project id or name"), "name": s(""), "color": s("#rrggbb"), "sections": {"type": "array", "items": {"type": "string"}, "description": "Sections to add"}, "archived": b(""), "page_id": s("Project document page id")}), &["id"]),
            tool("create_calendar", "Create a calendar (own name and colour).", json!({"name": s("Name"), "color": s("#rrggbb")}), &["name"]),
            tool("add_database_column", "Add a property (column) to a database. Types: text, number, select, checkbox, date, url, email, phone, page, relation (target: 'projects', 'tasks', 'pages' or another database's name).", json!({"database": s("Database id or name"), "name": s("Column name"), "type": s("Type"), "options": {"type": "array", "items": {"type": "string"}}, "target": s("For relations")}), &["database", "name"]),
        ]);
    }
    Value::Array(list)
}

// ---------------------------------------------------------------- helpers

fn arg_str(a: &Value, k: &str) -> Option<String> {
    match a.get(k)? {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}
fn arg_i64(a: &Value, k: &str) -> Option<i64> {
    match a.get(k)? {
        Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}
fn arg_bool(a: &Value, k: &str) -> Option<bool> {
    match a.get(k)? {
        Value::Bool(b) => Some(*b),
        Value::String(s) => Some(matches!(s.as_str(), "true" | "yes" | "1")),
        _ => None,
    }
}
fn arg_list(a: &Value, k: &str) -> Option<Vec<String>> {
    match a.get(k)? {
        Value::Array(v) => Some(v.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()),
        Value::String(s) => Some(s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()),
        _ => None,
    }
}

fn parse_day(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok().or_else(|| crate::nlp::parse(s, today()).due)
}

pub fn parse_datetime(s: &str) -> Option<(NaiveDateTime, bool)> {
    let s = s.trim().replace('T', " ");
    let s = s.trim_end_matches('Z');
    for f in ["%Y-%m-%d %H:%M:%S", "%Y-%m-%d %H:%M"] {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, f) {
            return Some((dt, false));
        }
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some((d.and_hms_opt(0, 0, 0)?, true));
    }
    let c = crate::nlp::parse(s, today());
    match (c.due, c.time) {
        (Some(d), Some(t)) => Some((d.and_time(t), false)),
        (Some(d), None) => Some((d.and_hms_opt(0, 0, 0)?, true)),
        _ => None,
    }
}

fn parse_recurrence(s: &str) -> Recurrence {
    let l = s.trim().to_lowercase();
    if l.is_empty() || l == "none" || l == "never" || l == "does not repeat" {
        return Recurrence::None;
    }
    let c = crate::nlp::parse(&l, today());
    c.recurrence
}

pub fn task_json(d: &WorkspaceData, t: &Task) -> Value {
    let subs = d.subtasks(&t.id);
    json!({
        "id": t.id,
        "title": t.title,
        "notes": t.notes,
        "project": d.project_name(&t.project_id),
        "section": t.section,
        "parent_id": t.parent_id,
        "priority": format!("p{}", t.priority),
        "due": t.due.map(|x| x.to_string()),
        "time": t.due_time.map(fmt_time),
        "recurrence": if t.recurrence.is_some() { Some(t.recurrence.label()) } else { None },
        "labels": t.labels,
        "page_id": t.page_id,
        "page_title": t.page_id.as_deref().and_then(|p| d.page(p)).map(|p| p.title.clone()),
        "completed": t.completed,
        "overdue": t.is_overdue(),
        "subtasks": subs.iter().map(|s| json!({"id": s.id, "title": s.title, "completed": s.completed})).collect::<Vec<_>>(),
        "reminder_minutes": t.reminder_min,
        "duration_minutes": t.duration_min,
    })
}

fn event_json(d: &WorkspaceData, e: &Event, start: NaiveDateTime) -> Value {
    json!({
        "id": e.id,
        "title": e.title,
        "calendar": d.calendar(&e.calendar_id).map(|c| c.name.clone()),
        "start": start.format("%Y-%m-%d %H:%M").to_string(),
        "end": (start + e.duration()).format("%Y-%m-%d %H:%M").to_string(),
        "all_day": e.all_day,
        "location": e.location,
        "notes": e.notes,
        "recurrence": if e.recurrence.is_some() { Some(e.recurrence.label()) } else { None },
        "page_id": e.page_id,
    })
}

fn parse_hex(s: &str) -> Option<[u8; 3]> {
    let h = s.trim().trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    let v = u32::from_str_radix(h, 16).ok()?;
    Some([(v >> 16) as u8, (v >> 8) as u8, v as u8])
}

fn find_db<'a>(d: &'a WorkspaceData, key: &str) -> Option<&'a Database> {
    d.databases.iter().find(|x| x.id == key).or_else(|| d.databases.iter().find(|x| x.name.eq_ignore_ascii_case(key.trim())))
}

fn row_json(db: &Database, r: &Row) -> Value {
    let mut m = Map::new();
    m.insert("row_id".into(), json!(r.id));
    for c in &db.columns {
        let v = r.cells.get(&c.id).cloned().unwrap_or_default();
        let val = match c.kind {
            ColKind::Checkbox => json!(v == "true"),
            ColKind::Number => v.parse::<f64>().map(|f| json!(f)).unwrap_or(json!(v)),
            _ => json!(v),
        };
        m.insert(c.name.clone(), val);
    }
    if let Some(p) = &r.page_id {
        m.insert("page_id".into(), json!(p));
    }
    Value::Object(m)
}

fn set_cells(db: &mut Database, row: &mut Row, values: &Value) -> Result<(), String> {
    let Some(obj) = values.as_object() else { return Err("values must be an object of column name → value".into()) };
    for (k, v) in obj {
        let Some(col) = db.col(k).cloned() else {
            return Err(format!("No column named '{k}'. Columns: {}", db.columns.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", ")));
        };
        let text = match v {
            Value::String(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::Null => String::new(),
            other => other.to_string(),
        };
        let text = match col.kind {
            ColKind::Checkbox => (matches!(text.to_lowercase().as_str(), "true" | "yes" | "1" | "x")).to_string(),
            ColKind::Date => parse_day(&text).map(|d| d.to_string()).unwrap_or(text),
            ColKind::Select => {
                if !text.is_empty() && !col.options.contains(&text) {
                    if let Some(c) = db.columns.iter_mut().find(|c| c.id == col.id) {
                        c.options.push(text.clone());
                    }
                }
                text
            }
            _ => text,
        };
        row.cells.insert(col.id.clone(), text);
    }
    Ok(())
}

fn ensure_project(d: &mut WorkspaceData, name: &str) -> Option<Id> {
    let name = name.trim();
    if name.is_empty() || name.eq_ignore_ascii_case("inbox") {
        return None;
    }
    if let Some(p) = d.project_by_name(name) {
        return Some(p.id.clone());
    }
    let palette: [[u8; 3]; 5] = [[219, 76, 63], [41, 171, 135], [20, 170, 245], [175, 56, 235], [255, 153, 51]];
    let p = Project { id: new_id(), name: name.to_string(), color: palette[d.projects.len() % 5], sections: vec![], page_id: None, archived: false, favorite: false, order: d.projects.len() as i64 + 1, updated: now_ms() };
    let id = p.id.clone();
    d.projects.push(p);
    Some(id)
}

/// Apply fields shared by create_task and update_task.
fn apply_task_fields(d: &mut WorkspaceData, t: &mut Task, a: &Value) {
    if let Some(v) = arg_str(a, "title") {
        if !v.trim().is_empty() {
            t.title = v;
        }
    }
    if let Some(v) = arg_str(a, "notes") {
        t.notes = v;
    }
    if let Some(v) = arg_str(a, "project") {
        t.project_id = ensure_project(d, &v);
    }
    if let Some(v) = arg_str(a, "section") {
        t.section = v;
    }
    if let Some(p) = arg_i64(a, "priority") {
        t.priority = p.clamp(1, 4) as u8;
    } else if let Some(p) = arg_str(a, "priority") {
        if let Ok(n) = p.trim_start_matches(['p', 'P']).parse::<u8>() {
            t.priority = n.clamp(1, 4);
        }
    }
    if let Some(v) = arg_str(a, "due") {
        t.due = if v.trim().is_empty() { None } else { parse_day(&v) };
        if t.due.is_none() {
            t.due_time = None;
        }
    }
    if let Some(v) = arg_str(a, "time") {
        t.due_time = if v.trim().is_empty() { None } else { NaiveTime::parse_from_str(v.trim(), "%H:%M").ok().or_else(|| crate::nlp::parse_time(&v)) };
        if t.due_time.is_some() && t.due.is_none() {
            t.due = Some(today());
        }
    }
    if let Some(v) = arg_str(a, "recurrence") {
        t.recurrence = parse_recurrence(&v);
        if t.recurrence.is_some() && t.due.is_none() {
            t.due = Some(today());
        }
    }
    if let Some(v) = arg_list(a, "labels") {
        t.labels = v;
    }
    if let Some(v) = arg_str(a, "parent_id") {
        t.parent_id = if v.is_empty() { None } else { Some(v) };
        if let Some(pid) = &t.parent_id {
            if let Some(parent) = d.task(pid) {
                t.project_id = parent.project_id.clone();
                t.section = parent.section.clone();
            }
        }
    }
    if let Some(v) = arg_str(a, "page_id") {
        t.page_id = if v.is_empty() { None } else { Some(v) };
    }
    if let Some(v) = arg_str(a, "page_title") {
        t.page_id = d.page_by_title(&v).map(|p| p.id.clone());
    }
    if let Some(m) = arg_i64(a, "reminder_minutes") {
        t.reminder_min = if m < 0 { None } else { Some(m) };
    }
    if let Some(m) = arg_i64(a, "duration_minutes") {
        t.duration_min = if m <= 0 { None } else { Some(m as u32) };
    }
    t.updated = now_ms();
}

fn find_project_id(d: &WorkspaceData, key: &str) -> Result<Id, String> {
    d.project(key).or_else(|| d.project_by_name(key)).map(|p| p.id.clone()).ok_or_else(|| format!("No project called '{key}'"))
}

fn record_json(r: &crate::resolve::Record) -> Value {
    json!({"id": r.id, "kind": r.kind.key(), "title": r.title, "body": r.body, "status": r.status, "parent_id": r.parent_id, "fields": r.fields, "links": r.links, "project_id": r.project_id})
}

fn context_json(d: &WorkspaceData, c: &crate::resolve::ProjectContext) -> Value {
    let a = crate::resolve::assess::assess(d, c);
    json!({
        "project_id": c.project_id, "project": d.project(&c.project_id).map(|p| p.name.clone()), "situation": c.situation,
        "understanding": c.understanding, "outcome": c.outcome, "outcome_confirmed": c.outcome_confirmed,
        "gap_answers": c.gap_answers, "tools": c.tools, "recommendation": c.recommendation, "plan": c.plan, "notes": c.notes,
        "progress": {"done": a.done, "total": a.total},
        "next_move": a.win.as_ref().and_then(|w| d.task(w)).map(|t| json!({"id": t.id, "title": t.title})),
        "what_matters_now": a.next.iter().map(|(r, id)| json!({"reason": r, "id": id})).collect::<Vec<_>>(),
    })
}

fn set_record_fields(r: &mut crate::resolve::Record, a: &Value) -> Result<(), String> {
    if let Some(v) = arg_str(a, "title") { r.title = v; }
    if let Some(v) = arg_str(a, "body") { r.body = v; }
    // fields before status, so a planned experiment can be fully specified and started in one call
    if let Some(f) = a.get("fields").and_then(|x| x.as_object()) {
        for (k, v) in f {
            let v = v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string());
            r.set_field(k, &v)?;
        }
    }
    if let Some(s) = arg_str(a, "status") { r.set_status(&s)?; }
    if let Some(l) = arg_list(a, "links") { r.links = l; }
    r.updated = now_ms();
    Ok(())
}
// ---------------------------------------------------------------- the dispatcher

/// Run one tool against the workspace. Returns JSON text for the assistant.
pub fn call(store: &mut Store, name: &str, a: &Value) -> Result<String, String> {
    if !store.settings.ai_access {
        return Err(format!("AI access is turned off for the workspace '{}'. Turn it on in Resolve 2 → Settings → AI & MCP.", store.meta.name));
    }
    if store.settings.ai_read_only && WRITE_TOOLS.contains(&name) {
        return Err("This workspace is read-only for AI assistants. Change it in Resolve 2 → Settings → AI & MCP.".into());
    }
    store.reload_if_changed();
    let out = run(store, name, a)?;
    // a completed task ticks its page to-do (and the other way round)
    let synced = store.data.sync_todo_links();
    if WRITE_TOOLS.contains(&name) || synced {
        store.save()?;
    }
    Ok(serde_json::to_string_pretty(&out).unwrap_or_default())
}

fn run(store: &mut Store, name: &str, a: &Value) -> Result<Value, String> {
    let ws_name = store.meta.name.clone();
    let d = &mut store.data;
    match name {
        "workspace_overview" => {
            let open: Vec<&Task> = d.tasks.iter().filter(|t| !t.completed).collect();
            let due_today: Vec<Value> = open.iter().filter(|t| t.due.map_or(false, |x| x <= today())).map(|t| json!({"id": t.id, "title": t.title, "due": t.due, "priority": t.priority})).collect();
            Ok(json!({
                "workspace": ws_name,
                "today": today().to_string(),
                "counts": {"pages": d.pages.len(), "open_tasks": open.len(), "events": d.events.len(), "databases": d.databases.len(), "projects": d.projects.len()},
                "due_today_or_overdue": due_today,
                "projects": d.projects.iter().map(|p| json!({"id": p.id, "name": p.name, "sections": p.sections})).collect::<Vec<_>>(),
                "calendars": d.calendars.iter().map(|c| json!({"id": c.id, "name": c.name})).collect::<Vec<_>>(),
                "databases": d.databases.iter().map(|x| json!({"id": x.id, "name": x.name, "rows": x.rows.len()})).collect::<Vec<_>>(),
                "favourite_pages": d.pages.iter().filter(|p| p.favorite).map(|p| json!({"id": p.id, "title": p.title})).collect::<Vec<_>>(),
            }))
        }
        "search" => {
            let q = arg_str(a, "query").unwrap_or_default();
            let limit = arg_i64(a, "limit").unwrap_or(20).clamp(1, 100) as usize;
            let hits = crate::search::search(d, &q, limit);
            Ok(json!(hits.iter().map(|h| json!({"kind": h.kind.label(), "id": h.id, "row_id": h.row_id, "title": h.title, "snippet": h.snippet})).collect::<Vec<_>>()))
        }
        "get_agenda" => {
            let start = arg_str(a, "date").and_then(|s| parse_day(&s)).unwrap_or_else(today);
            let days = arg_i64(a, "days").unwrap_or(7).clamp(1, 62);
            let mut out = vec![];
            for i in 0..days {
                let day = start + Duration::days(i);
                let mut tasks: Vec<Value> = d.tasks.iter().filter(|t| !t.completed && (t.due == Some(day) || (day == today() && t.due.map_or(false, |x| x < day)))).map(|t| task_json(d, t)).collect();
                tasks.sort_by_key(|v| v["time"].as_str().unwrap_or("99").to_string());
                let mut evs: Vec<(NaiveDateTime, Value)> = d.events.iter().filter_map(|e| e.occurrence_on(day).map(|s| (s, event_json(d, e, s)))).collect();
                evs.sort_by_key(|x| x.0);
                out.push(json!({"date": day.to_string(), "weekday": day.format("%A").to_string(), "events": evs.into_iter().map(|x| x.1).collect::<Vec<_>>(), "tasks": tasks}));
            }
            Ok(json!(out))
        }
        "list_pages" => Ok(json!(d.pages.iter().map(|p| json!({"id": p.id, "title": p.title, "icon": p.icon, "parent_id": p.parent, "favorite": p.favorite, "updated": chrono::DateTime::from_timestamp_millis(p.updated).map(|x| x.to_rfc3339())})).collect::<Vec<_>>())),
        "read_page" => {
            let page = arg_str(a, "id").and_then(|i| d.page(&i).cloned()).or_else(|| arg_str(a, "title").and_then(|t| d.page_by_title(&t).cloned())).ok_or("Page not found — use search or list_pages to find its id")?;
            Ok(json!({
                "id": page.id,
                "title": page.title,
                "parent_id": page.parent,
                "markdown": blocks_to_markdown(&page, d),
                "subpages": d.pages.iter().filter(|p| p.parent.as_deref() == Some(&page.id)).map(|p| json!({"id": p.id, "title": p.title})).collect::<Vec<_>>(),
                "backlinks": d.backlinks(&page.id).iter().map(|p| json!({"id": p.id, "title": p.title})).collect::<Vec<_>>(),
                "linked_tasks": d.tasks.iter().filter(|t| t.page_id.as_deref() == Some(&page.id)).map(|t| task_json(d, t)).collect::<Vec<_>>(),
            }))
        }
        "list_tasks" => {
            let view = arg_str(a, "view").unwrap_or_else(|| "all".into()).to_lowercase();
            let project = arg_str(a, "project");
            let q = arg_str(a, "query").map(|s| s.to_lowercase());
            let limit = arg_i64(a, "limit").unwrap_or(100).clamp(1, 1000) as usize;
            let t0 = today();
            let mut v: Vec<&Task> = d
                .tasks
                .iter()
                .filter(|t| match view.as_str() {
                    "completed" => t.completed,
                    "today" => !t.completed && t.due.map_or(false, |x| x <= t0),
                    "overdue" => t.is_overdue(),
                    "upcoming" => !t.completed && t.due.map_or(false, |x| x >= t0),
                    "inbox" => !t.completed && t.project_id.is_none(),
                    _ => !t.completed,
                })
                .filter(|t| project.as_ref().map_or(true, |p| d.project_name(&t.project_id).eq_ignore_ascii_case(p)))
                .filter(|t| q.as_ref().map_or(true, |q| t.title.to_lowercase().contains(q) || t.notes.to_lowercase().contains(q)))
                .collect();
            v.sort_by_key(|t| (t.due.unwrap_or(NaiveDate::MAX), t.due_time, t.priority, t.order));
            v.truncate(limit);
            Ok(json!(v.iter().map(|t| task_json(d, t)).collect::<Vec<_>>()))
        }
        "list_projects" => Ok(json!(d.projects.iter().map(|p| json!({
            "id": p.id, "name": p.name, "sections": p.sections,
            "open_tasks": d.tasks.iter().filter(|t| t.project_id.as_deref() == Some(&p.id) && !t.completed).count(),
            "page_id": p.page_id,
        })).collect::<Vec<_>>())),
        "list_calendars" => Ok(json!(d.calendars.iter().map(|c| json!({"id": c.id, "name": c.name, "visible": c.visible, "color": format!("#{:02x}{:02x}{:02x}", c.color[0], c.color[1], c.color[2])})).collect::<Vec<_>>())),
        "list_events" => {
            let from = arg_str(a, "from").and_then(|s| parse_day(&s)).unwrap_or_else(today);
            let to = arg_str(a, "to").and_then(|s| parse_day(&s)).unwrap_or(from + Duration::days(7));
            let cal = arg_str(a, "calendar").map(|s| s.to_lowercase());
            let mut out: Vec<(NaiveDateTime, Value)> = vec![];
            let mut day = from;
            let mut guard = 0;
            while day <= to && guard < 370 {
                for e in &d.events {
                    if let Some(c) = &cal {
                        if d.calendar(&e.calendar_id).map_or(true, |x| x.name.to_lowercase() != *c) {
                            continue;
                        }
                    }
                    if let Some(s) = e.occurrence_on(day) {
                        if day == e.start.date() || e.recurrence.is_some() || day == from {
                            out.push((s, event_json(d, e, s)));
                        }
                    }
                }
                day += Duration::days(1);
                guard += 1;
            }
            out.sort_by_key(|x| x.0);
            Ok(json!(out.into_iter().map(|x| x.1).collect::<Vec<_>>()))
        }
        "list_databases" => Ok(json!(d.databases.iter().map(|x| json!({
            "id": x.id, "name": x.name, "rows": x.rows.len(),
            "columns": x.columns.iter().map(|c| json!({"name": c.name, "type": c.kind.label(), "options": c.options})).collect::<Vec<_>>()
        })).collect::<Vec<_>>())),
        "query_database" => {
            let db = find_db(d, &arg_str(a, "database").unwrap_or_default()).ok_or("Database not found")?;
            let q = arg_str(a, "query").map(|s| s.to_lowercase());
            let limit = arg_i64(a, "limit").unwrap_or(200).clamp(1, 5000) as usize;
            let rows: Vec<Value> = db.rows.iter().filter(|r| q.as_ref().map_or(true, |q| r.cells.values().any(|v| v.to_lowercase().contains(q)))).take(limit).map(|r| row_json(db, r)).collect();
            Ok(json!({"id": db.id, "name": db.name, "rows": rows}))
        }
        "create_page" => {
            let title = arg_str(a, "title").unwrap_or_else(|| "Untitled".into());
            let mut page = Page::new(title);
            if let Some(md) = arg_str(a, "markdown") {
                let (t, blocks) = markdown_to_blocks(&md);
                if let Some(t) = t {
                    if page.title.trim().is_empty() {
                        page.title = t;
                    }
                }
                page.blocks = blocks;
            }
            if let Some(p) = arg_str(a, "parent_id") {
                if d.page(&p).is_some() {
                    page.parent = Some(p);
                }
            }
            if let Some(i) = arg_str(a, "icon") {
                page.icon = i;
            }
            let id = page.id.clone();
            d.pages.push(page);
            Ok(json!({"created": "page", "id": id}))
        }
        "update_page" => {
            let id = arg_str(a, "id").unwrap_or_default();
            let page = d.page_mut(&id).ok_or("Page not found")?;
            if let Some(t) = arg_str(a, "title") {
                page.title = t;
            }
            if let Some(md) = arg_str(a, "markdown") {
                let (t, blocks) = markdown_to_blocks(&md);
                if let (Some(t), None) = (t, arg_str(a, "title")) {
                    page.title = t;
                }
                // keep to-do ↔ task links for to-dos whose text didn't change
                let mut blocks = blocks;
                for nb in blocks.iter_mut().filter(|b| b.kind == BlockKind::Todo) {
                    if let Some(old) = page.blocks.iter().find(|o| o.kind == BlockKind::Todo && o.text.trim() == nb.text.trim() && o.task_id.is_some()) {
                        nb.task_id = old.task_id.clone();
                        nb.id = old.id.clone();
                    }
                }
                page.blocks = blocks;
            }
            if let Some(md) = arg_str(a, "append_markdown") {
                let (t, blocks) = markdown_to_blocks(&md);
                if let Some(t) = t {
                    page.blocks.push(Block::new(BlockKind::H1, t));
                }
                if page.blocks.len() == 1 && page.blocks[0].text.is_empty() {
                    page.blocks.clear();
                }
                page.blocks.extend(blocks);
            }
            if let Some(i) = arg_str(a, "icon") {
                page.icon = i;
            }
            if let Some(f) = arg_bool(a, "favorite") {
                page.favorite = f;
            }
            page.updated = now_ms();
            if let Some(parent) = arg_str(a, "parent_id") {
                let new_parent = if parent.is_empty() { None } else { Some(parent.clone()) };
                if let Some(np) = &new_parent {
                    if np == &id || d.page_path(np).iter().any(|x| x.id == id) {
                        return Err("A page can't be moved inside itself".into());
                    }
                    if d.page(np).is_none() {
                        return Err("Parent page not found".into());
                    }
                }
                if let Some(page) = d.page_mut(&id) {
                    page.parent = new_parent;
                }
            }
            Ok(json!({"updated": "page", "id": id}))
        }
        "create_task" => {
            let mut t = Task::new("");
            if let Some(text) = arg_str(a, "text") {
                let c = crate::nlp::parse(&text, today());
                t.title = c.title;
                t.due = c.due;
                t.due_time = c.time;
                t.priority = c.priority.unwrap_or(4);
                t.recurrence = c.recurrence;
                t.labels = c.labels;
                t.reminder_min = c.reminder_min;
                t.duration_min = c.duration_min;
                if let Some(p) = c.project {
                    t.project_id = ensure_project(d, &p);
                }
                if let Some(s) = c.section {
                    t.section = s;
                }
            }
            apply_task_fields(d, &mut t, a);
            if t.title.trim().is_empty() {
                return Err("A task needs a title (or text)".into());
            }
            let v = task_json(d, &t);
            d.tasks.push(t);
            Ok(json!({"created": "task", "task": v}))
        }
        "update_task" => {
            let id = arg_str(a, "id").unwrap_or_default();
            let mut t = d.task(&id).cloned().ok_or("Task not found")?;
            apply_task_fields(d, &mut t, a);
            let mut rolled = false;
            if let Some(c) = arg_bool(a, "completed") {
                if c && !t.completed {
                    rolled = t.complete();
                } else if !c {
                    t.completed = false;
                    t.completed_at = None;
                }
            }
            let v = task_json(d, &t);
            *d.task_mut(&id).unwrap() = t;
            Ok(json!({"updated": "task", "rolled_to_next_date": rolled, "task": v}))
        }
        "complete_task" => {
            let id = arg_str(a, "id").unwrap_or_default();
            let t = d.task_mut(&id).ok_or("Task not found")?;
            let rolled = t.complete();
            Ok(json!({"completed": !rolled, "rolled_to_next_date": rolled, "next_due": t.due.map(|x| x.to_string())}))
        }
        "create_project" => {
            let name = arg_str(a, "name").unwrap_or_default();
            let id = ensure_project(d, &name).ok_or("A project needs a name other than Inbox")?;
            if let Some(secs) = arg_list(a, "sections") {
                if let Some(p) = d.projects.iter_mut().find(|p| p.id == id) {
                    for s in secs {
                        if !p.sections.contains(&s) {
                            p.sections.push(s);
                        }
                    }
                    p.updated = now_ms();
                }
            }
            Ok(json!({"created": "project", "id": id}))
        }
        "create_event" | "update_event" => {
            let creating = name == "create_event";
            let mut e = if creating {
                let cal = d.default_calendar().ok_or("Create a calendar first")?;
                Event { id: new_id(), calendar_id: cal, title: String::new(), start: now_local(), end: now_local(), all_day: false, location: String::new(), notes: String::new(), recurrence: Recurrence::None, page_id: None, reminder_min: None, updated: now_ms() }
            } else {
                let id = arg_str(a, "id").unwrap_or_default();
                d.events.iter().find(|x| x.id == id).cloned().ok_or("Event not found")?
            };
            let old_len = e.duration();
            if let Some(t) = arg_str(a, "title") {
                e.title = t;
            }
            if let Some(s) = arg_str(a, "start") {
                let (dt, date_only) = parse_datetime(&s).ok_or(format!("Couldn't read the start '{s}' — use YYYY-MM-DD HH:MM"))?;
                e.start = dt;
                if date_only {
                    e.all_day = true;
                }
                e.end = dt + if creating { Duration::minutes(arg_i64(a, "duration_minutes").unwrap_or(if date_only { 1440 } else { 60 })) } else { old_len };
            }
            if let Some(s) = arg_str(a, "end") {
                if let Some((dt, date_only)) = parse_datetime(&s) {
                    e.end = if date_only && e.all_day { dt + Duration::days(1) } else { dt };
                }
            }
            if e.end <= e.start {
                e.end = e.start + Duration::minutes(30);
            }
            if let Some(b) = arg_bool(a, "all_day") {
                e.all_day = b;
            }
            if let Some(c) = arg_str(a, "calendar") {
                e.calendar_id = d.calendars.iter().find(|x| x.name.eq_ignore_ascii_case(c.trim()) || x.id == c).map(|x| x.id.clone()).ok_or(format!("No calendar named '{c}'"))?;
            }
            if let Some(v) = arg_str(a, "location") {
                e.location = v;
            }
            if let Some(v) = arg_str(a, "notes") {
                e.notes = v;
            }
            if let Some(v) = arg_str(a, "recurrence") {
                e.recurrence = parse_recurrence(&v);
            }
            if let Some(v) = arg_str(a, "page_id") {
                e.page_id = if v.is_empty() { None } else { Some(v) };
            }
            if let Some(m) = arg_i64(a, "reminder_minutes") {
                e.reminder_min = if m < 0 { None } else { Some(m) };
            }
            if e.title.trim().is_empty() {
                return Err("An event needs a title".into());
            }
            e.updated = now_ms();
            let v = event_json(d, &e, e.start);
            if creating {
                d.events.push(e);
            } else {
                let id = e.id.clone();
                *d.event_mut(&id).unwrap() = e;
            }
            Ok(json!({ "saved": if creating { "created" } else { "updated" }, "event": v }))
        }
        "create_database" => {
            let name = arg_str(a, "name").unwrap_or_else(|| "Untitled database".into());
            let mut cols = vec![];
            if let Some(Value::Array(list)) = a.get("columns") {
                for c in list {
                    let cname = c.get("name").and_then(|x| x.as_str()).unwrap_or("Column").to_string();
                    let kind = match c.get("type").and_then(|x| x.as_str()).unwrap_or("text").to_lowercase().as_str() {
                        "number" => ColKind::Number,
                        "select" | "status" => ColKind::Select,
                        "checkbox" | "bool" => ColKind::Checkbox,
                        "date" => ColKind::Date,
                        "url" | "link" => ColKind::Url,
                        _ => ColKind::Text,
                    };
                    let options = c.get("options").and_then(|o| o.as_array()).map(|o| o.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()).unwrap_or_default();
                    cols.push(Column { id: new_id(), name: cname, kind, options, width: 160.0, target: String::new() });
                }
            }
            if cols.is_empty() {
                cols.push(Column { id: new_id(), name: "Name".into(), kind: ColKind::Text, options: vec![], width: 220.0, target: String::new() });
            }
            let group_by = cols.iter().find(|c| c.kind == ColKind::Select).map(|c| c.id.clone());
            let db = Database { id: new_id(), name, icon: "▦".into(), columns: cols, rows: vec![], view: DbView::Table, group_by, sort_by: None, filters: vec![], template: false, updated: now_ms() };
            let id = db.id.clone();
            d.databases.push(db);
            Ok(json!({"created": "database", "id": id}))
        }
        "add_database_row" | "update_database_row" => {
            let key = arg_str(a, "database").unwrap_or_default();
            let id = find_db(d, &key).map(|x| x.id.clone()).ok_or("Database not found")?;
            let db = d.database_mut(&id).unwrap();
            let values = a.get("values").cloned().unwrap_or(json!({}));
            let mut row = if name == "add_database_row" {
                Row { id: new_id(), cells: Default::default(), page_id: None }
            } else {
                let rid = arg_str(a, "row_id").unwrap_or_default();
                db.rows.iter().find(|r| r.id == rid).cloned().ok_or("Row not found")?
            };
            set_cells(db, &mut row, &values)?;
            let v = row_json(db, &row);
            if let Some(existing) = db.rows.iter_mut().find(|r| r.id == row.id) {
                *existing = row;
            } else {
                db.rows.push(row);
            }
            db.updated = now_ms();
            Ok(json!({"saved": "row", "row": v}))
        }
        "list_page_todos" => {
            let items: Vec<Value> = d
                .page_todos()
                .into_iter()
                .map(|(pid, ptitle, bid, text, linked)| {
                    let proj = d.projects.iter().find(|p| p.page_id.as_deref() == Some(pid.as_str())).map(|p| p.name.clone());
                    json!({"page_id": pid, "page_title": ptitle, "block_id": bid, "text": text, "linked_task_id": linked, "page_is_doc_for_project": proj})
                })
                .collect();
            Ok(json!({"todos": items, "projects": d.projects.iter().filter(|p| !p.archived).map(|p| json!({"name": p.name, "sections": p.sections})).collect::<Vec<_>>()}))
        }
        "convert_todos_to_tasks" => {
            let Some(Value::Array(items)) = a.get("items") else { return Err("items must be a list".into()) };
            let link_projects = arg_bool(a, "link_projects_to_pages").unwrap_or(true);
            let mut made: Vec<Value> = vec![];
            let mut by_block: std::collections::HashMap<String, Id> = Default::default();
            let mut skipped = 0;
            // parents first, so subtasks can find them
            let mut ordered: Vec<&Value> = items.iter().filter(|i| arg_str(i, "parent_block_id").is_none()).collect();
            ordered.extend(items.iter().filter(|i| arg_str(i, "parent_block_id").is_some()));
            for it in ordered {
                let pid = arg_str(it, "page_id").unwrap_or_default();
                let bid = arg_str(it, "block_id").unwrap_or_default();
                let Some(block) = d.page(&pid).and_then(|p| p.blocks.iter().find(|b| b.id == bid)).cloned() else {
                    skipped += 1;
                    continue;
                };
                if block.task_id.as_ref().map_or(false, |t| d.task(t).is_some()) {
                    skipped += 1;
                    continue;
                }
                // start from quick-capture parsing of the to-do text, then apply the assistant's choices
                let c = crate::nlp::parse(&block.text, today());
                let mut t = Task::new(if c.title.is_empty() { block.text.clone() } else { c.title.clone() });
                t.due = c.due;
                t.due_time = c.time;
                t.priority = c.priority.unwrap_or(4);
                t.recurrence = c.recurrence;
                t.labels = c.labels.clone();
                // blank answers mean "keep what the to-do says", not "clear it"
                let mut it = it.clone();
                if let Some(obj) = it.as_object_mut() {
                    obj.retain(|_, v| !(v.is_null() || v.as_str().map_or(false, |s| s.trim().is_empty())));
                }
                let it = &it;
                let had_project = arg_str(it, "project").map(|p| d.project_by_name(&p).is_some() || p.eq_ignore_ascii_case("inbox")).unwrap_or(true);
                apply_task_fields(d, &mut t, it);
                if !had_project && link_projects {
                    if let Some(proj) = t.project_id.clone() {
                        if let Some(pr) = d.projects.iter_mut().find(|p| p.id == proj) {
                            if pr.page_id.is_none() {
                                pr.page_id = Some(pid.clone());
                                pr.updated = now_ms();
                            }
                        }
                    }
                }
                if !t.section.is_empty() {
                    if let Some(proj) = t.project_id.clone() {
                        if let Some(pr) = d.projects.iter_mut().find(|p| p.id == proj) {
                            if !pr.sections.contains(&t.section) {
                                pr.sections.push(t.section.clone());
                                pr.updated = now_ms();
                            }
                        }
                    }
                }
                if let Some(parent_block) = arg_str(it, "parent_block_id") {
                    if let Some(parent) = by_block.get(&parent_block) {
                        t.parent_id = Some(parent.clone());
                        if let Some(par) = d.task(parent) {
                            t.project_id = par.project_id.clone();
                            t.section = par.section.clone();
                        }
                    }
                }
                t.page_id = Some(pid.clone());
                t.updated = now_ms();
                let tid = t.id.clone();
                made.push(json!({"task_id": tid, "title": t.title, "project": d.project_name(&t.project_id), "section": t.section, "due": t.due.map(|x| x.to_string()), "priority": t.priority}));
                d.tasks.push(t);
                by_block.insert(bid.clone(), tid.clone());
                if let Some(page) = d.page_mut(&pid) {
                    if let Some(b) = page.blocks.iter_mut().find(|b| b.id == bid) {
                        b.task_id = Some(tid);
                    }
                    page.updated = now_ms();
                }
            }
            Ok(json!({"created": made, "skipped_already_linked_or_missing": skipped}))
        }
        "update_project" => {
            let key = arg_str(a, "id").unwrap_or_default();
            let id = d.project(&key).map(|p| p.id.clone()).or_else(|| d.project_by_name(&key).map(|p| p.id.clone())).ok_or("Project not found")?;
            let pr = d.projects.iter_mut().find(|p| p.id == id).unwrap();
            if let Some(n) = arg_str(a, "name") {
                pr.name = n;
            }
            if let Some(c) = arg_str(a, "color").and_then(|c| parse_hex(&c)) {
                pr.color = c;
            }
            if let Some(secs) = arg_list(a, "sections") {
                for s in secs {
                    if !pr.sections.contains(&s) {
                        pr.sections.push(s);
                    }
                }
            }
            if let Some(b) = arg_bool(a, "archived") {
                pr.archived = b;
            }
            if let Some(p) = arg_str(a, "page_id") {
                pr.page_id = if p.is_empty() { None } else { Some(p) };
            }
            pr.updated = now_ms();
            Ok(json!({"updated": "project", "id": id}))
        }
        "create_calendar" => {
            let name = arg_str(a, "name").unwrap_or_else(|| "Calendar".into());
            let color = arg_str(a, "color").and_then(|c| parse_hex(&c)).unwrap_or([3, 155, 229]);
            let c = Calendar { id: new_id(), name, color, visible: true, updated: now_ms(), ..Default::default() };
            let id = c.id.clone();
            d.calendars.push(c);
            Ok(json!({"created": "calendar", "id": id}))
        }
        "add_database_column" => {
            let key = arg_str(a, "database").unwrap_or_default();
            let id = find_db(d, &key).map(|x| x.id.clone()).ok_or("Database not found")?;
            let kind = match arg_str(a, "type").unwrap_or_default().to_lowercase().as_str() {
                "number" => ColKind::Number,
                "select" | "status" => ColKind::Select,
                "checkbox" => ColKind::Checkbox,
                "date" => ColKind::Date,
                "url" => ColKind::Url,
                "email" => ColKind::Email,
                "phone" => ColKind::Phone,
                "page" => ColKind::Page,
                "relation" => ColKind::Relation,
                _ => ColKind::Text,
            };
            let target = match arg_str(a, "target") {
                Some(t) if matches!(t.as_str(), "projects" | "tasks" | "pages") => t,
                Some(t) => find_db(d, &t).map(|x| format!("db:{}", x.id)).unwrap_or_default(),
                None => String::new(),
            };
            let db = d.database_mut(&id).unwrap();
            let col = Column { id: new_id(), name: arg_str(a, "name").unwrap_or_else(|| "Property".into()), kind, options: arg_list(a, "options").unwrap_or_default(), width: 150.0, target };
            let cid = col.id.clone();
            db.columns.push(col);
            db.updated = now_ms();
            Ok(json!({"added_column": cid}))
        }
        "import_pages" => {
            let Some(Value::Array(list)) = a.get("pages") else { return Err("pages must be a list".into()) };
            let mut made: Vec<Value> = vec![];
            for item in list {
                let title = arg_str(item, "title").unwrap_or_else(|| "Untitled".into());
                let mut page = Page::new(title.clone());
                if let Some(md) = arg_str(item, "markdown") {
                    page.blocks = markdown_to_blocks(&md).1;
                }
                if let Some(i) = arg_str(item, "icon") {
                    page.icon = i;
                }
                if let Some(pt) = arg_str(item, "parent_title") {
                    page.parent = d.page_by_title(&pt).map(|p| p.id.clone());
                }
                made.push(json!({"id": page.id, "title": title}));
                d.pages.push(page);
            }
            Ok(json!({"imported_pages": made}))
        }
        "import_tasks" => {
            let Some(Value::Array(list)) = a.get("lines") else { return Err("lines must be a list".into()) };
            let default_project = arg_str(a, "project").and_then(|p| ensure_project(d, &p));
            let mut last_parent: Option<Id> = None;
            let mut n = 0;
            for l in list.iter().filter_map(|x| x.as_str()) {
                let is_sub = l.starts_with("  ") || l.starts_with(char::from(9u8));
                let text = l.trim().trim_start_matches("- ").trim_start_matches("[ ] ").trim();
                if text.is_empty() {
                    continue;
                }
                let c = crate::nlp::parse(text, today());
                let mut t = Task::new(c.title.clone());
                t.due = c.due;
                t.due_time = c.time;
                t.priority = c.priority.unwrap_or(4);
                t.recurrence = c.recurrence;
                t.labels = c.labels.clone();
                t.reminder_min = c.reminder_min;
                t.duration_min = c.duration_min;
                t.section = c.section.clone().unwrap_or_default();
                t.project_id = match &c.project {
                    Some(p) => ensure_project(d, p),
                    None => default_project.clone(),
                };
                if is_sub {
                    if let Some(pid) = &last_parent {
                        t.parent_id = Some(pid.clone());
                        if let Some(par) = d.task(pid) {
                            t.project_id = par.project_id.clone();
                            t.section = par.section.clone();
                        }
                    }
                } else {
                    last_parent = Some(t.id.clone());
                }
                if !t.title.is_empty() {
                    d.tasks.push(t);
                    n += 1;
                }
            }
            Ok(json!({"imported_tasks": n}))
        }
        "import_csv" => {
            let name = arg_str(a, "name").unwrap_or_else(|| "Imported table".into());
            let db = crate::import::csv_to_database(&name, &arg_str(a, "csv").unwrap_or_default())?;
            let out = json!({"database_id": db.id, "rows": db.rows.len(), "columns": db.columns.iter().map(|c| json!({"name": c.name, "type": c.kind.label()})).collect::<Vec<_>>()});
            d.databases.push(db);
            Ok(out)
        }
        "import_ics" => {
            let text = arg_str(a, "ics").unwrap_or_default();
            if !text.contains("BEGIN:VCALENDAR") && !text.contains("BEGIN:VEVENT") {
                return Err("That isn't iCalendar text".into());
            }
            let (name, _) = crate::import::ics_events(&text, "");
            let cname = arg_str(a, "calendar").or(name).unwrap_or_else(|| "Imported".into());
            let cal_id = match d.calendars.iter().find(|c| c.name.eq_ignore_ascii_case(&cname)) {
                Some(c) => c.id.clone(),
                None => {
                    let c = Calendar { id: new_id(), name: cname.clone(), color: [3, 155, 229], visible: true, updated: now_ms(), ..Default::default() };
                    let id = c.id.clone();
                    d.calendars.push(c);
                    id
                }
            };
            let (_, events) = crate::import::ics_events(&text, &cal_id);
            let mut n = 0;
            for e in events {
                if !d.events.iter().any(|x| x.id == e.id) {
                    d.events.push(e);
                    n += 1;
                }
            }
            Ok(json!({"calendar": cname, "imported_events": n}))
        }
        "resolve_projects" => Ok(json!(d.contexts.iter().map(|c| {
            let a = crate::resolve::assess::assess(d, c);
            json!({"project_id": c.project_id, "project": d.project(&c.project_id).map(|p| p.name.clone()), "outcome": c.outcome, "outcome_confirmed": c.outcome_confirmed, "progress": format!("{}/{}", a.done, a.total), "next": a.next.first().map(|x| x.0.clone())})
        }).collect::<Vec<_>>())),
        "resolve_project" => {
            let pid = find_project_id(d, &arg_str(a, "project").unwrap_or_default())?;
            let c = d.context_for(&pid).ok_or("That project has no Resolve context yet (run Intake, or adopt it in the app).")?;
            let mut v = context_json(d, c);
            v["records"] = json!(crate::resolve::assess::records_for(d, &pid).iter().map(|r| record_json(r)).collect::<Vec<_>>());
            Ok(v)
        }
        "list_records" => {
            let pid = match arg_str(a, "project") { Some(p) if !p.is_empty() => Some(find_project_id(d, &p)?), _ => None };
            let kind = arg_str(a, "kind").filter(|k| !k.is_empty()).map(|k| crate::resolve::Kind::parse(&k).ok_or(format!("Unknown kind '{k}'"))).transpose()?;
            let open = arg_bool(a, "open_only").unwrap_or(false);
            Ok(json!(d.records.iter().filter(|r| pid.as_ref().map_or(true, |p| r.project_id.as_ref() == Some(p))).filter(|r| kind.map_or(true, |k| r.kind == k)).filter(|r| !open || r.is_open()).map(record_json).collect::<Vec<_>>()))
        }
        "add_record" => {
            let pid = find_project_id(d, &arg_str(a, "project").unwrap_or_default())?;
            let k = arg_str(a, "kind").unwrap_or_default();
            let kind = crate::resolve::Kind::parse(&k).ok_or(format!("Unknown kind '{k}'"))?;
            let title = arg_str(a, "title").unwrap_or_default();
            if title.trim().is_empty() { return Err("A record needs a title".into()); }
            let mut r = crate::resolve::Record::new(kind, Some(pid), title);
            if let Some(p) = arg_str(a, "parent_id").filter(|p| !p.is_empty()) {
                if d.record(&p).is_none() { return Err("parent_id isn't a record".into()); }
                r.parent_id = Some(p);
            }
            set_record_fields(&mut r, a)?;
            let v = record_json(&r);
            d.records.push(r);
            Ok(v)
        }
        "update_record" => {
            let id = arg_str(a, "id").unwrap_or_default();
            let r = d.record_mut(&id).ok_or("No record with that id")?;
            let mut copy = r.clone();
            set_record_fields(&mut copy, a)?;
            *r = copy;
            Ok(record_json(r))
        }        "delete_item" => {
            let id = arg_str(a, "id").unwrap_or_default();
            if d.page(&id).is_some() {
                d.delete_page(&id);
            } else if d.task(&id).is_some() {
                d.delete_task(&id);
            } else if d.events.iter().any(|e| e.id == id) {
                d.delete_event(&id);
            } else if d.project(&id).is_some() {
                d.delete_project(&id);
            } else if d.database(&id).is_some() {
                d.delete_database(&id);
            } else if d.record(&id).is_some() {
                d.delete_record(&id);
            } else {
                // maybe a database row
                let mut found = false;
                for db in d.databases.iter_mut() {
                    let before = db.rows.len();
                    db.rows.retain(|r| r.id != id);
                    if db.rows.len() != before {
                        db.updated = now_ms();
                        found = true;
                    }
                }
                if !found {
                    return Err("Nothing with that id".into());
                }
            }
            Ok(json!({"deleted": id}))
        }
        _ => Err(format!("Unknown tool '{name}'")),
    }
}

// ---------------------------------------------------------------- JSON-RPC

/// Handle one JSON-RPC message; None for notifications.
pub fn handle(store: &mut Store, msg: &Value) -> Option<Value> {
    let id = msg.get("id").cloned();
    let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = msg.get("params").cloned().unwrap_or(json!({}));
    id.as_ref()?;
    let id = id.unwrap();
    let result: Result<Value, (i64, String)> = match method {
        "initialize" => {
            let pv = params.get("protocolVersion").and_then(|v| v.as_str()).unwrap_or(PROTOCOL).to_string();
            Ok(json!({
                "protocolVersion": pv,
                "capabilities": {"tools": {"listChanged": false}, "prompts": {"listChanged": false}},
                "serverInfo": {"name": "resolve-2", "title": format!("Resolve 2 — {}", store.meta.name), "version": env!("CARGO_PKG_VERSION")},
                "instructions": format!("This is the user's '{}' workspace in Resolve 2: pages (Notion-style docs), databases, projects & tasks (Todoist-style, priorities p1..p4, recurring due dates) and calendars. Start with workspace_overview or search. Dates are local time, YYYY-MM-DD.", store.meta.name)
            }))
        }
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({"tools": tools(store.settings.ai_read_only)})),
        "tools/call" => {
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            match call(store, name, &args) {
                Ok(text) => Ok(json!({"content": [{"type": "text", "text": text}], "isError": false})),
                Err(e) => Ok(json!({"content": [{"type": "text", "text": e}], "isError": true})),
            }
        }
        "resources/list" => Ok(json!({"resources": []})),
        "prompts/list" => Ok(json!({"prompts": [
            {"name": "organise_todos", "title": "Organise my to-dos", "description": "Move the to-dos written in your pages into projects as real tasks with deadlines, linked back to the pages."},
            {"name": "plan_my_day", "title": "Plan my day", "description": "Order today's tasks and events into a realistic plan."},
            {"name": "weekly_review", "title": "Weekly review", "description": "Write a weekly review page from what was done, what slipped and what's next."}
        ]})),
        "prompts/get" => {
            let pname = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let text = match pname {
                "organise_todos" => Some(crate::ai::ORGANISE_PROMPT.to_string()),
                "plan_my_day" => Some("Use get_agenda for today (overdue tasks are included) and suggest a realistic order for my day around my events. Point out clashes and anything that won't fit.".to_string()),
                "weekly_review" => Some("Using list_tasks (completed, overdue, upcoming) and get_agenda for the next 7 days, write a short weekly review and save it with create_page as 'Weekly review — <today's date>'.".to_string()),
                _ => None,
            };
            match text {
                Some(t) => Ok(json!({"description": pname, "messages": [{"role": "user", "content": {"type": "text", "text": t}}]})),
                None => Err((-32602, format!("Unknown prompt: {pname}"))),
            }
        }
        _ => Err((-32601, format!("Method not found: {method}"))),
    };
    Some(match result {
        Ok(r) => json!({"jsonrpc": "2.0", "id": id, "result": r}),
        Err((code, m)) => json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": m}}),
    })
}

pub fn handle_text(store: &mut Store, body: &str) -> Option<String> {
    let v: Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => return Some(json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": format!("Parse error: {e}")}}).to_string()),
    };
    if let Value::Array(batch) = &v {
        let out: Vec<Value> = batch.iter().filter_map(|m| handle(store, m)).collect();
        if out.is_empty() {
            None
        } else {
            Some(Value::Array(out).to_string())
        }
    } else {
        handle(store, &v).map(|r| r.to_string())
    }
}

/// Serve MCP over stdin/stdout (newline-delimited JSON-RPC) until stdin closes.
pub fn run_stdio(mut store: Store) {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        // some Windows tools (e.g. PowerShell pipes) prefix a byte-order mark
        let line = line.trim_start_matches('\u{feff}');
        if line.trim().is_empty() {
            continue;
        }
        if let Some(resp) = handle_text(&mut store, line) {
            let _ = writeln!(out, "{resp}");
            let _ = out.flush();
        }
    }
}
