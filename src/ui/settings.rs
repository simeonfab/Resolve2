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

/// One-line advice about the chosen local model.
pub fn model_note(provider: &str, model: &str) -> Option<&'static str> {
    if provider != "ollama" {
        return None;
    }
    let m = model.to_lowercase();
    if m.starts_with("llama3.2") {
        Some("Fine for trying the flow, weak for real projects. qwen3 is recommended for the intake.")
    } else if m.is_empty() {
        Some("Recommended for the intake: qwen3.")
    } else {
        None
    }
}

/// Longer explanations live behind a "Learn more" toggle.
fn learn_more(ui: &mut egui::Ui, p: &Pal, id: &str, text: &str) {
    egui::CollapsingHeader::new(RichText::new("Learn more").size(size::SMALL).color(p.dim)).id_salt(("learn", id)).show(ui, |ui| {
        wrap_label(ui, RichText::new(text).size(size::SMALL).color(p.dim));
    });
}

fn badge(ui: &mut egui::Ui, p: &Pal, ok: bool, text: &str) {
    chip(ui, text, if ok { p.green } else { p.orange });
}

pub(super) fn open_folder(path: &std::path::Path) {
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
        .find_map(|c| crate::ai::cmd(c).arg("--version").output().ok().filter(|o| o.status.success()).map(|_| c.to_string()))
        .ok_or("Claude Code isn't installed (claude.com/claude-code)")?;
    let _ = crate::ai::cmd(&claude).args(["mcp", "remove", "--scope", "user", name]).output();
    let out = crate::ai::cmd(&claude).args(["mcp", "add", "--scope", "user", name, "--", exe, "--mcp", "--workspace", ws]).output().map_err(|e| e.to_string())?;
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
        "$s=New-Object -ComObject WScript.Shell; foreach($f in @([Environment]::GetFolderPath('Desktop'), (Join-Path ([Environment]::GetFolderPath('Programs')) '')) ){{ $l=$s.CreateShortcut((Join-Path $f 'Resolve 2.lnk')); $l.TargetPath='{}'; $l.WorkingDirectory='{}'; $l.Description='Resolve 2: pages, tasks and calendar'; $l.Save() }}",
        exe.replace('\'', "''"),
        dir.replace('\'', "''")
    );
    match crate::ai::cmd("powershell").args(["-NoProfile", "-NonInteractive", "-Command", &script]).output() {
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
    Ok(format!("Added to {}. Restart Codex.", path.display()))
}

impl App {
    pub fn settings_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        self.settings_page(ui, p);
    }

    /// Everything deleted in the last 60 days — by you, the assistant or an MCP client — restorable.
    pub fn recently_deleted(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let list: Vec<crate::history::ChangeSet> = self.store.data.recently_deleted().into_iter().take(50).cloned().collect();
        let mut restore: Option<String> = None;
        card(ui, p, |ui| {
            ui.label(RichText::new("🗑 Recently deleted").font(fb(16.0)));
            ui.label(RichText::new("Deletes are soft: anything deleted in the last 60 days (by you, the assistant or Claude/ChatGPT over MCP) can be put back.").size(12.0).color(p.faint));
            if list.is_empty() {
                ui.label(RichText::new("Nothing deleted recently.").color(p.dim));
            }
            for cs in &list {
                ui.horizontal(|ui| {
                    let when = chrono::DateTime::from_timestamp_millis(cs.created).map(|x| x.with_timezone(&chrono::Local).format("%-d %b %H:%M").to_string()).unwrap_or_default();
                    ui.label(RichText::new(when).size(11.5).color(p.faint));
                    ui.label(RichText::new(&cs.label).size(13.0));
                    let items: Vec<String> = cs.changes.iter().filter(|c| c.before.is_some() && c.after_updated == 0).take(4).map(crate::history::item_label).collect();
                    ui.label(RichText::new(items.join(" · ")).size(11.5).color(p.dim));
                    if ui.small_button("Restore").clicked() {
                        restore = Some(cs.id.clone());
                    }
                });
            }
        });
        if let Some(id) = restore {
            self.undo(&id);
        }
    }

    /// AI tab: the other providers (OpenAI, other local servers, Codex) and usage.
    pub(super) fn settings_ai_more(&mut self, ui: &mut egui::Ui, p: &Pal) {
        card(ui, p, |ui| {
            ui.label(RichText::new("Other AI options").font(fb(16.0)));
            ui.label(RichText::new("The AI Resolve uses now. Connect your AI (above) sets this for you; other providers are here.").size(12.5).color(p.dim));
            let mut changed = false;
            let before = self.reg.settings.ai_provider.clone();
            ui.label(RichText::new("API providers (pay per use, tracked below):").size(12.5).color(p.dim));
            ui.radio_value(&mut self.reg.settings.ai_provider, "anthropic".to_string(), "Claude API key");
            ui.radio_value(&mut self.reg.settings.ai_provider, "openai-cloud".to_string(), "OpenAI API key");
            ui.label(RichText::new("Local models (free, private):").size(12.5).color(p.dim));
            ui.radio_value(&mut self.reg.settings.ai_provider, "ollama".to_string(), "Ollama (runs on this PC)");
            ui.radio_value(&mut self.reg.settings.ai_provider, "openai".to_string(), "Other OpenAI-compatible server (LM Studio, llama.cpp, Jan, a hosted endpoint)");
            ui.label(RichText::new("Personal / development only, not for a commercial deployment:").size(12.5).color(p.orange));
            ui.radio_value(&mut self.reg.settings.ai_provider, "claude-code".to_string(), "Claude Code CLI (your own Claude subscription)");
            ui.radio_value(&mut self.reg.settings.ai_provider, "codex-cli".to_string(), "Codex CLI (your own ChatGPT subscription)");
            if self.reg.settings.ai_provider != before {
                changed = true;
                let (url, model) = match self.reg.settings.ai_provider.as_str() {
                    "claude-code" | "codex-cli" => ("", ""),
                    "anthropic" => ("https://api.anthropic.com", crate::ai::CLAUDE_MODEL),
                    "openai-cloud" => ("https://api.openai.com", ""),
                    "openai" => ("http://localhost:1234", ""),
                    _ => ("http://localhost:11434", "qwen3"),
                };
                self.reg.settings.ai_url = url.into();
                self.reg.settings.ai_model = model.into();
            }
            match self.reg.settings.ai_provider.as_str() {
                "claude-code" | "codex-cli" => {
                    let claude = self.reg.settings.ai_provider == "claude-code";
                    ui.label(RichText::new(if claude {
                        "Needs Claude Code on this PC, signed in with your Claude account: install it from claude.com/claude-code, open a terminal, type `claude` and sign in once. Usage counts against your subscription's normal limits, with no API bill."
                    } else {
                        "Needs Codex on this PC, signed in with ChatGPT: in a terminal run `npm install -g @openai/codex`, then `codex` and choose Sign in with ChatGPT. Usage counts against your plan's limits, with no API bill."
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
                    ui.label(RichText::new("Free and private. Install Ollama from ollama.com. For the Resolve intake use **qwen3** (`ollama pull qwen3`). It follows the intake's rules far better than llama3.2. Press Detect models below.").size(11.5).color(p.faint));
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
            if let Some(note) = model_note(&self.reg.settings.ai_provider, &self.reg.settings.ai_model) {
                ui.label(RichText::new(note).size(11.5).color(p.orange));
            }
            if changed {
                self.reg.save();
            }
            if let Some(rx) = &self.settings_ui.models_rx {
                if let Ok(r) = rx.try_recv() {
                    match r {
                        Ok(m) => {
                            self.settings_ui.message = if m.is_empty() { "Connected, but no models are installed yet (try: ollama pull qwen3)".into() } else { format!("Found {} models", m.len()) };
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
    }

    pub(super) fn settings_calendars(&mut self, ui: &mut egui::Ui, p: &Pal) {
        card(ui, p, |ui| {
            ui.label(RichText::new("Subscribe to an external calendar").font(fb(16.0)));
            ui.label(RichText::new("Paste an iCal link. In Google Calendar: Settings → (your calendar) → Integrate calendar → Secret address in iCal format. Outlook and Apple calendars have the same kind of link. Resolve 2 refreshes it every 30 minutes while open; subscribed events are read-only here (edit them at the source).").size(12.5).color(p.dim));
            let mut name = self.temp.get("sub_name").cloned().unwrap_or_default();
            let mut url = self.temp.get("sub_url").cloned().unwrap_or_default();
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut name).hint_text("Name, e.g. Work (Google)").desired_width(200.0));
                ui.add(egui::TextEdit::singleline(&mut url).hint_text("https://calendar.google.com/…/basic.ics").desired_width(380.0));
            });
            self.temp.insert("sub_name".into(), name.clone());
            self.temp.insert("sub_url".into(), url.clone());
            if primary_button(ui, p, "Subscribe").clicked() && url.trim().starts_with("http") {
                let cal = Calendar { id: new_id(), name: if name.trim().is_empty() { "Subscribed".into() } else { name.trim().to_string() }, color: PICK_COLOURS[0], visible: true, source_url: url.trim().to_string(), last_fetch: 0, updated: now_ms() };
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
