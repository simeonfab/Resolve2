//! "Connect your AI": the first screen of a fresh copy (before the first intake), and the top of
//! Settings → AI. Three choices in order — Claude Code (recommended), Ollama, Claude API key —
//! each detected where possible and proven with a real test reply. A passing test (or "Skip for
//! now", which uses the fixed question set) lets the person move on.
//!
//! Also here: "Report a problem" (version, Windows version, the AI in use, the last error — no
//! personal data or project content).

use super::theme::*;
use super::widgets::*;
use super::*;
use eframe::egui::{self, RichText, Stroke};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, OnceLock};

pub const OLLAMA_URL: &str = "http://localhost:11434";

#[derive(Default)]
pub struct ConnectUi {
    pub checked: bool,
    /// Claude Code: (installed, looks signed in)
    pub claude: (bool, bool),
    pub ollama_rx: Option<Receiver<Result<Vec<String>, String>>>,
    pub ollama: Option<Result<Vec<String>, String>>,
    /// The running test: (which choice, result).
    pub test_rx: Option<(&'static str, Receiver<Result<String, String>>)>,
    pub test_started: f64,
    /// Last test result per choice ("claude-code", "ollama", "anthropic").
    pub results: HashMap<&'static str, Result<String, String>>,
    pub pull_rx: Option<Receiver<Result<(), String>>>,
    pub pull_progress: Arc<Mutex<String>>,
    pub key: String,
}

fn windows_version() -> String {
    static V: OnceLock<String> = OnceLock::new();
    V.get_or_init(|| {
        let mut cmd = std::process::Command::new("cmd");
        cmd.args(["/c", "ver"]);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x0800_0000); // no console window
        }
        cmd.output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| std::env::consts::OS.to_string())
    })
    .clone()
}

impl App {
    /// Start detection once (Claude Code on PATH, Ollama answering).
    fn connect_detect(&mut self, force: bool) {
        if self.connect.checked && !force {
            return;
        }
        self.connect.checked = true;
        self.connect.claude = crate::ai::claude_code_state();
        self.connect.ollama = None;
        self.connect.ollama_rx = Some(crate::ai::list_models_async("ollama", OLLAMA_URL));
    }

    fn connect_poll(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.connect.ollama_rx {
            if let Ok(r) = rx.try_recv() {
                self.connect.ollama = Some(r);
                self.connect.ollama_rx = None;
            }
        }
        if let Some((which, rx)) = &self.connect.test_rx {
            if let Ok(r) = rx.try_recv() {
                let which = *which;
                if r.is_ok() {
                    self.connect_apply(which);
                }
                self.connect.results.insert(which, r);
                self.connect.test_rx = None;
            }
        }
        if let Some(rx) = &self.connect.pull_rx {
            if let Ok(r) = rx.try_recv() {
                self.connect.pull_rx = None;
                match r {
                    Ok(()) => {
                        if let Ok(mut p) = self.connect.pull_progress.lock() {
                            *p = "qwen3 is downloaded.".into();
                        }
                        self.connect_detect(true);
                    }
                    Err(e) => {
                        if let Ok(mut p) = self.connect.pull_progress.lock() {
                            *p = format!("⚠ The download stopped: {e}");
                        }
                    }
                }
            }
        }
        if self.connect.ollama_rx.is_some() || self.connect.test_rx.is_some() || self.connect.pull_rx.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
    }

    fn ollama_model(&self) -> Option<String> {
        let models = self.connect.ollama.as_ref()?.as_ref().ok()?;
        models.iter().find(|m| m.starts_with("qwen3")).or(models.first()).cloned()
    }

    fn connect_cfg(&self, which: &str) -> crate::ai::AiConfig {
        match which {
            "claude-code" => crate::ai::AiConfig { provider: "claude-code".into(), url: String::new(), model: String::new(), key: String::new() },
            "ollama" => crate::ai::AiConfig { provider: "ollama".into(), url: OLLAMA_URL.into(), model: self.ollama_model().unwrap_or_else(|| "qwen3".into()), key: String::new() },
            _ => crate::ai::AiConfig { provider: "anthropic".into(), url: "https://api.anthropic.com".into(), model: crate::ai::CLAUDE_MODEL.into(), key: self.connect.key.trim().to_string() },
        }
    }

    fn connect_test(&mut self, which: &'static str) {
        let cfg = self.connect_cfg(which);
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(crate::ai::test_connection(&cfg));
        });
        self.connect.results.remove(which);
        self.connect.test_rx = Some((which, rx));
        self.connect.test_started = self.now;
    }

    /// A passing test makes that AI the one Resolve uses.
    fn connect_apply(&mut self, which: &str) {
        let cfg = self.connect_cfg(which);
        let s = &mut self.reg.settings;
        s.ai_provider = cfg.provider;
        s.ai_url = cfg.url;
        s.ai_model = cfg.model;
        if which == "anthropic" {
            s.anthropic_key = cfg.key;
        }
        s.ai_setup_done = true;
        self.reg.save();
    }

    /// The full-screen first step.
    pub(super) fn connect_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let ctx = ui.ctx().clone();
        self.connect_detect(false);
        self.connect_poll(&ctx);
        let passed = self.connect.results.values().any(|r| r.is_ok());
        let mut go_on = false;
        let mut skip = false;
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.add_space(36.0);
            centered(ui, space::COLUMN + 80.0, |ui| {
                ui.label(RichText::new("Before you start").size(size::SMALL).color(p.eyebrow));
                ui.label(RichText::new("Connect your AI").font(fserif(size::STEP_TITLE)));
                wrap_label(ui, RichText::new("Resolve uses an AI to ask good questions and write your plan. Pick one and press Test. You can change it later in Settings.").size(size::BODY).color(p.dim));
                ui.add_space(14.0);
                self.connect_cards(ui, p);
                ui.add_space(16.0);
                ui.horizontal_wrapped(|ui| {
                    let cta = egui::Button::new(RichText::new("Continue →").font(fs(size::BODY)).color(p.on_ink)).fill(p.ink).rounding(space::RADIUS_SMALL).min_size(egui::vec2(0.0, 34.0));
                    go_on = ui.add_enabled(passed, cta).on_disabled_hover_text("Test one of the choices first, or skip for now").clicked();
                    ui.add_space(12.0);
                    skip = wrap_link(ui, RichText::new("Skip for now (use the fixed question set)").size(size::SMALL).color(p.faint)).clicked();
                });
                ui.add_space(40.0);
            });
        });
        if skip {
            self.reg.settings.ai_provider = "offline".into();
            self.reg.settings.ai_setup_done = true;
            self.reg.save();
            go_on = true;
        }
        if go_on {
            self.view = View::Dashboard;
            self.open_intake_if_new();
        }
    }

    /// The three choices (first screen and Settings → AI).
    pub(super) fn connect_cards(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let ctx = ui.ctx().clone();
        self.connect_detect(false);
        self.connect_poll(&ctx);
        // "in use" only once a choice has been made (a fresh copy has a default, not a choice)
        let current = if self.reg.settings.ai_setup_done { self.reg.settings.ai_provider.clone() } else { String::new() };
        let testing = self.connect.test_rx.as_ref().map(|x| x.0);
        let secs = (self.now - self.connect.test_started).max(0.0) as u64;
        let mut test: Option<&'static str> = None;
        let mut recheck = false;
        let mut pull = false;

        // 1. Claude Code
        let (installed, signed) = self.connect.claude;
        self.choice_card(ui, p, "1", "Claude Code", Some("Recommended"), "Free if you already pay for Claude Pro or Max.", current == "claude-code", |app, ui| {
            let status = match (installed, signed) {
                (true, true) => "✓ Installed and signed in on this PC.",
                (true, false) => "Installed, but it doesn't look signed in yet.",
                _ => "Not installed on this PC yet.",
            };
            ui.label(RichText::new(status).size(size::SMALL).color(if installed && signed { p.green } else { p.orange }));
            if !(installed && signed) {
                ui.label(RichText::new("Two setup steps:").size(size::SMALL).color(p.dim));
                wrap_label(ui, RichText::new("1. Install Claude Code: open PowerShell and run   irm https://claude.ai/install.ps1 | iex").size(size::SMALL));
                wrap_label(ui, RichText::new("2. In the same window type   claude   and sign in with your Claude Pro or Max account.").size(size::SMALL));
            }
            ui.horizontal_wrapped(|ui| {
                if ui.add_enabled(testing.is_none(), egui::Button::new("Test")).clicked() {
                    test = Some("claude-code");
                }
                if !(installed && signed) && ui.button("Check again").clicked() {
                    recheck = true;
                }
                app.test_result(ui, p, "claude-code", testing, secs);
            });
        });
        ui.add_space(10.0);

        // 2. Ollama
        let ollama = self.connect.ollama.clone();
        let progress = self.connect.pull_progress.lock().map(|x| x.clone()).unwrap_or_default();
        let pulling = self.connect.pull_rx.is_some();
        self.choice_card(ui, p, "2", "Ollama", None, "Free and private, runs on your PC, slower and less accurate.", current == "ollama", |app, ui| {
            match &ollama {
                None => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(RichText::new("Looking for Ollama…").size(size::SMALL).color(p.dim));
                    });
                }
                Some(Err(_)) => {
                    ui.label(RichText::new("Ollama isn't running on this PC.").size(size::SMALL).color(p.orange));
                    wrap_label(ui, RichText::new("1. Install it from ollama.com.   2. Open it once; it then runs in the background.").size(size::SMALL));
                }
                Some(Ok(models)) => {
                    let has_qwen = models.iter().any(|m| m.starts_with("qwen3"));
                    let txt = if models.is_empty() { "✓ Ollama is running, but no model is downloaded yet.".to_string() } else { format!("✓ Ollama is running. Models: {}", models.join(", ")) };
                    wrap_label(ui, RichText::new(txt).size(size::SMALL).color(p.green_text));
                    if !has_qwen {
                        if models.iter().all(|m| m.starts_with("llama3.2")) && !models.is_empty() {
                            ui.label(RichText::new("llama3.2: Fine for trying the flow, weak for real projects.").size(size::SMALL).color(p.orange));
                        }
                        ui.horizontal_wrapped(|ui| {
                            if ui.add_enabled(!pulling, egui::Button::new("Download qwen3 (recommended, about 5 GB)")).clicked() {
                                pull = true;
                            }
                            if pulling {
                                ui.spinner();
                            }
                        });
                    }
                }
            }
            if !progress.is_empty() {
                wrap_label(ui, RichText::new(&progress).size(size::TINY).color(p.dim));
            }
            ui.horizontal_wrapped(|ui| {
                let can = matches!(&ollama, Some(Ok(m)) if !m.is_empty());
                if ui.add_enabled(testing.is_none() && can, egui::Button::new("Test")).clicked() {
                    test = Some("ollama");
                }
                if ui.button("Check again").clicked() {
                    recheck = true;
                }
                app.test_result(ui, p, "ollama", testing, secs);
            });
        });
        ui.add_space(10.0);

        // 3. Claude API key
        self.choice_card(ui, p, "3", "Claude API key", None, "Fastest and best results, but you pay per use. Set a monthly spending limit in your Anthropic console first.", current == "anthropic", |app, ui| {
            if app.connect.key.is_empty() && !app.reg.settings.anthropic_key.is_empty() {
                app.connect.key = app.reg.settings.anthropic_key.clone();
            }
            ui.horizontal_wrapped(|ui| {
                ui.add(egui::TextEdit::singleline(&mut app.connect.key).password(true).hint_text("Paste your key (sk-ant-…)").desired_width(320.0));
                if ui.add_enabled(testing.is_none() && !app.connect.key.trim().is_empty(), egui::Button::new("Test")).clicked() {
                    test = Some("anthropic");
                }
                app.test_result(ui, p, "anthropic", testing, secs);
            });
            wrap_label(ui, RichText::new("Get a key at console.anthropic.com → API keys. It stays on this PC.").size(size::TINY).color(p.faint));
        });

        if recheck {
            self.connect_detect(true);
        }
        if pull {
            if let Ok(mut x) = self.connect.pull_progress.lock() {
                *x = "Starting the download…".into();
            }
            self.connect.pull_rx = Some(crate::ai::pull_model_async(OLLAMA_URL, "qwen3", self.connect.pull_progress.clone()));
        }
        if let Some(w) = test {
            self.connect_test(w);
        }
    }

    fn test_result(&self, ui: &mut egui::Ui, p: &Pal, which: &str, testing: Option<&str>, secs: u64) {
        if testing == Some(which) {
            ui.spinner();
            ui.label(RichText::new(format!("Waiting for a real reply… {secs} s")).size(size::SMALL).color(p.dim));
            return;
        }
        match self.connect.results.get(which) {
            Some(Ok(reply)) => {
                wrap_label(ui, RichText::new(format!("✓ Connected. It said: \"{}\"", reply.chars().take(120).collect::<String>())).size(size::SMALL).color(p.green_text));
            }
            Some(Err(e)) => {
                wrap_label(ui, RichText::new(format!("✗ {e}")).size(size::SMALL).color(p.orange));
            }
            None => {}
        }
    }

    fn choice_card(&mut self, ui: &mut egui::Ui, p: &Pal, n: &str, title: &str, tag: Option<&str>, line: &str, in_use: bool, body: impl FnOnce(&mut Self, &mut egui::Ui)) {
        egui::Frame::none().fill(p.card).stroke(Stroke::new(1.0, if in_use { p.accent } else { p.line })).rounding(space::RADIUS).inner_margin(space::CARD_PAD).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(n).font(fserif(size::TITLE)).color(p.accent_text));
                    ui.label(RichText::new(title).font(fs(size::TITLE)));
                    if let Some(t) = tag {
                        chip(ui, t, p.accent);
                    }
                    if in_use {
                        chip(ui, "in use", p.green);
                    }
                });
                wrap_label(ui, RichText::new(line).size(size::BODY).color(p.dim));
                ui.add_space(6.0);
                body(self, ui);
            });
        });
    }

    /// Help / Settings: copy version, Windows version, the AI in use and the last error.
    pub(super) fn problem_report(&self) -> String {
        let last = crate::ai::last_error();
        format!(
            "Resolve problem report\nVersion: {}\nWindows: {}\nAI in use: {}\nConnect your AI: {}\nLast error: {}\n",
            crate::store::VERSION,
            windows_version(),
            if self.reg.settings.ai_provider == "offline" { "none (fixed question set)".to_string() } else { self.provider_label() },
            if self.reg.settings.ai_setup_done { "done" } else { "not done" },
            if last.is_empty() { "none this session".to_string() } else { last }
        )
    }

    pub(super) fn report_problem_button(&mut self, ui: &mut egui::Ui, p: &Pal) {
        card(ui, p, |ui| {
            ui.label(RichText::new("Report a problem").font(fs(size::SECTION)));
            wrap_label(ui, RichText::new("Copies the app version, your Windows version, the AI in use and the last error message. No personal data or project content.").size(size::SMALL).color(p.dim));
            ui.horizontal_wrapped(|ui| {
                if ui.add(egui::Button::new(RichText::new("Copy problem report").color(p.on_accent)).fill(p.accent)).clicked() {
                    ui.ctx().copy_text(self.problem_report());
                    self.temp.insert("report_copied".into(), "1".into());
                }
                if self.temp.contains_key("report_copied") {
                    ui.label(RichText::new("✓ Copied. Paste this to Simeon.").size(size::SMALL).color(p.green_text));
                } else {
                    ui.label(RichText::new("Paste this to Simeon.").size(size::SMALL).color(p.faint));
                }
            });
        });
    }
}
