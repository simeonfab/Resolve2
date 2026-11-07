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
        "This is the Resolve alpha. Thank you for being one of the first testers. I made this, and I want to hear what works and what doesn't. Message Simeon on the number he gave you if you get stuck.\n\nIf something goes wrong, use **Report a problem** below: it copies what Simeon needs to know (no personal data or project content).",
        &["#INTRO"],
    ),
    (
        "Quick start",
        "1. **Connect your AI** — Claude Code, Ollama or a Claude API key (Settings → AI if you skipped it).\n2. **Start something new with Resolve** — the workspace menu at the top left, or New project.\n3. **Answer the questions** — one at a time, in your own words. At most 8.\n4. **Check the proposal** — your next move, the roadmap themes, your first week, the whole picture.\n5. **Open your project** — Adopt builds it; then Open my project.",
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
        "Pick a project in the sidebar's picker. Overview shows **Do this next**, progress, what matters now, and the context from your intake (edit it here; everything else reads from it). The ResolveOS buttons ask the AI to challenge assumptions, plan validation or prepare the build — nothing is added until you confirm. **Plan page** and **Tasks board** are here too.",
        &[],
    ),
    (
        "Roadmap",
        "Now, Next and Later — **By phase** (columns) or **By theme** (each theme with its tasks). Dates come from sequence, not the calendar: each card shows its step number.\n\n- **Add a theme**: type it, pick Now / Next / Later, press **+ Theme**. Move it with ← and →.\n- **Milestones**: type one in *New milestone…* and press **+ Milestone**. They're listed under the lanes; open Knowledge → Evidence & Records to set *Done when…* and mark them Reached.\n- **Dependencies**: on a theme's **⋯** menu, choose **Comes after** to say which theme it waits for. Waiting-on items (Dependency and Blocker records) are listed under the lanes.\n- **Validation gates**: on the **⋯** menu, choose a hypothesis; the theme waits until that hypothesis is Supported.\n- **Coming up**: dated tasks in the next 14 days, filled in from your tasks' due dates automatically.",
        &[],
    ),
    ("Tasks", "With a project picked: its tasks, sections and board. With *All projects*: **Captured**, **Today**, **Upcoming** and **Filters & labels** across everything.", &["## 2. Tasks"]),
    (
        "Frameworks",
        "**Hypothesis Zero**: H0 is the baseline hypothesis; every hypothesis has its negation; any hypothesis can have sub-hypotheses and experiments. Fix an experiment's thresholds before running it — they lock once it's Running. Results update the hypothesis (unresolved, supported, weakened, refuted) and can gate roadmap themes.",
        &[],
    ),
    (
        "Knowledge",
        "Everything the project knows, in tabs: **Brief & documents**, **Research**, **Cut list** (what you decided not to do yet), **Evidence & Records** (hypothesis results, experiments, decisions, evidence and history, linked to each other), and **Advanced: databases**.\n\n**Add files** (or drag files onto the window) takes .md, .txt, .docx and .pdf. Resolve reads each one and files it in the right place — with an AI connected it decides; without one it guesses from the file name and headings and says so. Click another place to move a file.",
        &["## 3. Pages", "## 4. Databases", "## 6. Search", "## 7. Bring your data in"],
    ),
    (
        "AI Team",
        "Five specialists — **General assistant**, **Product Director**, **Validation Lead**, **Growth Lead** and **Tech Lead** — each reading your project's real context. **Ask AI** in the sidebar is the General assistant: the same conversation. With Claude, changes they make can be undone (↶). Local models can read but not change anything.",
        &["## 10. AI assistants"],
    ),
    (
        "Handoffs",
        "Write a brief for someone else — Claude Code, Codex or a person. Say who it's for and what they should do; Resolve writes the brief from that and the project's real context (what already exists on it, why it matters, what's off limits, when it's done). With an AI connected, it rewrites the brief in a few seconds. Copy it, mark it Sent or Done.",
        &[],
    ),
    ("Calendar", "Your events and dated tasks: day, week and month.", &["## 5. Calendar"]),
    (
        "Tips",
        "- **Windows+H** starts voice typing in any text box — talk instead of typing your answers.\n- **Ctrl+Q** adds a task, **Ctrl+K** searches everything, **Ctrl+N** makes a new page.\n- **Ctrl+Z / Ctrl+Y** undo and redo (when you're not typing in a box).\n- **Esc** leaves the intake with everything saved; it picks up where you stopped.",
        &["## 11. Good to know"],
    ),
    (
        "Troubleshooting",
        "**The AI isn't connecting.** Settings → AI → press **Test** on your choice: it shows a green tick or says what's wrong.\n- *Claude Code*: open PowerShell, type `claude`, and sign in; then Check again.\n- *Ollama*: make sure Ollama is running (it starts with Windows after you install it) and **qwen3** is downloaded.\n- *API key*: check you copied the whole key, and that the account has credit.\n\n**Windows says it protected your PC (SmartScreen).** The alpha isn't signed yet. Click **More info**, then **Run anyway**. You only need to do this once.\n\n**Where your data lives.** Everything stays on this PC, in your user folder: `%APPDATA%\\Resolve2` (paste that into File Explorer's address bar). Settings → Advanced → *Where your files are* opens it. Backups are made automatically; Settings → Your data restores them.",
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
    let mut s = format!("# Resolve — help ({})\n\n", crate::store::VERSION);
    for i in 0..GUIDE.len() {
        s.push_str(&section_text(i));
        s.push('\n');
    }
    s
}

impl App {
    pub fn help_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        self.top_bar(ui, p, |app, ui| {
            if ui.button("Copy as text").clicked() {
                ui.ctx().copy_text(guide_text());
                app.toast("Help copied");
            }
        });
        let mut sel: usize = self.temp.get("help_section").and_then(|s| s.parse().ok()).unwrap_or(0).min(GUIDE.len() - 1);
        ui.horizontal_top(|ui| {
            ui.add_space(16.0);
            // the sections, down the left
            ui.vertical(|ui| {
                ui.set_width(210.0);
                ui.add_space(18.0);
                ui.label(RichText::new("Help").font(fb(size::TITLE)));
                ui.label(RichText::new(crate::store::VERSION).size(size::SMALL).color(p.dim));
                ui.add_space(10.0);
                for (i, (title, _, _)) in GUIDE.iter().enumerate() {
                    if nav_item(ui, p, "", title, None, sel == i, None).clicked() {
                        sel = i;
                    }
                }
            });
            ui.add_space(24.0);
            egui::ScrollArea::vertical().id_salt(("help", sel)).auto_shrink([false, false]).show(ui, |ui| {
                let w = 760.0_f32.min(ui.available_width() - 24.0).max(320.0);
                ui.vertical(|ui| {
                ui.set_width(w);
                ui.add_space(18.0);
                let (title, _, _) = GUIDE[sel];
                ui.label(RichText::new(title).font(fserif(size::STEP_TITLE)));
                ui.add_space(8.0);
                let text = section_text(sel);
                let body = text.splitn(2, '\n').nth(1).unwrap_or("").to_string();
                render_md(ui, p, &body);
                if sel == 0 {
                    ui.add_space(14.0);
                    self.report_problem_button(ui, p);
                }
                ui.add_space(60.0);
                });
            });
        });
        self.temp.insert("help_section".into(), sel.to_string());
    }
}

/// Draw Markdown (the subset the tutorial uses).
fn render_md(ui: &mut egui::Ui, p: &Pal, md: &str) {
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
