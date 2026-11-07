//! The workspace data model: pages, databases, projects, tasks, calendars and events.
//! Every item carries an `updated` timestamp so two copies (PC, phone, AI edits) can be merged.

use chrono::{Datelike, Local, Months, NaiveDate, NaiveDateTime, NaiveTime, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};

pub type Id = String;

pub fn now_ms() -> i64 {
    Local::now().timestamp_millis()
}

pub fn today() -> NaiveDate {
    now_local().date()
}

pub fn now_local() -> NaiveDateTime {
    Local::now().naive_local() + chrono::Duration::seconds(CLOCK_SHIFT_S.load(Ordering::Relaxed))
}

/// Screenshots only (--now): the app's clock reads `at` (the canvases are drawn on Friday
/// 6 November 2026, in the afternoon). Edit times (`now_ms`) stay real.
static CLOCK_SHIFT_S: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
pub fn set_clock(at: NaiveDateTime) {
    CLOCK_SHIFT_S.store((at - Local::now().naive_local()).num_seconds(), Ordering::Relaxed);
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Short unique id: time + process counter + a little randomness from the address space.
pub fn new_id() -> Id {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let c = COUNTER.fetch_add(1, Ordering::Relaxed);
    let salt = (&c as *const u64 as u64) ^ (std::process::id() as u64).rotate_left(17);
    let mix = t ^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ c.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    format!("{:x}{:04x}", mix & 0xFFFF_FFFF_FFFF, c & 0xFFFF)
}

// ---------------------------------------------------------------- pages

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BlockKind {
    #[default]
    Paragraph,
    H1,
    H2,
    H3,
    Bullet,
    Numbered,
    Todo,
    Quote,
    Code,
    Callout,
    Divider,
    /// `text` holds the id of the linked page.
    PageLink,
    /// `text` holds the id of an embedded database.
    Database,
}

impl BlockKind {
    pub const MENU: [BlockKind; 13] = [
        BlockKind::Paragraph,
        BlockKind::H1,
        BlockKind::H2,
        BlockKind::H3,
        BlockKind::Bullet,
        BlockKind::Numbered,
        BlockKind::Todo,
        BlockKind::Quote,
        BlockKind::Code,
        BlockKind::Callout,
        BlockKind::Divider,
        BlockKind::PageLink,
        BlockKind::Database,
    ];
    pub fn label(self) -> &'static str {
        match self {
            BlockKind::Paragraph => "Text",
            BlockKind::H1 => "Heading 1",
            BlockKind::H2 => "Heading 2",
            BlockKind::H3 => "Heading 3",
            BlockKind::Bullet => "Bulleted list",
            BlockKind::Numbered => "Numbered list",
            BlockKind::Todo => "To-do",
            BlockKind::Quote => "Quote",
            BlockKind::Code => "Code",
            BlockKind::Callout => "Callout",
            BlockKind::Divider => "Divider",
            BlockKind::PageLink => "Link to page",
            BlockKind::Database => "Embedded database",
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            BlockKind::Paragraph => "¶",
            BlockKind::H1 => "H1",
            BlockKind::H2 => "H2",
            BlockKind::H3 => "H3",
            BlockKind::Bullet => "•",
            BlockKind::Numbered => "1.",
            BlockKind::Todo => "☑",
            BlockKind::Quote => "❝",
            BlockKind::Code => "</>",
            BlockKind::Callout => "💡",
            BlockKind::Divider => "---",
            BlockKind::PageLink => "↗",
            BlockKind::Database => "▦",
        }
    }
    pub fn is_text(self) -> bool {
        !matches!(self, BlockKind::Divider | BlockKind::PageLink | BlockKind::Database)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Block {
    pub id: Id,
    pub kind: BlockKind,
    pub text: String,
    #[serde(default)]
    pub checked: bool,
    /// A to-do block can be tied to a task: ticking either one ticks the other.
    #[serde(default)]
    pub task_id: Option<Id>,
}

impl Block {
    pub fn new(kind: BlockKind, text: impl Into<String>) -> Self {
        Block { id: new_id(), kind, text: text.into(), checked: false, task_id: None }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Page {
    pub id: Id,
    pub title: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub parent: Option<Id>,
    pub blocks: Vec<Block>,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Templates are listed separately and copied when used.
    #[serde(default)]
    pub template: bool,
    pub created: i64,
    pub updated: i64,
}

impl Page {
    pub fn new(title: impl Into<String>) -> Self {
        let t = now_ms();
        Page {
            id: new_id(),
            title: title.into(),
            icon: "📄".into(),
            parent: None,
            blocks: vec![Block::new(BlockKind::Paragraph, "")],
            favorite: false,
            tags: vec![],
            template: false,
            created: t,
            updated: t,
        }
    }
    pub fn plain_text(&self) -> String {
        let mut s = String::new();
        for b in &self.blocks {
            if b.kind.is_text() {
                s.push_str(&b.text);
                s.push('\n');
            }
        }
        s
    }
    /// Titles referenced with [[Title]] in the page body.
    pub fn wiki_links(&self) -> Vec<String> {
        let mut out = vec![];
        for b in &self.blocks {
            if !b.kind.is_text() {
                continue;
            }
            let mut rest = b.text.as_str();
            while let Some(i) = rest.find("[[") {
                let after = &rest[i + 2..];
                if let Some(j) = after.find("]]") {
                    let t = after[..j].trim();
                    if !t.is_empty() && !out.iter().any(|x: &String| x.eq_ignore_ascii_case(t)) {
                        out.push(t.to_string());
                    }
                    rest = &after[j + 2..];
                } else {
                    break;
                }
            }
        }
        out
    }
}

// ---------------------------------------------------------------- recurrence

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Recurrence {
    #[default]
    None,
    Daily,
    Weekdays,
    Weekly,
    Monthly,
    Yearly,
    EveryDays(u32),
    EveryWeeks(u32),
}

impl Recurrence {
    pub const CHOICES: [Recurrence; 6] = [
        Recurrence::None,
        Recurrence::Daily,
        Recurrence::Weekdays,
        Recurrence::Weekly,
        Recurrence::Monthly,
        Recurrence::Yearly,
    ];
    pub fn label(self) -> String {
        match self {
            Recurrence::None => "Does not repeat".into(),
            Recurrence::Daily => "Every day".into(),
            Recurrence::Weekdays => "Every weekday".into(),
            Recurrence::Weekly => "Every week".into(),
            Recurrence::Monthly => "Every month".into(),
            Recurrence::Yearly => "Every year".into(),
            Recurrence::EveryDays(n) => format!("Every {n} days"),
            Recurrence::EveryWeeks(n) => format!("Every {n} weeks"),
        }
    }
    pub fn is_some(self) -> bool {
        self != Recurrence::None
    }
    /// Next occurrence strictly after `d`.
    pub fn next_after(self, d: NaiveDate) -> Option<NaiveDate> {
        match self {
            Recurrence::None => None,
            Recurrence::Daily => d.succ_opt(),
            Recurrence::Weekdays => {
                let mut n = d.succ_opt()?;
                while matches!(n.weekday(), Weekday::Sat | Weekday::Sun) {
                    n = n.succ_opt()?;
                }
                Some(n)
            }
            Recurrence::Weekly => d.checked_add_days(chrono::Days::new(7)),
            Recurrence::EveryDays(k) => d.checked_add_days(chrono::Days::new(k.max(1) as u64)),
            Recurrence::EveryWeeks(k) => d.checked_add_days(chrono::Days::new(7 * k.max(1) as u64)),
            Recurrence::Monthly => d.checked_add_months(Months::new(1)),
            Recurrence::Yearly => d.checked_add_months(Months::new(12)),
        }
    }
    /// Does a series that starts on `start` have an occurrence on `day`?
    pub fn occurs_on(self, start: NaiveDate, day: NaiveDate) -> bool {
        if day < start {
            return false;
        }
        let diff = (day - start).num_days();
        match self {
            Recurrence::None => day == start,
            Recurrence::Daily => true,
            Recurrence::Weekdays => !matches!(day.weekday(), Weekday::Sat | Weekday::Sun),
            Recurrence::Weekly => diff % 7 == 0,
            Recurrence::EveryDays(k) => diff % (k.max(1) as i64) == 0,
            Recurrence::EveryWeeks(k) => diff % (7 * k.max(1) as i64) == 0,
            Recurrence::Monthly => {
                day.day() == start.day()
                    || (start.day() > days_in_month(day) && day.day() == days_in_month(day))
            }
            Recurrence::Yearly => day.month() == start.month() && day.day() == start.day(),
        }
    }
}

pub fn days_in_month(d: NaiveDate) -> u32 {
    let first = NaiveDate::from_ymd_opt(d.year(), d.month(), 1).unwrap();
    let next = first.checked_add_months(Months::new(1)).unwrap();
    (next - first).num_days() as u32
}

// ---------------------------------------------------------------- tasks

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Project {
    pub id: Id,
    pub name: String,
    pub color: [u8; 3],
    #[serde(default)]
    pub sections: Vec<String>,
    /// The project's linked document.
    #[serde(default)]
    pub page_id: Option<Id>,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub order: i64,
    pub updated: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Task {
    pub id: Id,
    pub title: String,
    #[serde(default)]
    pub notes: String,
    /// None means the Inbox.
    #[serde(default)]
    pub project_id: Option<Id>,
    #[serde(default)]
    pub section: String,
    /// Parent task for subtasks.
    #[serde(default)]
    pub parent_id: Option<Id>,
    /// 1 (urgent) .. 4 (none), like Todoist's P1..P4.
    pub priority: u8,
    #[serde(default)]
    pub due: Option<NaiveDate>,
    #[serde(default)]
    pub due_time: Option<NaiveTime>,
    /// How long the task takes, so it can be shown as a block on the calendar.
    #[serde(default)]
    pub duration_min: Option<u32>,
    #[serde(default)]
    pub recurrence: Recurrence,
    /// Remind this many minutes before the due time.
    #[serde(default)]
    pub reminder_min: Option<i64>,
    #[serde(default)]
    pub reminded_for: String,
    #[serde(default)]
    pub labels: Vec<String>,
    /// Linked document.
    #[serde(default)]
    pub page_id: Option<Id>,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub completed_at: Option<i64>,
    pub created: i64,
    pub updated: i64,
    #[serde(default)]
    pub order: i64,
}

impl Task {
    pub fn new(title: impl Into<String>) -> Self {
        let t = now_ms();
        Task {
            id: new_id(),
            title: title.into(),
            priority: 4,
            created: t,
            updated: t,
            order: t,
            ..Default::default()
        }
    }
    pub fn due_datetime(&self) -> Option<NaiveDateTime> {
        self.due.map(|d| d.and_time(self.due_time.unwrap_or(NaiveTime::from_hms_opt(23, 59, 0).unwrap())))
    }
    pub fn is_overdue(&self) -> bool {
        if self.completed {
            return false;
        }
        match (self.due, self.due_time) {
            (Some(d), Some(t)) => d.and_time(t) < now_local() && d <= today(),
            (Some(d), None) => d < today(),
            _ => false,
        }
    }
    /// Complete the task. Recurring tasks roll forward to their next date instead (like Todoist).
    /// Returns true if the task rolled forward.
    pub fn complete(&mut self) -> bool {
        self.updated = now_ms();
        if self.recurrence.is_some() {
            let base = self.due.unwrap_or_else(today);
            let mut next = self.recurrence.next_after(base);
            // skip past dates so an old recurring task lands in the future
            while let Some(n) = next {
                if n > today() || (n == today() && base < today()) {
                    break;
                }
                next = self.recurrence.next_after(n);
            }
            if let Some(n) = next {
                self.due = Some(n);
                self.reminded_for.clear();
                return true;
            }
        }
        self.completed = true;
        self.completed_at = Some(now_ms());
        false
    }
}

pub fn priority_label(p: u8) -> &'static str {
    match p {
        1 => "P1 · Urgent",
        2 => "P2 · High",
        3 => "P3 · Medium",
        _ => "P4 · None",
    }
}

// ---------------------------------------------------------------- calendar

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Calendar {
    pub id: Id,
    pub name: String,
    pub color: [u8; 3],
    #[serde(default = "yes")]
    pub visible: bool,
    /// An external calendar subscribed by its iCal (.ics) link; its events are refreshed, read-only.
    #[serde(default)]
    pub source_url: String,
    #[serde(default)]
    pub last_fetch: i64,
    pub updated: i64,
}

fn yes() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    pub id: Id,
    pub calendar_id: Id,
    pub title: String,
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    #[serde(default)]
    pub all_day: bool,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub recurrence: Recurrence,
    #[serde(default)]
    pub page_id: Option<Id>,
    #[serde(default)]
    pub reminder_min: Option<i64>,
    pub updated: i64,
}

impl Event {
    pub fn duration(&self) -> chrono::Duration {
        (self.end - self.start).max(chrono::Duration::minutes(15))
    }
    /// The start time of the occurrence on `day`, if any.
    pub fn occurrence_on(&self, day: NaiveDate) -> Option<NaiveDateTime> {
        let sd = self.start.date();
        if self.recurrence.is_some() {
            if self.recurrence.occurs_on(sd, day) {
                return Some(day.and_time(self.start.time()));
            }
            return None;
        }
        // multi-day events show on each day they cover
        let ed = (self.end - chrono::Duration::seconds(1)).date();
        if day >= sd && day <= ed.max(sd) {
            if day == sd {
                Some(self.start)
            } else {
                Some(day.and_hms_opt(0, 0, 0).unwrap())
            }
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------- databases

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ColKind {
    #[default]
    Text,
    Number,
    Select,
    Checkbox,
    Date,
    Url,
    Page,
    /// Links to records elsewhere: rows of another database, projects, tasks or pages.
    Relation,
    Email,
    Phone,
}

impl ColKind {
    pub const ALL: [ColKind; 10] = [
        ColKind::Text,
        ColKind::Number,
        ColKind::Select,
        ColKind::Checkbox,
        ColKind::Date,
        ColKind::Url,
        ColKind::Email,
        ColKind::Phone,
        ColKind::Page,
        ColKind::Relation,
    ];
    pub fn label(self) -> &'static str {
        match self {
            ColKind::Text => "Text",
            ColKind::Number => "Number",
            ColKind::Select => "Select",
            ColKind::Checkbox => "Checkbox",
            ColKind::Date => "Date",
            ColKind::Url => "URL",
            ColKind::Page => "Linked page",
            ColKind::Relation => "Relation (linked records)",
            ColKind::Email => "Email",
            ColKind::Phone => "Phone",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Column {
    pub id: Id,
    pub name: String,
    pub kind: ColKind,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default = "default_width")]
    pub width: f32,
    /// For relations: "db:<database id>", "projects", "tasks" or "pages".
    #[serde(default)]
    pub target: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FilterOp {
    #[default]
    Contains,
    Is,
    IsNot,
    IsEmpty,
    IsNotEmpty,
    Checked,
    Unchecked,
    Before,
    After,
    Greater,
    Less,
}

impl FilterOp {
    pub const ALL: [FilterOp; 11] = [
        FilterOp::Contains,
        FilterOp::Is,
        FilterOp::IsNot,
        FilterOp::IsEmpty,
        FilterOp::IsNotEmpty,
        FilterOp::Checked,
        FilterOp::Unchecked,
        FilterOp::Before,
        FilterOp::After,
        FilterOp::Greater,
        FilterOp::Less,
    ];
    pub fn label(self) -> &'static str {
        match self {
            FilterOp::Contains => "contains",
            FilterOp::Is => "is",
            FilterOp::IsNot => "is not",
            FilterOp::IsEmpty => "is empty",
            FilterOp::IsNotEmpty => "is not empty",
            FilterOp::Checked => "is checked",
            FilterOp::Unchecked => "is unchecked",
            FilterOp::Before => "is before",
            FilterOp::After => "is after",
            FilterOp::Greater => ">",
            FilterOp::Less => "<",
        }
    }
    pub fn needs_value(self) -> bool {
        !matches!(self, FilterOp::IsEmpty | FilterOp::IsNotEmpty | FilterOp::Checked | FilterOp::Unchecked)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct DbFilter {
    pub col: Id,
    pub op: FilterOp,
    #[serde(default)]
    pub value: String,
}

impl DbFilter {
    /// `shown` is the human-readable cell value (relation names instead of ids).
    pub fn matches(&self, raw: &str, shown: &str) -> bool {
        let v = shown.to_lowercase();
        let q = self.value.trim().to_lowercase();
        match self.op {
            FilterOp::Contains => v.contains(&q),
            FilterOp::Is => v == q || raw.to_lowercase() == q,
            FilterOp::IsNot => v != q && raw.to_lowercase() != q,
            FilterOp::IsEmpty => raw.trim().is_empty(),
            FilterOp::IsNotEmpty => !raw.trim().is_empty(),
            FilterOp::Checked => raw == "true",
            FilterOp::Unchecked => raw != "true",
            FilterOp::Before => !raw.is_empty() && raw < self.value.trim(),
            FilterOp::After => !raw.is_empty() && raw > self.value.trim(),
            FilterOp::Greater => raw.parse::<f64>().ok().zip(q.parse::<f64>().ok()).map_or(false, |(a, b)| a > b),
            FilterOp::Less => raw.parse::<f64>().ok().zip(q.parse::<f64>().ok()).map_or(false, |(a, b)| a < b),
        }
    }
}

pub fn split_ids(s: &str) -> Vec<String> {
    s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()
}

fn default_width() -> f32 {
    160.0
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Row {
    pub id: Id,
    pub cells: BTreeMap<Id, String>,
    #[serde(default)]
    pub page_id: Option<Id>,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DbView {
    #[default]
    Table,
    Board,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Database {
    pub id: Id,
    pub name: String,
    #[serde(default)]
    pub icon: String,
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
    #[serde(default)]
    pub view: DbView,
    #[serde(default)]
    pub group_by: Option<Id>,
    #[serde(default)]
    pub sort_by: Option<(Id, bool)>,
    #[serde(default)]
    pub filters: Vec<DbFilter>,
    #[serde(default)]
    pub template: bool,
    pub updated: i64,
}

impl Database {
    pub fn col(&self, name_or_id: &str) -> Option<&Column> {
        self.columns
            .iter()
            .find(|c| c.id == name_or_id)
            .or_else(|| self.columns.iter().find(|c| c.name.eq_ignore_ascii_case(name_or_id)))
    }
    pub fn title_of(&self, row: &Row) -> String {
        self.columns
            .first()
            .and_then(|c| row.cells.get(&c.id))
            .cloned()
            .unwrap_or_default()
    }
}

// ---------------------------------------------------------------- the workspace

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Tombstone {
    pub id: Id,
    pub deleted: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct WorkspaceData {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub pages: Vec<Page>,
    #[serde(default)]
    pub projects: Vec<Project>,
    #[serde(default)]
    pub tasks: Vec<Task>,
    #[serde(default)]
    pub calendars: Vec<Calendar>,
    #[serde(default)]
    pub events: Vec<Event>,
    #[serde(default)]
    pub databases: Vec<Database>,
    #[serde(default)]
    pub tombstones: Vec<Tombstone>,
    // ---- the Resolve layer (intake, project context, knowledge records); see src/resolve
    #[serde(default)]
    pub records: Vec<crate::resolve::Record>,
    #[serde(default)]
    pub intakes: Vec<crate::resolve::Intake>,
    #[serde(default)]
    pub contexts: Vec<crate::resolve::ProjectContext>,
    /// Undo / soft-delete history (see src/history.rs).
    #[serde(default)]
    pub history: Vec<crate::history::ChangeSet>,
    /// AI Team conversations (one per role and project; Ask AI is the General assistant's).
    #[serde(default)]
    pub conversations: Vec<Conversation>,
}

/// A conversation with one AI Team role, about one project or the whole workspace.
/// Ported from Control Centre's model (Conversation / ChatMessage).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Conversation {
    pub id: Id,
    #[serde(default)]
    pub project_id: Option<Id>,
    /// Role id: "general", "product-director", "validation", "growth-lead", "tech-lead".
    pub agent: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub messages: Vec<ChatMessage>,
    pub updated: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ChatMessage {
    /// "user" or the role id that answered.
    pub role: String,
    pub text: String,
    pub at: i64,
    /// Which AI answered (shown under the message).
    #[serde(default)]
    pub via: String,
    #[serde(default)]
    pub error: bool,
}

trait Stamped {
    fn sid(&self) -> &str;
    fn stamp(&self) -> i64;
}
macro_rules! stamped {
    ($t:ty) => {
        impl Stamped for $t {
            fn sid(&self) -> &str {
                &self.id
            }
            fn stamp(&self) -> i64 {
                self.updated
            }
        }
    };
}
stamped!(Page);
stamped!(Project);
stamped!(Task);
stamped!(Calendar);
stamped!(Event);
stamped!(Database);
stamped!(crate::resolve::Record);
stamped!(crate::resolve::Intake);
stamped!(crate::resolve::ProjectContext);
stamped!(crate::history::ChangeSet);
stamped!(Conversation);

fn merge_vec<T: Stamped + Clone>(local: &mut Vec<T>, remote: &[T], dead: &HashMap<String, i64>) {
    let mut index: HashMap<String, usize> = HashMap::new();
    for (i, x) in local.iter().enumerate() {
        index.insert(x.sid().to_string(), i);
    }
    for r in remote {
        match index.get(r.sid()) {
            Some(&i) => {
                if r.stamp() > local[i].stamp() {
                    local[i] = r.clone();
                }
            }
            None => {
                index.insert(r.sid().to_string(), local.len());
                local.push(r.clone());
            }
        }
    }
    local.retain(|x| dead.get(x.sid()).map_or(true, |&d| x.stamp() > d));
}

impl WorkspaceData {
    /// Merge another copy into this one: per item, the newest edit wins; deletions win over older edits.
    pub fn merge(&mut self, other: &WorkspaceData) {
        let mut dead: HashMap<String, i64> = HashMap::new();
        for t in self.tombstones.iter().chain(other.tombstones.iter()) {
            let e = dead.entry(t.id.clone()).or_insert(0);
            *e = (*e).max(t.deleted);
        }
        merge_vec(&mut self.pages, &other.pages, &dead);
        merge_vec(&mut self.projects, &other.projects, &dead);
        merge_vec(&mut self.tasks, &other.tasks, &dead);
        merge_vec(&mut self.calendars, &other.calendars, &dead);
        merge_vec(&mut self.events, &other.events, &dead);
        merge_vec(&mut self.databases, &other.databases, &dead);
        merge_vec(&mut self.records, &other.records, &dead);
        merge_vec(&mut self.intakes, &other.intakes, &dead);
        merge_vec(&mut self.contexts, &other.contexts, &dead);
        merge_vec(&mut self.history, &other.history, &dead);
        merge_vec(&mut self.conversations, &other.conversations, &dead);
        // keep tombstones for 60 days
        let cutoff = now_ms() - 60 * 24 * 3600 * 1000;
        self.tombstones = dead
            .into_iter()
            .filter(|(_, d)| *d > cutoff)
            .map(|(id, deleted)| Tombstone { id, deleted })
            .collect();
        self.tombstones.sort_by(|a, b| a.id.cmp(&b.id));
    }

    pub fn bury(&mut self, id: &str) {
        self.tombstones.push(Tombstone { id: id.to_string(), deleted: now_ms() });
    }

    pub fn page(&self, id: &str) -> Option<&Page> {
        self.pages.iter().find(|p| p.id == id)
    }
    pub fn page_mut(&mut self, id: &str) -> Option<&mut Page> {
        self.pages.iter_mut().find(|p| p.id == id)
    }
    pub fn page_by_title(&self, title: &str) -> Option<&Page> {
        self.pages.iter().find(|p| p.title.eq_ignore_ascii_case(title.trim()))
    }
    pub fn task(&self, id: &str) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }
    pub fn task_mut(&mut self, id: &str) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }
    pub fn project(&self, id: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.id == id)
    }
    pub fn project_by_name(&self, name: &str) -> Option<&Project> {
        let norm = |s: &str| s.to_lowercase().replace([' ', '_', '-'], "");
        let n = norm(name);
        self.projects.iter().find(|p| norm(&p.name) == n)
    }
    pub fn calendar(&self, id: &str) -> Option<&Calendar> {
        self.calendars.iter().find(|c| c.id == id)
    }
    pub fn database(&self, id: &str) -> Option<&Database> {
        self.databases.iter().find(|d| d.id == id)
    }
    pub fn database_mut(&mut self, id: &str) -> Option<&mut Database> {
        self.databases.iter_mut().find(|d| d.id == id)
    }
    pub fn event_mut(&mut self, id: &str) -> Option<&mut Event> {
        self.events.iter_mut().find(|e| e.id == id)
    }
    pub fn project_name(&self, id: &Option<Id>) -> String {
        match id {
            None => "Inbox".into(),
            Some(i) => self.project(i).map(|p| p.name.clone()).unwrap_or_else(|| "Inbox".into()),
        }
    }
    pub fn project_color(&self, id: &Option<Id>) -> [u8; 3] {
        id.as_ref().and_then(|i| self.project(i)).map(|p| p.color).unwrap_or([120, 130, 150])
    }

    pub fn subtasks(&self, id: &str) -> Vec<&Task> {
        let mut v: Vec<&Task> = self.tasks.iter().filter(|t| t.parent_id.as_deref() == Some(id)).collect();
        v.sort_by_key(|t| t.order);
        v
    }

    /// Remove a task and its subtasks.
    pub fn delete_task(&mut self, id: &str) {
        let mut ids = vec![id.to_string()];
        let mut i = 0;
        while i < ids.len() {
            let cur = ids[i].clone();
            for t in &self.tasks {
                if t.parent_id.as_deref() == Some(cur.as_str()) {
                    ids.push(t.id.clone());
                }
            }
            i += 1;
        }
        for x in &ids {
            self.bury(x);
        }
        self.tasks.retain(|t| !ids.contains(&t.id));
    }

    /// Remove a page; its sub-pages move up to the page's parent.
    pub fn delete_page(&mut self, id: &str) {
        let parent = self.page(id).and_then(|p| p.parent.clone());
        let t = now_ms();
        for p in self.pages.iter_mut() {
            if p.parent.as_deref() == Some(id) {
                p.parent = parent.clone();
                p.updated = t;
            }
        }
        for task in self.tasks.iter_mut() {
            if task.page_id.as_deref() == Some(id) {
                task.page_id = None;
                task.updated = t;
            }
        }
        self.bury(id);
        self.pages.retain(|p| p.id != id);
    }

    pub fn delete_event(&mut self, id: &str) {
        self.bury(id);
        self.events.retain(|e| e.id != id);
    }

    pub fn delete_project(&mut self, id: &str) {
        let ids: Vec<Id> = self.tasks.iter().filter(|t| t.project_id.as_deref() == Some(id)).map(|t| t.id.clone()).collect();
        for t in ids {
            self.delete_task(&t);
        }
        // its Resolve context and records go with it
        let recs: Vec<Id> = self.records.iter().filter(|r| r.project_id.as_deref() == Some(id)).map(|r| r.id.clone()).chain(self.contexts.iter().filter(|c| c.project_id == id).map(|c| c.id.clone())).collect();
        for r in recs {
            self.bury(&r);
        }
        self.records.retain(|r| r.project_id.as_deref() != Some(id));
        self.contexts.retain(|c| c.project_id != id);
        self.bury(id);
        self.projects.retain(|p| p.id != id);
    }

    pub fn delete_record(&mut self, id: &str) {
        // children move up to the deleted record's parent (the hypothesis tree stays connected)
        let parent = self.records.iter().find(|r| r.id == id).and_then(|r| r.parent_id.clone());
        let t = now_ms();
        for r in self.records.iter_mut().filter(|r| r.parent_id.as_deref() == Some(id)) {
            r.parent_id = parent.clone();
            r.updated = t;
        }
        self.bury(id);
        self.records.retain(|r| r.id != id);
    }

    pub fn context_for(&self, project_id: &str) -> Option<&crate::resolve::ProjectContext> {
        self.contexts.iter().filter(|c| c.project_id == project_id).max_by_key(|c| c.updated)
    }
    pub fn context_for_mut(&mut self, project_id: &str) -> Option<&mut crate::resolve::ProjectContext> {
        self.contexts.iter_mut().filter(|c| c.project_id == project_id).max_by_key(|c| c.updated)
    }
    pub fn record(&self, id: &str) -> Option<&crate::resolve::Record> {
        self.records.iter().find(|r| r.id == id)
    }
    pub fn record_mut(&mut self, id: &str) -> Option<&mut crate::resolve::Record> {
        self.records.iter_mut().find(|r| r.id == id)
    }

    pub fn delete_database(&mut self, id: &str) {
        self.bury(id);
        self.databases.retain(|d| d.id != id);
    }

    pub fn delete_calendar(&mut self, id: &str) {
        let ids: Vec<Id> = self.events.iter().filter(|e| e.calendar_id == id).map(|e| e.id.clone()).collect();
        for e in ids {
            self.delete_event(&e);
        }
        self.bury(id);
        self.calendars.retain(|c| c.id != id);
    }

    /// Pages that link to `id` (via [[Title]] or a link block).
    pub fn backlinks(&self, id: &str) -> Vec<&Page> {
        let Some(target) = self.page(id) else { return vec![] };
        self.pages
            .iter()
            .filter(|p| p.id != id)
            .filter(|p| {
                p.blocks.iter().any(|b| b.kind == BlockKind::PageLink && b.text == id)
                    || p.wiki_links().iter().any(|l| l.eq_ignore_ascii_case(&target.title))
            })
            .collect()
    }

    pub fn page_path(&self, id: &str) -> Vec<&Page> {
        let mut out = vec![];
        let mut cur = self.page(id);
        let mut guard = 0;
        while let Some(p) = cur {
            out.push(p);
            guard += 1;
            if guard > 32 {
                break;
            }
            cur = p.parent.as_deref().and_then(|x| self.page(x));
        }
        out.reverse();
        out
    }

    pub fn default_calendar(&self) -> Option<Id> {
        self.calendars.iter().find(|c| c.source_url.is_empty()).or(self.calendars.first()).map(|c| c.id.clone())
    }

    /// Things a relation column can point at: (id, display name).
    pub fn relation_options(&self, target: &str) -> Vec<(Id, String)> {
        if let Some(dbid) = target.strip_prefix("db:") {
            return self.database(dbid).map(|db| db.rows.iter().map(|r| (r.id.clone(), { let t = db.title_of(r); if t.is_empty() { "Untitled".into() } else { t } })).collect()).unwrap_or_default();
        }
        match target {
            "projects" => self.projects.iter().map(|p| (p.id.clone(), format!("# {}", p.name))).collect(),
            "tasks" => self.tasks.iter().filter(|t| !t.completed).map(|t| (t.id.clone(), t.title.clone())).collect(),
            "pages" => self.pages.iter().filter(|p| !p.template).map(|p| (p.id.clone(), format!("{} {}", p.icon, if p.title.is_empty() { "Untitled" } else { &p.title }))).collect(),
            _ => vec![],
        }
    }

    pub fn relation_target_label(&self, target: &str) -> String {
        if let Some(dbid) = target.strip_prefix("db:") {
            return self.database(dbid).map(|d| d.name.clone()).unwrap_or_else(|| "(deleted database)".into());
        }
        match target {
            "projects" => "Projects".into(),
            "tasks" => "Tasks".into(),
            "pages" => "Pages".into(),
            _ => "(choose what to link)".into(),
        }
    }

    pub fn relation_name(&self, target: &str, id: &str) -> Option<String> {
        if let Some(dbid) = target.strip_prefix("db:") {
            let db = self.database(dbid)?;
            let r = db.rows.iter().find(|r| r.id == id)?;
            let t = db.title_of(r);
            return Some(if t.is_empty() { "Untitled".into() } else { t });
        }
        match target {
            "projects" => self.project(id).map(|p| p.name.clone()),
            "tasks" => self.task(id).map(|t| t.title.clone()),
            "pages" => self.page(id).map(|p| p.title.clone()),
            _ => None,
        }
    }

    /// The text a person sees for a cell (relations and page links resolved to names).
    pub fn cell_display(&self, col: &Column, raw: &str) -> String {
        match col.kind {
            ColKind::Relation => split_ids(raw).iter().filter_map(|id| self.relation_name(&col.target, id)).collect::<Vec<_>>().join(", "),
            ColKind::Page => self.page(raw).map(|p| p.title.clone()).unwrap_or_default(),
            ColKind::Checkbox => if raw == "true" { "Yes".into() } else { "No".into() },
            _ => raw.to_string(),
        }
    }

    /// Rows in any database whose relation columns point at `item_id`.
    /// Returns (database id, database name, column name, row id, row title).
    pub fn incoming_links(&self, item_id: &str) -> Vec<(Id, String, String, Id, String)> {
        let mut out = vec![];
        for db in &self.databases {
            for c in db.columns.iter().filter(|c| c.kind == ColKind::Relation) {
                for r in &db.rows {
                    if r.cells.get(&c.id).map_or(false, |v| split_ids(v).iter().any(|x| x == item_id)) {
                        out.push((db.id.clone(), db.name.clone(), c.name.clone(), r.id.clone(), db.title_of(r)));
                    }
                }
            }
        }
        out
    }

    /// Keep page to-dos and their tasks in step. Whichever side was edited last wins.
    /// Returns true if anything changed.
    pub fn sync_todo_links(&mut self) -> bool {
        let tasks: HashMap<String, (bool, i64)> = self.tasks.iter().map(|t| (t.id.clone(), (t.completed, t.updated))).collect();
        let mut to_tasks: Vec<(String, bool)> = vec![];
        let mut changed = false;
        let t = now_ms();
        for page in self.pages.iter_mut() {
            let mut page_changed = false;
            for b in page.blocks.iter_mut().filter(|b| b.kind == BlockKind::Todo) {
                let Some(tid) = b.task_id.clone() else { continue };
                match tasks.get(&tid) {
                    None => {
                        b.task_id = None;
                        page_changed = true;
                    }
                    Some(&(done, tupd)) => {
                        if done != b.checked {
                            if page.updated > tupd {
                                to_tasks.push((tid, b.checked));
                            } else {
                                b.checked = done;
                                page_changed = true;
                            }
                        }
                    }
                }
            }
            if page_changed {
                page.updated = t;
                changed = true;
            }
        }
        for (tid, checked) in to_tasks {
            if let Some(task) = self.task_mut(&tid) {
                if checked {
                    task.complete();
                } else {
                    task.completed = false;
                    task.completed_at = None;
                    task.updated = now_ms();
                }
                changed = true;
            }
        }
        changed
    }

    /// Unticked to-dos written in pages: (page id, page title, block id, text, linked task id).
    pub fn page_todos(&self) -> Vec<(Id, String, Id, String, Option<Id>)> {
        let mut out = vec![];
        for p in self.pages.iter().filter(|p| !p.template) {
            for b in p.blocks.iter().filter(|b| b.kind == BlockKind::Todo && !b.checked && !b.text.trim().is_empty()) {
                out.push((p.id.clone(), p.title.clone(), b.id.clone(), b.text.clone(), b.task_id.clone().filter(|t| self.task(t).is_some())));
            }
        }
        out
    }

    /// The database row whose page is `page_id`.
    pub fn row_for_page(&self, page_id: &str) -> Option<(Id, Id)> {
        for db in &self.databases {
            for r in &db.rows {
                if r.page_id.as_deref() == Some(page_id) {
                    return Some((db.id.clone(), r.id.clone()));
                }
            }
        }
        None
    }
}

// ---------------------------------------------------------------- markdown conversion

pub fn blocks_to_markdown(page: &Page, data: &WorkspaceData) -> String {
    let mut out = format!("# {}\n\n", page.title);
    let mut num = 0;
    for b in &page.blocks {
        if b.kind != BlockKind::Numbered {
            num = 0;
        }
        match b.kind {
            BlockKind::Paragraph => out.push_str(&format!("{}\n\n", b.text)),
            BlockKind::H1 => out.push_str(&format!("## {}\n\n", b.text)),
            BlockKind::H2 => out.push_str(&format!("### {}\n\n", b.text)),
            BlockKind::H3 => out.push_str(&format!("#### {}\n\n", b.text)),
            BlockKind::Bullet => out.push_str(&format!("- {}\n", b.text)),
            BlockKind::Numbered => {
                num += 1;
                out.push_str(&format!("{num}. {}\n", b.text))
            }
            BlockKind::Todo => out.push_str(&format!("- [{}] {}\n", if b.checked { "x" } else { " " }, b.text)),
            BlockKind::Quote => out.push_str(&format!("> {}\n\n", b.text.replace('\n', "\n> "))),
            BlockKind::Code => out.push_str(&format!("```\n{}\n```\n\n", b.text)),
            BlockKind::Callout => out.push_str(&format!("> 💡 {}\n\n", b.text)),
            BlockKind::Divider => out.push_str("---\n\n"),
            BlockKind::PageLink => {
                let t = data.page(&b.text).map(|p| p.title.clone()).unwrap_or_else(|| "missing page".into());
                out.push_str(&format!("[[{t}]]\n\n"))
            }
            BlockKind::Database => {
                if let Some(db) = data.database(&b.text) {
                    out.push_str(&database_to_markdown(db));
                    out.push('\n');
                }
            }
        }
    }
    out
}

pub fn database_to_markdown(db: &Database) -> String {
    let mut s = format!("**{}**\n\n| ", db.name);
    for c in &db.columns {
        s.push_str(&format!("{} | ", c.name));
    }
    s.push_str("\n|");
    for _ in &db.columns {
        s.push_str(" --- |");
    }
    s.push('\n');
    for r in &db.rows {
        s.push_str("| ");
        for c in &db.columns {
            s.push_str(&format!("{} | ", r.cells.get(&c.id).map(|v| v.replace('|', "/")).unwrap_or_default()));
        }
        s.push('\n');
    }
    s
}

/// Parse markdown into blocks. A leading `# Title` is returned separately.
pub fn markdown_to_blocks(md: &str) -> (Option<String>, Vec<Block>) {
    let mut title = None;
    let mut blocks = vec![];
    let mut para = String::new();
    let mut in_code = false;
    let mut code = String::new();
    let flush = |para: &mut String, blocks: &mut Vec<Block>| {
        if !para.trim().is_empty() {
            blocks.push(Block::new(BlockKind::Paragraph, para.trim().to_string()));
        }
        para.clear();
    };
    for raw in md.lines() {
        let line = raw.trim_end();
        if line.trim_start().starts_with("```") {
            if in_code {
                blocks.push(Block::new(BlockKind::Code, code.trim_end_matches('\n').to_string()));
                code.clear();
                in_code = false;
            } else {
                flush(&mut para, &mut blocks);
                in_code = true;
            }
            continue;
        }
        if in_code {
            code.push_str(raw);
            code.push('\n');
            continue;
        }
        let t = line.trim_start();
        if t.is_empty() {
            flush(&mut para, &mut blocks);
            continue;
        }
        let kind_text: Option<(BlockKind, String, bool)> = if let Some(r) = t.strip_prefix("# ") {
            if title.is_none() && blocks.is_empty() && para.is_empty() {
                title = Some(r.trim().to_string());
                continue;
            }
            Some((BlockKind::H1, r.into(), false))
        } else if let Some(r) = t.strip_prefix("#### ") {
            Some((BlockKind::H3, r.into(), false))
        } else if let Some(r) = t.strip_prefix("### ") {
            Some((BlockKind::H3, r.into(), false))
        } else if let Some(r) = t.strip_prefix("## ") {
            Some((BlockKind::H2, r.into(), false))
        } else if let Some(r) = t.strip_prefix("- [ ] ").or_else(|| t.strip_prefix("* [ ] ")) {
            Some((BlockKind::Todo, r.into(), false))
        } else if let Some(r) = t
            .strip_prefix("- [x] ")
            .or_else(|| t.strip_prefix("- [X] "))
            .or_else(|| t.strip_prefix("* [x] "))
        {
            Some((BlockKind::Todo, r.into(), true))
        } else if let Some(r) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")).or_else(|| t.strip_prefix("+ ")) {
            Some((BlockKind::Bullet, r.into(), false))
        } else if let Some(r) = t.strip_prefix("> 💡 ") {
            Some((BlockKind::Callout, r.into(), false))
        } else if let Some(r) = t.strip_prefix("> ").or_else(|| (t == ">").then_some("")) {
            Some((BlockKind::Quote, r.into(), false))
        } else if t == "---" || t == "***" || t == "___" {
            Some((BlockKind::Divider, String::new(), false))
        } else if let Some(pos) = t.find(". ") {
            if pos <= 3 && t[..pos].chars().all(|c| c.is_ascii_digit()) && pos > 0 {
                Some((BlockKind::Numbered, t[pos + 2..].into(), false))
            } else {
                None
            }
        } else {
            None
        };
        match kind_text {
            Some((k, text, checked)) => {
                flush(&mut para, &mut blocks);
                // merge consecutive quote lines
                if k == BlockKind::Quote {
                    if let Some(last) = blocks.last_mut() {
                        if last.kind == BlockKind::Quote {
                            last.text.push('\n');
                            last.text.push_str(&text);
                            continue;
                        }
                    }
                }
                let mut b = Block::new(k, text);
                b.checked = checked;
                blocks.push(b);
            }
            None => {
                if !para.is_empty() {
                    para.push(' ');
                }
                para.push_str(t);
            }
        }
    }
    if in_code && !code.is_empty() {
        blocks.push(Block::new(BlockKind::Code, code.trim_end().to_string()));
    }
    flush(&mut para, &mut blocks);
    if blocks.is_empty() {
        blocks.push(Block::new(BlockKind::Paragraph, ""));
    }
    (title, blocks)
}

pub fn fmt_date_short(d: NaiveDate) -> String {
    let t = today();
    let diff = (d - t).num_days();
    match diff {
        0 => "Today".into(),
        1 => "Tomorrow".into(),
        -1 => "Yesterday".into(),
        2..=6 => d.format("%A").to_string(),
        _ => {
            if d.year() == t.year() {
                d.format("%-d %b").to_string()
            } else {
                d.format("%-d %b %Y").to_string()
            }
        }
    }
}

pub fn fmt_time(t: NaiveTime) -> String {
    t.format("%H:%M").to_string()
}
