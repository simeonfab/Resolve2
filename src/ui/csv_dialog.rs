//! The CSV import dialog (C-E18, Pack C "Databases_Import"): the file's columns with a sample
//! value and the type Resolve guessed (you can change it), a new database or rows added to the
//! one you're in, and Import N rows. Rows that can't be read are listed with why; the rest go in
//! as one undoable change, and the failed ones can be brought in with their text as it is.

use super::theme::*;
use super::widgets::*;
use super::*;
use crate::import::{csv_append, csv_check, csv_new_database, csv_plan, CsvPlan};
use eframe::egui::{self, RichText};

/// What the dialog is showing.
pub struct CsvDialog {
    pub file: String,
    pub plan: CsvPlan,
    pub kinds: Vec<ColKind>,
    /// Make a new database (true) or add the rows to `target`.
    pub new_db: bool,
    pub target: Id,
}

#[derive(Default)]
pub struct CsvUi {
    pub dialog: Option<CsvDialog>,
    /// The last import's failed rows, for "Import them as text": (database, plan, types, row indexes).
    pub failed: Option<(Id, CsvPlan, Vec<ColKind>, Vec<usize>)>,
}

/// The types offered for a column (relations and page links are set up in the table itself).
const KINDS: [ColKind; 8] = [ColKind::Text, ColKind::Number, ColKind::Date, ColKind::Select, ColKind::Checkbox, ColKind::Url, ColKind::Email, ColKind::Phone];

impl App {
    /// Pick a .csv file and open the dialog for it. `db` is the database you're in ("" for none).
    pub(super) fn csv_pick(&mut self, db: &str) {
        let Some(path) = rfd::FileDialog::new().add_filter("CSV", &["csv"]).pick_file() else { return };
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                self.toast(format!("Couldn't open that file: {e}"));
                return;
            }
        };
        let file = path.file_name().map(|x| x.to_string_lossy().to_string()).unwrap_or_else(|| "file.csv".into());
        self.csv_open(&file, &text, db);
    }

    /// Open the dialog on CSV text (also used by screenshots).
    pub fn csv_open(&mut self, file: &str, text: &str, db: &str) {
        match csv_plan(text) {
            Ok(plan) => {
                let kinds = plan.kinds.clone();
                self.csv.dialog = Some(CsvDialog { file: file.to_string(), plan, kinds, new_db: true, target: db.to_string() });
            }
            Err(e) => self.toast(format!("Couldn't read {file}: {e}")),
        }
    }

    pub(super) fn csv_dialog_ui(&mut self, ctx: &egui::Context, p: &Pal) {
        let Some(dlg) = self.csv.dialog.as_mut() else { return };
        let screen = ctx.screen_rect();
        let work = self.toaster.work.unwrap_or(screen);
        let mut close = false;
        let mut go = false;
        egui::Area::new(egui::Id::new("csv-dim")).order(egui::Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
            close |= backdrop(ui, screen, shadow::BACKDROP_DIALOG);
        });
        let target_name = self.store.data.database(&dlg.target).map(|x| x.name.clone());
        let stem = dlg.file.trim_end_matches(".csv").replace(['-', '_'], " ");
        let new_name = {
            let mut c = stem.chars();
            c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
        };
        let w = space::NEW_PAGE_W.min(work.width() - space::S40);
        let (ok, failed) = csv_check(&dlg.plan, &dlg.kinds);
        egui::Area::new(egui::Id::new("csv")).order(egui::Order::Foreground).fixed_pos(egui::pos2(work.center().x, screen.top() + space::S40 * 1.75)).pivot(Align2::CENTER_TOP).show(ctx, |ui| {
            dialog_frame(p).inner_margin(egui::Margin::same(space::S22)).show(ui, |ui| {
                ui.set_width(w - space::S22 * 2.0);
                ui.spacing_mut().item_spacing.y = space::S10;
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("Import {}", dlg.file)).font(fserif(size::CARD_SERIF)).color(p.text));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if button(ui, p, ButtonKind::Secondary, "✕", true, true).on_hover_text("Close (Esc)").clicked() {
                            close = true;
                        }
                    });
                });
                ui.label(RichText::new(format!("{} rows, {} columns. Check what each column is.", dlg.plan.rows.len(), dlg.plan.header.len())).size(size::ROW_SMALL).color(p.text));
                egui::ScrollArea::vertical().max_height(space::S40 * 7.0).auto_shrink([false, true]).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = space::S8;
                    for (ci, h) in dlg.plan.header.iter().enumerate() {
                        let sample = dlg.plan.rows.iter().filter_map(|r| r.get(ci)).map(|s| s.trim()).find(|s| !s.is_empty()).unwrap_or("").to_string();
                        edit_row(ui, p, |ui| {
                            ui.horizontal(|ui| {
                                let cw = (ui.available_width() - space::S40 * 3.0) / 2.0;
                                ui.allocate_ui_with_layout(egui::vec2(cw * 0.7, space::S22), Layout::left_to_right(Align::Center), |ui| {
                                    ui.set_width(cw * 0.7);
                                    ui.label(RichText::new(h).font(fs(size::ROW_SMALL)).color(p.text));
                                });
                                ui.allocate_ui_with_layout(egui::vec2(cw * 1.3, space::S22), Layout::left_to_right(Align::Center), |ui| {
                                    ui.set_width(cw * 1.3);
                                    ui.add(egui::Label::new(RichText::new(sample).size(size::ROW_SMALL).color(p.text)).truncate());
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let cur = dlg.kinds.get(ci).copied().unwrap_or(ColKind::Text);
                                    egui::ComboBox::from_id_salt(("csv-kind", ci)).selected_text(cur.label()).width(space::S40 * 2.5).show_ui(ui, |ui| {
                                        for k in KINDS {
                                            if ui.selectable_label(cur == k, k.label()).clicked() {
                                                if let Some(x) = dlg.kinds.get_mut(ci) {
                                                    *x = k;
                                                }
                                            }
                                        }
                                    });
                                });
                            });
                        });
                    }
                });
                ui.radio_value(&mut dlg.new_db, true, format!("New database: {new_name}"));
                if let Some(t) = &target_name {
                    ui.radio_value(&mut dlg.new_db, false, format!("Add rows to {t}"));
                }
                if !failed.is_empty() {
                    ui.label(RichText::new(format!("{} row{} won't go in as they are; you'll see which and why.", failed.len(), if failed.len() == 1 { "" } else { "s" })).size(size::ROW_SMALL).color(p.orange));
                }
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = space::S8;
                        go = button(ui, p, ButtonKind::Dark, &format!("Import {} row{}", ok.len(), if ok.len() == 1 { "" } else { "s" }), false, !ok.is_empty() || failed.is_empty()).clicked();
                        if button(ui, p, ButtonKind::Secondary, "Cancel", false, true).clicked() {
                            close = true;
                        }
                    });
                });
            });
        });
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close = true;
        }
        if go {
            let Some(dlg) = self.csv.dialog.take() else { return };
            let (ok, failed) = csv_check(&dlg.plan, &dlg.kinds);
            let target = if dlg.new_db {
                let db = csv_new_database(&new_name, &dlg.plan, &dlg.kinds, &ok, false);
                let id = db.id.clone();
                self.undoable(&format!("Imported {} rows into a new database, {new_name}", ok.len()), "import", move |d| d.databases.push(db));
                id
            } else {
                let tid = dlg.target.clone();
                let (plan, kinds, take) = (dlg.plan.clone(), dlg.kinds.clone(), ok.clone());
                let name = target_name.clone().unwrap_or_default();
                self.undoable(&format!("Imported {} rows into {name}", ok.len()), "import", move |d| {
                    if let Some(db) = d.database_mut(&tid) {
                        csv_append(db, &plan, &kinds, &take, false);
                    }
                });
                dlg.target.clone()
            };
            if failed.is_empty() {
                self.temp.remove("csv_failed");
                self.csv.failed = None;
            } else {
                // the rows that didn't go in, named with why (shown on the database's page)
                let list = failed.iter().map(|(n, w)| format!("{n}|{}", w.replace(['|', ';', '\t'], " "))).collect::<Vec<_>>().join(";");
                self.temp.insert("csv_failed".into(), format!("{target}\t{}\t{list}\t{}", dlg.file, ok.len()));
                let idx: Vec<usize> = failed.iter().map(|(n, _)| n - 2).collect();
                self.csv.failed = Some((target.clone(), dlg.plan.clone(), dlg.kinds.clone(), idx));
            }
            self.go(View::Database(target));
            return;
        }
        if close {
            self.csv.dialog = None;
        }
    }

    /// "Import them as text": the rows that failed, with their values kept as they are (cells past
    /// the last column are still dropped).
    pub(super) fn csv_import_failed_as_text(&mut self, db: &str) {
        let Some((target, plan, kinds, idx)) = self.csv.failed.take() else {
            self.toast("Those rows aren't here any more. Import the file again.");
            self.temp.remove("csv_failed");
            return;
        };
        if target != db {
            self.csv.failed = Some((target, plan, kinds, idx));
            return;
        }
        let n = idx.len();
        let tid = target.clone();
        self.undoable(&format!("Imported {n} more row{} as text", if n == 1 { "" } else { "s" }), "import", move |d| {
            if let Some(x) = d.database_mut(&tid) {
                csv_append(x, &plan, &kinds, &idx, true);
            }
        });
        self.temp.remove("csv_failed");
        if self.view_state == ViewState::Error {
            self.view_state = ViewState::Full;
        }
    }
}
