//! Bring your data in (Notion Markdown & CSV export, Todoist CSV / backup zip, Google Calendar .ics)
//! and take it out again (CSV, .ics, Markdown) — your data stays yours.

use crate::model::*;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use std::collections::{BTreeMap, HashMap};
use std::io::Read;
use std::path::Path;

#[derive(Default, Debug, Clone)]
pub struct ImportReport {
    pub pages: usize,
    pub databases: usize,
    pub projects: usize,
    pub tasks: usize,
    pub calendars: usize,
    pub events: usize,
    pub skipped: usize,
    pub notes: Vec<String>,
}

impl ImportReport {
    pub fn summary(&self) -> String {
        let mut parts = vec![];
        for (n, w) in [
            (self.pages, "pages"),
            (self.databases, "databases"),
            (self.projects, "projects"),
            (self.tasks, "tasks"),
            (self.calendars, "calendars"),
            (self.events, "events"),
        ] {
            if n > 0 {
                parts.push(format!("{n} {w}"));
            }
        }
        if parts.is_empty() {
            "Nothing new was found to import".into()
        } else {
            let mut s = format!("Imported {}", parts.join(", "));
            if self.skipped > 0 {
                s.push_str(&format!(" ({} already present)", self.skipped));
            }
            s
        }
    }
}

/// A file from a folder or a zip: (relative path with '/', bytes)
type VFile = (String, Vec<u8>);

fn collect_files(path: &Path) -> Result<Vec<VFile>, String> {
    let mut out = vec![];
    if path.is_dir() {
        walk(path, path, &mut out)?;
    } else {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        if name.to_lowercase().ends_with(".zip") {
            unzip(&bytes, "", &mut out, 0)?;
        } else {
            out.push((name, bytes));
        }
    }
    Ok(out)
}

fn walk(base: &Path, dir: &Path, out: &mut Vec<VFile>) -> Result<(), String> {
    for e in std::fs::read_dir(dir).map_err(|e| e.to_string())?.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(base, &p, out)?;
        } else if let Ok(bytes) = std::fs::read(&p) {
            let rel = p.strip_prefix(base).unwrap_or(&p).to_string_lossy().replace('\\', "/");
            if rel.to_lowercase().ends_with(".zip") {
                unzip(&bytes, "", out, 0)?;
            } else {
                out.push((rel, bytes));
            }
        }
    }
    Ok(())
}

fn unzip(bytes: &[u8], prefix: &str, out: &mut Vec<VFile>, depth: u32) -> Result<(), String> {
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| format!("Not a readable zip: {e}"))?;
    for i in 0..z.len() {
        let mut f = z.by_index(i).map_err(|e| e.to_string())?;
        if f.is_dir() {
            continue;
        }
        let name = format!("{prefix}{}", f.name().replace('\\', "/"));
        let mut buf = vec![];
        f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        if name.to_lowercase().ends_with(".zip") && depth < 3 {
            // Notion puts the real export inside another zip
            unzip(&buf, "", out, depth + 1)?;
        } else {
            out.push((name, buf));
        }
    }
    Ok(())
}

fn text_of(b: &[u8]) -> String {
    let s = String::from_utf8_lossy(b).to_string();
    s.trim_start_matches('\u{feff}').to_string()
}

// ---------------------------------------------------------------- CSV

pub fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let mut rows = vec![];
    let mut row = vec![];
    let mut field = String::new();
    let mut in_q = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if in_q {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    in_q = false;
                }
            } else {
                field.push(c);
            }
        } else {
            match c {
                '"' => in_q = true,
                ',' => row.push(std::mem::take(&mut field)),
                '\r' => {}
                '\n' => {
                    row.push(std::mem::take(&mut field));
                    if !(row.len() == 1 && row[0].is_empty()) {
                        rows.push(std::mem::take(&mut row));
                    } else {
                        row.clear();
                    }
                }
                _ => field.push(c),
            }
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

fn csv_escape(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

pub fn database_to_csv(db: &Database) -> String {
    let mut s = db.columns.iter().map(|c| csv_escape(&c.name)).collect::<Vec<_>>().join(",");
    s.push('\n');
    for r in &db.rows {
        s.push_str(&db.columns.iter().map(|c| csv_escape(r.cells.get(&c.id).map(|x| x.as_str()).unwrap_or(""))).collect::<Vec<_>>().join(","));
        s.push('\n');
    }
    s
}

// ---------------------------------------------------------------- Notion

fn is_hex_id(s: &str) -> bool {
    s.len() >= 20 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// "Meeting notes 1a2b...(32 hex)" -> "Meeting notes"
pub fn strip_notion_id(stem: &str) -> String {
    let s = stem.trim();
    if let Some((head, last)) = s.rsplit_once(' ') {
        if is_hex_id(last) {
            return head.trim().to_string();
        }
    }
    s.to_string()
}

fn notion_id(stem: &str) -> Option<String> {
    stem.trim().rsplit_once(' ').map(|(_, l)| l).filter(|l| is_hex_id(l)).map(|l| l.to_string())
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = vec![];
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() && bytes[i + 1].is_ascii_hexdigit() && bytes[i + 2].is_ascii_hexdigit() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

/// [Title](Some%20Page%20abc123.md) -> [[Title]]
fn convert_links(md: &str) -> String {
    let mut out = String::with_capacity(md.len());
    let mut rest = md;
    while let Some(i) = rest.find('[') {
        let (before, after) = rest.split_at(i);
        out.push_str(before);
        let is_image = before.ends_with('!');
        if let Some(close) = after.find("](") {
            let label = &after[1..close];
            let tail = &after[close + 2..];
            if let Some(end) = tail.find(')') {
                let url = &tail[..end];
                if !is_image && !label.contains('\n') && (url.ends_with(".md") || url.ends_with(".csv")) {
                    let target = percent_decode(url.rsplit('/').next().unwrap_or(url));
                    let stem = target.trim_end_matches(".md").trim_end_matches(".csv");
                    let t = strip_notion_id(stem);
                    out.push_str(&format!("[[{}]]", if label.trim().is_empty() { t } else { label.trim().to_string() }));
                    rest = &tail[end + 1..];
                    continue;
                }
            }
        }
        out.push('[');
        rest = &after[1..];
    }
    out.push_str(rest);
    out
}

fn infer_kind(values: &[&str]) -> (ColKind, Vec<String>) {
    let non_empty: Vec<&str> = values.iter().copied().filter(|v| !v.trim().is_empty()).collect();
    if non_empty.is_empty() {
        return (ColKind::Text, vec![]);
    }
    if non_empty.iter().all(|v| matches!(v.to_lowercase().as_str(), "yes" | "no" | "true" | "false")) {
        return (ColKind::Checkbox, vec![]);
    }
    if non_empty.iter().all(|v| v.replace(',', "").parse::<f64>().is_ok()) {
        return (ColKind::Number, vec![]);
    }
    if non_empty.iter().all(|v| parse_loose_date(v).is_some()) {
        return (ColKind::Date, vec![]);
    }
    if non_empty.iter().all(|v| v.starts_with("http://") || v.starts_with("https://")) {
        return (ColKind::Url, vec![]);
    }
    let mut distinct: Vec<String> = vec![];
    for v in &non_empty {
        if !distinct.iter().any(|d| d == v) {
            distinct.push(v.to_string());
        }
    }
    if distinct.len() <= 8 && non_empty.len() >= 3 && distinct.len() < non_empty.len() && distinct.iter().all(|d| d.len() < 40) {
        return (ColKind::Select, distinct);
    }
    (ColKind::Text, vec![])
}

fn parse_loose_date(v: &str) -> Option<NaiveDate> {
    let v = v.trim();
    let head = v.split(['(', '→']).next().unwrap_or(v).trim();
    for f in ["%Y-%m-%d", "%B %d, %Y", "%b %d, %Y", "%d/%m/%Y", "%m/%d/%Y", "%d %B %Y", "%B %d, %Y %I:%M %p"] {
        if let Ok(d) = NaiveDate::parse_from_str(head, f) {
            return Some(d);
        }
        if let Ok(dt) = NaiveDateTime::parse_from_str(head, f) {
            return Some(dt.date());
        }
    }
    // "November 5, 2026 10:00 AM"
    let words: Vec<&str> = head.split_whitespace().collect();
    if words.len() >= 3 {
        let first3 = words[..3].join(" ");
        if let Ok(d) = NaiveDate::parse_from_str(&first3, "%B %d, %Y") {
            return Some(d);
        }
    }
    None
}

pub fn import_notion(path: &Path, d: &mut WorkspaceData) -> Result<ImportReport, String> {
    let files = collect_files(path)?;
    let mut rep = ImportReport::default();
    let t = now_ms();
    // key: path without extension -> page id
    let mut page_keys: HashMap<String, Id> = HashMap::new();
    let mut md_files: Vec<(String, String)> = vec![];
    let mut csv_files: Vec<(String, String)> = vec![];
    for (p, b) in &files {
        let lower = p.to_lowercase();
        if lower.ends_with(".md") {
            md_files.push((p.clone(), text_of(b)));
        } else if lower.ends_with(".csv") {
            csv_files.push((p.clone(), text_of(b)));
        }
    }
    if md_files.is_empty() && csv_files.is_empty() {
        return Err("No Markdown or CSV files were found. In Notion choose Settings → Export → Markdown & CSV, then pick the zip it gives you.".into());
    }
    md_files.sort_by(|a, b| a.0.cmp(&b.0));
    for (p, text) in &md_files {
        let key = p.trim_end_matches(".md").to_string();
        let stem = key.rsplit('/').next().unwrap_or(&key).to_string();
        let id = notion_id(&stem).map(|h| format!("n{}", &h[..h.len().min(24)])).unwrap_or_else(new_id);
        if d.page(&id).is_some() {
            rep.skipped += 1;
            page_keys.insert(key, id);
            continue;
        }
        let converted = convert_links(text);
        let (title, blocks) = markdown_to_blocks(&converted);
        let mut page = Page::new(title.unwrap_or_else(|| strip_notion_id(&stem)));
        page.id = id.clone();
        page.blocks = blocks;
        page.created = t;
        page.updated = t;
        page.tags = vec!["Imported from Notion".into()];
        d.pages.push(page);
        page_keys.insert(key, id);
        rep.pages += 1;
    }
    // parents: "A/B hex/C hex.md" has parent "A/B hex" (if that page exists)
    let keys: Vec<(String, Id)> = page_keys.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    for (k, id) in &keys {
        if let Some((dir, _)) = k.rsplit_once('/') {
            if let Some(pid) = page_keys.get(dir) {
                if pid != id {
                    if let Some(p) = d.page_mut(id) {
                        if p.parent.is_none() {
                            p.parent = Some(pid.clone());
                        }
                    }
                }
            }
        }
    }
    // databases: prefer "X_all.csv" over "X.csv"
    let names: Vec<String> = csv_files.iter().map(|(p, _)| p.clone()).collect();
    for (p, text) in &csv_files {
        let key = p.trim_end_matches(".csv").to_string();
        if !key.ends_with("_all") && names.contains(&format!("{key}_all.csv")) {
            continue;
        }
        let key = key.trim_end_matches("_all").to_string();
        let stem = key.rsplit('/').next().unwrap_or(&key).to_string();
        let rows = parse_csv(text);
        if rows.is_empty() {
            continue;
        }
        let id = notion_id(&stem).map(|h| format!("d{}", &h[..h.len().min(24)])).unwrap_or_else(new_id);
        if d.database(&id).is_some() {
            rep.skipped += 1;
            continue;
        }
        let header = &rows[0];
        let mut columns = vec![];
        for (ci, name) in header.iter().enumerate() {
            let vals: Vec<&str> = rows.iter().skip(1).map(|r| r.get(ci).map(|s| s.as_str()).unwrap_or("")).collect();
            let (kind, options) = if ci == 0 { (ColKind::Text, vec![]) } else { infer_kind(&vals) };
            columns.push(Column { id: new_id(), name: if name.trim().is_empty() { format!("Column {}", ci + 1) } else { name.trim().to_string() }, kind, options, width: if ci == 0 { 220.0 } else { 150.0 }, target: String::new() });
        }
        let mut db_rows = vec![];
        for r in rows.iter().skip(1) {
            let mut cells = BTreeMap::new();
            for (ci, c) in columns.iter().enumerate() {
                let raw = r.get(ci).cloned().unwrap_or_default();
                let v = match c.kind {
                    ColKind::Checkbox => (matches!(raw.to_lowercase().as_str(), "yes" | "true")).to_string(),
                    ColKind::Date => parse_loose_date(&raw).map(|d| d.to_string()).unwrap_or_default(),
                    ColKind::Number => raw.replace(',', ""),
                    _ => raw,
                };
                cells.insert(c.id.clone(), v);
            }
            // link to the row's page if Notion exported one
            let title = cells.get(&columns[0].id).cloned().unwrap_or_default();
            let page_id = page_keys.iter().find(|(k, _)| k.starts_with(&format!("{key}/")) && strip_notion_id(k.rsplit('/').next().unwrap_or("")) == title).map(|(_, v)| v.clone());
            if let Some(pid) = &page_id {
                if let Some(pg) = d.page_mut(pid) {
                    pg.parent = None;
                }
            }
            db_rows.push(Row { id: new_id(), cells, page_id });
        }
        let group_by = columns.iter().find(|c| c.kind == ColKind::Select).map(|c| c.id.clone());
        d.databases.push(Database { id, name: strip_notion_id(&stem), icon: "▦".into(), columns, rows: db_rows, view: DbView::Table, group_by, updated: t, ..Default::default() });
        rep.databases += 1;
    }
    Ok(rep)
}

// ---------------------------------------------------------------- the CSV import dialog (C-E18)

/// A CSV read for the import dialog: its header, its rows, and a type guessed for each column.
#[derive(Clone, Debug, Default)]
pub struct CsvPlan {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub kinds: Vec<ColKind>,
    /// The options of the columns guessed as Select.
    pub options: Vec<Vec<String>>,
}

/// A column's type when most of its values fit it (4 in 5), so the few that don't can be listed
/// as failed rows rather than turning the whole column into text.
fn infer_mostly(values: &[&str]) -> (ColKind, Vec<String>) {
    let non_empty: Vec<&str> = values.iter().copied().filter(|v| !v.trim().is_empty()).collect();
    if non_empty.len() >= 3 {
        let share = |ok: usize| ok * 5 >= non_empty.len() * 4;
        let nums = non_empty.iter().filter(|v| v.replace(',', "").parse::<f64>().is_ok()).count();
        if share(nums) && nums < non_empty.len() {
            return (ColKind::Number, vec![]);
        }
        let dates = non_empty.iter().filter(|v| parse_loose_date(v).is_some()).count();
        if share(dates) && dates < non_empty.len() {
            return (ColKind::Date, vec![]);
        }
    }
    infer_kind(values)
}

pub fn csv_plan(text: &str) -> Result<CsvPlan, String> {
    let rows = parse_csv(text.trim_start_matches('\u{feff}'));
    let Some(header) = rows.first().cloned() else { return Err("The file is empty.".into()) };
    let body: Vec<Vec<String>> = rows.into_iter().skip(1).filter(|r| r.iter().any(|c| !c.trim().is_empty())).collect();
    let mut kinds = vec![];
    let mut options = vec![];
    for ci in 0..header.len() {
        let vals: Vec<&str> = body.iter().map(|r| r.get(ci).map(|s| s.as_str()).unwrap_or("")).collect();
        let (k, o) = infer_mostly(&vals);
        kinds.push(k);
        options.push(o);
    }
    let header = header.iter().enumerate().map(|(i, h)| if h.trim().is_empty() { format!("Column {}", i + 1) } else { h.trim().to_string() }).collect();
    Ok(CsvPlan { header, rows: body, kinds, options })
}

/// Which rows import, and which fail and why (the rule from the Phase 4 checks: a row fails when
/// it has more cells than the header has columns, or when a value in a date or number column
/// can't be read as one). Failed rows are numbered as lines of the file (the header is line 1).
pub fn csv_check(plan: &CsvPlan, kinds: &[ColKind]) -> (Vec<usize>, Vec<(usize, String)>) {
    let (mut ok, mut failed) = (vec![], vec![]);
    for (i, r) in plan.rows.iter().enumerate() {
        let line = i + 2;
        if r.len() > plan.header.len() {
            failed.push((line, format!("it has {} cells but the file has {} columns", r.len(), plan.header.len())));
            continue;
        }
        let mut why = None;
        for (ci, raw) in r.iter().enumerate() {
            let raw = raw.trim();
            if raw.is_empty() {
                continue;
            }
            let name = plan.header.get(ci).cloned().unwrap_or_default();
            match kinds.get(ci) {
                Some(ColKind::Number) if raw.replace(',', "").parse::<f64>().is_err() => why = Some(format!("\"{raw}\" in {name} isn't a number")),
                Some(ColKind::Date) if parse_loose_date(raw).is_none() => why = Some(format!("\"{raw}\" in {name} isn't a date")),
                _ => {}
            }
            if why.is_some() {
                break;
            }
        }
        match why {
            Some(w) => failed.push((line, w)),
            None => ok.push(i),
        }
    }
    (ok, failed)
}

/// A cell as stored. `lenient` keeps a value that doesn't fit its column as text (Import them
/// anyway) instead of storing it empty.
pub fn csv_cell(kind: ColKind, raw: &str, lenient: bool) -> String {
    match kind {
        ColKind::Checkbox => (matches!(raw.trim().to_lowercase().as_str(), "yes" | "true")).to_string(),
        ColKind::Date => parse_loose_date(raw).map(|d| d.to_string()).unwrap_or_else(|| if lenient { raw.trim().to_string() } else { String::new() }),
        ColKind::Number => {
            let v = raw.trim().replace(',', "");
            if v.parse::<f64>().is_ok() || lenient {
                if v.parse::<f64>().is_ok() {
                    v
                } else {
                    raw.trim().to_string()
                }
            } else {
                String::new()
            }
        }
        _ => raw.to_string(),
    }
}

/// A new database from the plan's rows `take`.
pub fn csv_new_database(name: &str, plan: &CsvPlan, kinds: &[ColKind], take: &[usize], lenient: bool) -> Database {
    let columns: Vec<Column> = plan
        .header
        .iter()
        .enumerate()
        .map(|(ci, h)| {
            let kind = kinds.get(ci).copied().unwrap_or(ColKind::Text);
            let options = if kind == ColKind::Select {
                let mut o: Vec<String> = plan.options.get(ci).cloned().unwrap_or_default();
                for r in &plan.rows {
                    if let Some(v) = r.get(ci).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
                        if !o.contains(&v) {
                            o.push(v);
                        }
                    }
                }
                o
            } else {
                vec![]
            };
            Column { id: new_id(), name: h.clone(), kind, options, width: if ci == 0 { 220.0 } else { 150.0 }, target: String::new() }
        })
        .collect();
    let mut db = Database { id: new_id(), name: name.to_string(), icon: "▦".into(), columns, rows: vec![], view: DbView::Table, group_by: None, updated: now_ms(), ..Default::default() };
    csv_append(&mut db, plan, kinds, take, lenient);
    db.group_by = db.columns.iter().find(|c| c.kind == ColKind::Select).map(|c| c.id.clone());
    db
}

/// Add the plan's rows `take` to a database, matching columns by name; a column it doesn't have
/// is added. Returns how many rows were added.
pub fn csv_append(db: &mut Database, plan: &CsvPlan, kinds: &[ColKind], take: &[usize], lenient: bool) -> usize {
    let mut ids = vec![];
    for (ci, h) in plan.header.iter().enumerate() {
        let id = match db.columns.iter().find(|c| c.name.eq_ignore_ascii_case(h)) {
            Some(c) => c.id.clone(),
            None => {
                let c = Column { id: new_id(), name: h.clone(), kind: kinds.get(ci).copied().unwrap_or(ColKind::Text), options: vec![], width: 150.0, target: String::new() };
                let id = c.id.clone();
                db.columns.push(c);
                id
            }
        };
        ids.push(id);
    }
    let mut n = 0;
    for &i in take {
        let Some(r) = plan.rows.get(i) else { continue };
        let mut cells = BTreeMap::new();
        for (ci, cid) in ids.iter().enumerate() {
            let kind = db.columns.iter().find(|c| &c.id == cid).map(|c| c.kind).unwrap_or(ColKind::Text);
            cells.insert(cid.clone(), csv_cell(kind, r.get(ci).map(|s| s.as_str()).unwrap_or(""), lenient));
        }
        db.rows.push(Row { id: new_id(), cells, page_id: None });
        n += 1;
    }
    db.updated = now_ms();
    n
}

/// Turn CSV text (first row = headers) into a database with inferred column types.
pub fn csv_to_database(name: &str, text: &str) -> Result<Database, String> {
    let rows = parse_csv(text.trim_start_matches('\u{feff}'));
    if rows.is_empty() {
        return Err("The CSV is empty".into());
    }
    let header = &rows[0];
    let mut columns = vec![];
    for (ci, h) in header.iter().enumerate() {
        let vals: Vec<&str> = rows.iter().skip(1).map(|r| r.get(ci).map(|s| s.as_str()).unwrap_or("")).collect();
        let (kind, options) = if ci == 0 { (ColKind::Text, vec![]) } else { infer_kind(&vals) };
        columns.push(Column { id: new_id(), name: if h.trim().is_empty() { format!("Column {}", ci + 1) } else { h.trim().to_string() }, kind, options, width: if ci == 0 { 220.0 } else { 150.0 }, target: String::new() });
    }
    let mut out_rows = vec![];
    for r in rows.iter().skip(1) {
        let mut cells = BTreeMap::new();
        for (ci, c) in columns.iter().enumerate() {
            let raw = r.get(ci).cloned().unwrap_or_default();
            let v = match c.kind {
                ColKind::Checkbox => (matches!(raw.to_lowercase().as_str(), "yes" | "true")).to_string(),
                ColKind::Date => parse_loose_date(&raw).map(|d| d.to_string()).unwrap_or_default(),
                ColKind::Number => raw.replace(',', ""),
                _ => raw,
            };
            cells.insert(c.id.clone(), v);
        }
        out_rows.push(Row { id: new_id(), cells, page_id: None });
    }
    let group_by = columns.iter().find(|c| c.kind == ColKind::Select).map(|c| c.id.clone());
    Ok(Database { id: new_id(), name: name.to_string(), icon: "▦".into(), columns, rows: out_rows, view: DbView::Table, group_by, updated: now_ms(), ..Default::default() })
}

// ---------------------------------------------------------------- Todoist

pub fn import_todoist(path: &Path, d: &mut WorkspaceData) -> Result<ImportReport, String> {
    let files = collect_files(path)?;
    let mut rep = ImportReport::default();
    let csvs: Vec<&VFile> = files.iter().filter(|(p, _)| p.to_lowercase().ends_with(".csv")).collect();
    if csvs.is_empty() {
        return Err("No CSV files found. In Todoist open a project → ⋯ → Export as CSV, or Settings → Backups → Download.".into());
    }
    let today = today();
    let palette: [[u8; 3]; 6] = [[219, 76, 63], [255, 153, 51], [41, 171, 135], [20, 170, 245], [175, 56, 235], [136, 77, 255]];
    for (p, bytes) in csvs {
        let rows = parse_csv(&text_of(bytes));
        if rows.is_empty() {
            continue;
        }
        let header: Vec<String> = rows[0].iter().map(|h| h.trim().to_uppercase()).collect();
        let col = |name: &str| header.iter().position(|h| h == name);
        let (Some(ci_type), Some(ci_content)) = (col("TYPE"), col("CONTENT")) else {
            rep.notes.push(format!("{p}: not a Todoist CSV (no TYPE/CONTENT columns)"));
            continue;
        };
        let ci_desc = col("DESCRIPTION");
        let ci_prio = col("PRIORITY");
        let ci_indent = col("INDENT");
        let ci_date = col("DATE");
        let ci_dur = col("DURATION");
        // project from the file name
        let stem = p.rsplit('/').next().unwrap_or(p).trim_end_matches(".csv").trim_end_matches(".CSV");
        let mut pname = stem.to_string();
        if let Some(i) = pname.find(" [") {
            pname.truncate(i);
        }
        let pname = pname.trim().to_string();
        let project_id: Option<Id> = if pname.eq_ignore_ascii_case("inbox") {
            None
        } else if let Some(pr) = d.project_by_name(&pname) {
            Some(pr.id.clone())
        } else {
            let pr = Project {
                id: new_id(),
                name: pname.clone(),
                color: palette[d.projects.len() % palette.len()],
                sections: vec![],
                page_id: None,
                archived: false,
                favorite: false,
                order: d.projects.len() as i64 + 1,
                updated: now_ms(),
            };
            let id = pr.id.clone();
            d.projects.push(pr);
            rep.projects += 1;
            Some(id)
        };
        let mut section = String::new();
        let mut stack: Vec<(usize, Id)> = vec![];
        let mut last_task: Option<Id> = None;
        let mut order = now_ms();
        for r in rows.iter().skip(1) {
            let get = |i: Option<usize>| i.and_then(|i| r.get(i)).map(|s| s.trim().to_string()).unwrap_or_default();
            let kind = get(Some(ci_type)).to_lowercase();
            let content = get(Some(ci_content));
            match kind.as_str() {
                "section" => {
                    section = content.clone();
                    if let Some(pid) = &project_id {
                        if let Some(pr) = d.projects.iter_mut().find(|x| &x.id == pid) {
                            if !pr.sections.contains(&section) {
                                pr.sections.push(section.clone());
                                pr.updated = now_ms();
                            }
                        }
                    }
                    stack.clear();
                }
                "note" => {
                    if let Some(tid) = &last_task {
                        if let Some(t) = d.task_mut(tid) {
                            if !t.notes.is_empty() {
                                t.notes.push_str("\n\n");
                            }
                            t.notes.push_str(&content);
                        }
                    }
                }
                "task" => {
                    if content.is_empty() {
                        continue;
                    }
                    let indent: usize = get(ci_indent).parse().unwrap_or(1).max(1);
                    while stack.last().map_or(false, |(lvl, _)| *lvl >= indent) {
                        stack.pop();
                    }
                    let parent = stack.last().map(|(_, id)| id.clone());
                    // labels written inline as @label
                    let cap = crate::nlp::parse(&content, today);
                    let mut title = content.clone();
                    let labels: Vec<String> = content.split_whitespace().filter(|w| w.starts_with('@') && w.len() > 1).map(|w| w[1..].to_string()).collect();
                    for l in &labels {
                        title = title.replace(&format!("@{l}"), "");
                    }
                    let title = title.split_whitespace().collect::<Vec<_>>().join(" ");
                    let _ = cap;
                    if d.tasks.iter().any(|t| t.title == title && t.project_id == project_id && t.parent_id == parent && !t.completed) {
                        rep.skipped += 1;
                        if let Some(t) = d.tasks.iter().find(|t| t.title == title && t.project_id == project_id) {
                            stack.push((indent, t.id.clone()));
                        }
                        continue;
                    }
                    let mut task = Task::new(title);
                    task.project_id = project_id.clone();
                    task.section = section.clone();
                    task.parent_id = parent;
                    task.labels = labels;
                    task.notes = get(ci_desc);
                    // ASSUMPTION: the CSV writes 1 for P1 (most urgent) .. 4 for P4.
                    task.priority = get(ci_prio).parse::<u8>().ok().filter(|p| (1..=4).contains(p)).unwrap_or(4);
                    let date = get(ci_date);
                    if !date.is_empty() {
                        let dc = crate::nlp::parse(&date, today);
                        task.due = dc.due.or_else(|| parse_loose_date(&date));
                        task.due_time = dc.time;
                        task.recurrence = dc.recurrence;
                        if task.due.is_none() {
                            rep.notes.push(format!("Couldn't read the date \"{date}\" for \"{}\"", task.title));
                        }
                    }
                    if let Ok(m) = get(ci_dur).parse::<u32>() {
                        if m > 0 {
                            task.duration_min = Some(m);
                        }
                    }
                    order += 1;
                    task.order = order;
                    let id = task.id.clone();
                    d.tasks.push(task);
                    stack.push((indent, id.clone()));
                    last_task = Some(id);
                    rep.tasks += 1;
                }
                _ => {}
            }
        }
    }
    Ok(rep)
}

// ---------------------------------------------------------------- iCalendar

fn ics_unescape(s: &str) -> String {
    s.replace("\\n", "\n").replace("\\N", "\n").replace("\\,", ",").replace("\\;", ";").replace("\\\\", "\\")
}

fn ics_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace(';', "\\;").replace(',', "\\,").replace('\n', "\\n")
}

/// Returns (datetime in local time, is_date_only)
fn parse_ics_dt(params: &str, value: &str) -> Option<(NaiveDateTime, bool)> {
    let v = value.trim();
    if params.contains("VALUE=DATE") && !params.contains("DATE-TIME") || (v.len() == 8 && v.chars().all(|c| c.is_ascii_digit())) {
        let d = NaiveDate::parse_from_str(&v[..8.min(v.len())], "%Y%m%d").ok()?;
        return Some((d.and_hms_opt(0, 0, 0)?, true));
    }
    if let Some(u) = v.strip_suffix('Z') {
        let dt = NaiveDateTime::parse_from_str(u, "%Y%m%dT%H%M%S").ok()?;
        let local = chrono::Utc.from_utc_datetime(&dt).with_timezone(&chrono::Local);
        return Some((local.naive_local(), false));
    }
    // TZID times are taken as wall-clock time (usually the same zone as this PC)
    let dt = NaiveDateTime::parse_from_str(v, "%Y%m%dT%H%M%S").or_else(|_| NaiveDateTime::parse_from_str(v, "%Y%m%dT%H%M")).ok()?;
    Some((dt, false))
}

fn parse_rrule(v: &str) -> Recurrence {
    let mut freq = "";
    let mut interval = 1u32;
    let mut byday = "";
    for part in v.split(';') {
        if let Some((k, val)) = part.split_once('=') {
            match k {
                "FREQ" => freq = val,
                "INTERVAL" => interval = val.parse().unwrap_or(1),
                "BYDAY" => byday = val,
                _ => {}
            }
        }
    }
    match (freq, interval) {
        ("DAILY", 1) => Recurrence::Daily,
        ("DAILY", n) => Recurrence::EveryDays(n),
        ("WEEKLY", 1) => {
            let days: Vec<&str> = byday.split(',').filter(|s| !s.is_empty()).collect();
            if days.len() == 5 && ["MO", "TU", "WE", "TH", "FR"].iter().all(|d| days.contains(d)) {
                Recurrence::Weekdays
            } else {
                Recurrence::Weekly
            }
        }
        ("WEEKLY", n) => Recurrence::EveryWeeks(n),
        ("MONTHLY", _) => Recurrence::Monthly,
        ("YEARLY", _) => Recurrence::Yearly,
        _ => Recurrence::None,
    }
}

fn stable_id(prefix: &str, s: &str) -> String {
    // FNV-1a, so re-importing the same file updates instead of duplicating
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    format!("{prefix}{h:016x}")
}

/// Parse the events of one .ics text for calendar `cal_id`. Returns (X-WR-CALNAME, events).
pub fn ics_events(text: &str, cal_id: &str) -> (Option<String>, Vec<Event>) {
    let mut lines: Vec<String> = vec![];
    for raw in text.split('\n') {
        let l = raw.trim_end_matches('\r');
        if (l.starts_with(' ') || l.starts_with('\t')) && !lines.is_empty() {
            lines.last_mut().unwrap().push_str(&l[1..]);
        } else {
            lines.push(l.to_string());
        }
    }
    let cal_name = lines.iter().find(|l| l.starts_with("X-WR-CALNAME:")).map(|l| ics_unescape(&l["X-WR-CALNAME:".len()..]));
    let mut out = vec![];
    let mut cur: Option<HashMap<String, (String, String)>> = None;
    let mut depth_alarm = 0;
    let mut cancelled_overrides: std::collections::HashSet<String> = Default::default();
    for l in &lines {
        if l == "BEGIN:VEVENT" {
            cur = Some(HashMap::new());
            continue;
        }
        if l == "BEGIN:VALARM" {
            depth_alarm += 1;
            continue;
        }
        if l == "END:VALARM" {
            depth_alarm -= 1;
            continue;
        }
        if l == "END:VEVENT" {
            if let Some(props) = cur.take() {
                let get = |k: &str| props.get(k).cloned().unwrap_or_default();
                if get("STATUS").1.eq_ignore_ascii_case("CANCELLED") {
                    cancelled_overrides.insert(get("UID").1);
                    continue;
                }
                let (sp, sv) = get("DTSTART");
                let Some((start, date_only)) = parse_ics_dt(&sp, &sv) else { continue };
                let (ep, ev) = get("DTEND");
                let end = parse_ics_dt(&ep, &ev).map(|x| x.0).unwrap_or(if date_only { start + chrono::Duration::days(1) } else { start + chrono::Duration::hours(1) });
                let uid = get("UID").1;
                let recur_id = get("RECURRENCE-ID").1;
                let id = stable_id("i", &format!("{cal_id}{uid}{sv}{recur_id}"));
                let title = ics_unescape(&get("SUMMARY").1);
                out.push(Event {
                    id,
                    calendar_id: cal_id.to_string(),
                    title: if title.is_empty() { "(No title)".into() } else { title },
                    start,
                    end: end.max(start + chrono::Duration::minutes(15)),
                    all_day: date_only,
                    location: ics_unescape(&get("LOCATION").1),
                    notes: ics_unescape(&get("DESCRIPTION").1),
                    recurrence: parse_rrule(&get("RRULE").1),
                    page_id: None,
                    reminder_min: None,
                    updated: now_ms(),
                });
            }
            continue;
        }
        if depth_alarm > 0 {
            continue;
        }
        if let Some(props) = cur.as_mut() {
            if let Some((left, value)) = l.split_once(':') {
                let (name, params) = left.split_once(';').unwrap_or((left, ""));
                props.entry(name.to_uppercase()).or_insert((params.to_string(), value.to_string()));
            }
        }
    }
    (cal_name, out)
}

pub fn import_ics(path: &Path, d: &mut WorkspaceData) -> Result<ImportReport, String> {
    let files = collect_files(path)?;
    let mut rep = ImportReport::default();
    let icss: Vec<&VFile> = files.iter().filter(|(p, _)| p.to_lowercase().ends_with(".ics")).collect();
    if icss.is_empty() {
        return Err("No .ics calendar files found. In Google Calendar use Settings → Import & export → Export.".into());
    }
    let palette: [[u8; 3]; 6] = [[3, 155, 229], [63, 81, 181], [11, 128, 67], [244, 81, 30], [142, 36, 170], [246, 191, 38]];
    for (p, bytes) in icss {
        let text = text_of(bytes);
        let stem = p.rsplit('/').next().unwrap_or(p).trim_end_matches(".ics").to_string();
        let (name, _) = ics_events(&text, "");
        let cal_name = name.unwrap_or_else(|| stem.split('_').next().unwrap_or(&stem).to_string());
        let cal_id = if let Some(c) = d.calendars.iter().find(|c| c.name == cal_name) {
            c.id.clone()
        } else {
            let c = Calendar { id: new_id(), name: cal_name.clone(), color: palette[d.calendars.len() % palette.len()], visible: true, updated: now_ms(), ..Default::default() };
            let id = c.id.clone();
            d.calendars.push(c);
            rep.calendars += 1;
            id
        };
        let (_, events) = ics_events(&text, &cal_id);
        for e in events {
            if d.events.iter().any(|x| x.id == e.id) {
                rep.skipped += 1;
                continue;
            }
            d.events.push(e);
            rep.events += 1;
        }
    }
    Ok(rep)
}

/// Download text over HTTP(S). Uses Windows' built-in curl.exe so no TLS library is needed.
pub fn fetch_text(url: &str) -> Result<String, String> {
    let url = url.trim().replace("webcal://", "https://");
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("The link must start with https://".into());
    }
    let out = crate::ai::cmd("curl.exe")
        .args(["-sSL", "--max-time", "40", "-A", "Resolve 2-calendar/1.0", &url])
        .output()
        .map_err(|e| format!("Couldn't run curl: {e}"))?;
    if !out.status.success() {
        return Err(format!("Download failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    if !text.contains("BEGIN:VCALENDAR") {
        return Err("That link didn't return a calendar (.ics). Check it's the secret iCal address.".into());
    }
    Ok(text)
}

/// Replace a subscribed calendar's events with a fresh download. Returns the number of events.
pub fn apply_subscription(d: &mut WorkspaceData, cal_id: &str, text: &str) -> usize {
    let (_, events) = ics_events(text, cal_id);
    let fresh: std::collections::HashSet<String> = events.iter().map(|e| e.id.clone()).collect();
    let stale: Vec<Id> = d.events.iter().filter(|e| e.calendar_id == cal_id && !fresh.contains(&e.id)).map(|e| e.id.clone()).collect();
    for s in stale {
        d.delete_event(&s);
    }
    let n = events.len();
    for e in events {
        match d.events.iter_mut().find(|x| x.id == e.id) {
            Some(x) => {
                if x.title != e.title || x.start != e.start || x.end != e.end || x.location != e.location || x.notes != e.notes || x.recurrence != e.recurrence {
                    *x = e;
                }
            }
            None => d.events.push(e),
        }
    }
    if let Some(c) = d.calendars.iter_mut().find(|c| c.id == cal_id) {
        c.last_fetch = now_ms();
        c.updated = now_ms();
    }
    n
}

pub fn events_to_ics(d: &WorkspaceData) -> String {
    let mut s = String::from("BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Resolve 2//EN\r\n");
    for e in &d.events {
        s.push_str("BEGIN:VEVENT\r\n");
        s.push_str(&format!("UID:{}@resolve2\r\n", e.id));
        if e.all_day {
            s.push_str(&format!("DTSTART;VALUE=DATE:{}\r\n", e.start.format("%Y%m%d")));
            s.push_str(&format!("DTEND;VALUE=DATE:{}\r\n", e.end.format("%Y%m%d")));
        } else {
            s.push_str(&format!("DTSTART:{}\r\n", e.start.format("%Y%m%dT%H%M%S")));
            s.push_str(&format!("DTEND:{}\r\n", e.end.format("%Y%m%dT%H%M%S")));
        }
        s.push_str(&format!("SUMMARY:{}\r\n", ics_escape(&e.title)));
        if !e.location.is_empty() {
            s.push_str(&format!("LOCATION:{}\r\n", ics_escape(&e.location)));
        }
        if !e.notes.is_empty() {
            s.push_str(&format!("DESCRIPTION:{}\r\n", ics_escape(&e.notes)));
        }
        if let Some(cal) = d.calendar(&e.calendar_id) {
            s.push_str(&format!("CATEGORIES:{}\r\n", ics_escape(&cal.name)));
        }
        let rr = match e.recurrence {
            Recurrence::None => None,
            Recurrence::Daily => Some("FREQ=DAILY".to_string()),
            Recurrence::Weekdays => Some("FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR".to_string()),
            Recurrence::Weekly => Some("FREQ=WEEKLY".to_string()),
            Recurrence::Monthly => Some("FREQ=MONTHLY".to_string()),
            Recurrence::Yearly => Some("FREQ=YEARLY".to_string()),
            Recurrence::EveryDays(n) => Some(format!("FREQ=DAILY;INTERVAL={n}")),
            Recurrence::EveryWeeks(n) => Some(format!("FREQ=WEEKLY;INTERVAL={n}")),
        };
        if let Some(r) = rr {
            s.push_str(&format!("RRULE:{r}\r\n"));
        }
        s.push_str("END:VEVENT\r\n");
    }
    // dated tasks as all-day to-dos so other calendars can show them
    for t in d.tasks.iter().filter(|t| !t.completed && t.due.is_some()) {
        let due = t.due.unwrap();
        s.push_str("BEGIN:VTODO\r\n");
        s.push_str(&format!("UID:{}@resolve2\r\n", t.id));
        s.push_str(&format!("SUMMARY:{}\r\n", ics_escape(&t.title)));
        match t.due_time {
            Some(tm) => s.push_str(&format!("DUE:{}\r\n", due.and_time(tm).format("%Y%m%dT%H%M%S"))),
            None => s.push_str(&format!("DUE;VALUE=DATE:{}\r\n", due.format("%Y%m%d"))),
        }
        s.push_str(&format!("PRIORITY:{}\r\n", match t.priority { 1 => 1, 2 => 3, 3 => 5, _ => 9 }));
        s.push_str("END:VTODO\r\n");
    }
    s.push_str("END:VCALENDAR\r\n");
    s
}

#[allow(dead_code)]
pub fn time_or_midnight(t: Option<NaiveTime>) -> NaiveTime {
    t.unwrap_or(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn csv_and_links() {
        let r = parse_csv("a,b\n\"x, y\",\"he said \"\"hi\"\"\"\n");
        assert_eq!(r[1][0], "x, y");
        assert_eq!(r[1][1], "he said \"hi\"");
        assert_eq!(convert_links("See [Plan](Plan%20abcdefabcdefabcdefabcdefabcdefab.md) now"), "See [[Plan]] now");
        assert_eq!(strip_notion_id("Notes 0123456789abcdef0123456789abcdef"), "Notes");
    }
}
