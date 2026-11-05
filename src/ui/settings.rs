use super::theme::*;
use super::widgets::*;
use super::*;
use eframe::egui::{self, RichText, Sense};

fn code_box(ui: &mut egui::Ui, p: &Pal, text: &str) -> bool {
    let mut copied = false;
    egui::Frame::none().fill(p.code).rounding(6.0).inner_margin(egui::Margin::same(10.0)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal_top(|ui| {
            ui.add(egui::Label::new(RichText::new(text).monospace().size(12.0)).wrap().selectable(true));
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                if ui.small_button("Copy").clicked() {
                    ui.ctx().output_mut(|o| o.copied_text = text.to_string());
                    copied = true;
                }
            });
        });
    });
    copied
}

fn badge(ui: &mut egui::Ui, p: &Pal, ok: bool, text: &str) {
    chip(ui, text, if ok { p.green } else { p.orange });
}

fn open_folder(path: &std::path::Path) {
    let _ = std::process::Command::new("explorer").arg(path).spawn();
}

/// Merge our server into Claude Desktop's config (backing the file up first).
/// Where Claude Desktop really reads its settings. Newer Microsoft Store builds keep them in
/// %LOCALAPPDATA%\Claude-Data\<package>\, older Store builds inside the package's LocalCache,
/// and the classic installer in %APPDATA%\Claude.
pub(crate) fn claude_desktop_configs() -> Vec<std::path::PathBuf> {
    let mut out = vec![];
    let local = std::env::var("LOCALAPPDATA").map(std::path::PathBuf::from).ok();
    if let Some(l) = &local {
        if let Ok(rd) = std::fs::read_dir(l.join("Claude-Data")) {
            for e in rd.flatten() {
                let p = e.path().join("claude_desktop_config.json");
                if e.path().is_dir() && (p.exists() || e.file_name().to_string_lossy().to_lowercase().starts_with("claude")) {
                    out.push(p);
                }
            }
        }
        if let Ok(rd) = std::fs::read_dir(l.join("Packages")) {
            for e in rd.flatten() {
                if e.file_name().to_string_lossy().starts_with("Claude_") {
                    let p = e.path().join("LocalCache").join("Roaming").join("Claude").join("claude_desktop_config.json");
                    if p.exists() {
                        out.push(p);
                    }
                }
            }
        }
    }
    if let Ok(a) = std::env::var("APPDATA") {
        let p = std::path::PathBuf::from(a).join("Claude").join("claude_desktop_config.json");
        // only use the classic location if nothing newer exists
        if out.is_empty() {
            out.push(p);
        }
    }
    out
}

fn merge_claude_config(path: &std::path::Path, name: &str, exe: &str, ws: &str) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let mut v: serde_json::Value = if path.exists() {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        std::fs::write(path.with_file_name("claude_desktop_config.backup-by-resolve-2.json"), &text).map_err(|e| e.to_string())?;
        serde_json::from_str(&text).map_err(|e| format!("Claude's settings file isn't valid JSON ({e}); add the snippet by hand"))?
    } else {
        serde_json::json!({})
    };
    if !v.get("mcpServers").map_or(false, |m| m.is_object()) {
        v["mcpServers"] = serde_json::json!({});
    }
    v["mcpServers"][name] = serde_json::json!({"command": exe, "args": ["--mcp", "--workspace", ws]});
    std::fs::write(path, serde_json::to_string_pretty(&v).unwrap()).map_err(|e| e.to_string())
}

pub(crate) fn add_to_claude_desktop(name: &str, exe: &str, ws: &str) -> Result<String, String> {
    let targets = claude_desktop_configs();
    let mut done = vec![];
    for t in &targets {
        merge_claude_config(t, name, exe, ws)?;
        done.push(t.display().to_string());
    }
    Ok(format!("Added Resolve 2 to Claude Desktop ({}). Now fully quit Claude (right-click its icon by the clock → Quit) and open it again; Resolve 2 appears under the tools / connectors menu in a chat.", done.join("; ")))
}

/// Register the MCP server with Claude Code (user-wide), replacing an older entry of the same name.
pub(crate) fn add_to_claude_code(name: &str, exe: &str, ws: &str) -> Result<String, String> {
    let claude = ["claude.cmd", "claude.exe", "claude"]
        .iter()
        .find_map(|c| std::process::Command::new(c).arg("--version").output().ok().filter(|o| o.status.success()).map(|_| c.to_string()))
        .ok_or("Claude Code isn't installed (claude.com/claude-code)")?;
    let _ = std::process::Command::new(&claude).args(["mcp", "remove", "--scope", "user", name]).output();
    let out = std::process::Command::new(&claude).args(["mcp", "add", "--scope", "user", name, "--", exe, "--mcp", "--workspace", ws]).output().map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(format!("Added to Claude Code as '{name}' (all folders). Start a new Claude Code session to use it."))
    } else {
        Err(format!("Claude Code said: {}{}", String::from_utf8_lossy(&out.stderr).trim(), String::from_utf8_lossy(&out.stdout).trim()))
    }
}

/// Desktop + Start menu shortcuts through Windows' own scripting host.
pub(crate) fn make_shortcuts(exe: &str) -> String {
    let dir = std::path::Path::new(exe).parent().map(|d| d.display().to_string()).unwrap_or_default();
    let script = format!(
        "$s=New-Object -ComObject WScript.Shell; foreach($f in @([Environment]::GetFolderPath('Desktop'), (Join-Path ([Environment]::GetFolderPath('Programs')) '')) ){{ $l=$s.CreateShortcut((Join-Path $f 'Resolve 2.lnk')); $l.TargetPath='{}'; $l.WorkingDirectory='{}'; $l.Description='Resolve 2 — pages, tasks and calendar'; $l.Save() }}",
        exe.replace('\'', "''"),
        dir.replace('\'', "''")
    );
    match std::process::Command::new("powershell").args(["-NoProfile", "-NonInteractive", "-Command", &script]).output() {
        Ok(o) if o.status.success() => "Added a Resolve 2 shortcut to your desktop and Start menu.".into(),
        Ok(o) => format!("⚠ Couldn't make the shortcuts: {}", String::from_utf8_lossy(&o.stderr).trim()),
        Err(e) => format!("⚠ Couldn't make the shortcuts: {e}"),
    }
}

fn add_to_codex(name: &str, exe: &str, ws: &str) -> Result<String, String> {
    let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).map_err(|_| "No home folder")?;
    let dir = std::path::PathBuf::from(home).join(".codex");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("config.toml");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let header = format!("[mcp_servers.{name}]");
    if existing.contains(&header) {
        return Ok(format!("{name} is already in {}", path.display()));
    }
    if !existing.is_empty() {
        std::fs::write(dir.join("config.backup-by-resolve2.toml"), &existing).map_err(|e| e.to_string())?;
    }
    let block = format!("\n{header}\ncommand = '{exe}'\nargs = [\"--mcp\", \"--workspace\", \"{ws}\"]\n");
    std::fs::write(&path, format!("{existing}{block}")).map_err(|e| e.to_string())?;
    Ok(format!("Added to {} — restart Codex", path.display()))
}

impl App {
    pub fn settings_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        self.top_bar(ui, p, |_, _| {});
        let tabs = ["General", "Import & export", "Backups", "Sync & phone", "AI & MCP", "Calendars"];
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            centered(ui, 860.0, |ui| {
                view_title(ui, p, "Settings", Some(&format!("Workspace: {} — everything here is stored on this PC", self.store.meta.name)));
                ui.horizontal_wrapped(|ui| {
                    for (i, t) in tabs.iter().enumerate() {
                        if pill(ui, p, t, self.settings_ui.tab == i).clicked() {
                            self.settings_ui.tab = i;
                            self.settings_ui.message.clear();
                        }
                    }
                });
                ui.add_space(10.0);
                if !self.settings_ui.message.is_empty() {
                    egui::Frame::none().fill(p.callout).rounding(8.0).inner_margin(egui::Margin::same(10.0)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new(&self.settings_ui.message).size(13.5));
                    });
                    ui.add_space(8.0);
                }
                match self.settings_ui.tab {
                    0 => self.settings_general(ui, p),
                    1 => self.settings_import(ui, p),
                    2 => self.settings_backups(ui, p),
                    3 => self.settings_sync(ui, p),
                    4 => self.settings_ai(ui, p),
                    _ => self.settings_calendars(ui, p),
                }
                ui.add_space(40.0);
            });
        });
    }

    fn settings_general(&mut self, ui: &mut egui::Ui, p: &Pal) {
        card(ui, p, |ui| {
            ui.label(RichText::new("Appearance").font(fb(16.0)));
            ui.horizontal(|ui| {
                if pill(ui, p, "🌙 Dark", self.reg.settings.dark).clicked() {
                    self.reg.settings.dark = true;
                    self.reg.save();
                }
                if pill(ui, p, "☀ Light", !self.reg.settings.dark).clicked() {
                    self.reg.settings.dark = false;
                    self.reg.save();
                }
            });
        });
        ui.add_space(10.0);
        card(ui, p, |ui| {
            ui.label(RichText::new("This workspace").font(fb(16.0)));
            ui.label(RichText::new("Each workspace is private and isolated: its own folder, backups, sync file, phone token and AI access switch.").size(12.5).color(p.dim));
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("Name");
                let mut name = self.store.meta.name.clone();
                if ui.text_edit_singleline(&mut name).changed() && !name.trim().is_empty() {
                    self.store.meta.name = name.clone();
                    if let Some(w) = self.reg.workspaces.iter_mut().find(|w| w.id == self.store.meta.id) {
                        w.name = name;
                    }
                    self.reg.save();
                }
                for c in [[219u8, 76, 63], [63, 81, 181], [41, 171, 135], [255, 153, 51], [175, 56, 235], [20, 170, 245]] {
                    let (r, resp) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), Sense::click());
                    ui.painter().circle_filled(r.center(), 9.0, rgb(c));
                    if resp.clicked() {
                        self.store.meta.color = c;
                        if let Some(w) = self.reg.workspaces.iter_mut().find(|w| w.id == self.store.meta.id) {
                            w.color = c;
                        }
                        self.reg.save();
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Stored in {}", self.store.dir.display())).size(12.0).color(p.dim));
                if ui.small_button("Open folder").clicked() {
                    open_folder(&self.store.dir);
                }
            });
        });
        ui.add_space(10.0);
        card(ui, p, |ui| {
            ui.label(RichText::new("Opening Resolve 2").font(fb(16.0)));
            let exe = std::env::current_exe().map(|x| x.display().to_string()).unwrap_or_default();
            ui.label(RichText::new(format!("The program is {exe}")).size(12.0).color(p.dim));
            if primary_button(ui, p, "Put Resolve 2 on my desktop and Start menu").clicked() {
                self.settings_ui.message = make_shortcuts(&exe);
            }
            ui.label(RichText::new("Then open it like any other app: double-click the Resolve 2 icon, or press Windows and type Resolve 2. You can pin it to the taskbar from there.").size(12.0).color(p.faint));
        });
        ui.add_space(10.0);
        card(ui, p, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("All workspaces").font(fb(16.0)));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("+ New workspace").clicked() {
                        self.new_ws = Some(String::new());
                    }
                });
            });
            let mut switch: Option<Id> = None;
            for w in self.reg.workspaces.clone() {
                ui.horizontal(|ui| {
                    avatar(ui, &w.name.chars().next().unwrap_or('W').to_string(), rgb(w.color), 22.0);
                    ui.label(&w.name);
                    if w.id == self.store.meta.id {
                        badge(ui, p, true, "open");
                    } else if ui.small_button("Switch").clicked() {
                        switch = Some(w.id.clone());
                    }
                });
            }
            if let Some(s) = switch {
                self.switch_workspace(&s);
            }
            ui.add_space(6.0);
            if self.reg.workspaces.len() > 1 {
                if !self.settings_ui.confirm_remove_ws {
                    if ui.button(RichText::new("Remove this workspace from the list…").color(p.accent)).clicked() {
                        self.settings_ui.confirm_remove_ws = true;
                    }
                } else {
                    ui.label(RichText::new("It disappears from Resolve 2, but its folder (and backups) stay on disk so nothing is lost.").color(p.orange));
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Yes, remove it").color(p.accent)).clicked() {
                            let id = self.store.meta.id.clone();
                            self.reg.workspaces.retain(|w| w.id != id);
                            self.reg.save();
                            let next = self.reg.workspaces[0].id.clone();
                            self.store.meta.id = String::new();
                            self.switch_workspace(&next);
                            self.settings_ui.confirm_remove_ws = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.settings_ui.confirm_remove_ws = false;
                        }
                    });
                }
            }
        });
        ui.add_space(10.0);
        card(ui, p, |ui| {
            ui.label(RichText::new("Keyboard shortcuts").font(fb(16.0)));
            for (k, v) in [
                ("Ctrl+Q", "Quick add a task (natural language: dates, times, p1–p4, #project, @label, every …, remind 30m)"),
                ("Ctrl+E", "Quick add an event"),
                ("Ctrl+K", "Search everything / jump anywhere"),
                ("Ctrl+N", "New page"),
                ("/", "Block menu inside a page"),
                ("[[", "Link to another page"),
                ("Esc", "Close dialogs and panels"),
            ] {
                ui.horizontal(|ui| {
                    ui.add_sized([70.0, 18.0], egui::Label::new(RichText::new(k).monospace().color(p.accent)));
                    ui.label(RichText::new(v).size(13.0));
                });
            }
        });
    }

    fn settings_import(&mut self, ui: &mut egui::Ui, p: &Pal) {
        card(ui, p, |ui| {
            ui.label(RichText::new("Import").font(fb(16.0)));
            ui.label(RichText::new("A backup is made before every import. Importing the same file twice won't duplicate pages, databases or events.").size(12.5).color(p.dim));
            ui.add_space(8.0);
            let mut run: Option<(&str, std::path::PathBuf)> = None;
            ui.label(RichText::new("📄 Notion").font(fs(14.0)));
            ui.label(RichText::new("In Notion: Settings → Export all workspace content → Markdown & CSV (include subpages). Pages, nested pages, links and databases come across.").size(12.5).color(p.dim));
            ui.horizontal(|ui| {
                if ui.button("Choose export .zip…").clicked() {
                    if let Some(f) = rfd::FileDialog::new().add_filter("Notion export", &["zip"]).pick_file() {
                        run = Some(("notion", f));
                    }
                }
                if ui.button("Choose unzipped folder…").clicked() {
                    if let Some(f) = rfd::FileDialog::new().pick_folder() {
                        run = Some(("notion", f));
                    }
                }
            });
            ui.add_space(8.0);
            ui.label(RichText::new("✔ Todoist").font(fs(14.0)));
            ui.label(RichText::new("In Todoist: Settings → Backups → download a backup (.zip of CSVs), or open a project → ⋯ → Export as CSV. Projects, sections, subtasks, priorities, dates and repeats come across.").size(12.5).color(p.dim));
            if ui.button("Choose Todoist .zip or .csv…").clicked() {
                if let Some(f) = rfd::FileDialog::new().add_filter("Todoist", &["zip", "csv"]).pick_file() {
                    run = Some(("todoist", f));
                }
            }
            ui.add_space(8.0);
            ui.label(RichText::new("📆 Google Calendar (or any .ics)").font(fs(14.0)));
            ui.label(RichText::new("In Google Calendar: Settings → Import & export → Export (a .zip with one .ics per calendar). Each becomes its own calendar. To keep a calendar live instead, subscribe to it in the Calendars tab.").size(12.5).color(p.dim));
            if ui.button("Choose .zip or .ics…").clicked() {
                if let Some(f) = rfd::FileDialog::new().add_filter("Calendar", &["zip", "ics"]).pick_file() {
                    run = Some(("ics", f));
                }
            }
            if let Some((kind, path)) = run {
                let _ = self.store.backup_now();
                let res = match kind {
                    "notion" => crate::import::import_notion(&path, &mut self.store.data),
                    "todoist" => crate::import::import_todoist(&path, &mut self.store.data),
                    _ => crate::import::import_ics(&path, &mut self.store.data),
                };
                self.settings_ui.message = match res {
                    Ok(rep) => {
                        self.touch();
                        let mut m = rep.summary();
                        for n in rep.notes.iter().take(5) {
                            m.push_str(&format!("\n• {n}"));
                        }
                        m
                    }
                    Err(e) => format!("⚠ {e}"),
                };
            }
        });
        ui.add_space(10.0);
        card(ui, p, |ui| {
            ui.label(RichText::new("Export — your data, in open formats").font(fb(16.0)));
            ui.label(RichText::new("Writes every page as Markdown, every database as CSV, all events as .ics, tasks as a Markdown checklist, plus the full workspace as JSON.").size(12.5).color(p.dim));
            if ui.button("Export everything to a folder…").clicked() {
                if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                    self.settings_ui.message = match self.store.export_markdown(&dir) {
                        Ok(n) => format!("Exported {n} pages and databases to {}", dir.display()),
                        Err(e) => format!("⚠ Export failed: {e}"),
                    };
                }
            }
        });
    }

    fn settings_backups(&mut self, ui: &mut egui::Ui, p: &Pal) {
        card(ui, p, |ui| {
            ui.label(RichText::new("Backups").font(fb(16.0)));
            ui.horizontal(|ui| {
                if ui.checkbox(&mut self.store.settings.auto_backup, "Back up automatically every day").changed() {
                    self.store.save_settings();
                }
                ui.label("Keep");
                if ui.add(egui::DragValue::new(&mut self.store.settings.keep_backups).range(1..=365)).changed() {
                    self.store.save_settings();
                }
                ui.label("backups");
            });
            ui.horizontal(|ui| {
                if primary_button(ui, p, "Back up now").clicked() {
                    self.save_now();
                    self.settings_ui.message = match self.store.backup_now() {
                        Ok(pth) => format!("Backed up to {}", pth.display()),
                        Err(e) => format!("⚠ {e}"),
                    };
                }
                if ui.button("Open backups folder").clicked() {
                    let _ = std::fs::create_dir_all(self.store.backups_dir());
                    open_folder(&self.store.backups_dir());
                }
            });
            ui.label(RichText::new("Tip: point the sync folder at OneDrive/Google Drive, or copy the backups folder to a USB drive, for an off-PC copy.").size(12.0).color(p.dim));
            ui.add_space(8.0);
            let list = self.store.list_backups();
            if list.is_empty() {
                ui.label(RichText::new("No backups yet").color(p.dim));
            }
            for (path, when, size) in list.iter().take(30) {
                ui.horizontal(|ui| {
                    let dt: chrono::DateTime<chrono::Local> = (*when).into();
                    ui.label(RichText::new(dt.format("%a %-d %b %Y  %H:%M").to_string()).size(13.0));
                    ui.label(RichText::new(format!("{} KB", size / 1024)).size(12.0).color(p.dim));
                    if self.settings_ui.confirm_restore.as_ref() == Some(path) {
                        if ui.button(RichText::new("Confirm restore").color(p.accent)).clicked() {
                            let pth = path.clone();
                            self.settings_ui.message = match self.store.restore(&pth) {
                                Ok(()) => "Restored. (Your state just before restoring was backed up too.)".into(),
                                Err(e) => format!("⚠ {e}"),
                            };
                            self.settings_ui.confirm_restore = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.settings_ui.confirm_restore = None;
                        }
                    } else if ui.small_button("Restore…").clicked() {
                        self.settings_ui.confirm_restore = Some(path.clone());
                    }
                });
            }
        });
    }

    fn settings_sync(&mut self, ui: &mut egui::Ui, p: &Pal) {
        card(ui, p, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("📱 Android phone").font(fb(16.0)));
                badge(ui, p, self.server.is_some(), if self.server.is_some() { "running" } else { "off" });
            });
            ui.label(RichText::new("Your phone opens this workspace as a web app served by this PC: Today, Upcoming, Inbox, calendar agenda, search, quick add and completing tasks. Changes appear here within seconds. Works on the same Wi-Fi; nothing goes through anyone's cloud.").size(12.5).color(p.dim));
            ui.add_space(6.0);
            let mut on = self.store.settings.server_enabled;
            if ui.checkbox(&mut on, "Serve the phone page (and MCP over HTTP) on this network").changed() {
                self.store.settings.server_enabled = on;
                self.store.save_settings();
                self.ensure_server();
            }
            ui.horizontal(|ui| {
                ui.label("Port");
                let mut port = self.store.settings.server_port;
                if ui.add(egui::DragValue::new(&mut port).range(1024..=65000)).changed() {
                    self.store.settings.server_port = port;
                    self.store.save_settings();
                    self.ensure_server();
                }
                if ui.button("New secret token").on_hover_text("Old phone links and connectors stop working").clicked() {
                    self.store.settings.token = crate::store::new_token();
                    self.store.save_settings();
                }
            });
            if let Some(e) = &self.server_error {
                ui.label(RichText::new(format!("⚠ {e}")).color(p.accent));
            }
            let ip = crate::server::lan_ip().unwrap_or_else(|| "this-pc".into());
            let url = format!("http://{ip}:{}/?token={}", self.store.settings.server_port, self.store.settings.token);
            ui.label(RichText::new("Open this on your phone (then ⋮ → Add to Home screen):").size(12.5));
            code_box(ui, p, &url);
            ui.label(RichText::new("Needs extra setup to work away from home: install Tailscale (free) on the PC and phone and use the PC's Tailscale address instead of the Wi-Fi one. Windows may ask once to allow Resolve 2 through the firewall.").size(12.0).color(p.orange));
        });
        ui.add_space(10.0);
        card(ui, p, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🌍 Phone anywhere — always-on hub").font(fb(16.0)));
                badge(ui, p, false, "needs Tailscale (free) — set up when you're ready");
            });
            ui.label(RichText::new("One always-on machine is the hub: it serves the phone page, MCP for ChatGPT, and /sync for your other devices. Every device keeps a full offline copy and merges with the hub, so the hub can move without rebuilding anything or losing data.").size(12.5).color(p.dim));
            ui.add_space(6.0);
            ui.label(RichText::new("Today — this PC is the hub").font(fs(13.5)));
            ui.label(RichText::new("1. Turn on \"Serve the phone page\" above.  2. Install Tailscale on this PC and your Android phone and sign in to both with the same account.  3. On the phone open the link above, but with this PC's Tailscale name (e.g. http://my-pc:PORT/?token=…), and Add to Home screen. Tailscale encrypts the connection and nobody outside your account can reach it.").size(12.5));
            ui.add_space(6.0);
            ui.label(RichText::new("Later — move the hub to a cloud server").font(fs(13.5)));
            ui.label(RichText::new("1. On the server run the same program headless:").size(12.5));
            code_box(ui, p, &format!("resolve2 --serve {} --data-dir /srv/resolve2 --workspace \"{}\" --token {}", self.store.settings.server_port, self.store.meta.name, self.store.settings.token));
            ui.label(RichText::new("2. Put its address below and press Sync now — this PC uploads everything and the two stay merged from then on. 3. Point the phone (and ChatGPT) at the server's address. Your PC keeps working offline and catches up whenever it reconnects. Nothing is lost: each device holds a full copy, and backups continue here.").size(12.5));
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("Hub address");
                let mut url = self.store.settings.hub_url.clone();
                if ui.add(egui::TextEdit::singleline(&mut url).hint_text("http://my-server:8797  (empty = this PC is the hub)").desired_width(360.0)).changed() {
                    self.store.settings.hub_url = url;
                    self.store.save_settings();
                }
            });
            ui.horizontal(|ui| {
                ui.label("Hub token  ");
                let mut tok = self.store.settings.hub_token.clone();
                if ui.add(egui::TextEdit::singleline(&mut tok).hint_text("the --token you gave the hub").password(true).desired_width(360.0)).changed() {
                    self.store.settings.hub_token = tok;
                    self.store.save_settings();
                }
            });
            ui.horizontal(|ui| {
                if ui.add_enabled(!self.store.settings.hub_url.trim().is_empty(), egui::Button::new("Sync with hub now")).clicked() {
                    self.save_now();
                    self.hub_tick(true);
                }
                if !self.hub_status.is_empty() {
                    ui.label(RichText::new(&self.hub_status).size(12.0).color(if self.hub_status.starts_with("hub:") && !self.hub_status.contains("syncing") { p.orange } else { p.dim }));
                }
            });
        });
        ui.add_space(10.0);
        card(ui, p, |ui| {
            ui.label(RichText::new("🔄 Sync folder").font(fb(16.0)));
            ui.label(RichText::new("Keeps this workspace in step with other PCs (or a backup drive) through any shared folder: Syncthing (free, open-source, peer-to-peer), OneDrive, Google Drive or Dropbox. Edits merge item by item — the newest change wins and deletions are respected. Every minute while open.").size(12.5).color(p.dim));
            ui.horizontal(|ui| {
                let mut dir = self.store.settings.sync_dir.clone();
                if ui.add(egui::TextEdit::singleline(&mut dir).hint_text("C:\\Users\\you\\OneDrive\\Resolve 2").desired_width(420.0)).changed() {
                    self.store.settings.sync_dir = dir;
                    self.store.save_settings();
                }
                if ui.button("Choose…").clicked() {
                    if let Some(f) = rfd::FileDialog::new().pick_folder() {
                        self.store.settings.sync_dir = f.display().to_string();
                        self.store.save_settings();
                    }
                }
            });
            ui.horizontal(|ui| {
                if primary_button(ui, p, "Sync now").clicked() {
                    self.save_now();
                    self.settings_ui.message = match self.store.sync() {
                        Ok(m) => {
                            self.sync_status = format!("synced {}", chrono::Local::now().format("%H:%M"));
                            m
                        }
                        Err(e) => format!("⚠ {e}"),
                    };
                }
                if self.store.settings.last_sync > 0 {
                    let t = chrono::DateTime::from_timestamp_millis(self.store.settings.last_sync).map(|x| x.with_timezone(&chrono::Local).format("%-d %b %H:%M").to_string()).unwrap_or_default();
                    ui.label(RichText::new(format!("Last synced {t}")).size(12.0).color(p.dim));
                }
            });
            if let Some(f) = self.store.sync_file() {
                ui.label(RichText::new(format!("File: {}", f.display())).size(11.5).color(p.faint));
            }
        });
    }

    fn settings_ai(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let exe = std::env::current_exe().map(|x| x.display().to_string()).unwrap_or_else(|_| "resolve2.exe".into());
        let ws = self.store.meta.id.clone();
        let sname = format!("resolve2_{}", crate::store::sanitize(&self.store.meta.name).to_lowercase().replace([' ', '-', '.'], "_"));
        card(ui, p, |ui| {
            ui.label(RichText::new("🔐 AI access to this workspace").font(fb(16.0)));
            ui.label(RichText::new("Assistants connect through MCP (Model Context Protocol) and only ever see the workspace you name in their settings. No subscription to Notion AI needed — use the assistant you already have.").size(12.5).color(p.dim));
            if ui.checkbox(&mut self.store.settings.ai_access, "Allow AI assistants to open this workspace").changed() {
                self.store.save_settings();
            }
            if ui.checkbox(&mut self.store.settings.ai_read_only, "Read-only (they can read and search, but not change anything)").changed() {
                self.store.save_settings();
            }
            ui.add_space(4.0);
            let names: Vec<String> = crate::mcp::tools(self.store.settings.ai_read_only).as_array().map(|a| a.iter().filter_map(|t| t["name"].as_str().map(|s| s.to_string())).collect()).unwrap_or_default();
            ui.label(RichText::new(format!("{} tools: {}", names.len(), names.join(", "))).size(11.5).color(p.faint));
        });
        ui.add_space(10.0);
        card(ui, p, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Claude Desktop").font(fb(15.0)));
                badge(ui, p, true, "works locally");
            });
            let snippet = format!("{{\n  \"mcpServers\": {{\n    \"{sname}\": {{\n      \"command\": {},\n      \"args\": [\"--mcp\", \"--workspace\", \"{ws}\"]\n    }}\n  }}\n}}", serde_json::to_string(&exe).unwrap());
            code_box(ui, p, &snippet);
            if primary_button(ui, p, "Add to Claude Desktop for me").clicked() {
                self.settings_ui.message = add_to_claude_desktop(&sname, &exe, &ws).unwrap_or_else(|e| format!("⚠ {e}"));
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Claude Code").font(fb(15.0)));
                badge(ui, p, true, "works locally");
            });
            code_box(ui, p, &format!("claude mcp add --scope user {sname} -- \"{exe}\" --mcp --workspace {ws}"));
            if ui.button("Add to Claude Code for me").clicked() {
                self.settings_ui.message = add_to_claude_code(&sname, &exe, &ws).unwrap_or_else(|e| format!("⚠ {e}"));
            }
            ui.label(RichText::new(format!("Claude Desktop settings file(s) used: {}", claude_desktop_configs().iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("; "))).size(11.0).color(p.faint));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Codex (CLI / IDE)").font(fb(15.0)));
                badge(ui, p, true, "works locally");
            });
            code_box(ui, p, &format!("# ~/.codex/config.toml\n[mcp_servers.{sname}]\ncommand = '{exe}'\nargs = [\"--mcp\", \"--workspace\", \"{ws}\"]"));
            if ui.button("Add to Codex config for me").clicked() {
                self.settings_ui.message = add_to_codex(&sname, &exe, &ws).unwrap_or_else(|e| format!("⚠ {e}"));
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("ChatGPT").font(fb(15.0)));
                badge(ui, p, false, "needs extra setup");
            });
            ui.label(RichText::new("ChatGPT connects from OpenAI's servers, so it needs a public HTTPS address for this PC. 1) Turn on the phone page/HTTP server (Sync & phone tab). 2) Run a free tunnel such as `cloudflared tunnel --url http://localhost:PORT` (or Tailscale Funnel). 3) In ChatGPT → Settings → Apps & Connectors → Developer mode → create a connector with the https address below (no authentication; the token is in the link). Keep the link private.").size(12.5).color(p.dim));
            code_box(ui, p, &format!("https://<your-tunnel-address>/mcp?token={}", self.store.settings.token));
            ui.label(RichText::new(format!("Local test address: http://localhost:{}/mcp?token=…", self.store.settings.server_port)).size(11.5).color(p.faint));
        });
        ui.add_space(10.0);
        card(ui, p, |ui| {
            ui.label(RichText::new("✨ The in-app assistant (like Notion AI)").font(fb(16.0)));
            ui.label(RichText::new("Powers ✨ Ask AI in the sidebar: it answers questions and, if you let it, makes changes for you — writes pages, creates tasks and events, fills in database rows. Pick who does the thinking:").size(12.5).color(p.dim));
            let mut changed = false;
            let before = self.reg.settings.ai_provider.clone();
            ui.label(RichText::new("API providers (pay per use, tracked below):").size(12.5).color(p.dim));
            ui.radio_value(&mut self.reg.settings.ai_provider, "anthropic".to_string(), "Claude API key");
            ui.radio_value(&mut self.reg.settings.ai_provider, "openai-cloud".to_string(), "OpenAI API key");
            ui.label(RichText::new("Local models (free, private):").size(12.5).color(p.dim));
            ui.radio_value(&mut self.reg.settings.ai_provider, "ollama".to_string(), "Ollama — runs on this PC");
            ui.radio_value(&mut self.reg.settings.ai_provider, "openai".to_string(), "Other OpenAI-compatible server (LM Studio, llama.cpp, Jan, a hosted endpoint)");
            ui.label(RichText::new("Personal / development only — not for a commercial deployment:").size(12.5).color(p.orange));
            ui.radio_value(&mut self.reg.settings.ai_provider, "claude-code".to_string(), "Claude Code CLI (your own Claude subscription)");
            ui.radio_value(&mut self.reg.settings.ai_provider, "codex-cli".to_string(), "Codex CLI (your own ChatGPT subscription)");
            if self.reg.settings.ai_provider != before {
                changed = true;
                let (url, model) = match self.reg.settings.ai_provider.as_str() {
                    "claude-code" | "codex-cli" => ("", ""),
                    "anthropic" => ("https://api.anthropic.com", crate::ai::CLAUDE_MODEL),
                    "openai-cloud" => ("https://api.openai.com", ""),
                    "openai" => ("http://localhost:1234", ""),
                    _ => ("http://localhost:11434", "llama3.2"),
                };
                self.reg.settings.ai_url = url.into();
                self.reg.settings.ai_model = model.into();
            }
            match self.reg.settings.ai_provider.as_str() {
                "claude-code" | "codex-cli" => {
                    let claude = self.reg.settings.ai_provider == "claude-code";
                    ui.label(RichText::new(if claude {
                        "Needs Claude Code on this PC, signed in with your Claude account: install it from claude.com/claude-code, open a terminal, type `claude` and sign in once. Usage counts against your subscription's normal limits — no API bill."
                    } else {
                        "Needs Codex on this PC, signed in with ChatGPT: in a terminal run `npm install -g @openai/codex`, then `codex` and choose Sign in with ChatGPT. Usage counts against your plan's limits — no API bill."
                    }).size(11.5).color(p.faint));
                    if ui.button("Check it's installed").clicked() {
                        self.settings_ui.message = crate::ai::cli_status(&self.reg.settings.ai_provider).unwrap_or_else(|e| format!("⚠ {e}"));
                    }
                }
                "anthropic" => {
                    ui.horizontal(|ui| {
                        ui.label("API key");
                        changed |= ui.add(egui::TextEdit::singleline(&mut self.reg.settings.anthropic_key).password(true).hint_text("sk-ant-…").desired_width(340.0)).changed();
                    });
                    ui.label(RichText::new("Get a key at console.anthropic.com → API keys. A Claude Pro/Max subscription doesn't include API use; to use your subscription instead, chat in the Claude app connected through MCP (above).").size(11.5).color(p.faint));
                }
                "openai-cloud" => {
                    ui.horizontal(|ui| {
                        ui.label("API key");
                        changed |= ui.add(egui::TextEdit::singleline(&mut self.reg.settings.openai_key).password(true).hint_text("sk-…").desired_width(340.0)).changed();
                    });
                    ui.label(RichText::new("Get a key at platform.openai.com → API keys, and type the model name your account offers. A ChatGPT Plus subscription doesn't include API use; to use your subscription instead, connect ChatGPT through MCP (above).").size(11.5).color(p.faint));
                }
                _ => {
                    ui.label(RichText::new("Free and private. Install Ollama from ollama.com, then run `ollama pull llama3.2` (small, any PC) or `ollama pull qwen3` (better at taking actions, needs more memory). Press Detect models below.").size(11.5).color(p.faint));
                    ui.horizontal(|ui| {
                        ui.label("Address");
                        changed |= ui.text_edit_singleline(&mut self.reg.settings.ai_url).changed();
                    });
                }
            }
            let uses_subscription = matches!(self.reg.settings.ai_provider.as_str(), "claude-code" | "codex-cli");
            if !uses_subscription { ui.horizontal(|ui| {
                ui.label("Model");
                changed |= ui.text_edit_singleline(&mut self.reg.settings.ai_model).changed();
                if matches!(self.reg.settings.ai_provider.as_str(), "ollama" | "openai") && ui.button("Detect models").clicked() {
                    self.settings_ui.models_rx = Some(crate::ai::list_models_async(&self.reg.settings.ai_provider, &self.reg.settings.ai_url));
                }
            }); }
            if changed {
                self.reg.save();
            }
            if let Some(rx) = &self.settings_ui.models_rx {
                if let Ok(r) = rx.try_recv() {
                    match r {
                        Ok(m) => {
                            self.settings_ui.message = if m.is_empty() { "Connected, but no models are installed yet (try: ollama pull llama3.2)".into() } else { format!("Found {} models", m.len()) };
                            self.settings_ui.models = m;
                        }
                        Err(e) => self.settings_ui.message = format!("⚠ {e}"),
                    }
                    self.settings_ui.models_rx = None;
                } else {
                    ui.spinner();
                }
            }
            let models = self.settings_ui.models.clone();
            ui.horizontal_wrapped(|ui| {
                for m in models {
                    if pill(ui, p, &m, self.reg.settings.ai_model == m).clicked() {
                        self.reg.settings.ai_model = m;
                        self.reg.save();
                    }
                }
            });
        });
        ui.add_space(10.0);
        card(ui, p, |ui| {
            ui.label(RichText::new("💷 AI usage and cost (this month)").font(fb(16.0)));
            let now = chrono::Local::now();
            let since = chrono::NaiveDate::from_ymd_opt(chrono::Datelike::year(&now), chrono::Datelike::month(&now), 1)
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .and_then(|d| d.and_local_timezone(chrono::Local).single())
                .map(|d| d.timestamp_millis())
                .unwrap_or(0);
            let u = crate::ai::usage_since(since);
            let s = &mut self.reg.settings;
            let mut changed = false;
            ui.horizontal(|ui| {
                ui.label("Price per million input tokens");
                changed |= ui.add(egui::DragValue::new(&mut s.price_in_per_m).speed(0.05).range(0.0..=1000.0).prefix("$")).changed();
                ui.label("output");
                changed |= ui.add(egui::DragValue::new(&mut s.price_out_per_m).speed(0.05).range(0.0..=1000.0).prefix("$")).changed();
            });
            ui.horizontal(|ui| {
                ui.label("Monthly budget");
                changed |= ui.add(egui::DragValue::new(&mut s.monthly_budget).speed(0.5).range(0.0..=100000.0).prefix("$")).changed();
                ui.label(RichText::new("(0 = no limit; guided AI steps stop when it's reached)").size(11.5).color(p.faint));
            });
            let cost = (u.input as f64 * s.price_in_per_m + u.output as f64 * s.price_out_per_m) / 1_000_000.0;
            ui.label(RichText::new(format!("{} calls · {} input + {} output tokens · ≈ ${:.2} at the prices above", u.calls, u.input, u.output, cost)).size(13.0));
            for (m, c, i, o) in &u.by_model {
                ui.label(RichText::new(format!("{m}: {c} calls, {i} in / {o} out")).size(11.5).color(p.faint));
            }
            ui.label(RichText::new("Enter your provider's current prices; Resolve 2 doesn't guess them. CLI subscription providers don't report tokens.").size(11.0).color(p.faint));
            if changed {
                self.reg.save();
            }
        });
    }

    fn settings_calendars(&mut self, ui: &mut egui::Ui, p: &Pal) {
        card(ui, p, |ui| {
            ui.label(RichText::new("Subscribe to an external calendar").font(fb(16.0)));
            ui.label(RichText::new("Paste an iCal link — in Google Calendar: Settings → (your calendar) → Integrate calendar → Secret address in iCal format. Outlook and Apple calendars have the same kind of link. Resolve 2 refreshes it every 30 minutes while open; subscribed events are read-only here (edit them at the source).").size(12.5).color(p.dim));
            let mut name = self.temp.get("sub_name").cloned().unwrap_or_default();
            let mut url = self.temp.get("sub_url").cloned().unwrap_or_default();
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut name).hint_text("Name, e.g. Work (Google)").desired_width(200.0));
                ui.add(egui::TextEdit::singleline(&mut url).hint_text("https://calendar.google.com/…/basic.ics").desired_width(380.0));
            });
            self.temp.insert("sub_name".into(), name.clone());
            self.temp.insert("sub_url".into(), url.clone());
            if primary_button(ui, p, "Subscribe").clicked() && url.trim().starts_with("http") {
                let cal = Calendar { id: new_id(), name: if name.trim().is_empty() { "Subscribed".into() } else { name.trim().to_string() }, color: [97, 97, 97], visible: true, source_url: url.trim().to_string(), last_fetch: 0, updated: now_ms() };
                let id = cal.id.clone();
                self.store.data.calendars.push(cal);
                self.touch();
                self.refresh_subscription(&id);
                self.temp.remove("sub_name");
                self.temp.remove("sub_url");
            }
        });
        ui.add_space(10.0);
        card(ui, p, |ui| {
            ui.label(RichText::new("Calendars in this workspace").font(fb(16.0)));
            for c in self.store.data.calendars.clone() {
                ui.horizontal(|ui| {
                    ui.colored_label(rgb(c.color), "●");
                    ui.label(&c.name);
                    let n = self.store.data.events.iter().filter(|e| e.calendar_id == c.id).count();
                    ui.label(RichText::new(format!("{n} events")).size(12.0).color(p.dim));
                    if !c.source_url.is_empty() {
                        badge(ui, p, true, "subscribed");
                        if ui.small_button("Refresh").clicked() {
                            self.refresh_subscription(&c.id);
                        }
                    }
                });
            }
            ui.label(RichText::new("Add, rename, recolour and hide calendars from the Calendar screen. Your own calendars can be exported as .ics from Import & export.").size(12.0).color(p.faint));
        });
    }

    /// Fetch a subscribed calendar in the background.
    pub fn refresh_subscription(&mut self, cal_id: &str) {
        let Some(cal) = self.store.data.calendar(cal_id) else { return };
        if cal.source_url.is_empty() {
            return;
        }
        let url = cal.source_url.clone();
        let id = cal_id.to_string();
        let tx = self.sub_tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send((id, crate::import::fetch_text(&url)));
        });
        self.settings_ui.message = "Fetching the calendar…".into();
    }
}
