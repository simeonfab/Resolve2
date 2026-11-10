//! Help as a guide: sections listed down the left, each clickable. The written tutorial
//! (TUTORIAL.md, shipped inside the program) is kept and filed under these sections.

use super::theme::*;
use super::widgets::*;
use super::*;
use eframe::egui::{self, RichText};

/// The written tutorial, shipped inside the program.
pub const TUTORIAL: &str = include_str!("../../TUTORIAL.md");

/// (title, the guide's own text, the tutorial sections filed here — matched by heading start)
const GUIDE: &[(&str, &str, &[&str])] = &[
    (
        "Welcome",
        "This is the Resolve alpha. Thank you for being one of the first testers. I made this, and I want to hear what works and what doesn't. Message Simeon on the number he gave you if you get stuck.\n\nIf something goes wrong, use Report a problem below. It copies what Simeon needs to know, with no personal data or project content.",
        &["#INTRO"],
    ),
    (
        "Quick start",
        "1. **Connect your AI.** Claude Code, Ollama or a Claude API key. Settings, AI and MCP if you skipped it.\n2. **Start something new.** The workspace menu at the top left, or New project.\n3. **Answer the questions.** One at a time, in your own words. At most 8.\n4. **Check the proposal.** Your next move, the roadmap themes, your first week, the whole picture.\n5. **Open your project.** Adopt builds it, then Open my project.",
        &["## Start a project with Resolve"],
    ),
    (
        "Requirements",
        "- **Windows 10 or 11.** Nothing else to install for the app itself.\n- **One AI**, any of:\n  - **Claude Code** (free with Claude Pro or Max): install it and sign in once.\n  - **Ollama** (free, runs on your PC, slower): install from ollama.com and download **qwen3**.\n  - **A Claude API key** (pay per use): set a monthly spending limit in your Anthropic console first.\n- Without any AI, Resolve still works with a fixed set of questions.",
        &[],
    ),
    (
        "Home",
        "Your day across all projects: the capture bar, **Today** (what's due), **Captured** (tasks not in a project yet), your schedule, the next 7 days, recent pages and each guided project's next move.",
        &["## 1. Find your way around"],
    ),
    (
        "Projects",
        "Every project, with its progress and how many roadmap themes are in Now, Next and Later. Click one to open its Overview. **New project** opens the intake. Archived projects are listed at the bottom.",
        &[],
    ),
    (
        "Overview",
        "Pick a project in the sidebar's picker. Overview shows **Do this next**, progress, what matters now, and the context from your intake (edit it here; everything else reads from it). The ResolveOS buttons ask the AI to challenge assumptions, plan validation or prepare the build. Nothing is added until you confirm. **Plan page** and **Tasks board** are here too.",
        &[],
    ),
    (
        "Roadmap",
        "Now, Next and Later, shown **By phase** (columns) or **By theme** (a list). Open a theme to see its tasks, add one, move it, or open its experiment. **Whole picture** shows each theme against the hypothesis it rests on.\n\n- **+ Add a theme** adds a theme in a phase, or a milestone with its date.\n- A theme's **⋯** menu sets its phase, what it **comes after**, its **validation gate** (the theme waits until that hypothesis is Supported) and its status. Every change can be undone.\n- On the right: **Milestones** by date, **Dependencies and blockers**, and **Coming up** (dated tasks in the next two weeks, then work behind a closed gate).",
        &[],
    ),
    ("Tasks", "With a project picked: its tasks, sections and board. With *All projects*: **Captured**, **Today**, **Upcoming** and **Filters & labels** across everything.", &["## 2. Tasks"]),
    (
        "Frameworks",
        "**Hypothesis Zero**: H0 is the baseline hypothesis; every hypothesis has its negation; any hypothesis can have sub-hypotheses and experiments. Fix an experiment's thresholds before running it: they lock once it's Running. Results update the hypothesis (unresolved, supported, weakened, refuted) and can gate roadmap themes.",
        &[],
    ),
    (
        "Knowledge base",
        "Everything the project knows, in tabs: **Brief & documents**, **Research**, **Cut list** (what you decided not to do yet), **Evidence & Records** (hypothesis results, experiments, decisions, evidence and history, linked to each other), and **Advanced: databases**.\n\n**Add files** (or drag files onto the window) takes .md, .txt, .docx and .pdf. Resolve reads each one and files it in the right place. With an AI connected it decides; without one it guesses from the file name and headings and says so. Click another place to move a file.",
        &["## 3. Pages", "## 4. Databases", "## 6. Search", "## 7. Bring your data in"],
    ),
    (
        "AI Team",
        "Five specialists (**General assistant**, **Product Director**, **Validation Lead**, **Growth Lead** and **Tech Lead**), each reading your project's real context. **Ask AI** in the sidebar is the General assistant: the same conversation. No AI changes your workspace from a chat: each one proposes, and you add a change with **Add it** (↶ undoes it).",
        &["## 10. AI assistants"],
    ),
    (
        "Handoffs",
        "Write a brief for someone else: Claude Code, Codex or a person. Say who it's for and what they should do; Resolve writes the brief from that and the project's real context (what already exists on it, why it matters, what's off limits, when it's done). With an AI connected, it rewrites the brief in a few seconds. Copy it, mark it Sent or Done.",
        &[],
    ),
    ("Calendar", "Your events and dated tasks: day, week and month.", &["## 5. Calendar"]),
    (
        "Tips",
        "- **Windows+H** starts voice typing in any text box, so you can talk instead of typing your answers.\n- **Ctrl+Q** adds a task, **Ctrl+K** searches everything, **Ctrl+N** makes a new page.\n- **Ctrl+Z / Ctrl+Y** undo and redo (when you're not typing in a box).\n- **Esc** leaves the intake with everything saved; it picks up where you stopped.",
        &["## 11. Good to know"],
    ),
    (
        "Troubleshooting",
        "**The AI isn't connecting.** Settings, AI and MCP: press **Test** on your choice: it shows a green tick or says what's wrong.\n- *Claude Code*: open PowerShell, type `claude`, and sign in; then Check again.\n- *Ollama*: make sure Ollama is running (it starts with Windows after you install it) and **qwen3** is downloaded.\n- *API key*: check you copied the whole key, and that the account has credit.\n\n**Windows says it protected your PC (SmartScreen).** The alpha isn't signed yet. Click **More info**, then **Run anyway**. You only need to do this once.\n\n**Where your data lives.** Everything stays on this PC, in your user folder: `%APPDATA%\\Resolve2` (paste that into File Explorer's address bar). Settings, About, *Where your files are* opens it. Backups are made automatically; Settings, Sync and phone restores them.",
        &["## 0. Opening", "## 8. Backups", "## 9. Your phone", "## Test Intake"],
    ),
];

/// A tutorial section by the start of its heading (to the next `## `). "#INTRO" = before the first.
fn tutorial_part(head: &str) -> String {
    if head == "#INTRO" {
        return TUTORIAL.split("\n## ").next().unwrap_or("").lines().filter(|l| !l.starts_with("# ")).collect::<Vec<_>>().join("\n");
    }
    let Some(start) = TUTORIAL.find(head) else { return String::new() };
    let rest = &TUTORIAL[start..];
    let end = rest[3..].find("\n## ").map(|e| e + 3).unwrap_or(rest.len());
    rest[..end].to_string()
}

fn section_text(i: usize) -> String {
    let (title, own, parts) = GUIDE[i];
    let mut s = format!("## {title}\n\n{own}\n");
    for h in parts.iter() {
        let t = tutorial_part(h);
        if !t.trim().is_empty() {
            // the tutorial's own heading becomes a sub-heading here
            let t = t.replacen("## ", "### ", 1);
            s.push_str(&format!("\n{}\n", t.trim()));
        }
    }
    s
}

/// The whole guide as text (Copy as text).
pub fn guide_text() -> String {
    let mut s = format!("# Resolve help ({})\n\n", crate::store::VERSION);
    for i in 0..GUIDE.len() {
        s.push_str(&section_text(i));
        s.push('\n');
    }
    s
}

/// The Quick start steps: (lead, rest), from the guide's own numbered list.
pub fn quick_steps() -> Vec<(String, String)> {
    GUIDE[1]
        .1
        .lines()
        .filter_map(|l| {
            let l = l.trim();
            let rest = l.split_once(". ")?.1;
            let rest = rest.strip_prefix("**")?;
            let (lead, tail) = rest.split_once("**")?;
            Some((lead.to_string(), tail.trim().to_string()))
        })
        .collect()
}

/// The report's one line of details: version, Windows, the AI in use and the last error. No
/// names, no project text.
pub fn report_details(version: &str, os: &str, ai: &str, last_error: &str) -> String {
    format!("Alpha {version} · {os} · AI: {ai} · last error: {}", if last_error.trim().is_empty() { "none this session" } else { last_error.trim() })
}

/// Copy text to the Windows clipboard and say whether it worked (the text goes through an
/// environment variable, so any characters survive).
fn copy_checked(text: &str) -> Result<(), String> {
    let mut cmd = std::process::Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", "Set-Clipboard -Value $env:RESOLVE_REPORT"]).env("RESOLVE_REPORT", text);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    match cmd.output() {
        Ok(o) if o.status.success() => Ok(()),
        Ok(o) => Err(String::from_utf8_lossy(&o.stderr).trim().chars().take(160).collect()),
        Err(e) => Err(e.to_string()),
    }
}

/// The Report a problem dialog: what you write, the details, and where the copy got to.
#[derive(Default)]
pub struct ReportUi {
    pub open: bool,
    pub text: String,
    /// The copy running in the background.
    pub job: Option<std::sync::mpsc::Receiver<Result<(), String>>>,
    /// Some(Ok) copied, Some(Err) the clipboard couldn't be reached.
    pub done: Option<Result<(), String>>,
}

impl App {
    pub fn open_report(&mut self) {
        self.report = ReportUi { open: true, ..Default::default() };
    }

    fn report_text(&self) -> String {
        let d = self.report_details_line();
        let what = self.report.text.trim();
        if what.is_empty() {
            d
        } else {
            format!("{what} · {d}")
        }
    }

    fn report_details_line(&self) -> String {
        let ai = if self.reg.settings.ai_provider == "offline" { "none (fixed question set)".to_string() } else { self.provider_label() };
        report_details(env!("CARGO_PKG_VERSION"), &super::connect::windows_short(), &ai, &crate::ai::last_error())
    }

    pub fn help_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let mut sel: usize = self.temp.get("help_section").and_then(|s| s.parse().ok()).unwrap_or(0).min(GUIDE.len() - 1);
        let mut report = false;
        let mut copy_all = false;
        egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt(("help", sel)).show(ui, |ui| {
            egui::Frame::none().inner_margin(egui::Margin { left: space::WORK_SIDE - space::S4, right: space::WORK_SIDE - space::S4, top: space::S22, bottom: space::S40 * 2.0 }).show(ui, |ui| {
                let total = space::HELP_LIST + space::S24 + space::HELP_COL;
                let w = ui.available_width();
                let left = ((w - total.min(w)) / 2.0).max(0.0);
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.add_space(left);
                    // the sections
                    ui.allocate_ui_with_layout(egui::vec2(space::HELP_LIST, 0.0), Layout::top_down(Align::Min), |ui| {
                        ui.set_width(space::HELP_LIST);
                        ui.spacing_mut().item_spacing.y = space::S4 / 2.0;
                        ui.label(RichText::new("Help").font(fserif(size::GREETING)).color(p.text));
                        ui.add_space(space::S10);
                        for (i, (title, _, _)) in GUIDE.iter().enumerate() {
                            if help_item(ui, p, space::HELP_LIST, title, sel == i).clicked() {
                                sel = i;
                            }
                        }
                        ui.add_space(space::S14);
                        ui.horizontal(|ui| {
                            ui.add_space(space::S10);
                            copy_all = link(ui, p, "Copy the guide as text").clicked();
                        });
                    });
                    ui.add_space(space::S24);
                    let cw = space::HELP_COL.min(w - space::HELP_LIST - space::S24).max(space::S40 * 8.0);
                    ui.allocate_ui_with_layout(egui::vec2(cw, 0.0), Layout::top_down(Align::Min), |ui| {
                        ui.set_width(cw);
                        ui.set_max_width(cw);
                        ui.spacing_mut().item_spacing.y = space::S14;
                        let (title, own, _) = GUIDE[sel];
                        if sel == 0 {
                            // the dark ALPHA card, then Quick start
                            report = welcome_card(ui, p, own);
                            quick_card(ui, p);
                            let intro = tutorial_part("#INTRO");
                            if !intro.trim().is_empty() {
                                guide_card(ui, p, "", &intro);
                            }
                        } else if sel == 1 {
                            quick_card(ui, p);
                            let text = section_text(sel);
                            let rest: String = text.split("\n### ").skip(1).map(|s| format!("### {s}")).collect::<Vec<_>>().join("\n");
                            if !rest.trim().is_empty() {
                                guide_card(ui, p, "", &rest);
                            }
                        } else {
                            let text = section_text(sel);
                            let body = text.splitn(2, '\n').nth(1).unwrap_or("").to_string();
                            guide_card(ui, p, title, &body);
                        }
                    });
                });
            });
        });
        self.temp.insert("help_section".into(), sel.to_string());
        if report {
            self.open_report();
        }
        if copy_all {
            ui.ctx().copy_text(guide_text());
            self.toast("Help copied");
        }
    }

    /// The Report a problem dialog (C-E23): what happened (with the mic, C-E24), what's also
    /// included, and Copy the report; Copied, or the text to copy by hand if the clipboard
    /// can't be reached.
    pub fn report_dialog(&mut self, ctx: &egui::Context, p: &Pal) {
        if !self.report.open {
            return;
        }
        if let Some(rx) = &self.report.job {
            if let Ok(r) = rx.try_recv() {
                self.report.done = Some(r);
                self.report.job = None;
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            }
        }
        let screen = ctx.screen_rect();
        let work = self.toaster.work.unwrap_or(screen);
        let mut close = false;
        let mut copy = false;
        egui::Area::new(egui::Id::new("report-dim")).order(egui::Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
            close |= backdrop(ui, screen, shadow::BACKDROP_DIALOG);
        });
        let details = self.report_details_line();
        let full = self.report_text();
        let copying = self.report.job.is_some();
        let w = space::REPORT_W.min(work.width() - space::S40);
        egui::Area::new(egui::Id::new("report")).order(egui::Order::Foreground).fixed_pos(egui::pos2(work.center().x, screen.top() + space::DIALOG_TOP - space::S10)).pivot(Align2::CENTER_TOP).show(ctx, |ui| {
            dialog_frame(p).inner_margin(egui::Margin::same(space::S22)).show(ui, |ui| {
                ui.set_width(w - space::S22 * 2.0);
                ui.spacing_mut().item_spacing.y = space::S10;
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Report a problem").font(fserif(size::CARD_SERIF)).color(p.text));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if button(ui, p, ButtonKind::Secondary, "✕", true, true).on_hover_text("Close (Esc)").clicked() {
                            close = true;
                        }
                    });
                });
                match &self.report.done {
                    Some(Ok(())) => {
                        ui.label(RichText::new("Copied. Paste it into a message to Simeon. Thank you.").size(size::ROW).color(p.text));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if button(ui, p, ButtonKind::Dark, "Done", false, true).clicked() {
                                close = true;
                            }
                        });
                    }
                    Some(Err(_)) => {
                        egui::Frame::none().fill(p.orange_tint).rounding(egui::Rounding::same(radius::BUTTON)).inner_margin(egui::Margin::symmetric(space::S14, space::S10)).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            let mut job = egui::text::LayoutJob::default();
                            job.append("Resolve couldn't reach the clipboard. ", 0.0, egui::text::TextFormat { font_id: fs(size::ROW_SMALL), color: p.orange, ..Default::default() });
                            job.append("Select the text below and copy it yourself.", 0.0, egui::text::TextFormat { font_id: fp(size::ROW_SMALL), color: p.orange, ..Default::default() });
                            job.wrap.max_width = ui.available_width();
                            ui.label(job);
                        });
                        let mut t = full.clone();
                        egui::Frame::none().stroke(egui::Stroke::new(stroke::HAIR, p.line)).rounding(egui::Rounding::same(radius::BUTTON)).inner_margin(egui::Margin::symmetric(space::S12, space::S10)).show(ui, |ui| {
                            ui.add(egui::TextEdit::multiline(&mut t).frame(false).font(egui::FontId::monospace(size::SMALL)).text_color(p.text).desired_width(ui.available_width()).desired_rows(2));
                        });
                    }
                    None => {
                        field_label(ui, p, "What happened?");
                        let fid = egui::Id::new("report-text");
                        let fw = ui.available_width();
                        ui.horizontal_top(|ui| {
                            text_field(ui, p, fid, &mut self.report.text, "What you were doing, and what went wrong", true, fw - space::MIC - space::S8);
                            mic_for(ui, p, fid, false);
                        });
                        egui::Frame::none().fill(p.surface_raised).rounding(egui::Rounding::same(radius::BUTTON)).inner_margin(egui::Margin::symmetric(space::S12, space::S10)).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.spacing_mut().item_spacing.y = space::S4;
                            ui.label(RichText::new("Also included").font(fs(size::ROW_SMALL)).color(p.text));
                            ui.add(egui::Label::new(RichText::new(&details).monospace().size(size::SMALL).color(p.text)).wrap());
                            ui.label(RichText::new("No names, no project text.").size(size::ROW_SMALL).color(p.dim));
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.spacing_mut().item_spacing.x = space::S8;
                            copy = button(ui, p, ButtonKind::Dark, if copying { "Copying…" } else { "Copy the report" }, false, !copying).clicked();
                            if button(ui, p, ButtonKind::Secondary, "Cancel", false, true).clicked() {
                                close = true;
                            }
                        });
                    }
                }
            });
        });
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close = true;
        }
        if copy {
            let text = full.clone();
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let _ = tx.send(copy_checked(&text));
            });
            self.report.job = Some(rx);
        }
        if close {
            self.report = ReportUi::default();
        }
    }
}

/// Help's Welcome: the dark ALPHA card with Report a problem. Returns its click.
fn welcome_card(ui: &mut egui::Ui, p: &Pal, text: &str) -> bool {
    let mut clicked = false;
    egui::Frame::none().fill(tok::INK).rounding(egui::Rounding::same(radius::DIALOG)).inner_margin(egui::Margin::symmetric(space::S24, space::S22)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = space::S10;
        ui.label(RichText::new("ALPHA").font(fs(size::SMALL)).color(tok::AMBER));
        ui.label(RichText::new("Welcome").font(fserif(size::HERO_TITLE)).color(tok::TEXT_ON_DARK));
        let mut paras = text.split("\n\n");
        if let Some(a) = paras.next() {
            ui.add(egui::Label::new(RichText::new(a).size(size::ROW + 1.5).color(tok::TEXT_ON_DARK)).wrap());
        }
        for b in paras {
            ui.add(egui::Label::new(RichText::new(b).size(size::ROW + 0.5).color(tok::TEXT_ON_DARK_SOFT)).wrap());
        }
        ui.add_space(space::S4);
        clicked = button(ui, p, ButtonKind::Primary, "Report a problem", false, true).clicked();
    });
    clicked
}

/// Quick start: five numbered steps.
fn quick_card(ui: &mut egui::Ui, p: &Pal) {
    guide_frame(ui, p, |ui| {
        ui.label(RichText::new("Quick start").font(fserif(size::CARD_SERIF)).color(p.text));
        for (i, (lead, rest)) in quick_steps().iter().enumerate() {
            numbered_step(ui, p, i + 1, lead, rest);
        }
    });
}

fn guide_frame(ui: &mut egui::Ui, p: &Pal, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::none().fill(p.card).stroke(egui::Stroke::new(stroke::HAIR, p.line)).rounding(egui::Rounding::same(radius::DIALOG)).inner_margin(egui::Margin::symmetric(space::S22, space::S18)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = space::S12;
        add(ui);
    });
}

/// A guide section as a white card: its title (serif) and its text.
fn guide_card(ui: &mut egui::Ui, p: &Pal, title: &str, md: &str) {
    guide_frame(ui, p, |ui| {
        if !title.is_empty() {
            ui.label(RichText::new(title).font(fserif(size::CARD_SERIF)).color(p.text));
        }
        ui.spacing_mut().item_spacing.y = space::S4;
        render_md(ui, p, md);
    });
}

/// Draw Markdown (the subset the tutorial uses).
pub(super) fn render_md(ui: &mut egui::Ui, p: &Pal, md: &str) {
    let (_, blocks) = markdown_to_blocks(md);
    let mut number = 0;
    for b in &blocks {
        number = if b.kind == BlockKind::Numbered { number + 1 } else { 0 };
        let (sz, bold) = match b.kind {
            BlockKind::H1 => (size::TITLE, true),
            BlockKind::H2 => (18.0, true),
            BlockKind::H3 => (16.0, true),
            _ => (size::BODY + 0.5, false),
        };
        let job = super::pages::inline_job(&b.text, sz, bold, p.text, p, false, false, false);
        match b.kind {
            BlockKind::H1 | BlockKind::H2 | BlockKind::H3 => {
                ui.add_space(14.0);
                ui.add(egui::Label::new(job).wrap());
                ui.add_space(2.0);
            }
            BlockKind::Bullet | BlockKind::Todo => {
                ui.horizontal_top(|ui| {
                    ui.label(RichText::new("  •").size(size::BODY));
                    ui.add(egui::Label::new(job).wrap());
                });
            }
            BlockKind::Numbered => {
                ui.horizontal_top(|ui| {
                    ui.label(RichText::new(format!("  {number}.")).size(size::BODY));
                    ui.add(egui::Label::new(job).wrap());
                });
            }
            BlockKind::Callout | BlockKind::Quote => {
                egui::Frame::none().fill(p.callout).rounding(space::RADIUS_SMALL).inner_margin(egui::Margin::symmetric(16.0, 12.0)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.add(egui::Label::new(job).wrap());
                });
            }
            BlockKind::Code => {
                egui::Frame::none().fill(p.code).rounding(space::RADIUS_SMALL).inner_margin(egui::Margin::same(10.0)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.add(egui::Label::new(RichText::new(&b.text).monospace()).wrap());
                });
            }
            BlockKind::Divider => {
                ui.separator();
            }
            _ => {
                ui.add_space(4.0);
                ui.add(egui::Label::new(job).wrap());
            }
        }
    }
}
