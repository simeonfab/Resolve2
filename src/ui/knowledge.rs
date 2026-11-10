//! Frameworks (the small registry, Hypothesis Zero first) and Knowledge (project brief and
//! documents, research, the cut list, Evidence & Records, and databases as an advanced area).
//!
//! Ported from Control Centre (build-me-a-single-productivity/app/src/ui/sections.rs:
//! `frameworks_view`, `knowledge_view`, `FRAMEWORKS`, `KNOWLEDGE_TABS`). Changed for Resolve 2:
//! Hypothesis Zero and Evidence & Records use Resolve 2's own screens (cockpit.rs); pages and
//! databases are the ones that used to be listed in the sidebar; the cut list is the project
//! context's "not yet" list.

use super::nav::{frameworks, knowledge_tabs};
use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::{assess, Kind, Record};
use eframe::egui::{self, RichText};

impl App {
    // ------------------------------------------------------------ Frameworks

    pub(super) fn frameworks_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        ui.horizontal_wrapped(|ui| {
            for f in frameworks(self.mode()) {
                pill(ui, p, f.name, true).on_hover_text(f.id);
            }
            ui.label(RichText::new("More methods can be added to the registry later.").size(size::TINY).color(p.faint));
        });
        if let Some(f) = frameworks(self.mode()).first() {
            wrap_label(ui, RichText::new(f.about).size(size::SMALL).color(p.dim));
        }
        ui.add_space(10.0);
        if let Some(pid) = self.current_project.clone() {
            self.cockpit_validation(ui, p, &pid);
            return;
        }
        // all projects: each project's hypothesis tree, read only; Open to work on it
        let mut open: Option<Id> = None;
        for pid in self.scope_ids() {
            let tree = assess::hypothesis_tree(&self.store.data, &pid);
            let Some(pr) = self.store.data.project(&pid).cloned() else { continue };
            let rows: Vec<(usize, Record)> = tree.iter().take(12).filter_map(|(d, id)| self.store.data.record(id).cloned().map(|r| (*d, r))).collect();
            card(ui, p, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(format!("● {}", pr.name)).font(fs(size::SECTION)).color(rgb(pr.color)));
                    if ui.small_button("Open").clicked() {
                        open = Some(pid.clone());
                    }
                });
                if rows.is_empty() {
                    ui.label(RichText::new("No hypotheses yet.").size(size::SMALL).color(p.faint));
                }
                for (depth, r) in &rows {
                    ui.horizontal(|ui| {
                        ui.add_space(*depth as f32 * 16.0);
                        chip(ui, r.kind.icon(), if r.kind == Kind::Experiment { p.blue } else { p.accent });
                        wrap_label(ui, RichText::new(format!("{} · {}", r.title, r.status)).size(size::SMALL));
                    });
                }
            });
            ui.add_space(6.0);
        }
        if let Some(pid) = open {
            self.select_project(Some(pid));
        }
    }

    // ------------------------------------------------------------ Knowledge

    pub(super) fn knowledge_view(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let tabs = knowledge_tabs(self.mode());
        let mut tab = self.resolve_ui.knowledge_tab.min(tabs.len() - 1);
        ui.horizontal_wrapped(|ui| {
            for (i, (label, icon)) in tabs.iter().enumerate() {
                if pill(ui, p, &format!("{icon} {label}"), tab == i).clicked() {
                    tab = i;
                }
            }
        });
        self.resolve_ui.knowledge_tab = tab;
        ui.add_space(6.0);
        self.add_files_bar(ui, p);
        ui.add_space(10.0);
        let pid = self.current_project.clone();
        match tab {
            0 => self.knowledge_docs(ui, p, pid.as_deref()),
            1 => self.knowledge_research(ui, p, pid.as_deref()),
            2 => match pid {
                Some(pid) => self.knowledge_cut(ui, p, &pid),
                None => self.pick_project(ui, p, "The cut list belongs to a project: what you decided not to do yet, and why. Pick one:"),
            },
            3 => match pid {
                Some(pid) => {
                    wrap_label(ui, RichText::new("Hypothesis results, experiments, decisions, supporting evidence and the project's history. Evidence stays linked to the decisions and hypotheses it supports. Open a record to see its links.").size(size::SMALL).color(p.dim));
                    ui.add_space(6.0);
                    self.cockpit_records(ui, p, &pid);
                }
                None => self.pick_project(ui, p, "Evidence & Records belong to a project. Pick one:"),
            },
            _ => self.knowledge_databases(ui, p),
        }
    }

    fn brief_of(&self, pid: &str) -> Option<Id> {
        let d = &self.store.data;
        d.context_for(pid).and_then(|c| c.page_id.clone()).or_else(|| d.project(pid).and_then(|x| x.page_id.clone())).filter(|x| d.page(x).is_some())
    }

    fn page_link(&mut self, ui: &mut egui::Ui, p: &Pal, id: &str) {
        let Some(pg) = self.store.data.page(id).cloned() else { return };
        let t = if pg.title.is_empty() { "Untitled".to_string() } else { pg.title.clone() };
        let icon = if pg.icon.is_empty() { "📄".to_string() } else { pg.icon.clone() };
        if wrap_link(ui, RichText::new(format!("{icon}  {t}")).size(size::BODY).color(p.text)).clicked() {
            self.go(View::Page(id.to_string()));
        }
    }

    fn knowledge_docs(&mut self, ui: &mut egui::Ui, p: &Pal, pid: Option<&str>) {
        let brief = pid.and_then(|x| self.brief_of(x));
        self.imported_files_card(ui, p, brief);
        ui.add_space(10.0);
        if let Some(pid) = pid {
            let brief = self.brief_of(pid);
            let mut create = false;
            card(ui, p, |ui| {
                ui.label(RichText::new("Project brief").font(fs(size::SECTION)));
                match &brief {
                    Some(b) => {
                        let b = b.clone();
                        self.page_link(ui, p, &b);
                        let kids: Vec<Id> = self.store.data.pages.iter().filter(|x| x.parent.as_deref() == Some(b.as_str()) && !x.tags.iter().any(|t| t == "research")).map(|x| x.id.clone()).collect();
                        for k in kids {
                            ui.horizontal(|ui| {
                                ui.add_space(16.0);
                                self.page_link(ui, p, &k);
                            });
                        }
                    }
                    None => {
                        ui.label(RichText::new("No brief yet.").size(size::SMALL).color(p.faint));
                        create = ui.button("Create the project brief").clicked();
                    }
                }
            });
            if create {
                self.create_brief(pid);
                return;
            }
            ui.add_space(10.0);
        }
        // every page in the workspace (what the sidebar's Pages and Favorites lists used to show)
        card(ui, p, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("All documents").font(fs(size::SECTION)));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    self.new_page_menu(ui, p);
                });
            });
            let favs: Vec<Id> = self.store.data.pages.iter().filter(|x| x.favorite && !x.template).map(|x| x.id.clone()).collect();
            if !favs.is_empty() {
                ui.label(RichText::new("★ Favourites").size(size::TINY).color(p.faint));
                for f in favs {
                    self.page_link(ui, p, &f);
                }
                ui.add_space(4.0);
            }
            self.page_tree(ui, p, None, 0);
        });
    }

    fn create_brief(&mut self, pid: &str) {
        let name = self.store.data.project(pid).map(|x| x.name.clone()).unwrap_or_default();
        let mut pg = Page::new(format!("{name}: Project brief"));
        pg.icon = "🧭".into();
        pg.blocks = vec![Block::new(BlockKind::H2, "Goal"), Block::new(BlockKind::Paragraph, ""), Block::new(BlockKind::H2, "Notes"), Block::new(BlockKind::Paragraph, "")];
        let id = pg.id.clone();
        self.store.data.pages.push(pg);
        if let Some(pr) = self.store.data.projects.iter_mut().find(|x| x.id == pid) {
            pr.page_id = Some(id.clone());
            pr.updated = now_ms();
        }
        if let Some(c) = self.store.data.context_for_mut(pid) {
            c.page_id = Some(id.clone());
            c.updated = now_ms();
        }
        self.touch();
        self.go(View::Page(id));
    }

    /// The "+ page" menu that used to sit next to Pages in the sidebar.
    fn new_page_menu(&mut self, ui: &mut egui::Ui, p: &Pal) {
        ui.menu_button(RichText::new("+ New page").size(size::SMALL), |ui| {
            if ui.button("📄 Blank page   (Ctrl+N)").clicked() {
                self.new_page(None);
                ui.close_menu();
            }
            ui.separator();
            ui.label(RichText::new("From a template").size(size::TINY).color(p.dim));
            for (i, t) in crate::templates::PAGE_TEMPLATES.iter().enumerate() {
                if ui.button(format!("{}  {}", t.icon, t.name)).clicked() {
                    self.page_from_builtin(i);
                    ui.close_menu();
                }
            }
            let mine: Vec<(Id, String, String)> = self.store.data.pages.iter().filter(|x| x.template).map(|x| (x.id.clone(), x.icon.clone(), x.title.clone())).collect();
            if !mine.is_empty() {
                ui.separator();
                ui.label(RichText::new("Your templates").size(size::TINY).color(p.dim));
                for (id, icon, title) in mine {
                    ui.horizontal(|ui| {
                        if ui.button(format!("{icon}  {title}")).clicked() {
                            self.page_from_template(&id);
                            ui.close_menu();
                        }
                        if ui.small_button("edit").clicked() {
                            self.go(View::Page(id.clone()));
                            ui.close_menu();
                        }
                    });
                }
            }
        });
    }

    fn knowledge_research(&mut self, ui: &mut egui::Ui, p: &Pal, pid: Option<&str>) {
        let brief = pid.and_then(|x| self.brief_of(x));
        let mut new_note = false;
        let notes: Vec<Id> = self
            .store
            .data
            .pages
            .iter()
            .filter(|x| x.tags.iter().any(|t| t == "research"))
            .filter(|x| pid.is_none() || brief.as_ref().map_or(false, |b| x.parent.as_deref() == Some(b.as_str())))
            .map(|x| x.id.clone())
            .collect();
        card(ui, p, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Research notes").font(fs(size::SECTION)));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    new_note = ui.button("+ Research note").clicked();
                });
            });
            wrap_label(ui, RichText::new("Pages tagged \"research\": interviews, desk research, market notes.").size(size::TINY).color(p.faint));
            if notes.is_empty() {
                ui.label(RichText::new("No research notes yet.").size(size::SMALL).color(p.faint));
            }
            for n in &notes {
                self.page_link(ui, p, n);
            }
        });
        if new_note {
            let mut pg = Page::new("Research note");
            pg.icon = "🔎".into();
            pg.tags = vec!["research".into()];
            pg.parent = brief.clone();
            let id = pg.id.clone();
            self.store.data.pages.push(pg);
            self.touch();
            self.go(View::Page(id));
            return;
        }
        if let Some(pid) = pid {
            ui.add_space(10.0);
            let qs: Vec<Record> = self.store.data.records.iter().filter(|r| r.project_id.as_deref() == Some(pid) && matches!(r.kind, Kind::Question | Kind::Assumption) && r.is_open()).cloned().collect();
            let mut pick: Option<Id> = None;
            card(ui, p, |ui| {
                ui.label(RichText::new("Open questions and untested assumptions").font(fs(size::SECTION)));
                if qs.is_empty() {
                    ui.label(RichText::new("None open.").size(size::SMALL).color(p.faint));
                }
                for q in &qs {
                    if wrap_link(ui, RichText::new(format!("{} {}", q.kind.icon(), q.title)).size(size::SMALL)).on_hover_text("Open in Evidence & Records").clicked() {
                        pick = Some(q.id.clone());
                    }
                }
            });
            if let Some(id) = pick {
                self.resolve_ui.sel_record = Some(id);
                self.resolve_ui.knowledge_tab = 3;
            }
        }
    }

    fn knowledge_cut(&mut self, ui: &mut egui::Ui, p: &Pal, pid: &str) {
        let Some(mut ctx) = self.ensure_context(pid) else { return };
        let before = ctx.recommendation.do_not_yet.clone();
        let mut remove: Option<usize> = None;
        let mut add = false;
        card(ui, p, |ui| {
            ui.label(RichText::new("Cut list").font(fs(size::SECTION)));
            wrap_label(ui, RichText::new("What you decided not to do yet, and why. Themes you leave off the roadmap during the intake land here.").size(size::TINY).color(p.faint));
            if ctx.recommendation.do_not_yet.is_empty() {
                ui.label(RichText::new("Nothing cut yet.").size(size::SMALL).color(p.faint));
            }
            for (i, n) in ctx.recommendation.do_not_yet.iter_mut().enumerate() {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("✂").color(p.dim));
                    ui.add(egui::TextEdit::singleline(&mut n.item).font(fs(size::BODY)).desired_width(ui.available_width() - 40.0));
                    if ui.small_button("×").on_hover_text("Remove from the cut list").clicked() {
                        remove = Some(i);
                    }
                });
                ui.add(egui::TextEdit::multiline(&mut n.reason).hint_text("Why not now").desired_rows(1).desired_width(f32::INFINITY));
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.resolve_ui.new_cut).hint_text("Something you're choosing not to do yet…").desired_width(ui.available_width() - 80.0));
                add = ui.add_enabled(!self.resolve_ui.new_cut.trim().is_empty(), egui::Button::new("Add")).clicked();
            });
        });
        if let Some(i) = remove {
            ctx.recommendation.do_not_yet.remove(i);
        }
        if add {
            ctx.recommendation.do_not_yet.push(crate::resolve::intake::NotYet { item: self.resolve_ui.new_cut.trim().to_string(), reason: String::new() });
            self.resolve_ui.new_cut.clear();
        }
        if ctx.recommendation.do_not_yet != before {
            ctx.updated = now_ms();
            if let Some(x) = self.store.data.contexts.iter_mut().find(|x| x.id == ctx.id) {
                *x = ctx;
            }
            self.touch();
        }
    }

    fn knowledge_databases(&mut self, ui: &mut egui::Ui, p: &Pal) {
        card(ui, p, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Databases").font(fs(size::SECTION)));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.menu_button(RichText::new("+ New database").size(size::SMALL), |ui| {
                        if ui.button("▦ Blank database").clicked() {
                            self.new_database();
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.label(RichText::new("Templates").size(size::TINY).color(p.dim));
                        if ui.button("🤝 CRM: companies, contacts, opportunities (linked)").clicked() {
                            self.create_db_from_template("crm");
                            ui.close_menu();
                        }
                        if ui.button("📊 Project tracker (linked to projects & tasks)").clicked() {
                            self.create_db_from_template("projects");
                            ui.close_menu();
                        }
                        if ui.button("🗓 Content calendar").clicked() {
                            self.create_db_from_template("content");
                            ui.close_menu();
                        }
                        let mine: Vec<(Id, String)> = self.store.data.databases.iter().filter(|x| x.template).map(|x| (x.id.clone(), x.name.clone())).collect();
                        for (id, name) in mine {
                            if ui.button(format!("⧉ {name}")).clicked() {
                                self.create_db_from_template(&id);
                                ui.close_menu();
                            }
                        }
                    });
                });
            });
            wrap_label(ui, RichText::new("Advanced: structured tables (CRM, trackers) with linked rows. Most projects don't need one.").size(size::TINY).color(p.faint));
            let dbs: Vec<(Id, String, String)> = self.store.data.databases.iter().filter(|x| !x.template).map(|x| (x.id.clone(), x.icon.clone(), x.name.clone())).collect();
            if dbs.is_empty() {
                ui.label(RichText::new("No databases.").size(size::SMALL).color(p.faint));
            }
            for (id, icon, name) in dbs {
                let icon = if icon.is_empty() { "▦".to_string() } else { icon };
                if wrap_link(ui, RichText::new(format!("{icon}  {name}")).size(size::BODY)).clicked() {
                    self.go(View::Database(id));
                }
            }
        });
    }
}
