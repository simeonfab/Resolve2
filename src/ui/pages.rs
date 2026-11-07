use super::theme::*;
use super::widgets::*;
use super::*;
use eframe::egui::{self, text::LayoutJob, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, TextFormat};

const EMOJI: [&str; 40] = [
    "📄", "📝", "📌", "📚", "💡", "🚀", "🎯", "✅", "📅", "🧭", "🧪", "💼", "🏠", "❤", "⭐", "🔥", "🎨", "🛠", "📊", "🧠", "🌱", "🎵", "✈", "🍽", "💰", "📦", "🔒", "🌍", "⚙", "🗂", "🧾", "🎉", "☕", "🏃", "📷", "🎮", "🧩", "🗒", "🚨", "📈",
];

enum BAct {
    Split(usize, usize),
    MergeUp(usize),
    Delete(usize),
    InsertAfter(usize, BlockKind),
    Move(usize, i32),
    Dup(usize),
    Turn(usize, BlockKind),
    SubPage(usize),
    NewDb(usize),
    Focus(usize, usize),
}

#[derive(Clone, Copy)]
enum SlashChoice {
    Kind(BlockKind),
    SubPage,
}

fn slash_items(filter: &str) -> Vec<(String, &'static str, SlashChoice)> {
    let f = filter.trim().to_lowercase();
    let mut v: Vec<(String, &'static str, SlashChoice)> = BlockKind::MENU.iter().map(|k| (k.label().to_string(), k.icon(), SlashChoice::Kind(*k))).collect();
    v.insert(11, ("Sub-page".into(), "📄", SlashChoice::SubPage));
    v.into_iter().filter(|(l, _, _)| f.is_empty() || l.to_lowercase().contains(&f) || l.to_lowercase().replace(' ', "").contains(&f)).collect()
}

/// Build a LayoutJob for inline markdown: **bold**, *italic*, ~~strike~~, `code`, [[link]].
pub(crate) fn inline_job(text: &str, size: f32, family_bold: bool, color: Color32, p: &Pal, focused: bool, strike_all: bool, mono: bool) -> LayoutJob {
    let mut job = LayoutJob::default();
    let base_font = if mono { FontId::monospace(size - 2.0) } else if family_bold { fb(size) } else { fp(size) };
    let marker_fmt = if focused {
        TextFormat { font_id: FontId::new(size * 0.8, base_font.family.clone()), color: p.faint, ..Default::default() }
    } else {
        TextFormat { font_id: FontId::new(1.0, base_font.family.clone()), color: Color32::TRANSPARENT, ..Default::default() }
    };
    let normal = TextFormat { font_id: base_font.clone(), color, strikethrough: if strike_all { Stroke::new(1.0, color) } else { Stroke::NONE }, ..Default::default() };
    if mono {
        job.append(text, 0.0, normal);
        return job;
    }
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut buf = String::new();
    let flush = |job: &mut LayoutJob, buf: &mut String, f: &TextFormat| {
        if !buf.is_empty() {
            job.append(buf, 0.0, f.clone());
            buf.clear();
        }
    };
    let find = |from: usize, pat: &[char]| -> Option<usize> {
        let n = pat.len();
        let mut j = from;
        while j + n <= chars.len() {
            if &chars[j..j + n] == pat {
                return Some(j);
            }
            j += 1;
        }
        None
    };
    while i < chars.len() {
        let rest = &chars[i..];
        let (open, close, kind): (&[char], &[char], u8) = if rest.starts_with(&['*', '*']) {
            (&['*', '*'], &['*', '*'], 1)
        } else if rest.starts_with(&['~', '~']) {
            (&['~', '~'], &['~', '~'], 2)
        } else if rest.starts_with(&['[', '[']) {
            (&['[', '['], &[']', ']'], 3)
        } else if rest[0] == '`' {
            (&['`'], &['`'], 4)
        } else if rest[0] == '*' || (rest[0] == '_' && (i == 0 || !chars[i - 1].is_alphanumeric())) {
            if rest[0] == '*' {
                (&['*'], &['*'], 5)
            } else {
                (&['_'], &['_'], 5)
            }
        } else {
            buf.push(rest[0]);
            i += 1;
            continue;
        };
        let start = i + open.len();
        match find(start, close) {
            Some(end) if end > start => {
                flush(&mut job, &mut buf, &normal);
                let inner: String = chars[start..end].iter().collect();
                job.append(&open.iter().collect::<String>(), 0.0, marker_fmt.clone());
                let mut f = normal.clone();
                match kind {
                    1 => f.font_id = fb(size),
                    2 => f.strikethrough = Stroke::new(1.0, color),
                    3 => {
                        f.color = p.blue;
                        f.underline = Stroke::new(1.0, p.blue.linear_multiply(0.6));
                    }
                    4 => {
                        f.font_id = FontId::monospace(size - 2.0);
                        f.background = p.code;
                        f.color = p.orange;
                    }
                    _ => f.italics = true,
                }
                job.append(&inner, 0.0, f);
                job.append(&close.iter().collect::<String>(), 0.0, marker_fmt.clone());
                i = end + close.len();
            }
            _ => {
                buf.push(rest[0]);
                i += 1;
            }
        }
    }
    flush(&mut job, &mut buf, &normal);
    job
}

fn block_font(k: BlockKind) -> (f32, bool) {
    match k {
        BlockKind::H1 => (30.0, true),
        BlockKind::H2 => (24.0, true),
        BlockKind::H3 => (19.0, true),
        BlockKind::Code => (14.5, false),
        _ => (16.0, false),
    }
}

fn md_shortcut(text: &str) -> Option<(BlockKind, usize, bool)> {
    let pairs: [(&str, BlockKind, bool); 11] = [
        ("# ", BlockKind::H1, false),
        ("## ", BlockKind::H2, false),
        ("### ", BlockKind::H3, false),
        ("- ", BlockKind::Bullet, false),
        ("* ", BlockKind::Bullet, false),
        ("1. ", BlockKind::Numbered, false),
        ("[] ", BlockKind::Todo, false),
        ("[ ] ", BlockKind::Todo, false),
        ("[x] ", BlockKind::Todo, true),
        ("> ", BlockKind::Quote, false),
        ("``` ", BlockKind::Code, false),
    ];
    for (pre, k, c) in pairs {
        if text.starts_with(pre) {
            return Some((k, pre.chars().count(), c));
        }
    }
    if text == "---" {
        return Some((BlockKind::Divider, 3, false));
    }
    None
}

fn set_cursor(ctx: &egui::Context, id: egui::Id, pos: usize) {
    let mut state = egui::text_edit::TextEditState::load(ctx, id).unwrap_or_default();
    state.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(pos))));
    state.store(ctx, id);
}

fn cursor_of(ctx: &egui::Context, id: egui::Id) -> Option<(usize, usize)> {
    let state = egui::text_edit::TextEditState::load(ctx, id)?;
    let r = state.cursor.char_range()?;
    Some((r.primary.index, r.secondary.index))
}

impl App {
    pub fn page_view(&mut self, ui: &mut egui::Ui, p: &Pal, pid: &str) {
        let Some(page) = self.store.data.page(pid).cloned() else { return };
        // ---- top bar
        let mut delete_page = false;
        ui.horizontal(|ui| {
            ui.set_height(34.0);
            ui.add_space(12.0);
            if ui.add_enabled(!self.back.is_empty(), egui::Button::new(RichText::new("⏴").color(p.dim)).frame(false)).clicked() {
                if let Some(v) = self.back.pop() {
                    self.view = v;
                }
            }
            let path: Vec<(Id, String, String)> = self.store.data.page_path(pid).iter().map(|x| (x.id.clone(), x.icon.clone(), x.title.clone())).collect();
            for (i, (id, icon, title)) in path.iter().enumerate() {
                if i > 0 {
                    ui.label(RichText::new("/").color(p.faint));
                }
                let t = if title.is_empty() { "Untitled" } else { title };
                if ghost_button(ui, RichText::new(format!("{icon} {t}")).size(13.5)).clicked() && id != pid {
                    self.go(View::Page(id.clone()));
                }
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(12.0);
                ui.menu_button(RichText::new("⋯").size(17.0), |ui| {
                    if ui.button("📋 Copy as Markdown").clicked() {
                        let md = blocks_to_markdown(&page, &self.store.data);
                        ui.ctx().output_mut(|o| o.copied_text = md);
                        self.toast("Copied page as Markdown");
                        ui.close_menu();
                    }
                    if !page.template && ui.button("⧉ Save a copy as template").clicked() {
                        let mut tpl = page.clone();
                        tpl.id = new_id();
                        tpl.template = true;
                        tpl.parent = None;
                        tpl.favorite = false;
                        tpl.title = format!("{} (template)", if page.title.is_empty() { "Untitled" } else { &page.title });
                        tpl.blocks = tpl.blocks.iter().map(|b| Block { id: new_id(), ..b.clone() }).collect();
                        self.store.data.pages.push(tpl);
                        self.touch();
                        self.toast("Saved as a template. Use it from Pages → +");
                        ui.close_menu();
                    }
                    if ui.button("✔ Turn this page's to-dos into tasks").on_hover_text("Without AI: dates, times, p1-p4, #project and @labels written in each to-do are picked up. For smart sorting into projects use ✨ Ask AI → Organise my to-dos.").clicked() {
                        let n = self.page_todos_to_tasks(pid);
                        self.toast(format!("{n} to-dos are now tasks (ticking either ticks both)"));
                        ui.close_menu();
                    }
                    if ui.button("📄 Add sub-page").clicked() {
                        self.new_page(Some(pid.to_string()));
                        ui.close_menu();
                    }
                    ui.menu_button("↪ Move to…", |ui| {
                        egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                            if ui.button("Top level").clicked() {
                                if let Some(pg) = self.store.data.page_mut(pid) {
                                    pg.parent = None;
                                    pg.updated = now_ms();
                                }
                                self.touch();
                                ui.close_menu();
                            }
                            let others: Vec<(Id, String)> = self.store.data.pages.iter().filter(|x| x.id != pid && !self.store.data.page_path(&x.id).iter().any(|a| a.id == pid)).map(|x| (x.id.clone(), format!("{} {}", x.icon, x.title))).collect();
                            for (id, t) in others {
                                if ui.button(t).clicked() {
                                    if let Some(pg) = self.store.data.page_mut(pid) {
                                        pg.parent = Some(id.clone());
                                        pg.updated = now_ms();
                                    }
                                    self.expanded.insert(id);
                                    self.touch();
                                    ui.close_menu();
                                }
                            }
                        });
                    });
                    ui.separator();
                    if ui.button(RichText::new("🗑 Delete page").color(p.accent_text)).clicked() {
                        delete_page = true;
                        ui.close_menu();
                    }
                });
                let ai_label = if self.page_ui.ai_open { "✨ AI ✓" } else { "✨ AI" };
                if ui.button(ai_label).on_hover_text("Local AI assistant (Ollama / LM Studio): free, runs on this PC").clicked() {
                    self.page_ui.ai_open = !self.page_ui.ai_open;
                }
                let star = if page.favorite { RichText::new("★").color(p.accent).size(17.0) } else { RichText::new("☆").size(17.0).color(p.dim) };
                if ghost_button(ui, star).on_hover_text("Favourite").clicked() {
                    if let Some(pg) = self.store.data.page_mut(pid) {
                        pg.favorite = !pg.favorite;
                        pg.updated = now_ms();
                    }
                    self.touch();
                }
                let edited = chrono::DateTime::from_timestamp_millis(page.updated).map(|x| x.with_timezone(&chrono::Local).format("Edited %-d %b").to_string()).unwrap_or_default();
                ui.label(RichText::new(edited).size(12.5).color(p.dim));
            });
        });
        if delete_page {
            let title = page.title.clone();
            { let pg = pid.to_string(); let name = self.store.data.page(&pg).map(|p| p.title.clone()).unwrap_or_default(); self.undoable(&format!("Deleted page \"{name}\""), "delete", |d| d.delete_page(&pg)); }
            self.touch();
            self.view = page.parent.clone().map(View::Page).unwrap_or(View::Dashboard);
            self.toast(format!("Deleted \"{}\"", if title.is_empty() { "Untitled" } else { &title }));
            return;
        }

        let mut acts: Vec<BAct> = vec![];
        let mut open: Option<View> = None;
        egui::ScrollArea::vertical().id_salt(("page-scroll", pid)).auto_shrink([false, false]).show(ui, |ui| {
            centered(ui, 760.0, |ui| {
                ui.add_space(48.0);
                // icon
                let icon = if page.icon.is_empty() { "📄".to_string() } else { page.icon.clone() };
                let ir = ui.add(egui::Label::new(RichText::new(&icon).size(60.0)).sense(Sense::click())).on_hover_text("Change icon");
                if ir.clicked() {
                    self.page_ui.emoji_open = !self.page_ui.emoji_open;
                }
                if self.page_ui.emoji_open {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_max_width(420.0);
                        ui.horizontal_wrapped(|ui| {
                            for e in EMOJI {
                                if ui.add(egui::Button::new(RichText::new(e).size(22.0)).frame(false)).clicked() {
                                    if let Some(pg) = self.store.data.page_mut(pid) {
                                        pg.icon = e.to_string();
                                        pg.updated = now_ms();
                                    }
                                    self.touch();
                                    self.page_ui.emoji_open = false;
                                }
                            }
                        });
                    });
                }
                ui.add_space(6.0);
                // title
                let mut title = page.title.clone();
                let tid = egui::Id::new(("page-title", pid));
                let tr = ui.add(egui::TextEdit::multiline(&mut title).id(tid).hint_text("Untitled").font(fb(40.0)).frame(false).desired_rows(1).desired_width(f32::INFINITY));
                if self.temp.get("focus_title").map(|s| s == pid).unwrap_or(false) {
                    tr.request_focus();
                    self.temp.remove("focus_title");
                }
                if tr.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    title = title.replace('\n', "");
                    acts.push(BAct::Focus(0, 0));
                }
                if title != page.title {
                    let title = title.replace('\n', "");
                    if let Some(pg) = self.store.data.page_mut(pid) {
                        pg.title = title;
                        pg.updated = now_ms();
                    }
                    self.touch();
                }
                if page.template {
                    ui.label(RichText::new("⧉ This is a template. New pages made from it copy everything below.").size(12.5).color(p.orange));
                }
                if let Some((dbid, rid)) = self.store.data.row_for_page(pid) {
                    ui.add_space(6.0);
                    self.row_properties(ui, p, &dbid, &rid);
                }
                ui.add_space(14.0);
                // blocks
                let n = page.blocks.len();
                let mut number = 0;
                for i in 0..n {
                    let Some(mut b) = self.store.data.page(pid).and_then(|x| x.blocks.get(i).cloned()) else { break };
                    if b.kind == BlockKind::Numbered {
                        number += 1;
                    } else {
                        number = 0;
                    }
                    let before = (b.kind, b.text.clone(), b.checked);
                    self.block_ui(ui, p, pid, i, n, &mut b, number, &mut acts, &mut open);
                    if (b.kind, b.text.clone(), b.checked) != before {
                        if let Some(pg) = self.store.data.page_mut(pid) {
                            if let Some(slot) = pg.blocks.get_mut(i) {
                                *slot = b;
                            }
                            pg.updated = now_ms();
                        }
                        self.touch();
                    }
                }
                // click below to add
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 60.0), Sense::click());
                if resp.hovered() {
                    ui.painter().text(rect.left_top() + egui::vec2(4.0, 8.0), Align2::LEFT_TOP, "Click to add a block, or type / for commands", fp(14.0), p.faint);
                }
                if resp.clicked() {
                    let last_empty = self.store.data.page(pid).and_then(|x| x.blocks.last()).map(|b| b.kind == BlockKind::Paragraph && b.text.is_empty()).unwrap_or(false);
                    if last_empty {
                        acts.push(BAct::Focus(n - 1, 0));
                    } else {
                        acts.push(BAct::InsertAfter(n.saturating_sub(1), BlockKind::Paragraph));
                    }
                }
                // connections
                self.page_connections(ui, p, pid, &mut open);
                ui.add_space(80.0);
            });
        });
        self.apply_block_acts(ui.ctx(), pid, acts);
        if let Some(v) = open {
            self.go(v);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn block_ui(&mut self, ui: &mut egui::Ui, p: &Pal, pid: &str, i: usize, n: usize, b: &mut Block, number: usize, acts: &mut Vec<BAct>, open: &mut Option<View>) {
        let row_id = egui::Id::new(("blk-row", pid, &b.id));
        let edit_id = egui::Id::new(("blk", pid, &b.id));
        let prev_rect: Option<Rect> = ui.data(|d| d.get_temp(row_id));
        let pointer = ui.ctx().pointer_hover_pos();
        let hovered = prev_rect.map_or(false, |r| pointer.map_or(false, |pp| r.expand2(egui::vec2(60.0, 0.0)).contains(pp)));
        let has_focus = ui.memory(|m| m.has_focus(edit_id));
        let ctx = ui.ctx().clone();

        // pending focus request
        if let Some((fp_id, fi, cur)) = self.page_ui.focus.clone() {
            if fp_id == pid && fi == i {
                ctx.memory_mut(|m| m.request_focus(edit_id));
                set_cursor(&ctx, edit_id, cur.min(b.text.chars().count()));
                self.page_ui.focus = None;
            }
        }

        // slash menu / link suggestions (computed before keys so Enter can pick)
        let slash = has_focus && b.kind.is_text() && b.text.starts_with('/') && !b.text.contains(' ') && b.kind != BlockKind::Code;
        let slash_list = if slash { slash_items(&b.text[1..]) } else { vec![] };
        let link_partial: Option<String> = if has_focus && b.kind.is_text() {
            b.text.rfind("[[").and_then(|k| {
                let after = &b.text[k + 2..];
                if after.contains("]]") || after.len() > 40 {
                    None
                } else {
                    Some(after.to_string())
                }
            })
        } else {
            None
        };
        let link_list: Vec<(String, String)> = link_partial
            .as_ref()
            .map(|q| {
                let ql = q.to_lowercase();
                self.store.data.pages.iter().filter(|x| x.id != pid && x.title.to_lowercase().contains(&ql) && !x.title.is_empty()).take(8).map(|x| (x.icon.clone(), x.title.clone())).collect()
            })
            .unwrap_or_default();
        let menu_open = !slash_list.is_empty() || !link_list.is_empty();

        // keyboard handling for the focused block
        if has_focus {
            let cur = cursor_of(&ctx, edit_id);
            let at_start = cur.map_or(false, |(a, s)| a == 0 && s == 0);
            let len = b.text.chars().count();
            let at_end = cur.map_or(false, |(a, s)| a == len && s == len);
            if menu_open && ctx.input_mut(|inp| inp.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
                if let Some((_, _, choice)) = slash_list.first() {
                    self.apply_slash(b, i, *choice, acts);
                } else if let Some((_, t)) = link_list.first() {
                    if let Some(k) = b.text.rfind("[[") {
                        b.text.truncate(k);
                        b.text.push_str(&format!("[[{t}]] "));
                        let pos = b.text.chars().count();
                        acts.push(BAct::Focus(i, pos));
                    }
                }
            } else if b.kind != BlockKind::Code && ctx.input_mut(|inp| inp.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
                let pos = cur.map(|c| c.0.max(c.1)).unwrap_or(len);
                if b.text.is_empty() && matches!(b.kind, BlockKind::Bullet | BlockKind::Numbered | BlockKind::Todo | BlockKind::Quote | BlockKind::Callout) {
                    b.kind = BlockKind::Paragraph;
                } else {
                    acts.push(BAct::Split(i, pos));
                }
            } else if b.kind == BlockKind::Code && ctx.input_mut(|inp| inp.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter)) {
                acts.push(BAct::InsertAfter(i, BlockKind::Paragraph));
            } else if at_start && ctx.input(|inp| inp.key_pressed(egui::Key::Backspace)) {
                ctx.input_mut(|inp| inp.consume_key(egui::Modifiers::NONE, egui::Key::Backspace));
                if b.kind != BlockKind::Paragraph {
                    b.kind = BlockKind::Paragraph;
                } else if i > 0 {
                    acts.push(BAct::MergeUp(i));
                }
            } else if at_start && i > 0 && ctx.input_mut(|inp| inp.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)) {
                acts.push(BAct::Focus(i - 1, usize::MAX));
            } else if at_end && i + 1 < n && ctx.input_mut(|inp| inp.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)) {
                acts.push(BAct::Focus(i + 1, 0));
            }
        }

        let top = ui.cursor().min;
        let gutter = 52.0;
        let width = ui.available_width();
        let pad = match b.kind {
            BlockKind::H1 => 14.0,
            BlockKind::H2 => 10.0,
            BlockKind::H3 => 6.0,
            _ => 1.0,
        };
        ui.add_space(pad);
        let mut text_rect = Rect::NOTHING;
        ui.horizontal_top(|ui| {
            // left gutter with + and drag-handle menu, shown on hover
            ui.allocate_ui_with_layout(egui::vec2(gutter, 24.0), Layout::left_to_right(Align::Center), |ui| {
                ui.set_width(gutter);
                ui.spacing_mut().item_spacing.x = 2.0;
                if hovered || self.page_ui.menu_block == Some(i) {
                    if ui.add(egui::Button::new(RichText::new("+").color(p.faint).size(16.0)).frame(false)).on_hover_text("Add a block below").clicked() {
                        acts.push(BAct::InsertAfter(i, BlockKind::Paragraph));
                    }
                    let mr = ui.menu_button(RichText::new("⠿").color(p.faint).size(16.0), |ui| {
                        self.page_ui.menu_block = Some(i);
                        ui.menu_button("Turn into", |ui| {
                            for k in BlockKind::MENU {
                                if k == BlockKind::Database || k == BlockKind::PageLink {
                                    continue;
                                }
                                if ui.button(format!("{}  {}", k.icon(), k.label())).clicked() {
                                    acts.push(BAct::Turn(i, k));
                                    ui.close_menu();
                                }
                            }
                        });
                        if i > 0 && ui.button("↑ Move up").clicked() {
                            acts.push(BAct::Move(i, -1));
                            ui.close_menu();
                        }
                        if i + 1 < n && ui.button("↓ Move down").clicked() {
                            acts.push(BAct::Move(i, 1));
                            ui.close_menu();
                        }
                        if ui.button("⧉ Duplicate").clicked() {
                            acts.push(BAct::Dup(i));
                            ui.close_menu();
                        }
                        if ui.button("✔ Make it a task").on_hover_text("Creates an Inbox task linked to this page").clicked() {
                            let text = b.text.clone();
                            if !text.trim().is_empty() {
                                self.add_task_text(&text, None, "", None, None, Some(pid.to_string()));
                                self.toast("Task created and linked to this page");
                            }
                            ui.close_menu();
                        }
                        if ui.button(RichText::new("🗑 Delete").color(p.accent_text)).clicked() {
                            acts.push(BAct::Delete(i));
                            ui.close_menu();
                        }
                    });
                    if mr.inner.is_none() && self.page_ui.menu_block == Some(i) {
                        self.page_ui.menu_block = None;
                    }
                }
            });
            let avail = width - gutter - 8.0;
            ui.vertical(|ui| {
                ui.set_width(avail);
                match b.kind {
                    BlockKind::Divider => {
                        let (r, resp) = ui.allocate_exact_size(egui::vec2(avail, 18.0), Sense::click());
                        ui.painter().line_segment([r.left_center(), r.right_center()], Stroke::new(1.0, p.line));
                        if resp.clicked() {
                            acts.push(BAct::InsertAfter(i, BlockKind::Paragraph));
                        }
                        text_rect = r;
                    }
                    BlockKind::PageLink => {
                        let target = self.store.data.page(&b.text).map(|x| (x.icon.clone(), x.title.clone()));
                        match target {
                            Some((icon, t)) => {
                                let r = ui.add(egui::Label::new(RichText::new(format!("{icon}  {}", if t.is_empty() { "Untitled".to_string() } else { t })).size(16.0).underline()).sense(Sense::click()));
                                if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                    *open = Some(View::Page(b.text.clone()));
                                }
                            }
                            None => {
                                let mut chosen: Option<Id> = None;
                                egui::ComboBox::from_id_salt(("link-pick", &b.id)).selected_text("↗ Choose a page to link…").width(300.0).show_ui(ui, |ui| {
                                    for pg in &self.store.data.pages {
                                        if pg.id != pid && ui.button(format!("{} {}", pg.icon, pg.title)).clicked() {
                                            chosen = Some(pg.id.clone());
                                        }
                                    }
                                });
                                if let Some(c) = chosen {
                                    b.text = c;
                                }
                            }
                        }
                    }
                    BlockKind::Database => {
                        if self.store.data.database(&b.text).is_some() {
                            let dbid = b.text.clone();
                            ui.horizontal(|ui| {
                                let name = self.store.data.database(&dbid).map(|x| format!("{} {}", x.icon, x.name)).unwrap_or_default();
                                if ui.add(egui::Label::new(RichText::new(name).font(fb(17.0))).sense(Sense::click())).on_hover_text("Open full page").clicked() {
                                    *open = Some(View::Database(dbid.clone()));
                                }
                                ui.label(RichText::new("↗").color(p.dim));
                            });
                            self.db_table(ui, p, &dbid, true);
                        } else {
                            ui.label(RichText::new("▦ This database was deleted").color(p.dim));
                        }
                    }
                    _ => {
                        let (size, boldf) = block_font(b.kind);
                        let checked = b.kind == BlockKind::Todo && b.checked;
                        let col = if checked { p.dim } else { p.text };
                        let mono = b.kind == BlockKind::Code;
                        let focused_now = has_focus;
                        let pp = *p;
                        let mut layouter = move |ui: &egui::Ui, s: &str, wrap: f32| {
                            let mut job = inline_job(s, size, boldf, col, &pp, focused_now, checked, mono);
                            job.wrap.max_width = wrap;
                            ui.fonts(|f| f.layout_job(job))
                        };
                        let hint = if has_focus && b.text.is_empty() {
                            match b.kind {
                                BlockKind::Paragraph => "Type '/' for commands, [[ to link a page",
                                BlockKind::H1 => "Heading 1",
                                BlockKind::H2 => "Heading 2",
                                BlockKind::H3 => "Heading 3",
                                BlockKind::Todo => "To-do",
                                BlockKind::Quote => "Quote",
                                BlockKind::Code => "Code",
                                _ => "List",
                            }
                        } else if b.text.is_empty() && n == 1 {
                            "Start writing, or type '/' for commands"
                        } else {
                            ""
                        };
                        let mut draw_text = |ui: &mut egui::Ui, b: &mut Block, w: f32| -> egui::Response {
                            ui.add(egui::TextEdit::multiline(&mut b.text).id(edit_id).frame(false).desired_rows(1).desired_width(w).hint_text(RichText::new(hint).color(p.faint).size(size)).layouter(&mut layouter).margin(egui::vec2(0.0, 2.0)))
                        };
                        let resp = match b.kind {
                            BlockKind::Bullet | BlockKind::Numbered | BlockKind::Todo => {
                                ui.horizontal_top(|ui| {
                                    ui.spacing_mut().item_spacing.x = 4.0;
                                    let (r, cresp) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), if b.kind == BlockKind::Todo { Sense::click() } else { Sense::hover() });
                                    match b.kind {
                                        BlockKind::Bullet => {
                                            ui.painter().circle_filled(r.center(), 2.8, p.text);
                                        }
                                        BlockKind::Numbered => {
                                            ui.painter().text(r.center(), Align2::CENTER_CENTER, format!("{number}."), fp(16.0), p.text);
                                        }
                                        _ => {
                                            let br = Rect::from_center_size(r.center(), egui::vec2(16.0, 16.0));
                                            if b.checked {
                                                ui.painter().rect_filled(br, 3.0, p.blue);
                                                ui.painter().line_segment([br.center() + egui::vec2(-4.0, 0.0), br.center() + egui::vec2(-1.0, 3.0)], Stroke::new(1.8, Color32::WHITE));
                                                ui.painter().line_segment([br.center() + egui::vec2(-1.0, 3.0), br.center() + egui::vec2(4.5, -3.5)], Stroke::new(1.8, Color32::WHITE));
                                            } else {
                                                ui.painter().rect_stroke(br, 3.0, Stroke::new(1.5, if cresp.hovered() { p.blue } else { p.dim }));
                                            }
                                            if cresp.clicked() {
                                                b.checked = !b.checked;
                                            }
                                        }
                                    }
                                    let linked: Option<(String, Id)> = if b.kind == BlockKind::Todo {
                                        b.task_id.as_ref().and_then(|t| self.store.data.task(t)).map(|t| {
                                            let mut s = format!("→ # {}", self.store.data.project_name(&t.project_id));
                                            if let Some(d) = t.due {
                                                s.push_str(&format!(" · {}", fmt_date_short(d)));
                                            }
                                            (s, t.id.clone())
                                        })
                                    } else {
                                        None
                                    };
                                    let w = avail - 30.0 - if linked.is_some() { 170.0 } else { 0.0 };
                                    let r = draw_text(ui, b, w);
                                    if let Some((label, tid)) = linked {
                                        if chip(ui, &label, p.accent).on_hover_text("This to-do is a task: ticking either one ticks both. Click to open the task.").clicked() {
                                            self.open_task = Some(tid);
                                        }
                                    }
                                    r
                                })
                                .inner
                            }
                            BlockKind::Quote => ui
                                .horizontal_top(|ui| {
                                    let (r, _) = ui.allocate_exact_size(egui::vec2(3.0, 24.0), Sense::hover());
                                    let resp = draw_text(ui, b, avail - 16.0);
                                    let bar = Rect::from_min_max(r.min, Pos2::new(r.max.x, resp.rect.bottom()));
                                    ui.painter().rect_filled(bar, 1.0, p.text);
                                    resp
                                })
                                .inner,
                            BlockKind::Callout => egui::Frame::none()
                                .fill(p.callout)
                                .rounding(6.0)
                                .inner_margin(egui::Margin::symmetric(16.0, 14.0))
                                .show(ui, |ui| {
                                    ui.horizontal_top(|ui| {
                                        ui.label(RichText::new("💡").size(18.0));
                                        draw_text(ui, b, avail - 70.0)
                                    })
                                    .inner
                                })
                                .inner,
                            BlockKind::Code => egui::Frame::none()
                                .fill(p.code)
                                .rounding(6.0)
                                .inner_margin(egui::Margin::same(14.0))
                                .show(ui, |ui| {
                                    ui.label(RichText::new("code · Ctrl+Enter to leave").size(10.5).color(p.faint));
                                    draw_text(ui, b, avail - 30.0)
                                })
                                .inner,
                            _ => draw_text(ui, b, avail),
                        };
                        text_rect = resp.rect;
                        // markdown shortcuts while typing
                        if resp.changed() && b.kind == BlockKind::Paragraph {
                            if let Some((k, cut, checked)) = md_shortcut(&b.text) {
                                b.kind = k;
                                b.checked = checked;
                                b.text = b.text.chars().skip(cut).collect();
                                if k == BlockKind::Divider {
                                    b.text.clear();
                                    acts.push(BAct::InsertAfter(i, BlockKind::Paragraph));
                                } else {
                                    acts.push(BAct::Focus(i, b.text.chars().count()));
                                }
                            }
                        }
                    }
                }
            });
        });
        ui.add_space(pad.min(4.0));
        let bottom = ui.cursor().min.y;
        let row_rect = Rect::from_min_max(top, Pos2::new(top.x + width, bottom));
        ui.data_mut(|d| d.insert_temp(row_id, row_rect));

        // popups
        if menu_open {
            let pos = Pos2::new(text_rect.left(), text_rect.bottom() + 4.0);
            egui::Area::new(egui::Id::new(("blk-pop", &b.id))).order(egui::Order::Foreground).fixed_pos(pos).show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_min_width(260.0);
                    if !slash_list.is_empty() {
                        ui.label(RichText::new("Basic blocks").size(11.5).color(p.dim));
                        for (j, (label, icon, choice)) in slash_list.iter().enumerate() {
                            let r = ui.add(egui::SelectableLabel::new(j == 0, format!("{icon}   {label}")));
                            if r.clicked() {
                                self.apply_slash(b, i, *choice, acts);
                            }
                        }
                    } else {
                        ui.label(RichText::new("Link to page").size(11.5).color(p.dim));
                        for (j, (icon, t)) in link_list.iter().enumerate() {
                            if ui.add(egui::SelectableLabel::new(j == 0, format!("{icon}  {t}"))).clicked() {
                                if let Some(k) = b.text.rfind("[[") {
                                    b.text.truncate(k);
                                    b.text.push_str(&format!("[[{t}]] "));
                                    acts.push(BAct::Focus(i, b.text.chars().count()));
                                }
                            }
                        }
                    }
                });
            });
        }
    }

    /// Without AI: every unticked, unlinked to-do on the page becomes a linked task.
    /// Goes to the project whose document this page is, else the Inbox, unless the to-do says #project.
    pub fn page_todos_to_tasks(&mut self, pid: &str) -> usize {
        let proj = self.store.data.projects.iter().find(|p| p.page_id.as_deref() == Some(pid)).map(|p| p.id.clone());
        let todos: Vec<(Id, String)> = self.store.data.page_todos().into_iter().filter(|t| t.0 == pid && t.4.is_none()).map(|t| (t.2, t.3)).collect();
        let mut n = 0;
        for (bid, text) in todos {
            if let Some(tid) = self.add_task_text(&text, proj.clone(), "", None, None, Some(pid.to_string())) {
                if let Some(pg) = self.store.data.page_mut(pid) {
                    if let Some(b) = pg.blocks.iter_mut().find(|b| b.id == bid) {
                        b.task_id = Some(tid);
                    }
                    pg.updated = now_ms();
                }
                n += 1;
            }
        }
        self.touch();
        n
    }

    fn apply_slash(&mut self, b: &mut Block, i: usize, choice: SlashChoice, acts: &mut Vec<BAct>) {
        b.text.clear();
        match choice {
            SlashChoice::Kind(BlockKind::Database) => {
                b.kind = BlockKind::Paragraph;
                acts.push(BAct::NewDb(i));
            }
            SlashChoice::Kind(BlockKind::Divider) => {
                b.kind = BlockKind::Divider;
                acts.push(BAct::InsertAfter(i, BlockKind::Paragraph));
            }
            SlashChoice::Kind(k) => {
                b.kind = k;
                acts.push(BAct::Focus(i, 0));
            }
            SlashChoice::SubPage => {
                b.kind = BlockKind::Paragraph;
                acts.push(BAct::SubPage(i));
            }
        }
    }

    fn apply_block_acts(&mut self, ctx: &egui::Context, pid: &str, acts: Vec<BAct>) {
        if acts.is_empty() {
            return;
        }
        let mut focus: Option<(usize, usize)> = None;
        let mut go_page: Option<Id> = None;
        for a in acts {
            let Some(pg) = self.store.data.page_mut(pid) else { return };
            match a {
                BAct::Split(i, pos) => {
                    if let Some(b) = pg.blocks.get_mut(i) {
                        let chars: Vec<char> = b.text.chars().collect();
                        let pos = pos.min(chars.len());
                        let head: String = chars[..pos].iter().collect();
                        let tail: String = chars[pos..].iter().collect();
                        b.text = head;
                        let kind = match b.kind {
                            BlockKind::Bullet | BlockKind::Numbered | BlockKind::Todo => b.kind,
                            _ => BlockKind::Paragraph,
                        };
                        pg.blocks.insert(i + 1, Block::new(kind, tail));
                        focus = Some((i + 1, 0));
                    }
                }
                BAct::MergeUp(i) => {
                    if i > 0 && i < pg.blocks.len() {
                        if pg.blocks[i - 1].kind.is_text() {
                            let cur = pg.blocks.remove(i);
                            let prev = &mut pg.blocks[i - 1];
                            let at = prev.text.chars().count();
                            prev.text.push_str(&cur.text);
                            focus = Some((i - 1, at));
                        } else {
                            // previous is a divider / link / database: remove it instead
                            pg.blocks.remove(i - 1);
                            focus = Some((i - 1, 0));
                        }
                    }
                }
                BAct::Delete(i) => {
                    if i < pg.blocks.len() {
                        pg.blocks.remove(i);
                    }
                    if pg.blocks.is_empty() {
                        pg.blocks.push(Block::new(BlockKind::Paragraph, ""));
                    }
                    focus = Some((i.saturating_sub(1), usize::MAX));
                }
                BAct::InsertAfter(i, k) => {
                    let at = (i + 1).min(pg.blocks.len());
                    pg.blocks.insert(at, Block::new(k, ""));
                    focus = Some((at, 0));
                }
                BAct::Move(i, dir) => {
                    let j = i as i32 + dir;
                    if j >= 0 && (j as usize) < pg.blocks.len() {
                        pg.blocks.swap(i, j as usize);
                    }
                }
                BAct::Dup(i) => {
                    if let Some(b) = pg.blocks.get(i).cloned() {
                        let mut nb = b;
                        nb.id = new_id();
                        pg.blocks.insert(i + 1, nb);
                    }
                }
                BAct::Turn(i, k) => {
                    if let Some(b) = pg.blocks.get_mut(i) {
                        if !b.kind.is_text() {
                            b.text.clear();
                        }
                        b.kind = k;
                    }
                    focus = Some((i, usize::MAX));
                }
                BAct::Focus(i, pos) => focus = Some((i, pos)),
                BAct::SubPage(i) => {
                    let mut child = Page::new("");
                    child.parent = Some(pid.to_string());
                    let cid = child.id.clone();
                    if let Some(b) = pg.blocks.get_mut(i) {
                        b.kind = BlockKind::PageLink;
                        b.text = cid.clone();
                    }
                    pg.updated = now_ms();
                    self.store.data.pages.push(child);
                    self.expanded.insert(pid.to_string());
                    go_page = Some(cid);
                }
                BAct::NewDb(i) => {
                    let name = Column { id: new_id(), name: "Name".into(), kind: ColKind::Text, options: vec![], width: 220.0, target: String::new() };
                    let status = Column { id: new_id(), name: "Status".into(), kind: ColKind::Select, options: vec!["Not started".into(), "In progress".into(), "Done".into()], width: 140.0, target: String::new() };
                    let gb = Some(status.id.clone());
                    let mut row = Row { id: new_id(), cells: Default::default(), page_id: None };
                    row.cells.insert(name.id.clone(), String::new());
                    let db = Database { id: new_id(), name: "Untitled database".into(), icon: "▦".into(), columns: vec![name, status], rows: vec![row], view: DbView::Table, group_by: gb, sort_by: None, filters: vec![], template: false, updated: now_ms() };
                    if let Some(b) = pg.blocks.get_mut(i) {
                        b.kind = BlockKind::Database;
                        b.text = db.id.clone();
                    }
                    if i + 1 >= pg.blocks.len() {
                        pg.blocks.push(Block::new(BlockKind::Paragraph, ""));
                    }
                    pg.updated = now_ms();
                    self.store.data.databases.push(db);
                }
            }
            if let Some(pg) = self.store.data.page_mut(pid) {
                pg.updated = now_ms();
            }
        }
        self.touch();
        if let Some((i, pos)) = focus {
            let len = self.store.data.page(pid).map(|x| x.blocks.len()).unwrap_or(0);
            if len > 0 {
                let i = i.min(len - 1);
                let tl = self.store.data.page(pid).and_then(|x| x.blocks.get(i)).map(|b| b.text.chars().count()).unwrap_or(0);
                self.page_ui.focus = Some((pid.to_string(), i, pos.min(tl)));
                ctx.request_repaint();
            }
        }
        if let Some(c) = go_page {
            self.go(View::Page(c.clone()));
            self.temp.insert("focus_title".into(), c);
        }
    }

    fn page_connections(&mut self, ui: &mut egui::Ui, p: &Pal, pid: &str, open: &mut Option<View>) {
        let d = &self.store.data;
        let Some(page) = d.page(pid) else { return };
        let subs: Vec<&Page> = d.pages.iter().filter(|x| x.parent.as_deref() == Some(pid)).collect();
        let back = d.backlinks(pid);
        let links = page.wiki_links();
        let tasks: Vec<&Task> = d.tasks.iter().filter(|t| t.page_id.as_deref() == Some(pid)).collect();
        let events: Vec<&Event> = d.events.iter().filter(|e| e.page_id.as_deref() == Some(pid)).collect();
        let projects: Vec<&Project> = d.projects.iter().filter(|x| x.page_id.as_deref() == Some(pid)).collect();
        let mut create_link: Option<String> = None;
        let mut toggle: Option<Id> = None;
        let mut open_task: Option<Id> = None;
        let mut add_task = false;
        let mut open_event: Option<Event> = None;
        ui.add_space(24.0);
        hline(ui, p);
        ui.add_space(8.0);
        if !projects.is_empty() {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Project doc for").size(12.5).color(p.dim));
                for pr in &projects {
                    if chip(ui, &format!("# {}", pr.name), rgb(pr.color)).clicked() {
                        *open = Some(View::Project(pr.id.clone()));
                    }
                }
            });
        }
        if !subs.is_empty() {
            ui.label(RichText::new("Sub-pages").font(fs(13.0)).color(p.dim));
            for s in subs {
                if ui.add(egui::Label::new(RichText::new(format!("{}  {}", s.icon, if s.title.is_empty() { "Untitled" } else { &s.title })).size(14.5)).sense(Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    *open = Some(View::Page(s.id.clone()));
                }
            }
            ui.add_space(8.0);
        }
        if !links.is_empty() {
            ui.label(RichText::new("Links on this page").font(fs(13.0)).color(p.dim));
            ui.horizontal_wrapped(|ui| {
                for l in &links {
                    match d.page_by_title(l) {
                        Some(t) => {
                            if chip(ui, &format!("↗ {}", t.title), p.blue).clicked() {
                                *open = Some(View::Page(t.id.clone()));
                            }
                        }
                        None => {
                            if chip(ui, &format!("+ Create \"{l}\""), p.dim).on_hover_text("No page with this title yet").clicked() {
                                create_link = Some(l.clone());
                            }
                        }
                    }
                }
            });
            ui.add_space(8.0);
        }
        ui.label(RichText::new(format!("Backlinks · {}", back.len())).font(fs(13.0)).color(p.dim));
        if back.is_empty() {
            ui.label(RichText::new("No pages link here yet. Type [[ in another page to link to this one.").size(12.5).color(p.faint));
        }
        for bp in back {
            if ui.add(egui::Label::new(RichText::new(format!("↙ {} {}", bp.icon, bp.title)).size(14.0)).sense(Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                *open = Some(View::Page(bp.id.clone()));
            }
        }
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Linked tasks · {}", tasks.len())).font(fs(13.0)).color(p.dim));
            if ghost_button(ui, RichText::new("+ Add task for this page").size(12.5).color(p.accent_text)).clicked() {
                add_task = true;
            }
        });
        for t in tasks {
            ui.horizontal(|ui| {
                let (r, resp) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), Sense::click());
                paint_circle(ui, r.center(), p, t.priority, t.completed, resp.hovered());
                if resp.clicked() {
                    toggle = Some(t.id.clone());
                }
                let mut rt = RichText::new(&t.title).size(14.0);
                if t.completed {
                    rt = rt.strikethrough().color(p.dim);
                }
                if ui.add(egui::Label::new(rt).sense(Sense::click())).clicked() {
                    open_task = Some(t.id.clone());
                }
                if let Some((s, c)) = due_style(t, p) {
                    ui.label(RichText::new(s).size(12.0).color(c));
                }
            });
        }
        if !events.is_empty() {
            ui.add_space(8.0);
            ui.label(RichText::new("Linked events").font(fs(13.0)).color(p.dim));
            for e in events {
                if ui.add(egui::Label::new(RichText::new(format!("📅 {} · {}", e.title, e.start.format("%a %-d %b %H:%M"))).size(14.0)).sense(Sense::click())).clicked() {
                    open_event = Some(e.clone());
                }
            }
        }
        if let Some(t) = create_link {
            let mut np = Page::new(t);
            np.parent = Some(pid.to_string());
            let id = np.id.clone();
            self.store.data.pages.push(np);
            self.touch();
            *open = Some(View::Page(id));
        }
        if let Some(t) = toggle {
            self.toggle_task(&t);
        }
        if let Some(t) = open_task {
            self.open_task = Some(t);
        }
        if let Some(e) = open_event {
            self.event_edit = Some(calendar::draft_from(&e));
        }
        if add_task {
            self.quick = Some(QuickAdd { text: String::new(), kind: QuickKind::Task, project: None, section: String::new(), due: None, parent: None, page: Some(pid.to_string()), focus: true, opened: f64::NAN, error: false });
        }
    }

}
