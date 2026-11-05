//! Full-text search across pages, tasks, events, projects and database rows.

use crate::model::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HitKind {
    Page,
    Task,
    Event,
    Project,
    Row,
}

impl HitKind {
    pub fn label(self) -> &'static str {
        match self {
            HitKind::Page => "Page",
            HitKind::Task => "Task",
            HitKind::Event => "Event",
            HitKind::Project => "Project",
            HitKind::Row => "Database",
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            HitKind::Page => "📄",
            HitKind::Task => "○",
            HitKind::Event => "📅",
            HitKind::Project => "#",
            HitKind::Row => "▦",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Hit {
    pub kind: HitKind,
    /// id of the item; for rows this is the database id
    pub id: Id,
    pub row_id: Option<Id>,
    pub title: String,
    pub snippet: String,
    pub score: i32,
}

fn snippet(body: &str, tokens: &[String]) -> String {
    let lower = body.to_lowercase();
    let pos = tokens.iter().filter_map(|t| lower.find(t.as_str())).min().unwrap_or(0);
    // work in chars to stay on UTF-8 boundaries
    let char_pos = lower[..pos].chars().count();
    let chars: Vec<char> = body.chars().collect();
    let start = char_pos.saturating_sub(40);
    let end = (char_pos + 90).min(chars.len());
    let mut s: String = chars[start..end].iter().collect::<String>().replace('\n', " · ");
    s = s.replace("**", "").replace("[[", "").replace("]]", "");
    if start > 0 {
        s = format!("…{s}");
    }
    if end < chars.len() {
        s.push('…');
    }
    s
}

fn score(title: &str, body: &str, q: &str, tokens: &[String]) -> Option<i32> {
    let tl = title.to_lowercase();
    let bl = body.to_lowercase();
    let mut s = 0;
    for t in tokens {
        let in_title = tl.contains(t.as_str());
        let in_body = bl.contains(t.as_str());
        if !in_title && !in_body {
            return None;
        }
        if in_title {
            s += 10;
            if tl.split(|c: char| !c.is_alphanumeric()).any(|w| w.starts_with(t.as_str())) {
                s += 5;
            }
        }
        if in_body {
            s += 1 + bl.matches(t.as_str()).count().min(5) as i32;
        }
    }
    if tl == q {
        s += 50;
    } else if tl.starts_with(q) {
        s += 20;
    } else if tl.contains(q) {
        s += 10;
    }
    Some(s)
}

/// Optional prefixes narrow the search: `page:`, `task:`, `event:`, `db:`.
pub fn search(d: &WorkspaceData, query: &str, limit: usize) -> Vec<Hit> {
    let mut q = query.trim().to_lowercase();
    let mut only: Option<HitKind> = None;
    for (p, k) in [("page:", HitKind::Page), ("task:", HitKind::Task), ("event:", HitKind::Event), ("db:", HitKind::Row), ("project:", HitKind::Project)] {
        if let Some(r) = q.strip_prefix(p) {
            only = Some(k);
            q = r.trim().to_string();
        }
    }
    let tokens: Vec<String> = q.split_whitespace().map(|s| s.to_string()).collect();
    let mut hits = vec![];
    if tokens.is_empty() {
        return hits;
    }
    let want = |k: HitKind| only.map_or(true, |o| o == k);
    if want(HitKind::Page) {
        for p in &d.pages {
            let body = p.plain_text();
            if let Some(s) = score(&p.title, &body, &q, &tokens) {
                hits.push(Hit { kind: HitKind::Page, id: p.id.clone(), row_id: None, title: format!("{} {}", p.icon, p.title).trim().to_string(), snippet: snippet(&body, &tokens), score: s + 2 });
            }
        }
    }
    if want(HitKind::Task) {
        for t in &d.tasks {
            let body = format!("{} {} {}", t.notes, t.labels.join(" "), d.project_name(&t.project_id));
            if let Some(s) = score(&t.title, &body, &q, &tokens) {
                let mut sn = d.project_name(&t.project_id);
                if let Some(due) = t.due {
                    sn.push_str(&format!(" · {}", fmt_date_short(due)));
                }
                if t.completed {
                    sn.push_str(" · done");
                }
                hits.push(Hit { kind: HitKind::Task, id: t.id.clone(), row_id: None, title: t.title.clone(), snippet: sn, score: s - if t.completed { 5 } else { 0 } });
            }
        }
    }
    if want(HitKind::Event) {
        for e in &d.events {
            let body = format!("{} {}", e.location, e.notes);
            if let Some(s) = score(&e.title, &body, &q, &tokens) {
                let cal = d.calendar(&e.calendar_id).map(|c| c.name.clone()).unwrap_or_default();
                hits.push(Hit {
                    kind: HitKind::Event,
                    id: e.id.clone(),
                    row_id: None,
                    title: e.title.clone(),
                    snippet: format!("{} · {} {}", cal, e.start.format("%a %-d %b %H:%M"), if e.recurrence.is_some() { "🔁" } else { "" }),
                    score: s,
                });
            }
        }
    }
    if want(HitKind::Project) {
        for p in &d.projects {
            if let Some(s) = score(&p.name, "", &q, &tokens) {
                let n = d.tasks.iter().filter(|t| t.project_id.as_deref() == Some(&p.id) && !t.completed).count();
                hits.push(Hit { kind: HitKind::Project, id: p.id.clone(), row_id: None, title: p.name.clone(), snippet: format!("{n} open tasks"), score: s + 3 });
            }
        }
    }
    if want(HitKind::Row) {
        for db in &d.databases {
            if let Some(s) = score(&db.name, "", &q, &tokens) {
                hits.push(Hit { kind: HitKind::Row, id: db.id.clone(), row_id: None, title: format!("{} {}", db.icon, db.name).trim().to_string(), snippet: format!("Database · {} rows", db.rows.len()), score: s + 3 });
            }
            for r in &db.rows {
                let title = db.title_of(r);
                let body: Vec<String> = db.columns.iter().skip(1).filter_map(|c| r.cells.get(&c.id).cloned()).collect();
                let body = body.join(" · ");
                if let Some(s) = score(&title, &body, &q, &tokens) {
                    hits.push(Hit { kind: HitKind::Row, id: db.id.clone(), row_id: Some(r.id.clone()), title, snippet: format!("{} · {}", db.name, body), score: s });
                }
            }
        }
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score).then(a.title.cmp(&b.title)));
    hits.truncate(limit);
    hits
}
