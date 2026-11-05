//! Resolve screens: Intake (guided entry) and the project cockpit (ResolveOS Runtime Surface on
//! top of the existing project, tasks and pages). Model calls run on background threads; their
//! results are applied in `resolve_poll`, and every step is saved as it happens.

use super::theme::*;
use super::widgets::*;
use super::*;
use crate::resolve::intake::{self, Intake, OutcomeStep, Phase, Recommendation, Understanding};
use crate::resolve::roles::{self, Proposal};
use crate::resolve::{Kind, Record};
use std::sync::mpsc::{channel, Receiver};

pub enum JobOut {
    Understand(Id, Result<Understanding, String>),
    Judge(Id, String, Result<(bool, bool), String>),
    Recommend(Id, Result<Recommendation, String>),
    Plan(Id, Result<intake::ProjectPlan, String>),
    Team(Id, Result<Vec<(String, String)>, String>),
    Proposals(Id, String, Result<(String, Vec<Proposal>), String>),
    // ---- intake journey (v2)
    Answer(Id, crate::resolve::journey::AnswerOut),
    Understood(Id, Result<Understanding, String>, Vec<crate::ai::Trace>),
    Proposed(Id, Option<String>, Result<crate::resolve::journey::Proposal, String>, Vec<crate::ai::Trace>),
}

#[derive(Default)]
pub struct ResolveUi {
    pub rx: Option<Receiver<JobOut>>,
    pub busy: String,
    pub error: String,
    pub tab: usize,
    pub chat: String,
    pub chat_role: String,
    /// (project id, action, summary, items, accepted)
    pub proposals: Option<(Id, String, String, Vec<Proposal>, Vec<bool>)>,
    pub sel_record: Option<Id>,
    pub kind_filter: Option<Kind>,
    pub new_kind: Kind,
    pub new_title: String,
    pub show_import: bool,
    pub new_tool: String,
    // ---- intake journey (v2)
    /// Adopt in progress: (intake id, next step, state, snapshot for Undo).
    pub adopt: Option<(Id, usize, crate::resolve::journey::AdoptState, std::collections::BTreeMap<(String, Id), (serde_json::Value, i64)>)>,
    /// The Adopt change set, offered as Undo once the project opens.
    pub adopt_cs: Option<Id>,
    pub focus_qa: Option<Id>,
    pub show_why: bool,
    pub inspector: bool,
}

pub fn amber() -> Color32 {
    Color32::from_rgb(200, 160, 64)
}

impl App {
    pub(super) fn spawn_job(&mut self, label: &str, f: impl FnOnce() -> JobOut + Send + 'static) {
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let _ = tx.send(f());
        });
        self.resolve_ui.rx = Some(rx);
        self.resolve_ui.busy = label.to_string();
        self.resolve_ui.error.clear();
    }

    pub(super) fn intake_mut(&mut self, id: &str) -> Option<&mut Intake> {
        self.store.data.intakes.iter_mut().find(|x| x.id == id)
    }

    pub(super) fn put_intake(&mut self, it: &Intake) {
        if let Some(x) = self.intake_mut(&it.id) {
            *x = it.clone();
        }
    }

    pub(super) fn get_intake(&self, id: &str) -> Option<Intake> {
        self.store.data.intakes.iter().find(|x| x.id == id).cloned()
    }

    /// New project, from anywhere: resume an unfinished intake, else start one.
    pub fn start_intake(&mut self) {
        let open = self.store.data.intakes.iter().filter(|x| x.is_v2() && x.status == "in_progress" && x.stage != "name").max_by_key(|x| x.updated).map(|x| x.id.clone());
        if let Some(id) = open {
            self.go(View::Intake(id));
            return;
        }
        self.new_intake();
    }

    /// Always a fresh intake. The name is asked once, then remembered from the last intake.
    pub fn new_intake(&mut self) {
        let mut i = Intake::new();
        if let Some(last) = self.store.data.intakes.iter().filter(|x| !x.name.is_empty()).max_by_key(|x| x.updated) {
            i.name = last.name.clone();
            i.stage = "bring".into();
        }
        let id = i.id.clone();
        self.store.data.intakes.push(i);
        self.touch();
        self.go(View::Intake(id));
    }

    /// Open Test Intake: a separate window and process, with its own sandbox data folder.
    pub fn launch_testlab(&mut self) {
        let exe = std::env::current_exe().unwrap_or_default();
        match std::process::Command::new(exe).arg("--testlab").spawn() {
            Ok(_) => self.toast(format!("Test Intake opened in its own window (sandbox: {})", crate::resolve::testlab::default_root().display())),
            Err(e) => self.toast(format!("Couldn't open Test Intake: {e}")),
        }
    }

    /// A brand-new workspace opens straight into the intake.
    pub fn open_intake_if_new(&mut self) {
        let d = &self.store.data;
        let fresh = d.projects.is_empty() && d.tasks.is_empty() && !d.intakes.iter().any(|i| i.status == "done");
        if fresh && self.shot.is_none() {
            self.start_intake();
            self.back.clear();
        }
    }

    /// Apply finished background work. Called every frame.
    pub fn resolve_poll(&mut self, ctx: &egui::Context) {
        let Some(rx) = &self.resolve_ui.rx else { return };
        ctx.request_repaint_after(std::time::Duration::from_millis(150));
        let out = match rx.try_recv() {
            Ok(o) => o,
            Err(std::sync::mpsc::TryRecvError::Empty) => return,
            Err(_) => {
                self.resolve_ui.rx = None;
                self.resolve_ui.busy.clear();
                self.resolve_ui.error = "The AI request stopped unexpectedly".into();
                return;
            }
        };
        self.resolve_ui.rx = None;
        self.resolve_ui.busy.clear();
        match out {
            JobOut::Answer(id, out) => {
                if let Some(i) = self.intake_mut(&id) {
                    crate::resolve::journey::apply_answer(i, out);
                    self.resolve_ui.focus_qa = None;
                }
                self.touch();
            }
            JobOut::Understood(id, r, calls) => {
                if let Some(i) = self.intake_mut(&id) {
                    crate::resolve::journey::keep_trace(i, calls);
                    match r {
                        Ok(u) => {
                            i.understanding = Some(u);
                            i.last_error.clear();
                        }
                        Err(e) => i.last_error = e,
                    }
                    i.touch();
                }
                self.touch();
            }
            JobOut::Proposed(id, section, r, calls) => {
                if let Some(i) = self.intake_mut(&id) {
                    crate::resolve::journey::keep_trace(i, calls);
                    match r {
                        Ok(fresh) => {
                            match (section.as_deref(), i.proposal.as_mut()) {
                                (Some(s), Some(p)) if s != "all" => crate::resolve::journey::merge_section(p, fresh, s),
                                _ => i.proposal = Some(fresh),
                            }
                            let snapshot = i.clone();
                            if let Some(p) = i.proposal.as_mut() {
                                crate::resolve::journey::sanitise(p, &snapshot);
                            }
                            i.last_error.clear();
                        }
                        Err(e) => i.last_error = e,
                    }
                    i.touch();
                }
                self.touch();
            }
            JobOut::Understand(id, r) => match r {
                Ok(u) => {
                    if let Some(i) = self.intake_mut(&id) {
                        i.tools = u.mentioned_tools.clone();
                        i.understanding = Some(u);
                        i.sync_gap_answers();
                        i.manual = false;
                        i.step = 2;
                        i.last_error.clear();
                        i.touch();
                    }
                    self.touch();
                }
                Err(e) => self.intake_error(&id, e),
            },
            JobOut::Judge(id, answer, r) => {
                let verdict = r.as_ref().ok().copied();
                let step = self.intake_mut(&id).map(|i| intake::apply_verdict(i, &answer, verdict));
                self.touch();
                if step == Some(OutcomeStep::Proceed) {
                    self.run_recommend(&id);
                }
            }
            JobOut::Recommend(id, r) => match r {
                Ok(rec) => {
                    if let Some(i) = self.intake_mut(&id) {
                        i.recommendation = Some(rec);
                        i.plan = None;
                        i.step = 3;
                        i.last_error.clear();
                        i.touch();
                    }
                    self.touch();
                }
                Err(e) => self.intake_error(&id, e),
            },
            JobOut::Plan(id, r) => match r {
                Ok(plan) => {
                    if let Some(i) = self.intake_mut(&id) {
                        i.plan = Some(plan);
                        i.last_error.clear();
                        i.touch();
                    }
                    self.touch();
                    self.finish_intake(&id);
                }
                Err(e) => self.intake_error(&id, format!("{e}\nYou can try again, or create the project without the plan sections (readiness, risks, sharing paragraph).")),
            },
            JobOut::Team(pid, r) => match r {
                Ok(replies) => {
                    for (role, text) in replies {
                        let mut g = Record::new(Kind::Guidance, Some(pid.clone()), text.lines().next().unwrap_or("").chars().take(80).collect::<String>());
                        g.body = text;
                        g.fields.insert("role".into(), role);
                        g.fields.insert("provider".into(), self.reg.settings.ai_provider.clone());
                        self.store.data.records.push(g);
                    }
                    self.touch();
                }
                Err(e) => self.resolve_ui.error = e,
            },
            JobOut::Proposals(pid, action, r) => match r {
                Ok((summary, items)) => {
                    let n = items.len();
                    let mut g = Record::new(Kind::Guidance, Some(pid.clone()), roles::ACTIONS.iter().find(|a| a.id == action).map(|a| a.label).unwrap_or("Guidance").to_string());
                    g.body = summary.clone();
                    g.fields.insert("role".into(), if action == "prepare-build" { "tech-lead".into() } else { "product-director".into() });
                    g.fields.insert("action".into(), action.clone());
                    self.store.data.records.push(g);
                    self.resolve_ui.proposals = Some((pid, action, summary, items, vec![true; n]));
                    self.touch();
                }
                Err(e) => self.resolve_ui.error = e,
            },
        }
    }

    pub(super) fn intake_error(&mut self, id: &str, e: String) {
        if let Some(i) = self.intake_mut(id) {
            i.last_error = e;
            i.touch();
        }
        self.touch();
    }

    pub(super) fn run_recommend(&mut self, id: &str) {
        let Some(i) = self.get_intake(id) else { return };
        if i.manual {
            if let Some(x) = self.intake_mut(id) {
                if x.recommendation.is_none() {
                    x.recommendation = Some(Recommendation {
                        roadmap: vec![Phase { phase: "Now".into(), ..Default::default() }, Phase { phase: "Next".into(), ..Default::default() }, Phase { phase: "Later".into(), ..Default::default() }],
                        top_card_type: "action".into(),
                        ..Default::default()
                    });
                }
                x.step = 3;
                x.touch();
            }
            self.touch();
            return;
        }
        let cfg = crate::ai::current_config();
        let (u, g, t) = (i.understanding.clone().unwrap_or_default(), i.gap_answers.clone(), i.tools.clone());
        let id2 = id.to_string();
        self.spawn_job("Weighing it up… checking this against everything you confirmed", move || JobOut::Recommend(id2, intake::run_recommend(&cfg, &u, &g, &t)));
    }

    pub(super) fn finish_intake(&mut self, id: &str) {
        match intake::handoff(&mut self.store.data, id) {
            Ok(pid) => {
                self.save_now();
                self.toast("Your project is set up — first move pinned");
                self.resolve_ui.tab = 0;
                self.go(View::Resolve(pid));
            }
            Err(e) => self.intake_error(id, e),
        }
    }

    pub(super) fn loader(&mut self, ui: &mut egui::Ui, p: &Pal) {
        card(ui, p, |ui| {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(RichText::new(&self.resolve_ui.busy).font(fs(15.0)));
            });
            ui.label(RichText::new(format!("Using {} · {}", self.reg.settings.ai_provider, if self.reg.settings.ai_model.is_empty() { "default model" } else { &self.reg.settings.ai_model })).size(11.5).color(p.faint));
        });
    }
}

/// Unfinished intakes, newest first (for the sidebar and Home).
pub fn open_intakes(d: &WorkspaceData) -> Vec<(Id, String, u8)> {
    let mut v: Vec<&Intake> = d.intakes.iter().filter(|i| i.status == "in_progress" && (i.step > 0 || !i.input.is_empty())).collect();
    v.sort_by_key(|i| std::cmp::Reverse(i.updated));
    v.into_iter().map(|i| (i.id.clone(), i.title(), i.step)).collect()
}
