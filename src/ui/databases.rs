use super::theme::*;
use super::widgets::*;
use super::*;
use eframe::egui::{self, Color32, RichText, Sense, Stroke};

pub fn select_color(opt: &str) -> Color32 {
    let palette = SERIES;
    let mut h: u32 = 7;
    for b in opt.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as u32);
    }
    palette[(h % palette.len() as u32) as usize]
}

/// Edit one cell. Returns true if the value changed.
pub fn cell_ui(ui: &mut egui::Ui, p: &Pal, d: &WorkspaceData, col: &Column, value: &mut String, salt: &str, width: f32) -> bool {
    let mut changed = false;
    match col.kind {
        ColKind::Checkbox => {
            let mut b = value == "true";
            if ui.checkbox(&mut b, "").changed() {
                *value = b.to_string();
                changed = true;
            }
        }
        ColKind::Select => {
            let shown = if value.is_empty() { RichText::new("—").color(p.faint) } else { RichText::new(value.as_str()).color(select_color(value)) };
            egui::ComboBox::from_id_salt(("sel", salt)).selected_text(shown).width(width - 8.0).show_ui(ui, |ui| {
                if ui.selectable_label(value.is_empty(), "—").clicked() {
                    value.clear();
                    changed = true;
                }
                for o in &col.options {
                    if ui.selectable_label(value == o, RichText::new(o).color(select_color(o))).clicked() {
                        *value = o.clone();
                        changed = true;
                    }
                }
            });
        }
        ColKind::Date => {
            match chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                Ok(mut date) => {
                    ui.horizontal(|ui| {
                        if ui.add(egui_extras::DatePickerButton::new(&mut date).id_salt(salt).calendar_week(false).format("%-d %b %Y")).changed() {
                            *value = date.to_string();
                            changed = true;
                        }
                        if ghost_button(ui, RichText::new("✕").size(10.0).color(p.faint)).clicked() {
                            value.clear();
                            changed = true;
                        }
                    });
                }
                Err(_) => {
                    if ui.add(egui::Button::new(RichText::new("+ date").color(p.faint).size(12.0)).frame(false)).clicked() {
                        *value = today().to_string();
                        changed = true;
                    }
                }
            }
        }
        ColKind::Page => {
            let cur = d.page(value).map(|x| format!("{} {}", x.icon, x.title)).unwrap_or_else(|| "—".into());
            egui::ComboBox::from_id_salt(("pg", salt)).selected_text(cur).width(width - 8.0).show_ui(ui, |ui| {
                if ui.selectable_label(value.is_empty(), "—").clicked() {
                    value.clear();
                    changed = true;
                }
                for pg in d.pages.iter().filter(|x| !x.template) {
                    if ui.selectable_label(value == &pg.id, format!("{} {}", pg.icon, pg.title)).clicked() {
                        *value = pg.id.clone();
                        changed = true;
                    }
                }
            });
        }
        ColKind::Relation => {
            let ids = split_ids(value);
            let names: Vec<String> = ids.iter().filter_map(|id| d.relation_name(&col.target, id)).collect();
            let label = if names.is_empty() { "—".to_string() } else { names.join(", ") };
            let resp = ui.menu_button(RichText::new(format!("↗ {label}")).size(13.0).color(if names.is_empty() { p.faint } else { p.blue }), |ui| {
                ui.set_min_width(240.0);
                ui.label(RichText::new(format!("Link to {}", d.relation_target_label(&col.target))).size(12.0).color(p.dim));
                let key = egui::Id::new(("relsearch", salt));
                let mut q: String = ui.data_mut(|m| m.get_temp(key)).unwrap_or_default();
                if ui.add(egui::TextEdit::singleline(&mut q).hint_text("Search…")).changed() {
                    ui.data_mut(|m| m.insert_temp(key, q.clone()));
                }
                egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                    let ql = q.to_lowercase();
                    for (id, name) in d.relation_options(&col.target) {
                        if !ql.is_empty() && !name.to_lowercase().contains(&ql) {
                            continue;
                        }
                        let mut on = ids.contains(&id);
                        if ui.checkbox(&mut on, name).changed() {
                            let mut v = split_ids(value);
                            if on {
                                v.push(id.clone());
                            } else {
                                v.retain(|x| x != &id);
                            }
                            *value = v.join(",");
                            changed = true;
                        }
                    }
                });
            });
            resp.response.on_hover_text(if names.is_empty() { "Click to link records".to_string() } else { names.join("\n") });
        }
        ColKind::Number => {
            let r = ui.add(egui::TextEdit::singleline(value).desired_width(width - 8.0).horizontal_align(Align::Max).frame(false));
            if r.changed() {
                value.retain(|c| c.is_ascii_digit() || c == '.' || c == '-');
                changed = true;
            }
        }
        _ => {
            let r = ui.add(egui::TextEdit::singleline(value).desired_width(width - 8.0).frame(false).hint_text(match col.kind {
                ColKind::Url => "https://",
                ColKind::Email => "name@example.com",
                ColKind::Phone => "+44…",
                _ => "",
            }));
            if r.changed() {
                changed = true;
            }
            if matches!(col.kind, ColKind::Url | ColKind::Email) && !value.is_empty() && r.hovered() {
                let link = if col.kind == ColKind::Email { format!("mailto:{value}") } else { value.clone() };
                if ui.input(|i| i.modifiers.command && i.pointer.primary_clicked()) {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(link));
                }
            }
        }
    }
    changed
}

/// Which rows are visible after filters, quick search and sort.
pub fn visible_rows(d: &WorkspaceData, db: &Database, search: &str) -> Vec<usize> {
    let q = search.trim().to_lowercase();
    let mut idx: Vec<usize> = (0..db.rows.len())
        .filter(|&i| {
            let r = &db.rows[i];
            let pass_filters = db.filters.iter().all(|f| {
                let Some(col) = db.columns.iter().find(|c| c.id == f.col) else { return true };
                let raw = r.cells.get(&col.id).cloned().unwrap_or_default();
                f.matches(&raw, &d.cell_display(col, &raw))
            });
            let pass_search = q.is_empty() || db.columns.iter().any(|c| d.cell_display(c, r.cells.get(&c.id).map(|s| s.as_str()).unwrap_or("")).to_lowercase().contains(&q));
            pass_filters && pass_search
        })
        .collect();
    if let Some((cid, asc)) = &db.sort_by {
        if let Some(col) = db.columns.iter().find(|c| &c.id == cid) {
            idx.sort_by(|&a, &b| {
                let va = db.rows[a].cells.get(cid).cloned().unwrap_or_default();
                let vb = db.rows[b].cells.get(cid).cloned().unwrap_or_default();
                let ord = if col.kind == ColKind::Number {
                    va.parse::<f64>().unwrap_or(f64::MIN).partial_cmp(&vb.parse::<f64>().unwrap_or(f64::MIN)).unwrap_or(std::cmp::Ordering::Equal)
                } else if col.kind == ColKind::Select {
                    col.options.iter().position(|o| o == &va).unwrap_or(999).cmp(&col.options.iter().position(|o| o == &vb).unwrap_or(999))
                } else {
                    d.cell_display(col, &va).to_lowercase().cmp(&d.cell_display(col, &vb).to_lowercase())
                };
                if *asc {
                    ord
                } else {
                    ord.reverse()
                }
            });
        }
    }
    idx
}

impl App {
    pub fn database_view(&mut self, ui: &mut egui::Ui, p: &Pal, id: &str) {
        let mut delete = false;
        let mut as_template = false;
        self.top_bar(ui, p, |app, ui| {
            ui.menu_button(RichText::new("⋯").size(17.0), |ui| {
                if ui.button("📋 Copy as CSV").clicked() {
                    if let Some(db) = app.store.data.database(id) {
                        ui.ctx().output_mut(|o| o.copied_text = crate::import::database_to_csv(db));
                    }
                    app.toast("Copied as CSV");
                    ui.close_menu();
                }
                if ui.button("⧉ Save a copy as template").clicked() {
                    as_template = true;
                    ui.close_menu();
                }
                if ui.button(RichText::new("🗑 Delete database").color(p.accent_text)).clicked() {
                    delete = true;
                    ui.close_menu();
                }
            });
        });
        if delete {
            { let did = id.to_string(); let name = self.store.data.database(&did).map(|d| d.name.clone()).unwrap_or_default(); self.undoable(&format!("Deleted database \"{name}\""), "delete", |d| d.delete_database(&did)); }
            for pg in self.store.data.pages.iter_mut() {
                let before = pg.blocks.len();
                pg.blocks.retain(|b| !(b.kind == BlockKind::Database && b.text == id));
                if pg.blocks.len() != before {
                    pg.updated = now_ms();
                }
            }
            self.touch();
            self.view = View::Dashboard;
            return;
        }
        if as_template {
            if let Some(mut db) = self.store.data.database(id).cloned() {
                db.id = new_id();
                db.name = format!("{} (template)", db.name);
                db.rows.clear();
                db.template = true;
                db.updated = now_ms();
                self.store.data.databases.push(db);
                self.touch();
                self.toast("Saved as a template — find it under New database");
            }
        }
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                ui.add_space(40.0);
                ui.vertical(|ui| {
                    let Some(db) = self.store.data.database(id).cloned() else { return };
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(if db.icon.is_empty() { "▦" } else { &db.icon }).size(34.0));
                        let mut name = db.name.clone();
                        if ui.add(egui::TextEdit::singleline(&mut name).font(fb(32.0)).frame(false).desired_width(600.0)).changed() {
                            if let Some(x) = self.store.data.database_mut(id) {
                                x.name = name;
                                x.updated = now_ms();
                            }
                            self.touch();
                        }
                    });
                    if db.template {
                        ui.label(RichText::new("This is a template. New databases made from it copy its columns.").size(12.0).color(p.orange));
                    }
                    ui.add_space(6.0);
                    self.db_table(ui, p, id, false);
                });
            });
            ui.add_space(40.0);
        });
    }

    /// Toolbar + table/board for a database. Used full-page and embedded in pages.
    pub fn db_table(&mut self, ui: &mut egui::Ui, p: &Pal, id: &str, embedded: bool) {
        let Some(mut db) = self.store.data.database(id).cloned() else { return };
        let before = serde_json::to_string(&db).unwrap_or_default();
        let search_key = format!("dbsearch-{id}");
        let mut search = self.temp.get(&search_key).cloned().unwrap_or_default();
        let mut open_row: Option<String> = None;
        // ---- toolbar
        ui.horizontal(|ui| {
            for (v, l) in [(DbView::Table, "▦ Table"), (DbView::Board, "▥ Board")] {
                if ui.selectable_label(db.view == v, l).clicked() {
                    db.view = v;
                }
            }
            ui.separator();
            if db.view == DbView::Board {
                let selects: Vec<&Column> = db.columns.iter().filter(|c| c.kind == ColKind::Select).collect();
                let cur = db.group_by.as_ref().and_then(|g| db.columns.iter().find(|c| &c.id == g)).map(|c| c.name.clone()).unwrap_or_else(|| "choose".into());
                let mut gb = db.group_by.clone();
                egui::ComboBox::from_id_salt(("gb", id)).selected_text(format!("Group by: {cur}")).show_ui(ui, |ui| {
                    for c in selects {
                        ui.selectable_value(&mut gb, Some(c.id.clone()), &c.name);
                    }
                });
                db.group_by = gb;
            }
            let nf = db.filters.len();
            ui.menu_button(if nf > 0 { format!("⏷ Filter ({nf})") } else { "⏷ Filter".into() }, |ui| {
                ui.set_min_width(420.0);
                let mut remove: Option<usize> = None;
                for (fi, f) in db.filters.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        let cname = db.columns.iter().find(|c| c.id == f.col).map(|c| c.name.clone()).unwrap_or_default();
                        egui::ComboBox::from_id_salt(("fcol", id, fi)).selected_text(cname).width(120.0).show_ui(ui, |ui| {
                            for c in &db.columns {
                                ui.selectable_value(&mut f.col, c.id.clone(), &c.name);
                            }
                        });
                        egui::ComboBox::from_id_salt(("fop", id, fi)).selected_text(f.op.label()).width(100.0).show_ui(ui, |ui| {
                            for op in FilterOp::ALL {
                                ui.selectable_value(&mut f.op, op, op.label());
                            }
                        });
                        if f.op.needs_value() {
                            ui.add(egui::TextEdit::singleline(&mut f.value).desired_width(110.0).hint_text("value"));
                        }
                        if ui.small_button("✕").clicked() {
                            remove = Some(fi);
                        }
                    });
                }
                if let Some(r) = remove {
                    db.filters.remove(r);
                }
                if ui.button("+ Add filter").clicked() {
                    if let Some(c) = db.columns.first() {
                        db.filters.push(DbFilter { col: c.id.clone(), op: FilterOp::Contains, value: String::new() });
                    }
                }
                ui.label(RichText::new("Dates use YYYY-MM-DD. Filters are saved with the database.").size(11.0).color(p.faint));
            });
            let cur_sort = db.sort_by.as_ref().and_then(|(c, a)| db.columns.iter().find(|x| &x.id == c).map(|x| format!("{} {}", x.name, if *a { "↑" } else { "↓" }))).unwrap_or_else(|| "Sort".into());
            ui.menu_button(format!("⇅ {cur_sort}"), |ui| {
                if ui.button("None").clicked() {
                    db.sort_by = None;
                    ui.close_menu();
                }
                for c in &db.columns {
                    ui.horizontal(|ui| {
                        if ui.button(format!("{} ↑", c.name)).clicked() {
                            db.sort_by = Some((c.id.clone(), true));
                            ui.close_menu();
                        }
                        if ui.button("↓").clicked() {
                            db.sort_by = Some((c.id.clone(), false));
                            ui.close_menu();
                        }
                    });
                }
            });
            let r = ui.add(egui::TextEdit::singleline(&mut search).hint_text("🔍 Search").desired_width(150.0));
            if r.changed() {
                self.temp.insert(search_key.clone(), search.clone());
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if primary_button(ui, p, "New").clicked() {
                    let mut row = Row { id: new_id(), cells: Default::default(), page_id: None };
                    // new rows match the active filters where that's simple
                    for f in &db.filters {
                        if f.op == FilterOp::Is || f.op == FilterOp::Contains {
                            row.cells.insert(f.col.clone(), f.value.clone());
                        }
                        if f.op == FilterOp::Checked {
                            row.cells.insert(f.col.clone(), "true".into());
                        }
                    }
                    db.rows.push(row);
                }
            });
        });
        ui.add_space(4.0);
        let d = &self.store.data;
        let rows = visible_rows(d, &db, &search);
        let mut delete_row: Option<usize> = None;
        let mut add_col = false;
        let mut col_action: Option<(usize, &'static str)> = None;
        if db.view == DbView::Table {
            let total_w: f32 = db.columns.iter().map(|c| c.width).sum::<f32>() + 90.0;
            egui::ScrollArea::horizontal().id_salt(("dbh", id, embedded)).auto_shrink([false, true]).show(ui, |ui| {
                ui.set_min_width(total_w);
                // header
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.add_space(44.0);
                    let ncols = db.columns.len();
                    for (ci, c) in db.columns.iter_mut().enumerate() {
                        let w = c.width;
                        ui.allocate_ui_with_layout(egui::vec2(w, 28.0), Layout::left_to_right(Align::Center), |ui| {
                            ui.set_width(w);
                            let icon = match c.kind {
                                ColKind::Text => "Aa",
                                ColKind::Number => "#",
                                ColKind::Select => "◉",
                                ColKind::Checkbox => "☑",
                                ColKind::Date => "📅",
                                ColKind::Url => "🔗",
                                ColKind::Page => "📄",
                                ColKind::Relation => "↗",
                                ColKind::Email => "@",
                                ColKind::Phone => "☎",
                            };
                            ui.menu_button(RichText::new(format!("{icon} {}", c.name)).size(13.0).color(p.dim), |ui| {
                                ui.set_min_width(260.0);
                                ui.label(RichText::new("Property name").size(11.0).color(p.dim));
                                ui.text_edit_singleline(&mut c.name);
                                ui.label(RichText::new("Type").size(11.0).color(p.dim));
                                egui::ComboBox::from_id_salt(("ctype", &c.id)).selected_text(c.kind.label()).width(230.0).show_ui(ui, |ui| {
                                    for k in ColKind::ALL {
                                        ui.selectable_value(&mut c.kind, k, k.label());
                                    }
                                });
                                if c.kind == ColKind::Relation {
                                    ui.label(RichText::new("Links to").size(11.0).color(p.dim));
                                    egui::ComboBox::from_id_salt(("ctarget", &c.id)).selected_text(d.relation_target_label(&c.target)).width(230.0).show_ui(ui, |ui| {
                                        for (t, l) in [("projects", "Projects"), ("tasks", "Tasks"), ("pages", "Pages")] {
                                            ui.selectable_value(&mut c.target, t.to_string(), l);
                                        }
                                        for other in &d.databases {
                                            if !other.template {
                                                ui.selectable_value(&mut c.target, format!("db:{}", other.id), format!("▦ {}", other.name));
                                            }
                                        }
                                    });
                                }
                                if c.kind == ColKind::Select {
                                    ui.label(RichText::new("Options").size(11.0).color(p.dim));
                                    let mut rm: Option<usize> = None;
                                    for (oi, o) in c.options.iter_mut().enumerate() {
                                        ui.horizontal(|ui| {
                                            ui.colored_label(select_color(o), "●");
                                            ui.add(egui::TextEdit::singleline(o).desired_width(170.0));
                                            if ui.small_button("✕").clicked() {
                                                rm = Some(oi);
                                            }
                                        });
                                    }
                                    if let Some(r) = rm {
                                        c.options.remove(r);
                                    }
                                    ui.horizontal(|ui| {
                                        let k = egui::Id::new(("newopt", &c.id));
                                        let mut t: String = ui.data_mut(|m| m.get_temp(k)).unwrap_or_default();
                                        let r = ui.add(egui::TextEdit::singleline(&mut t).hint_text("New option").desired_width(170.0));
                                        ui.data_mut(|m| m.insert_temp(k, t.clone()));
                                        if (ui.small_button("Add").clicked() || (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && !t.trim().is_empty() {
                                            c.options.push(t.trim().to_string());
                                            ui.data_mut(|m| m.insert_temp(k, String::new()));
                                        }
                                    });
                                }
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Width").size(11.0).color(p.dim));
                                    ui.add(egui::Slider::new(&mut c.width, 60.0..=420.0));
                                });
                                ui.horizontal(|ui| {
                                    if ci > 0 && ui.button("← Move").clicked() {
                                        col_action = Some((ci, "left"));
                                        ui.close_menu();
                                    }
                                    if ci + 1 < ncols && ui.button("Move →").clicked() {
                                        col_action = Some((ci, "right"));
                                        ui.close_menu();
                                    }
                                });
                                if ci > 0 && ui.button(RichText::new("🗑 Delete property").color(p.accent_text)).clicked() {
                                    col_action = Some((ci, "delete"));
                                    ui.close_menu();
                                }
                            });
                        });
                    }
                    if ui.add(egui::Button::new(RichText::new(" + ").color(p.dim)).frame(false)).on_hover_text("Add a property").clicked() {
                        add_col = true;
                    }
                });
                hline(ui, p);
                let max_rows = if embedded { 50 } else { 1000 };
                for &ri in rows.iter().take(max_rows) {
                    let highlight = self.db_ui.highlight.as_deref() == Some(db.rows[ri].id.as_str());
                    let row_top = ui.cursor().min;
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.allocate_ui_with_layout(egui::vec2(44.0, 28.0), Layout::left_to_right(Align::Center), |ui| {
                            ui.set_width(44.0);
                            if ui.add(egui::Button::new(RichText::new("↗").size(12.0).color(p.dim)).frame(false)).on_hover_text("Open as page (notes, linked tasks)").clicked() {
                                open_row = Some(db.rows[ri].id.clone());
                            }
                            if ui.add(egui::Button::new(RichText::new("✕").size(11.0).color(p.faint)).frame(false)).on_hover_text("Delete row").clicked() {
                                delete_row = Some(ri);
                            }
                        });
                        let row_id = db.rows[ri].id.clone();
                        for c in db.columns.clone() {
                            let w = c.width;
                            ui.allocate_ui_with_layout(egui::vec2(w, 28.0), Layout::left_to_right(Align::Center), |ui| {
                                ui.set_width(w);
                                let mut v = db.rows[ri].cells.get(&c.id).cloned().unwrap_or_default();
                                if cell_ui(ui, p, d, &c, &mut v, &format!("{row_id}{}", c.id), w) {
                                    db.rows[ri].cells.insert(c.id.clone(), v);
                                }
                            });
                        }
                    });
                    let rr = egui::Rect::from_min_max(row_top, egui::pos2(row_top.x + total_w, ui.cursor().min.y));
                    if highlight {
                        ui.painter().rect_stroke(rr, 4.0, Stroke::new(1.5, p.accent));
                    }
                    hline(ui, p);
                }
                if rows.len() > max_rows {
                    ui.label(RichText::new(format!("{} more rows — open the full database to see them", rows.len() - max_rows)).color(p.dim));
                }
                if ui.add(egui::Button::new(RichText::new("+ New row").color(p.dim)).frame(false)).clicked() {
                    db.rows.push(Row { id: new_id(), cells: Default::default(), page_id: None });
                }
                ui.label(RichText::new(format!("{} of {} rows", rows.len(), db.rows.len())).size(11.5).color(p.faint));
            });
        } else {
            // ---- board
            let group = db.group_by.as_ref().and_then(|g| db.columns.iter().find(|c| &c.id == g && c.kind == ColKind::Select)).cloned();
            match group {
                None => {
                    ui.label(RichText::new("Add a Select property (like Status) to group the board by.").color(p.dim));
                    if ui.button("+ Add a Status property").clicked() {
                        let c = Column { id: new_id(), name: "Status".into(), kind: ColKind::Select, options: vec!["Not started".into(), "In progress".into(), "Done".into()], width: 140.0, target: String::new() };
                        db.group_by = Some(c.id.clone());
                        db.columns.push(c);
                    }
                }
                Some(gc) => {
                    let mut lanes: Vec<String> = vec![String::new()];
                    lanes.extend(gc.options.iter().cloned());
                    let mut moved: Option<(String, String)> = None;
                    let mut add_in: Option<String> = None;
                    egui::ScrollArea::horizontal().id_salt(("dbb", id, embedded)).auto_shrink([false, true]).show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            for lane in &lanes {
                                let frame = egui::Frame::none().fill(p.surface_sunk).rounding(10.0).inner_margin(egui::Margin::same(8.0));
                                let (_, dropped) = ui.dnd_drop_zone::<String, ()>(frame, |ui| {
                                    ui.set_width(250.0);
                                    ui.set_min_height(160.0);
                                    let in_lane: Vec<usize> = rows.iter().copied().filter(|&ri| db.rows[ri].cells.get(&gc.id).cloned().unwrap_or_default() == *lane || (lane.is_empty() && !gc.options.contains(&db.rows[ri].cells.get(&gc.id).cloned().unwrap_or_default()))).collect();
                                    ui.horizontal(|ui| {
                                        let name = if lane.is_empty() { "No value".to_string() } else { lane.clone() };
                                        chip(ui, &name, if lane.is_empty() { p.dim } else { select_color(lane) });
                                        ui.label(RichText::new(in_lane.len().to_string()).size(12.0).color(p.dim));
                                    });
                                    ui.add_space(4.0);
                                    for ri in in_lane {
                                        let r = &db.rows[ri];
                                        ui.dnd_drag_source(egui::Id::new(("dbcard", &r.id)), r.id.clone(), |ui| {
                                            egui::Frame::none().fill(p.card).stroke(Stroke::new(1.0, p.line)).rounding(8.0).inner_margin(egui::Margin::same(10.0)).show(ui, |ui| {
                                                ui.set_width(228.0);
                                                let t = db.title_of(r);
                                                if ui.add(egui::Label::new(RichText::new(if t.is_empty() { "Untitled" } else { &t }).font(fs(14.0))).sense(Sense::click())).clicked() {
                                                    open_row = Some(r.id.clone());
                                                }
                                                for c in db.columns.iter().skip(1).filter(|c| c.id != gc.id).take(4) {
                                                    let raw = r.cells.get(&c.id).cloned().unwrap_or_default();
                                                    let shown = d.cell_display(c, &raw);
                                                    if !raw.is_empty() && !(c.kind == ColKind::Checkbox && raw != "true") {
                                                        ui.label(RichText::new(format!("{}: {shown}", c.name)).size(12.0).color(p.dim));
                                                    }
                                                }
                                            });
                                        });
                                        ui.add_space(5.0);
                                    }
                                    if ui.add(egui::Button::new(RichText::new("+ New").color(p.dim)).frame(false)).clicked() {
                                        add_in = Some(lane.clone());
                                    }
                                });
                                if let Some(rid) = dropped {
                                    moved = Some(((*rid).clone(), lane.clone()));
                                }
                                ui.add_space(6.0);
                            }
                        });
                    });
                    if let Some((rid, lane)) = moved {
                        if let Some(r) = db.rows.iter_mut().find(|r| r.id == rid) {
                            r.cells.insert(gc.id.clone(), lane);
                        }
                    }
                    if let Some(lane) = add_in {
                        let mut row = Row { id: new_id(), cells: Default::default(), page_id: None };
                        row.cells.insert(gc.id.clone(), lane);
                        db.rows.push(row);
                    }
                }
            }
        }
        if let Some(ri) = delete_row {
            db.rows.remove(ri);
        }
        if add_col {
            db.columns.push(Column { id: new_id(), name: format!("Property {}", db.columns.len() + 1), kind: ColKind::Text, options: vec![], width: 150.0, target: String::new() });
        }
        if let Some((ci, what)) = col_action {
            match what {
                "left" if ci > 0 => db.columns.swap(ci, ci - 1),
                "right" if ci + 1 < db.columns.len() => db.columns.swap(ci, ci + 1),
                "delete" => {
                    let cid = db.columns[ci].id.clone();
                    db.columns.remove(ci);
                    for r in db.rows.iter_mut() {
                        r.cells.remove(&cid);
                    }
                    db.filters.retain(|f| f.col != cid);
                    if db.group_by.as_deref() == Some(cid.as_str()) {
                        db.group_by = None;
                    }
                }
                _ => {}
            }
        }
        let after = serde_json::to_string(&db).unwrap_or_default();
        if after != before {
            db.updated = now_ms();
            if delete_row.is_some() {
                // a deleted row is undoable (soft delete)
                let dbid = id.to_string();
                let label = format!("Deleted a row in {}", db.name);
                self.undoable(&label, "delete", move |d| {
                    if let Some(slot) = d.database_mut(&dbid) {
                        *slot = db;
                    }
                });
            } else {
                if let Some(slot) = self.store.data.database_mut(id) {
                    *slot = db;
                }
                self.touch();
            }
        }
        if let Some(rid) = open_row {
            self.open_row_page(id, &rid);
        }
    }

    /// Every row can be opened as a page (notes + linked tasks), like Notion.
    pub fn open_row_page(&mut self, dbid: &str, rid: &str) {
        let Some(db) = self.store.data.database(dbid) else { return };
        let Some(row) = db.rows.iter().find(|r| r.id == rid) else { return };
        if let Some(pg) = row.page_id.clone().filter(|x| self.store.data.page(x).is_some()) {
            self.go(View::Page(pg));
            return;
        }
        let title = db.title_of(row);
        let icon = if db.icon.is_empty() { "📄".to_string() } else { db.icon.clone() };
        let mut pg = Page::new(title);
        pg.icon = icon;
        let pgid = pg.id.clone();
        self.store.data.pages.push(pg);
        if let Some(db) = self.store.data.database_mut(dbid) {
            if let Some(r) = db.rows.iter_mut().find(|r| r.id == rid) {
                r.page_id = Some(pgid.clone());
            }
            db.updated = now_ms();
        }
        self.touch();
        self.go(View::Page(pgid));
    }

    /// Properties panel shown at the top of a row's page.
    pub fn row_properties(&mut self, ui: &mut egui::Ui, p: &Pal, dbid: &str, rid: &str) {
        let Some(mut db) = self.store.data.database(dbid).cloned() else { return };
        let Some(ri) = db.rows.iter().position(|r| r.id == rid) else { return };
        let mut changed = false;
        let mut open_db = false;
        egui::Frame::none().stroke(Stroke::new(1.0, p.line)).rounding(8.0).inner_margin(egui::Margin::same(12.0)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new("Record in").size(12.0).color(p.dim));
                if ghost_button(ui, RichText::new(format!("{} {}", db.icon, db.name)).size(12.5).color(p.blue)).clicked() {
                    open_db = true;
                }
            });
            let d = &self.store.data;
            egui::Grid::new(("rowprops", rid)).num_columns(2).spacing([16.0, 8.0]).show(ui, |ui| {
                for c in db.columns.clone() {
                    ui.label(RichText::new(&c.name).color(p.dim));
                    let mut v = db.rows[ri].cells.get(&c.id).cloned().unwrap_or_default();
                    if cell_ui(ui, p, d, &c, &mut v, &format!("rp{rid}{}", c.id), 320.0) {
                        db.rows[ri].cells.insert(c.id.clone(), v);
                        changed = true;
                    }
                    ui.end_row();
                }
            });
            // records elsewhere that link to this one
            let incoming = d.incoming_links(rid);
            if !incoming.is_empty() {
                ui.add_space(6.0);
                ui.label(RichText::new("Linked from").size(12.0).color(p.dim));
                ui.horizontal_wrapped(|ui| {
                    for (odb, dbname, cname, orow, title) in incoming {
                        if chip(ui, &format!("{dbname} · {cname}: {title}"), p.blue).clicked() {
                            self.db_ui.highlight = Some(orow.clone());
                            self.temp.insert("goto_db".into(), odb);
                        }
                    }
                });
            }
        });
        if changed {
            // keep the page title in step with the row title
            if let Some(c0) = db.columns.first() {
                let t = db.rows[ri].cells.get(&c0.id).cloned().unwrap_or_default();
                if let Some(pg) = db.rows[ri].page_id.clone().and_then(|x| self.store.data.page_mut(&x).map(|_| x)) {
                    if let Some(page) = self.store.data.page_mut(&pg) {
                        if !t.is_empty() {
                            page.title = t;
                        }
                    }
                }
            }
            db.updated = now_ms();
            if let Some(slot) = self.store.data.database_mut(dbid) {
                *slot = db;
            }
            self.touch();
        }
        if open_db {
            self.go(View::Database(dbid.to_string()));
        }
        if let Some(odb) = self.temp.remove("goto_db") {
            self.go(View::Database(odb));
        }
    }

    /// Create a database from a template. Returns the id of the main database.
    pub fn create_db_from_template(&mut self, which: &str) -> Id {
        let col = |name: &str, kind: ColKind, opts: &[&str], w: f32| Column { id: new_id(), name: name.into(), kind, options: opts.iter().map(|s| s.to_string()).collect(), width: w, target: String::new() };
        let mk = |name: &str, icon: &str, columns: Vec<Column>| Database { id: new_id(), name: name.into(), icon: icon.into(), group_by: columns.iter().find(|c| c.kind == ColKind::Select).map(|c| c.id.clone()), columns, updated: now_ms(), ..Default::default() };
        let main_id;
        match which {
            "crm" => {
                let mut companies = mk("Companies", "🏢", vec![col("Company", ColKind::Text, &[], 200.0), col("Industry", ColKind::Select, &["Software", "Retail", "Finance", "Other"], 130.0), col("Website", ColKind::Url, &[], 180.0)]);
                let mut contacts = mk("Contacts", "👤", vec![col("Name", ColKind::Text, &[], 180.0), col("Email", ColKind::Email, &[], 190.0), col("Phone", ColKind::Phone, &[], 130.0), col("Stage", ColKind::Select, &["Lead", "Contacted", "Customer", "Partner"], 120.0), col("Last contacted", ColKind::Date, &[], 130.0)]);
                let mut deals = mk("Opportunities", "💰", vec![col("Deal", ColKind::Text, &[], 200.0), col("Stage", ColKind::Select, &["Qualify", "Proposal", "Negotiation", "Won", "Lost"], 120.0), col("Value", ColKind::Number, &[], 100.0), col("Close date", ColKind::Date, &[], 130.0)]);
                // relations: contact → company, deal → company / contacts / project
                let mut c_company = col("Company", ColKind::Relation, &[], 160.0);
                c_company.target = format!("db:{}", companies.id);
                contacts.columns.push(c_company);
                let mut d_company = col("Company", ColKind::Relation, &[], 160.0);
                d_company.target = format!("db:{}", companies.id);
                let mut d_contacts = col("Contacts", ColKind::Relation, &[], 170.0);
                d_contacts.target = format!("db:{}", contacts.id);
                let mut d_project = col("Project", ColKind::Relation, &[], 150.0);
                d_project.target = "projects".into();
                let mut d_tasks = col("Next steps", ColKind::Relation, &[], 170.0);
                d_tasks.target = "tasks".into();
                deals.columns.extend([d_company, d_contacts, d_project, d_tasks]);
                companies.view = DbView::Table;
                deals.view = DbView::Board;
                main_id = deals.id.clone();
                // a hub page that embeds all three
                let mut hub = Page::new("CRM");
                hub.icon = "🤝".into();
                hub.blocks = vec![
                    Block::new(BlockKind::Paragraph, "Contacts belong to companies; opportunities link companies, contacts, a project and next-step tasks. Open any row (↗) to write notes about it."),
                    Block::new(BlockKind::Database, deals.id.clone()),
                    Block::new(BlockKind::Database, contacts.id.clone()),
                    Block::new(BlockKind::Database, companies.id.clone()),
                ];
                let hub_id = hub.id.clone();
                self.store.data.databases.extend([companies, contacts, deals]);
                self.store.data.pages.push(hub);
                self.touch();
                self.go(View::Page(hub_id));
                return main_id;
            }
            "projects" => {
                let mut db = mk("Project tracker", "📊", vec![col("Initiative", ColKind::Text, &[], 220.0), col("Status", ColKind::Select, &["Planned", "In progress", "Blocked", "Done"], 120.0), col("Owner", ColKind::Text, &[], 120.0), col("Due", ColKind::Date, &[], 130.0), col("Doc", ColKind::Page, &[], 160.0)]);
                let mut pr = col("Project", ColKind::Relation, &[], 150.0);
                pr.target = "projects".into();
                let mut tk = col("Tasks", ColKind::Relation, &[], 170.0);
                tk.target = "tasks".into();
                db.columns.extend([pr, tk]);
                main_id = db.id.clone();
                self.store.data.databases.push(db);
            }
            "content" => {
                let db = mk("Content calendar", "🗓", vec![col("Title", ColKind::Text, &[], 220.0), col("Channel", ColKind::Select, &["Blog", "Newsletter", "Social", "Video"], 120.0), col("Status", ColKind::Select, &["Idea", "Drafting", "Scheduled", "Published"], 120.0), col("Publish on", ColKind::Date, &[], 130.0), col("Link", ColKind::Url, &[], 180.0)]);
                main_id = db.id.clone();
                self.store.data.databases.push(db);
            }
            other => {
                if let Some(tpl) = self.store.data.database(other).cloned() {
                    let mut db = tpl;
                    db.id = new_id();
                    db.name = db.name.replace(" (template)", "");
                    db.template = false;
                    db.rows.clear();
                    db.updated = now_ms();
                    main_id = db.id.clone();
                    self.store.data.databases.push(db);
                } else {
                    return self.new_database();
                }
            }
        }
        self.touch();
        self.go(View::Database(main_id.clone()));
        main_id
    }
}
