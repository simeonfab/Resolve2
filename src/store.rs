//! Local-first storage. Each workspace lives in its own folder as plain JSON you own:
//!   <root>/workspaces.json            list of workspaces + app settings
//!   <root>/workspaces/<id>/data.json  pages, tasks, events, databases
//!   <root>/workspaces/<id>/settings.json
//!   <root>/workspaces/<id>/backups/*.json
//! Writes are atomic and merge with whatever is on disk, so the app, the MCP server
//! and the phone page can all edit the same workspace safely.

use crate::model::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

static ROOT: OnceLock<PathBuf> = OnceLock::new();
pub const SAMPLE_NAME: &str = "Sample (demo data)";
static WRITE_LOCK: Mutex<()> = Mutex::new(());

pub fn set_root(p: PathBuf) {
    let _ = ROOT.set(p);
}

pub fn root() -> PathBuf {
    ROOT.get_or_init(|| {
        if let Ok(p) = std::env::var("RESOLVE2_DATA") {
            return PathBuf::from(p);
        }
        if let Ok(a) = std::env::var("APPDATA") {
            return PathBuf::from(a).join("Resolve2"); // separate from Resolve Beta (%APPDATA%\Weave) on purpose
        }
        if let Ok(h) = std::env::var("HOME") {
            return PathBuf::from(h).join(".resolve2");
        }
        PathBuf::from("resolve2-data")
    })
    .clone()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkspaceMeta {
    pub id: Id,
    pub name: String,
    pub color: [u8; 3],
    #[serde(default)]
    pub icon: String,
    pub created: i64,
    /// Who owns this workspace. Single-user today ("local"); the field exists so a hosted,
    /// multi-user Resolve can scope every workspace to an account without a data migration.
    #[serde(default = "local_owner")]
    pub owner: String,
}

pub fn local_owner() -> String {
    "local".into()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    #[serde(default = "yes")]
    pub dark: bool,
    #[serde(default)]
    pub last_workspace: Option<Id>,
    #[serde(default = "default_provider")]
    pub ai_provider: String,
    #[serde(default = "default_ai_url")]
    pub ai_url: String,
    #[serde(default = "default_ai_model")]
    pub ai_model: String,
    #[serde(default = "yes")]
    pub sidebar_open: bool,
    /// Your own API keys for the in-app assistant (kept only in this PC's settings file).
    #[serde(default)]
    pub anthropic_key: String,
    #[serde(default)]
    pub openai_key: String,
    /// Cost tracking: the user's own prices (USD per million tokens) and an optional monthly cap.
    #[serde(default)]
    pub price_in_per_m: f64,
    #[serde(default)]
    pub price_out_per_m: f64,
    #[serde(default)]
    pub monthly_budget: f64,
}

fn yes() -> bool {
    true
}
fn default_provider() -> String {
    // Resolve 2 is provider-independent: default to a free local model; API keys or the
    // personal CLI subscriptions are chosen explicitly in Settings → AI & MCP.
    "ollama".into()
}
fn default_ai_url() -> String {
    "http://localhost:11434".into()
}
fn default_ai_model() -> String {
    "llama3.2".into()
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            dark: true,
            last_workspace: None,
            ai_provider: default_provider(),
            ai_url: default_ai_url(),
            ai_model: default_ai_model(),
            sidebar_open: true,
            anthropic_key: String::new(),
            openai_key: String::new(),
            price_in_per_m: 0.0,
            price_out_per_m: 0.0,
            monthly_budget: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Registry {
    pub workspaces: Vec<WorkspaceMeta>,
    #[serde(default)]
    pub settings: AppSettings,
}

impl Registry {
    pub fn path() -> PathBuf {
        root().join("workspaces.json")
    }
    pub fn load() -> Registry {
        let mut reg: Registry = fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        if reg.workspaces.is_empty() {
            // your real workspace starts clean; the demo content lives in its own workspace
            let personal = WorkspaceMeta { id: new_id(), name: "Personal".into(), color: [219, 76, 63], icon: "P".into(), created: now_ms(), owner: local_owner() };
            let dir = workspace_dir(&personal.id);
            let _ = fs::create_dir_all(&dir);
            let mut data = WorkspaceData::default();
            crate::seed::seed_empty(&mut data);
            let _ = write_json_atomic(&dir.join("data.json"), &data);
            let sample = WorkspaceMeta { id: new_id(), name: SAMPLE_NAME.into(), color: [120, 120, 130], icon: "S".into(), created: now_ms(), owner: local_owner() };
            let dir = workspace_dir(&sample.id);
            let _ = fs::create_dir_all(&dir);
            let mut data = WorkspaceData::default();
            crate::seed::seed(&mut data);
            let _ = write_json_atomic(&dir.join("data.json"), &data);
            reg.settings.last_workspace = Some(sample.id.clone());
            reg.workspaces.push(personal);
            reg.workspaces.push(sample);
            reg.save();
        }
        reg
    }
    pub fn save(&self) {
        let _ = fs::create_dir_all(root());
        let _ = write_json_atomic(&Self::path(), self);
    }
    pub fn find(&self, key: &str) -> Option<&WorkspaceMeta> {
        self.workspaces
            .iter()
            .find(|w| w.id == key)
            .or_else(|| self.workspaces.iter().find(|w| w.name.eq_ignore_ascii_case(key)))
    }
    pub fn create(&mut self, name: &str, color: [u8; 3]) -> WorkspaceMeta {
        let meta = WorkspaceMeta {
            id: new_id(),
            name: name.to_string(),
            color,
            icon: name.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default(),
            created: now_ms(),
            owner: local_owner(),
        };
        let dir = workspace_dir(&meta.id);
        let _ = fs::create_dir_all(&dir);
        let mut data = WorkspaceData::default();
        crate::seed::seed_empty(&mut data);
        let _ = write_json_atomic(&dir.join("data.json"), &data);
        self.workspaces.push(meta.clone());
        self.save();
        meta
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkspaceSettings {
    #[serde(default)]
    pub sync_dir: String,
    #[serde(default = "yes")]
    pub auto_backup: bool,
    #[serde(default = "default_keep")]
    pub keep_backups: usize,
    /// Let AI assistants (MCP) open this workspace at all.
    #[serde(default = "yes")]
    pub ai_access: bool,
    /// AI assistants may read and search but not change anything.
    #[serde(default)]
    pub ai_read_only: bool,
    /// Serve the phone page and MCP over HTTP on the local network.
    #[serde(default)]
    pub server_enabled: bool,
    #[serde(default = "default_port")]
    pub server_port: u16,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub last_backup: i64,
    #[serde(default)]
    pub last_sync: i64,
    /// Always-on hub this device syncs with (another PC over Tailscale, or a cloud server).
    #[serde(default)]
    pub hub_url: String,
    #[serde(default)]
    pub hub_token: String,
    #[serde(default)]
    pub last_hub_sync: i64,
}

/// Send this device's copy to a hub's /sync and get the merged workspace back.
/// http:// goes direct (fine inside Tailscale, which encrypts); https:// uses Windows' curl.exe.
pub fn hub_exchange(url: &str, token: &str, data: &WorkspaceData) -> Result<WorkspaceData, String> {
    let base = url.trim().trim_end_matches('/');
    let base = base.strip_suffix("/sync").unwrap_or(base);
    if !(base.starts_with("http://") || base.starts_with("https://")) {
        return Err("The hub address must start with http:// or https://".into());
    }
    let endpoint = format!("{base}/sync?token={}", token.trim());
    let body = serde_json::to_string(data).map_err(|e| e.to_string())?;
    let text = if base.starts_with("http://") {
        ureq::post(&endpoint)
            .timeout(std::time::Duration::from_secs(60))
            .set("Content-Type", "application/json")
            .send_string(&body)
            .map_err(|e| match e {
                ureq::Error::Status(401, _) => "The hub refused the token".to_string(),
                ureq::Error::Status(c, r) => format!("Hub error {c}: {}", r.into_string().unwrap_or_default()),
                ureq::Error::Transport(t) => format!("Can't reach the hub ({t}). Is it running, and is Tailscale connected?"),
            })?
            .into_string()
            .map_err(|e| e.to_string())?
    } else {
        let tmp = std::env::temp_dir().join(format!("resolve2-hub-{}.json", std::process::id()));
        fs::write(&tmp, &body).map_err(|e| e.to_string())?;
        let out = std::process::Command::new("curl.exe")
            .args(["-sS", "--fail-with-body", "--max-time", "60", "-X", "POST", "-H", "Content-Type: application/json", "--data-binary"])
            .arg(format!("@{}", tmp.display()))
            .arg(&endpoint)
            .output()
            .map_err(|e| format!("Couldn't run curl: {e}"));
        let _ = fs::remove_file(&tmp);
        let out = out?;
        if !out.status.success() {
            return Err(format!("Hub sync failed: {} {}", String::from_utf8_lossy(&out.stderr).trim(), String::from_utf8_lossy(&out.stdout).chars().take(200).collect::<String>()));
        }
        String::from_utf8_lossy(&out.stdout).to_string()
    };
    serde_json::from_str(&text).map_err(|e| format!("The hub answered something unexpected: {e}"))
}

fn default_keep() -> usize {
    30
}
fn default_port() -> u16 {
    8797
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        WorkspaceSettings {
            sync_dir: String::new(),
            auto_backup: true,
            keep_backups: 30,
            ai_access: true,
            ai_read_only: false,
            server_enabled: false,
            server_port: 8797,
            token: String::new(),
            last_backup: 0,
            last_sync: 0,
            hub_url: String::new(),
            hub_token: String::new(),
            last_hub_sync: 0,
        }
    }
}

pub fn new_token() -> String {
    format!("{}{}", new_id(), new_id())
}

pub fn workspace_dir(id: &str) -> PathBuf {
    root().join("workspaces").join(id)
}

pub fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> std::io::Result<()> {
    let text = serde_json::to_string_pretty(value).map_err(std::io::Error::other)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension(format!("tmp{}", std::process::id()));
    fs::write(&tmp, text)?;
    fs::rename(&tmp, path)
}

fn mtime(p: &Path) -> Option<SystemTime> {
    fs::metadata(p).ok().and_then(|m| m.modified().ok())
}

pub fn read_data(p: &Path) -> Option<WorkspaceData> {
    let s = fs::read_to_string(p).ok()?;
    serde_json::from_str(&s).ok()
}

pub struct Store {
    pub meta: WorkspaceMeta,
    pub dir: PathBuf,
    pub data: WorkspaceData,
    pub settings: WorkspaceSettings,
    disk_mtime: Option<SystemTime>,
    pub last_error: Option<String>,
}

impl Store {
    pub fn open(meta: WorkspaceMeta) -> Store {
        let dir = workspace_dir(&meta.id);
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("data.json");
        let data = read_data(&path).unwrap_or_else(|| {
            let mut d = WorkspaceData::default();
            crate::seed::seed_empty(&mut d);
            d
        });
        let mut settings: WorkspaceSettings = fs::read_to_string(dir.join("settings.json"))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        if settings.token.is_empty() {
            settings.token = new_token();
            let _ = write_json_atomic(&dir.join("settings.json"), &settings);
        }
        Store { meta, disk_mtime: mtime(&path), dir, data, settings, last_error: None }
    }

    pub fn data_path(&self) -> PathBuf {
        self.dir.join("data.json")
    }

    /// Pull in edits other programs (MCP, phone page, sync) made to the file.
    pub fn reload_if_changed(&mut self) -> bool {
        let path = self.data_path();
        let m = mtime(&path);
        if m.is_some() && m != self.disk_mtime {
            if let Some(disk) = read_data(&path) {
                self.data.merge(&disk);
                self.disk_mtime = m;
                return true;
            }
        }
        false
    }

    pub fn save(&mut self) -> Result<(), String> {
        let _g = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        self.reload_if_changed();
        let path = self.data_path();
        let r = write_json_atomic(&path, &self.data).map_err(|e| e.to_string());
        self.disk_mtime = mtime(&path);
        self.last_error = r.as_ref().err().cloned();
        r
    }

    pub fn save_settings(&self) {
        let _ = write_json_atomic(&self.dir.join("settings.json"), &self.settings);
    }

    pub fn backups_dir(&self) -> PathBuf {
        self.dir.join("backups")
    }

    pub fn backup_now(&mut self) -> Result<PathBuf, String> {
        let dir = self.backups_dir();
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let name = format!("backup-{}.json", chrono::Local::now().format("%Y-%m-%d_%H-%M-%S-%3f"));
        let p = dir.join(name);
        write_json_atomic(&p, &self.data).map_err(|e| e.to_string())?;
        self.settings.last_backup = now_ms();
        self.save_settings();
        // prune the oldest
        let mut list = self.list_backups();
        while list.len() > self.settings.keep_backups.max(1) {
            let (old, _, _) = list.remove(list.len() - 1);
            let _ = fs::remove_file(old);
        }
        Ok(p)
    }

    /// Newest first: (path, modified, size)
    pub fn list_backups(&self) -> Vec<(PathBuf, SystemTime, u64)> {
        let mut v: Vec<(PathBuf, SystemTime, u64)> = fs::read_dir(self.backups_dir())
            .map(|rd| {
                rd.flatten()
                    .filter(|e| e.path().extension().map_or(false, |x| x == "json"))
                    .filter_map(|e| {
                        let md = e.metadata().ok()?;
                        Some((e.path(), md.modified().ok()?, md.len()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v
    }

    pub fn auto_backup_if_due(&mut self) {
        if self.settings.auto_backup && now_ms() - self.settings.last_backup > 24 * 3600 * 1000 {
            let _ = self.backup_now();
        }
    }

    /// Replace the workspace with a backup (the current state is backed up first).
    pub fn restore(&mut self, p: &Path) -> Result<(), String> {
        let restored = read_data(p).ok_or("That backup could not be read")?;
        self.backup_now()?;
        // a restore is a deliberate rewind: stamp everything so it wins over newer copies
        let mut d = restored;
        let t = now_ms();
        for x in d.pages.iter_mut() {
            x.updated = t;
        }
        for x in d.tasks.iter_mut() {
            x.updated = t;
        }
        for x in d.projects.iter_mut() {
            x.updated = t;
        }
        for x in d.events.iter_mut() {
            x.updated = t;
        }
        for x in d.calendars.iter_mut() {
            x.updated = t;
        }
        for x in d.databases.iter_mut() {
            x.updated = t;
        }
        for x in d.records.iter_mut() {
            x.updated = t;
        }
        for x in d.intakes.iter_mut() {
            x.updated = t;
        }
        for x in d.contexts.iter_mut() {
            x.updated = t;
        }
        // anything that exists now but not in the backup gets removed
        let keep: std::collections::HashSet<String> = d
            .pages
            .iter()
            .map(|x| x.id.clone())
            .chain(d.tasks.iter().map(|x| x.id.clone()))
            .chain(d.projects.iter().map(|x| x.id.clone()))
            .chain(d.events.iter().map(|x| x.id.clone()))
            .chain(d.calendars.iter().map(|x| x.id.clone()))
            .chain(d.databases.iter().map(|x| x.id.clone()))
            .chain(d.records.iter().map(|x| x.id.clone()))
            .chain(d.intakes.iter().map(|x| x.id.clone()))
            .chain(d.contexts.iter().map(|x| x.id.clone()))
            .collect();
        let gone: Vec<String> = self
            .data
            .pages
            .iter()
            .map(|x| x.id.clone())
            .chain(self.data.tasks.iter().map(|x| x.id.clone()))
            .chain(self.data.projects.iter().map(|x| x.id.clone()))
            .chain(self.data.events.iter().map(|x| x.id.clone()))
            .chain(self.data.calendars.iter().map(|x| x.id.clone()))
            .chain(self.data.databases.iter().map(|x| x.id.clone()))
            .chain(self.data.records.iter().map(|x| x.id.clone()))
            .chain(self.data.intakes.iter().map(|x| x.id.clone()))
            .chain(self.data.contexts.iter().map(|x| x.id.clone()))
            .filter(|id| !keep.contains(id))
            .collect();
        for g in gone {
            d.tombstones.push(Tombstone { id: g, deleted: t });
        }
        d.tombstones.retain(|tb| !keep.contains(&tb.id));
        let _g = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        self.data = d;
        let path = self.data_path();
        write_json_atomic(&path, &self.data).map_err(|e| e.to_string())?;
        self.disk_mtime = mtime(&path);
        Ok(())
    }

    pub fn sync_file(&self) -> Option<PathBuf> {
        if self.settings.sync_dir.trim().is_empty() {
            None
        } else {
            Some(PathBuf::from(self.settings.sync_dir.trim()).join(format!("resolve2-{}.json", self.meta.id)))
        }
    }

    /// Two-way sync through a shared folder (Syncthing, OneDrive, Google Drive, a USB stick...).
    pub fn sync(&mut self) -> Result<String, String> {
        let Some(file) = self.sync_file() else { return Err("No sync folder chosen".into()) };
        let dir = file.parent().unwrap().to_path_buf();
        if !dir.exists() {
            return Err(format!("The sync folder {} doesn't exist", dir.display()));
        }
        let mut merged = 0;
        // the main copy plus any conflict copies the sync tool made
        let prefix = format!("resolve2-{}", self.meta.id);
        if let Ok(rd) = fs::read_dir(&dir) {
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with(&prefix) && name.ends_with(".json") {
                    if let Some(remote) = read_data(&e.path()) {
                        self.data.merge(&remote);
                        merged += 1;
                    }
                }
            }
        }
        self.save()?;
        write_json_atomic(&file, &self.data).map_err(|e| e.to_string())?;
        self.settings.last_sync = now_ms();
        self.save_settings();
        Ok(if merged > 0 { "Synced: merged changes from the shared folder".into() } else { "Synced: first copy written to the shared folder".into() })
    }

    pub fn export_markdown(&self, out: &Path) -> Result<usize, String> {
        let base = out.join(format!("{} export {}", self.meta.name, chrono::Local::now().format("%Y-%m-%d")));
        fs::create_dir_all(base.join("Pages")).map_err(|e| e.to_string())?;
        let mut n = 0;
        for p in &self.data.pages {
            let md = blocks_to_markdown(p, &self.data);
            let name = sanitize(&p.title);
            fs::write(base.join("Pages").join(format!("{name} {}.md", &p.id[..6.min(p.id.len())])), md).map_err(|e| e.to_string())?;
            n += 1;
        }
        // tasks as markdown checklists per project
        let mut tasks_md = String::from("# Tasks\n\n");
        let mut groups: Vec<(Option<Id>, String)> = vec![(None, "Inbox".into())];
        for p in &self.data.projects {
            groups.push((Some(p.id.clone()), p.name.clone()));
        }
        for (pid, name) in groups {
            tasks_md.push_str(&format!("## {name}\n\n"));
            for t in self.data.tasks.iter().filter(|t| t.project_id == pid && t.parent_id.is_none()) {
                let due = t.due.map(|d| format!(" (due {d})")).unwrap_or_default();
                tasks_md.push_str(&format!("- [{}] {} p{}{}\n", if t.completed { "x" } else { " " }, t.title, t.priority, due));
                for s in self.data.subtasks(&t.id) {
                    tasks_md.push_str(&format!("  - [{}] {}\n", if s.completed { "x" } else { " " }, s.title));
                }
            }
            tasks_md.push('\n');
        }
        fs::write(base.join("Tasks.md"), tasks_md).map_err(|e| e.to_string())?;
        for db in &self.data.databases {
            fs::write(base.join(format!("{}.csv", sanitize(&db.name))), crate::import::database_to_csv(db)).map_err(|e| e.to_string())?;
            n += 1;
        }
        fs::write(base.join("Calendar.ics"), crate::import::events_to_ics(&self.data)).map_err(|e| e.to_string())?;
        write_json_atomic(&base.join("workspace.json"), &self.data).map_err(|e| e.to_string())?;
        Ok(n)
    }
}

pub fn sanitize(s: &str) -> String {
    let t: String = s
        .chars()
        .map(|c| if "<>:\"/\\|?*".contains(c) || c.is_control() { '_' } else { c })
        .collect();
    let t = t.trim().trim_end_matches('.').to_string();
    if t.is_empty() {
        "Untitled".into()
    } else {
        t.chars().take(80).collect()
    }
}
