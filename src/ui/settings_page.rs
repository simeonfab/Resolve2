//! Settings in four tabs (C-E21, Pack C "Settings"): AI and MCP (Connect your AI, the MCP setup
//! rows and the Spending card, labelled Estimated), Sync and phone (the phone link as text, the
//! sync folder, backups, the hub), Workspaces (all workspaces, Open Resolve 2 with, import and
//! export) and About (version, privacy, Report a problem, the data folder, appearance,
//! shortcuts and the changelog). Everything that was in the earlier tabs is still here, folded
//! under each tab where the canvas doesn't draw it.

use super::nav::SETTINGS_TABS;
use super::settings::{add_to_claude_code, add_to_claude_desktop, claude_desktop_configs};
use super::theme::*;
use super::widgets::*;
use super::*;
use eframe::egui::{self, RichText};

/// The changelog, shipped inside the program: About shows the same text, newest first.
pub const CHANGELOG: &str = include_str!("../../CHANGELOG.md");

/// The four AI choices: (key, name, line).
const CHOICES: [(&str, &str, &str); 4] = [
    ("claude-code", "Claude Code", "Free with Claude Pro or Max. Signed in once on this PC."),
    ("ollama", "Ollama", "Free, runs on this PC, slower. Uses qwen3."),
    ("anthropic", "Claude API key", "Pay per use. Set a limit in your Anthropic console too."),
    ("other", "Other", "OpenAI key, another local server, or Codex"),
];

/// Which choice the provider in use belongs to.
fn choice_of(provider: &str) -> &'static str {
    match provider {
        "claude-code" => "claude-code",
        "ollama" => "ollama",
        "anthropic" => "anthropic",
        "openai-cloud" | "openai" | "codex-cli" => "other",
        _ => "",
    }
}

/// How many projects a workspace has (read from its file, not opened).
fn project_count(id: &str) -> usize {
    let path = crate::store::workspace_dir(id).join("data.json");
    let Ok(text) = std::fs::read_to_string(path) else { return 0 };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else { return 0 };
    v["projects"].as_array().map(|a| a.iter().filter(|p| !p["archived"].as_bool().unwrap_or(false)).count()).unwrap_or(0)
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

impl App {
    pub fn settings_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let tab = self.settings_ui.tab.min(SETTINGS_TABS.len() - 1);
        let sub = format!("Alpha {} · workspace: {} · everything here is stored on this PC", env!("CARGO_PKG_VERSION"), self.store.meta.name);
        egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt(("settings", tab)).show(ui, |ui| {
            egui::Frame::none().inner_margin(egui::Margin { left: space::WORK_SIDE - space::S4, right: space::WORK_SIDE - space::S4, top: space::S22, bottom: space::S40 * 2.0 }).show(ui, |ui| {
                capped(ui, space::CAP_SETTINGS, |ui| {
                    ui.label(RichText::new("Settings").font(fserif(size::GREETING)).color(p.text));
                    ui.label(RichText::new(sub).size(size::ROW_SMALL).color(p.dim));
                    ui.add_space(space::S10);
                    if let Some(t) = underline_tabs(ui, p, &SETTINGS_TABS, tab) {
                        self.settings_ui.tab = t;
                        self.settings_ui.message.clear();
                    }
                    ui.add_space(space::S14);
                    if !self.settings_ui.message.is_empty() {
                        let m = self.settings_ui.message.clone();
                        let bad = m.starts_with('⚠');
                        egui::Frame::none().fill(if bad { p.orange_tint } else { p.surface_raised }).rounding(egui::Rounding::same(radius::BUTTON)).inner_margin(egui::Margin::symmetric(space::S14, space::S10)).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.add(egui::Label::new(RichText::new(m.trim_start_matches('⚠').trim()).size(size::ROW_SMALL).color(if bad { p.orange } else { p.text })).wrap());
                        });
                        ui.add_space(space::S10);
                    }
                    let w = ui.available_width();
                    let lw = ((w - space::SET_GAP) * space::SET_LEFT_SHARE).floor();
                    let rw = w - space::SET_GAP - lw;
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.allocate_ui_with_layout(egui::vec2(lw, 0.0), Layout::top_down(Align::Min), |ui| {
                            ui.set_width(lw);
                            ui.spacing_mut().item_spacing = egui::vec2(space::S8, space::S12);
                            match tab {
                                0 => self.set_ai_left(ui, p),
                                1 => self.set_phone(ui, p),
                                2 => self.set_workspaces(ui, p),
                                _ => self.set_about_left(ui, p),
                            }
                        });
                        ui.add_space(space::SET_GAP);
                        ui.allocate_ui_with_layout(egui::vec2(rw, 0.0), Layout::top_down(Align::Min), |ui| {
                            ui.set_width(rw);
                            ui.spacing_mut().item_spacing = egui::vec2(space::S8, space::S12);
                            match tab {
                                0 => self.set_ai_right(ui, p),
                                1 => self.set_sync_right(ui, p),
                                2 => self.set_import_export(ui, p),
                                _ => self.set_about_right(ui, p),
                            }
                        });
                    });
                });
            });
        });
    }

    // ------------------------------------------------------------ AI and MCP

    fn set_ai_left(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let ctx = ui.ctx().clone();
        self.connect_detect(false);
        self.connect_poll(&ctx);
        let in_use = if self.reg.settings.ai_setup_done { choice_of(&self.reg.settings.ai_provider) } else { "" };
        let testing = self.connect.test_rx.as_ref().map(|x| x.0);
        let (installed, signed) = self.connect.claude;
        let ollama_ok = matches!(&self.connect.ollama, Some(Ok(m)) if !m.is_empty());
        let has_key = !self.reg.settings.anthropic_key.trim().is_empty() || !self.connect.key.trim().is_empty();
        let open = self.temp.get("conn_open").cloned().unwrap_or_default();
        let mut test: Option<&'static str> = None;
        let mut toggle: Option<&'static str> = None;
        let mut failed: Option<(&'static str, String)> = None;
        set_card(ui, p, "Connect your AI", |ui| {
            ui.label(RichText::new("Pick the AI Resolve uses, and press Test to prove it replies.").size(size::ROW_SMALL).color(p.dim));
            for (key, name, line) in CHOICES {
                let status = if testing == Some(key) {
                    Some(("Testing…", p.accent_text))
                } else if matches!(self.connect.results.get(key), Some(Err(_))) {
                    Some(("No reply", p.orange))
                } else if in_use == key || matches!(self.connect.results.get(key), Some(Ok(_))) {
                    Some(("Connected", p.green_text))
                } else if key == "claude-code" && !(installed && signed) {
                    Some(("Not set up", p.dim))
                } else {
                    None
                };
                if let Some(Err(e)) = self.connect.results.get(key) {
                    failed = Some((key, e.clone()));
                }
                let action = match key {
                    "claude-code" => "Test",
                    "ollama" if ollama_ok => "Test",
                    "anthropic" if has_key => "Test",
                    "anthropic" => "Add key",
                    _ => "Set up",
                };
                let (row, act) = conn_row(ui, p, egui::Id::new(("conn", key)), in_use == key, name, line, status, action);
                if act && action == "Test" && testing.is_none() {
                    test = Some(key);
                } else if act || row {
                    toggle = Some(key);
                }
                if open == key {
                    self.conn_detail(ui, p, key);
                }
            }
            if let Some((key, e)) = &failed {
                let name = CHOICES.iter().find(|c| c.0 == *key).map(|c| c.1).unwrap_or("The AI");
                let why = if *key == "claude-code" && installed && !signed { "It's installed but not signed in. Open a terminal, run claude once and sign in, then press Test again.".to_string() } else { e.clone() };
                egui::Frame::none().fill(p.orange_tint).stroke(egui::Stroke::new(stroke::HAIR, p.orange.gamma_multiply(0.4))).rounding(egui::Rounding::same(radius::BUTTON)).inner_margin(egui::Margin::symmetric(space::S14, space::S10)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    let mut job = egui::text::LayoutJob::default();
                    job.append(&format!("{name} didn't reply. "), 0.0, egui::text::TextFormat { font_id: fs(size::ROW_SMALL), color: p.orange, ..Default::default() });
                    job.append(&why, 0.0, egui::text::TextFormat { font_id: fp(size::ROW_SMALL), color: p.orange, ..Default::default() });
                    job.wrap.max_width = ui.available_width();
                    ui.label(job);
                });
            }
        });
        if let Some(w) = testing {
            let name = CHOICES.iter().find(|c| c.0 == w).map(|c| c.1).unwrap_or("the AI");
            ui.label(RichText::new(format!("Testing {name}… this takes up to 20 seconds.")).size(size::ROW_SMALL).color(p.dim));
        }
        if let Some(k) = toggle {
            if open == k {
                self.temp.remove("conn_open");
            } else {
                self.temp.insert("conn_open".into(), k.into());
            }
        }
        if let Some(k) = test {
            self.connect_test(k);
        }
        self.set_mcp(ui, p);
    }

    /// The setup under an AI choice: what's missing and how to fix it.
    fn conn_detail(&mut self, ui: &mut egui::Ui, p: &Pal, key: &str) {
        egui::Frame::none().fill(p.surface_raised).rounding(egui::Rounding::same(radius::BUTTON)).inner_margin(egui::Margin::symmetric(space::S14, space::S10)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = space::S6;
            match key {
                "claude-code" => {
                    let (installed, signed) = self.connect.claude;
                    let line = match (installed, signed) {
                        (true, true) => "Installed and signed in on this PC.",
                        (true, false) => "Installed, but it doesn't look signed in yet.",
                        _ => "Not installed on this PC yet.",
                    };
                    ui.label(RichText::new(line).size(size::ROW_SMALL).color(if installed && signed { p.green_text } else { p.orange }));
                    if !(installed && signed) {
                        ui.add(egui::Label::new(RichText::new("1. Install Claude Code: open PowerShell and run   irm https://claude.ai/install.ps1 | iex").size(size::SMALL + 0.5).color(p.text)).wrap());
                        ui.add(egui::Label::new(RichText::new("2. In the same window type   claude   and sign in with your Claude Pro or Max account.").size(size::SMALL + 0.5).color(p.text)).wrap());
                    }
                    if link(ui, p, "Check again").clicked() {
                        self.connect_detect(true);
                    }
                }
                "ollama" => {
                    match self.connect.ollama.clone() {
                        None => {
                            ui.label(RichText::new("Looking for Ollama…").size(size::ROW_SMALL).color(p.dim));
                        }
                        Some(Err(_)) => {
                            ui.label(RichText::new("Ollama isn't running on this PC.").size(size::ROW_SMALL).color(p.orange));
                            ui.add(egui::Label::new(RichText::new("1. Install it from ollama.com.   2. Open it once; it then runs in the background.").size(size::SMALL + 0.5).color(p.text)).wrap());
                        }
                        Some(Ok(models)) => {
                            let txt = if models.is_empty() { "Ollama is running, but no model is downloaded yet.".to_string() } else { format!("Ollama is running. Models: {}", models.join(", ")) };
                            ui.add(egui::Label::new(RichText::new(txt).size(size::ROW_SMALL).color(p.green_text)).wrap());
                            if !models.iter().any(|m| m.starts_with("qwen3")) {
                                let pulling = self.connect.pull_rx.is_some();
                                if button(ui, p, ButtonKind::Secondary, "Download qwen3 (recommended, about 5 GB)", true, !pulling).clicked() {
                                    if let Ok(mut x) = self.connect.pull_progress.lock() {
                                        *x = "Starting the download…".into();
                                    }
                                    self.connect.pull_rx = Some(crate::ai::pull_model_async(super::connect::OLLAMA_URL, "qwen3", self.connect.pull_progress.clone()));
                                }
                            }
                        }
                    }
                    let progress = self.connect.pull_progress.lock().map(|x| x.clone()).unwrap_or_default();
                    if !progress.is_empty() {
                        ui.label(RichText::new(progress).size(size::SMALL).color(p.dim));
                    }
                    if link(ui, p, "Check again").clicked() {
                        self.connect_detect(true);
                    }
                }
                "anthropic" => {
                    if self.connect.key.is_empty() && !self.reg.settings.anthropic_key.is_empty() {
                        self.connect.key = self.reg.settings.anthropic_key.clone();
                    }
                    let w = ui.available_width();
                    text_field(ui, p, egui::Id::new("conn-key"), &mut self.connect.key, "Paste your key (sk-ant-…)", false, w);
                    ui.label(RichText::new("Get a key at console.anthropic.com, API keys. It stays on this PC.").size(size::SMALL).color(p.dim));
                }
                _ => {
                    self.settings_ai_more(ui, p);
                }
            }
        });
    }

    /// The MCP setup rows: copy each app's setup, and the two access switches.
    fn set_mcp(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let exe = std::env::current_exe().map(|x| x.display().to_string()).unwrap_or_else(|_| "resolve2.exe".into());
        let ws = self.store.meta.id.clone();
        let sname = format!("resolve2_{}", crate::store::sanitize(&self.store.meta.name).to_lowercase().replace([' ', '-', '.'], "_"));
        let snippets = [
            ("Claude Desktop", format!("{{\n  \"mcpServers\": {{\n    \"{sname}\": {{\n      \"command\": {},\n      \"args\": [\"--mcp\", \"--workspace\", \"{ws}\"]\n    }}\n  }}\n}}", serde_json::to_string(&exe).unwrap_or_default())),
            ("Claude Code", format!("claude mcp add --scope user {sname} -- \"{exe}\" --mcp --workspace {ws}")),
            ("Codex (CLI or IDE)", format!("# ~/.codex/config.toml\n[mcp_servers.{sname}]\ncommand = '{exe}'\nargs = [\"--mcp\", \"--workspace\", \"{ws}\"]")),
            ("ChatGPT", format!("https://<your-tunnel-address>/mcp?token={}", self.store.settings.token)),
        ];
        let mut copied: Option<&str> = None;
        let mut add: Option<&str> = None;
        set_card(ui, p, "Use Resolve from other AI apps (MCP)", |ui| {
            let cw = ((ui.available_width() - space::S8) / 2.0).floor();
            for pair in snippets.chunks(2) {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = space::S8;
                    for (name, text) in pair {
                        if mini_row(ui, p, cw, name, "Copy setup") {
                            ui.ctx().copy_text(text.clone());
                            copied = Some(name);
                        }
                    }
                });
            }
            if ui.checkbox(&mut self.store.settings.ai_access, RichText::new("Other AI apps can read this workspace").size(size::ROW_SMALL)).changed() {
                self.store.save_settings();
            }
            if ui.checkbox(&mut self.store.settings.ai_read_only, RichText::new("Read only: they can't change anything").size(size::ROW_SMALL)).changed() {
                self.store.save_settings();
            }
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Or let Resolve add it:").size(size::SMALL).color(p.dim));
                if link(ui, p, "Claude Desktop").clicked() {
                    add = Some("desktop");
                }
                if link(ui, p, "Claude Code").clicked() {
                    add = Some("code");
                }
            });
            ui.label(RichText::new("ChatGPT reaches this PC from OpenAI's servers, so it needs a public https address (a tunnel such as cloudflared). Keep that link private.").size(size::SMALL).color(p.dim));
        });
        if let Some(n) = copied {
            self.toast(format!("Copied the {n} setup"));
        }
        match add {
            Some("desktop") => self.settings_ui.message = add_to_claude_desktop(&sname, &exe, &ws).unwrap_or_else(|e| format!("⚠ {e}")),
            Some(_) => self.settings_ui.message = add_to_claude_code(&sname, &exe, &ws).unwrap_or_else(|e| format!("⚠ {e}")),
            None => {}
        }
        let _ = claude_desktop_configs;
    }

    fn set_ai_right(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let u = crate::ai::usage_since(crate::ai::month_start_ms());
        let (pin, pout, budget) = (self.reg.settings.price_in_per_m, self.reg.settings.price_out_per_m, self.reg.settings.monthly_budget);
        let cost = crate::ai::estimated_cost(&u, pin, pout);
        let mut changed = false;
        set_card(ui, p, "", |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Spending").font(fs(size::SECTION)).color(p.text));
                tag(ui, "Estimated", p.dim, p.surface_raised).on_hover_text("Worked out from the tokens Resolve logged and the prices you enter here. Your provider's bill is the real figure.");
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("${cost:.2}")).font(fserif(size::HERO_TITLE)).color(p.text));
                let of = if budget > 0.0 { format!("of ${budget:.2} this month") } else { "this month, no limit".to_string() };
                ui.label(RichText::new(of).size(size::ROW).color(p.dim));
            });
            let frac = if budget > 0.0 { (cost / budget) as f32 } else { 0.0 };
            progress_line(ui, p, frac, ui.available_width());
            ui.add_space(space::S4);
            field_label(ui, p, "Monthly spending limit");
            let w = ui.available_width();
            changed |= money_field(ui, p, egui::Id::new("set-budget"), &mut self.reg.settings.monthly_budget, w);
            ui.add(egui::Label::new(RichText::new("When it's reached, Resolve stops calling the AI and tells you. Everything else keeps working. 0 means no limit.").size(size::ROW_SMALL).color(p.dim)).wrap());
            let hw = ((w - space::S10) / 2.0).floor();
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = space::S10;
                ui.allocate_ui_with_layout(egui::vec2(hw, 0.0), Layout::top_down(Align::Min), |ui| {
                    field_label(ui, p, "Price per million input");
                    changed |= money_field(ui, p, egui::Id::new("set-pin"), &mut self.reg.settings.price_in_per_m, hw);
                });
                ui.allocate_ui_with_layout(egui::vec2(hw, 0.0), Layout::top_down(Align::Min), |ui| {
                    field_label(ui, p, "Output");
                    changed |= money_field(ui, p, egui::Id::new("set-pout"), &mut self.reg.settings.price_out_per_m, hw);
                });
            });
            ui.add(egui::Label::new(RichText::new("Only API keys cost per use. Claude Code and Ollama show $0 here.").size(size::ROW_SMALL).color(p.dim)).wrap());
            if u.calls > 0 {
                ui.label(RichText::new(format!("This month: {}, {} input and {} output tokens.", plural(u.calls as usize, "call", "calls"), u.input, u.output)).size(size::SMALL).color(p.dim));
            }
        });
        if changed {
            self.reg.save();
        }
        fold_card(ui, p, egui::Id::new("set-ai-more"), "More AI options", |ui| {
            ui.add(egui::Label::new(RichText::new("Other providers and models are under Other, in Connect your AI.").size(size::ROW_SMALL).color(p.dim)).wrap());
            let mut fast = self.reg.settings.fast_validation;
            if ui.checkbox(&mut fast, RichText::new("Faster proposal: write the validation half on Sonnet (Claude Code and the Claude API only)").size(size::SMALL)).changed() {
                self.reg.settings.fast_validation = fast;
                self.reg.save();
            }
            for (m, c, i, o) in &u.by_model {
                ui.label(RichText::new(format!("{m}: {}, {i} in, {o} out", plural(*c as usize, "call", "calls"))).size(size::SMALL).color(p.dim));
            }
        });
        fold_card(ui, p, egui::Id::new("set-testlab"), "Test Intake", |ui| {
            ui.add(egui::Label::new(RichText::new("Try the intake on sample scenarios in its own window, without touching your workspaces. Choose the AI per run, see why each question was asked, compare runs and export them.").size(size::ROW_SMALL).color(p.dim)).wrap());
            if button(ui, p, ButtonKind::Secondary, "Open Test Intake", true, true).clicked() {
                self.launch_testlab();
            }
        });
    }

    // ------------------------------------------------------------ Sync and phone

    fn set_phone(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let ip = crate::server::lan_ip().unwrap_or_else(|| "this-pc".into());
        let port = self.store.settings.server_port;
        let base = format!("http://{ip}:{port}");
        let full = format!("{base}/?token={}", self.store.settings.token);
        let mut copy: Option<(String, &str)> = None;
        set_card(ui, p, "Your phone", |ui| {
            let mut on = self.store.settings.server_enabled;
            if ui.checkbox(&mut on, RichText::new("Let your phone reach this PC on your home network").font(fs(size::ROW_SMALL))).changed() {
                self.store.settings.server_enabled = on;
                self.store.save_settings();
                self.ensure_server();
            }
            let mut job = egui::text::LayoutJob::default();
            let f = |font: egui::FontId| egui::text::TextFormat { font_id: font, color: p.text, ..Default::default() };
            job.append("Open ", 0.0, f(fp(size::ROW_SMALL)));
            job.append(&base, 0.0, f(egui::FontId::monospace(size::ROW_SMALL - 1.0)));
            job.append(" on your phone and paste the token.", 0.0, f(fp(size::ROW_SMALL)));
            job.wrap.max_width = ui.available_width();
            ui.label(job);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = space::S6;
                ui.label(RichText::new("Port").size(size::ROW_SMALL).color(p.text));
                let mut port2 = port;
                if ui.add(egui::DragValue::new(&mut port2).range(1024..=65000)).changed() {
                    self.store.settings.server_port = port2;
                    self.store.save_settings();
                    self.ensure_server();
                }
                ui.label(RichText::new("· token ••••••••").size(size::ROW_SMALL).color(p.text));
                if link(ui, p, "Copy").clicked() {
                    copy = Some((self.store.settings.token.clone(), "the token"));
                }
                if link(ui, p, "Copy the whole link").clicked() {
                    copy = Some((full.clone(), "the link"));
                }
            });
            if let Some(e) = &self.server_error {
                ui.label(RichText::new(e).size(size::ROW_SMALL).color(p.orange));
            }
            ui.add(egui::Label::new(RichText::new("Your phone sees tasks, calendar and search. It works while this PC is awake and on the same network.").size(size::ROW_SMALL).color(p.dim)).wrap());
            if link(ui, p, "Make a new token (old phone links stop working)").clicked() {
                self.store.settings.token = crate::store::new_token();
                self.store.save_settings();
            }
        });
        if let Some((t, what)) = copy {
            ui.ctx().copy_text(t);
            self.toast(format!("Copied {what}"));
        }
        fold_card(ui, p, egui::Id::new("set-deleted"), "Recently deleted", |ui| self.recently_deleted_list(ui, p));
        fold_card(ui, p, egui::Id::new("set-cals"), "Calendar subscriptions", |ui| self.settings_calendars(ui, p));
    }

    fn set_sync_right(&mut self, ui: &mut egui::Ui, p: &Pal) {
        set_card(ui, p, "Sync folder", |ui| {
            let dir = self.store.settings.sync_dir.clone();
            if dir.trim().is_empty() {
                ui.add(egui::Label::new(RichText::new("Not set. Pick a shared folder (OneDrive, Google Drive, Dropbox or Syncthing) to keep other PCs in step.").size(size::ROW_SMALL).color(p.dim)).wrap());
            } else {
                ui.add(egui::Label::new(RichText::new(&dir).monospace().size(size::ROW_SMALL - 1.0).color(p.text)).wrap());
            }
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = space::S6;
                if self.store.settings.last_sync > 0 {
                    let when = chrono::DateTime::from_timestamp_millis(self.store.settings.last_sync).map(|x| x.with_timezone(&chrono::Local)).map(|x| {
                        let day = super::kb::day_label(self.store.settings.last_sync, today());
                        format!("Last synced {} at {}", if day == "Today" { "today".to_string() } else { day }, x.format("%H:%M"))
                    });
                    ui.label(RichText::new(format!("{} ·", when.unwrap_or_default())).size(size::ROW_SMALL).color(p.text));
                }
                if link(ui, p, if dir.trim().is_empty() { "Choose a folder" } else { "Change folder" }).clicked() {
                    if let Some(f) = rfd::FileDialog::new().pick_folder() {
                        self.store.settings.sync_dir = f.display().to_string();
                        self.store.save_settings();
                    }
                }
                if !dir.trim().is_empty() && link(ui, p, "Sync now").clicked() {
                    self.save_now();
                    self.settings_ui.message = match self.store.sync() {
                        Ok(m) => {
                            self.sync_status = format!("synced {}", chrono::Local::now().format("%H:%M"));
                            m
                        }
                        Err(e) => format!("⚠ Couldn't sync: {e}"),
                    };
                }
            });
        });
        let list = self.store.list_backups();
        let restore_open = self.temp.contains_key("restore_open");
        let mut toggle = false;
        set_card(ui, p, "Backups", |ui| {
            if ui.checkbox(&mut self.store.settings.auto_backup, RichText::new("Back up automatically").size(size::ROW_SMALL)).changed() {
                self.store.save_settings();
            }
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = space::S6;
                ui.label(RichText::new("Keep").size(size::ROW_SMALL).color(p.text));
                if ui.add(egui::DragValue::new(&mut self.store.settings.keep_backups).range(1..=365)).changed() {
                    self.store.save_settings();
                }
                ui.label(RichText::new("backups ·").size(size::ROW_SMALL).color(p.text));
                if link(ui, p, "Restore one").clicked() {
                    toggle = true;
                }
                ui.label(RichText::new("·").size(size::ROW_SMALL).color(p.text));
                if link(ui, p, "Back up now").clicked() {
                    self.save_now();
                    self.settings_ui.message = match self.store.backup_now() {
                        Ok(_) => "Backed up.".into(),
                        Err(e) => format!("⚠ Couldn't back up: {e}"),
                    };
                }
            });
            if restore_open {
                if list.is_empty() {
                    ui.label(RichText::new("No backups yet.").size(size::ROW_SMALL).color(p.dim));
                }
                for (path, when, size_b) in list.iter().take(30) {
                    ui.horizontal(|ui| {
                        let dt: chrono::DateTime<chrono::Local> = (*when).into();
                        ui.label(RichText::new(dt.format("%a %-d %b %Y  %H:%M").to_string()).size(size::ROW_SMALL).color(p.text));
                        ui.label(RichText::new(format!("{} KB", size_b / 1024)).size(size::SMALL).color(p.dim));
                        if self.settings_ui.confirm_restore.as_ref() == Some(path) {
                            if button(ui, p, ButtonKind::Dark, "Restore this one", true, true).clicked() {
                                let pth = path.clone();
                                self.settings_ui.message = match self.store.restore(&pth) {
                                    Ok(()) => "Restored. (Your state just before restoring was backed up too.)".into(),
                                    Err(e) => format!("⚠ Couldn't restore: {e}"),
                                };
                                self.settings_ui.confirm_restore = None;
                            }
                            if link(ui, p, "Cancel").clicked() {
                                self.settings_ui.confirm_restore = None;
                            }
                        } else if link(ui, p, "Restore").clicked() {
                            self.settings_ui.confirm_restore = Some(path.clone());
                        }
                    });
                }
                if link(ui, p, "Open the backups folder").clicked() {
                    let _ = std::fs::create_dir_all(self.store.backups_dir());
                    super::settings::open_folder(&self.store.backups_dir());
                }
            }
        });
        if toggle {
            if restore_open {
                self.temp.remove("restore_open");
            } else {
                self.temp.insert("restore_open".into(), "1".into());
            }
        }
        fold_card(ui, p, egui::Id::new("set-hub"), "Advanced: hub address and token", |ui| {
            ui.add(egui::Label::new(RichText::new("One always-on machine can be the hub: other devices merge with it, so the hub can move without losing anything. Leave the address empty and this PC is the hub.").size(size::ROW_SMALL).color(p.dim)).wrap());
            let w = ui.available_width();
            field_label(ui, p, "Hub address");
            let mut url = self.store.settings.hub_url.clone();
            if text_field(ui, p, egui::Id::new("set-hub-url"), &mut url, "http://my-server:8797", false, w).changed() {
                self.store.settings.hub_url = url;
                self.store.save_settings();
            }
            field_label(ui, p, "Hub token");
            let mut tok = self.store.settings.hub_token.clone();
            if ui.add(egui::TextEdit::singleline(&mut tok).password(true).hint_text("the --token you gave the hub").desired_width(w)).changed() {
                self.store.settings.hub_token = tok;
                self.store.save_settings();
            }
            ui.horizontal(|ui| {
                if button(ui, p, ButtonKind::Secondary, "Sync with the hub now", true, !self.store.settings.hub_url.trim().is_empty()).clicked() {
                    self.save_now();
                    self.hub_tick(true);
                }
                if !self.hub_status.is_empty() {
                    ui.label(RichText::new(&self.hub_status).size(size::SMALL).color(p.dim));
                }
            });
            ui.label(RichText::new("On the hub, run the same program headless:").size(size::SMALL).color(p.dim));
            ui.add(egui::Label::new(RichText::new(format!("resolve2 --serve {} --data-dir /srv/resolve2 --workspace \"{}\" --token <a token>", self.store.settings.server_port, self.store.meta.name)).monospace().size(size::SMALL)).wrap());
        });
    }

    /// Recently deleted, without its card (for the fold).
    fn recently_deleted_list(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let list: Vec<crate::history::ChangeSet> = self.store.data.recently_deleted().into_iter().take(50).cloned().collect();
        let mut restore: Option<String> = None;
        ui.add(egui::Label::new(RichText::new("Anything deleted in the last 60 days, by you, the AI or another app, can be put back.").size(size::ROW_SMALL).color(p.dim)).wrap());
        if list.is_empty() {
            ui.label(RichText::new("Nothing deleted recently.").size(size::ROW_SMALL).color(p.dim));
        }
        for cs in &list {
            ui.horizontal(|ui| {
                let when = chrono::DateTime::from_timestamp_millis(cs.created).map(|x| x.with_timezone(&chrono::Local).format("%-d %b %H:%M").to_string()).unwrap_or_default();
                ui.label(RichText::new(when).size(size::SMALL).color(p.dim));
                ui.add(egui::Label::new(RichText::new(&cs.label).size(size::ROW_SMALL).color(p.text)).truncate());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if link(ui, p, "Restore").clicked() {
                        restore = Some(cs.id.clone());
                    }
                });
            });
        }
        if let Some(id) = restore {
            self.undo(&id);
        }
    }

    // ------------------------------------------------------------ Workspaces

    fn set_workspaces(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let list = self.reg.workspaces.clone();
        let current = self.store.meta.id.clone();
        let renaming = self.temp.contains_key("ws_rename");
        let (mut new_ws, mut switch, mut rename) = (false, None::<Id>, false);
        let card = egui::Frame::none().fill(p.card).stroke(egui::Stroke::new(stroke::HAIR, p.line)).rounding(egui::Rounding::same(radius::TOAST)).inner_margin(egui::Margin::symmetric(space::S18, space::S16));
        card.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = space::S8;
            ui.horizontal(|ui| {
                ui.label(RichText::new("All workspaces").font(fs(size::SECTION)).color(p.text));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    new_ws = button(ui, p, ButtonKind::Primary, "+ New workspace", true, true).clicked();
                });
            });
            for w in &list {
                let open = w.id == current;
                let n = if open { self.store.data.projects.iter().filter(|x| !x.archived).count() } else { project_count(&w.id) };
                let line = if open { format!("open now · {}", plural(n, "project", "projects")) } else { plural(n, "project", "projects") };
                let initial = w.name.chars().next().unwrap_or('W').to_string();
                let (row, act) = conn_row(ui, p, egui::Id::new(("ws-row", &w.id)), open, &w.name, &line, None, if open { "Rename" } else { "Open" });
                let _ = initial;
                if act && open {
                    rename = true;
                } else if act || (row && !open) {
                    switch = Some(w.id.clone());
                }
                if open && renaming {
                    egui::Frame::none().fill(p.surface_raised).rounding(egui::Rounding::same(radius::BUTTON)).inner_margin(egui::Margin::symmetric(space::S14, space::S10)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        let mut name = self.store.meta.name.clone();
                        let ww = ui.available_width();
                        if text_field(ui, p, egui::Id::new("ws-name"), &mut name, "Workspace name", false, ww).changed() && !name.trim().is_empty() {
                            self.store.meta.name = name.clone();
                            if let Some(x) = self.reg.workspaces.iter_mut().find(|x| x.id == current) {
                                x.name = name;
                            }
                            self.reg.save();
                        }
                        ui.horizontal(|ui| {
                            for c in PICK_COLOURS {
                                let (r, resp) = ui.allocate_exact_size(egui::vec2(space::S22, space::S22), egui::Sense::click());
                                ui.painter().circle_filled(r.center(), space::S10 - 1.0, rgb(c));
                                if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                    self.store.meta.color = c;
                                    if let Some(x) = self.reg.workspaces.iter_mut().find(|x| x.id == current) {
                                        x.color = c;
                                    }
                                    self.reg.save();
                                }
                            }
                        });
                    });
                }
            }
            ui.add_space(space::S4);
            field_label(ui, p, "Open Resolve 2 with");
            let names: Vec<(Option<Id>, String)> = std::iter::once((None, "The last workspace I used".to_string())).chain(list.iter().map(|w| (Some(w.id.clone()), w.name.clone()))).collect();
            let cur = names.iter().find(|n| n.0 == self.reg.settings.open_with).map(|n| n.1.clone()).unwrap_or_else(|| names[0].1.clone());
            let mut pick = self.reg.settings.open_with.clone();
            field_visuals(ui, p);
            egui::ComboBox::from_id_salt("open-with").width(ui.available_width() - space::S8).selected_text(RichText::new(cur).size(size::ROW)).show_ui(ui, |ui| {
                for (id, n) in &names {
                    ui.selectable_value(&mut pick, id.clone(), n);
                }
            });
            if pick != self.reg.settings.open_with {
                self.reg.settings.open_with = pick;
                self.reg.save();
            }
            if list.len() > 1 {
                if !self.settings_ui.confirm_remove_ws {
                    if link(ui, p, "Remove this workspace from the list").clicked() {
                        self.settings_ui.confirm_remove_ws = true;
                    }
                } else {
                    ui.add(egui::Label::new(RichText::new("It disappears from Resolve 2, but its folder and backups stay on disk, so nothing is lost.").size(size::ROW_SMALL).color(p.orange)).wrap());
                    ui.horizontal(|ui| {
                        if button(ui, p, ButtonKind::Dark, "Yes, remove it", true, true).clicked() {
                            let id = self.store.meta.id.clone();
                            self.reg.workspaces.retain(|w| w.id != id);
                            self.reg.save();
                            let next = self.reg.workspaces[0].id.clone();
                            self.store.meta.id = String::new();
                            self.switch_workspace(&next);
                            self.settings_ui.confirm_remove_ws = false;
                        }
                        if link(ui, p, "Cancel").clicked() {
                            self.settings_ui.confirm_remove_ws = false;
                        }
                    });
                }
            }
        });
        if new_ws {
            self.new_ws = Some(String::new());
        }
        if rename {
            if renaming {
                self.temp.remove("ws_rename");
            } else {
                self.temp.insert("ws_rename".into(), "1".into());
            }
        }
        if let Some(s) = switch {
            self.switch_workspace(&s);
        }
    }

    fn set_import_export(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let mut run: Option<(&str, std::path::PathBuf)> = None;
        let mut csv = false;
        set_card(ui, p, "Import", |ui| {
            ui.add(egui::Label::new(RichText::new("Bring in Markdown pages, CSV tables and .ics calendars.").size(size::ROW_SMALL).color(p.text)).wrap());
            let b = button(ui, p, ButtonKind::Secondary, "Choose files", false, true);
            let menu = egui::Id::new("set-import");
            if b.clicked() {
                ui.memory_mut(|m| m.toggle_popup(menu));
            }
            egui::popup_below_widget(ui, menu, &b, egui::PopupCloseBehavior::CloseOnClick, |ui| {
                ui.set_min_width(space::S40 * 7.0);
                if ui.button("A Notion export (.zip, Markdown and CSV)").clicked() {
                    if let Some(f) = rfd::FileDialog::new().add_filter("Notion export", &["zip"]).pick_file() {
                        run = Some(("notion", f));
                    }
                }
                if ui.button("A Notion export, unzipped (a folder)").clicked() {
                    if let Some(f) = rfd::FileDialog::new().pick_folder() {
                        run = Some(("notion", f));
                    }
                }
                if ui.button("A CSV table (becomes a database)").clicked() {
                    csv = true;
                }
                if ui.button("Calendars (.ics, or a .zip of them)").clicked() {
                    if let Some(f) = rfd::FileDialog::new().add_filter("Calendar", &["zip", "ics"]).pick_file() {
                        run = Some(("ics", f));
                    }
                }
                if ui.button("A Todoist backup (.zip or .csv)").clicked() {
                    if let Some(f) = rfd::FileDialog::new().add_filter("Todoist", &["zip", "csv"]).pick_file() {
                        run = Some(("todoist", f));
                    }
                }
            });
            ui.label(RichText::new("A backup is made before every import, and importing the same file twice won't duplicate anything.").size(size::SMALL).color(p.dim));
        });
        if csv {
            self.csv_pick("");
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
                        m.push_str(&format!("\n{n}"));
                    }
                    m
                }
                Err(e) => format!("⚠ Couldn't import that: {e}"),
            };
        }
        set_card(ui, p, "Export: your data, in open formats", |ui| {
            ui.add(egui::Label::new(RichText::new("Every page as Markdown, every database as CSV, all events as .ics, tasks as a Markdown checklist, plus the full workspace as JSON.").size(size::ROW_SMALL).color(p.text)).wrap());
            if button(ui, p, ButtonKind::Secondary, "Export everything", false, true).clicked() {
                if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                    self.settings_ui.message = match self.store.export_markdown(&dir) {
                        Ok(n) => format!("Exported {n} pages and databases to {}", dir.display()),
                        Err(e) => format!("⚠ Couldn't export: {e}"),
                    };
                }
            }
        });
    }

    // ------------------------------------------------------------ About

    fn set_about_left(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let (mut report, mut folder) = (false, false);
        set_card(ui, p, "", |ui| {
            ui.label(RichText::new("Resolve 2").font(fserif(size::HERO_TITLE)).color(p.text));
            ui.label(RichText::new(format!("Alpha {}", env!("CARGO_PKG_VERSION"))).size(size::ROW).color(p.text));
            ui.add(egui::Label::new(RichText::new("Everything you make is stored on this PC. Resolve sends text to the AI you connect, and nothing to anyone else.").size(size::ROW_SMALL).color(p.dim)).wrap());
            ui.add_space(space::S4);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = space::S8;
                report = button(ui, p, ButtonKind::Dark, "Report a problem", true, true).clicked();
                folder = button(ui, p, ButtonKind::Secondary, "Open the data folder", true, true).clicked();
            });
        });
        if report {
            self.open_report();
        }
        if folder {
            super::settings::open_folder(&crate::store::root());
        }
        set_card(ui, p, "What's new", |ui| {
            ui.label(RichText::new("The changelog, newest first.").size(size::ROW_SMALL).color(p.dim));
            super::help::render_md(ui, p, CHANGELOG.splitn(2, '\n').nth(1).unwrap_or(""));
        });
    }

    fn set_about_right(&mut self, ui: &mut egui::Ui, p: &Pal) {
        set_card(ui, p, "Appearance", |ui| {
            let sel = if self.reg.settings.dark { 1 } else { 0 };
            if let Some(i) = segmented(ui, p, egui::Id::new("set-theme"), &["Light", "Dark"], sel) {
                self.reg.settings.dark = i == 1;
                self.reg.save();
            }
            let mut rm = self.reg.settings.reduced_motion;
            if ui.checkbox(&mut rm, RichText::new("Reduce motion: every movement becomes a short fade").size(size::ROW_SMALL)).changed() {
                self.reg.settings.reduced_motion = rm;
                crate::ui::motion::set_reduced(rm);
                self.reg.save();
            }
        });
        set_card(ui, p, "Keyboard shortcuts", |ui| {
            ui.spacing_mut().item_spacing.y = space::S4;
            for (what, keys) in [("Search", "Ctrl K"), ("Undo", "Ctrl Z"), ("Redo", "Ctrl Y"), ("Collapse the sidebar", "Ctrl \\"), ("Add a task", "Ctrl Q"), ("Add an event", "Ctrl E"), ("New page", "Ctrl N"), ("Block menu in a page", "/"), ("Link to another page", "[["), ("Close dialogs and panels", "Esc")] {
                shortcut_row(ui, p, what, keys);
            }
        });
        fold_card(ui, p, egui::Id::new("set-files"), "Where your files are", |ui| {
            let exe = std::env::current_exe().ok();
            let root = crate::store::root();
            let rows: Vec<(&str, std::path::PathBuf, bool)> = vec![
                ("All Resolve 2 data (workspaces, settings, AI keys)", root.clone(), true),
                ("This workspace", self.store.dir.clone(), true),
                ("This workspace's backups", self.store.backups_dir(), true),
                ("AI usage log", crate::ai::usage_path(), false),
                ("The program", exe.clone().unwrap_or_default(), false),
            ];
            for (label, path, is_dir) in rows {
                if path.as_os_str().is_empty() {
                    continue;
                }
                let target = if is_dir { path.clone() } else { path.parent().map(|x| x.to_path_buf()).unwrap_or_default() };
                ui.horizontal(|ui| {
                    ui.label(RichText::new(label).size(size::ROW_SMALL).color(p.text));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if link(ui, p, "Copy").clicked() {
                            ui.ctx().copy_text(path.display().to_string());
                        }
                        if target.exists() && link(ui, p, "Open").clicked() {
                            super::settings::open_folder(&target);
                        }
                    });
                });
                ui.add(egui::Label::new(RichText::new(path.display().to_string()).monospace().size(size::SMALL - 1.0).color(p.dim)).wrap());
            }
            ui.label(RichText::new("Resolve 2 never reads or writes Resolve Beta's folder (%APPDATA%\\Weave).").size(size::SMALL).color(p.dim));
        });
        fold_card(ui, p, egui::Id::new("set-open"), "Opening Resolve 2", |ui| {
            let exe = std::env::current_exe().map(|x| x.display().to_string()).unwrap_or_default();
            if button(ui, p, ButtonKind::Secondary, "Put Resolve 2 on my desktop and Start menu", true, true).clicked() {
                self.settings_ui.message = super::settings::make_shortcuts(&exe);
            }
            ui.add(egui::Label::new(RichText::new("Then open it like any other app: double-click the Resolve 2 icon, or press Windows and type Resolve 2.").size(size::SMALL).color(p.dim)).wrap());
        });
    }
}
