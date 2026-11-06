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
    /// (intake, answers it was computed from, result, calls)
    Understood(Id, usize, Result<Understanding, String>, Vec<crate::ai::Trace>),
    /// (intake, section, understanding key it was computed from, result, calls)
    Proposed(Id, Option<String>, String, Result<crate::resolve::journey::Proposal, String>, Vec<crate::ai::Trace>),
    /// A background job's result with how long it took.
    Timed(String, f64, Box<JobOut>),
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
    // ---- speed: background work and wait timing
    /// Background jobs (classification, precomputed understanding and proposal): (label, receiver).
    pub bg: Vec<(String, Receiver<JobOut>)>,
    /// When the foreground job started (app clock, seconds) and what it is: (intake id, kind).
    pub busy_since: f64,
    pub fg: Option<(Id, &'static str)>,
    /// Precompute bookkeeping: (intake id, key) of the understanding / proposal being computed.
    pub bg_understand: Option<(Id, usize)>,
    pub bg_propose: Option<(Id, String)>,
    /// Per-step timings this session (step, seconds) — shown in the inspector and the report.
    pub timings: Vec<(String, f64)>,
    /// Ending: the current theme card is being corrected ("Not quite").
    pub theme_edit: bool,
    /// Bridge: when the last adopt step ran (so each real step stays visible briefly).
    pub adopt_t: f64,
    // ---- sections (Part 4 navigation)
    /// Knowledge tab (see nav::KNOWLEDGE_TABS).
    pub knowledge_tab: usize,
    /// Roadmap grouped by theme instead of by phase.
    pub roadmap_by_theme: bool,
    pub new_theme: String,
    pub new_theme_lane: String,
    pub new_milestone: String,
    pub new_cut: String,
    /// AI Team: the selected role, the message being typed, and the running answer
    /// (conversation id, job, started (app clock), which AI, started (ms, for Undo)).
    pub agent: String,
    pub chat_input: String,
    pub chat_job: Option<(Id, crate::ai::AiJob)>,
    pub chat_started: f64,
    pub chat_via: String,
    pub chat_started_ms: i64,
    pub handoff_for: String,
    pub handoff_focus: String,
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
        self.resolve_ui.busy_since = self.now;
        self.resolve_ui.fg = None;
        self.resolve_ui.error.clear();
    }

    /// A foreground job the intake may give up on after 15 s (kind "next" falls back to the fixed question set).
    pub(super) fn spawn_job_kind(&mut self, label: &str, intake: &str, kind: &'static str, f: impl FnOnce() -> JobOut + Send + 'static) {
        self.spawn_job(label, f);
        self.resolve_ui.fg = Some((intake.to_string(), kind));
    }

    /// Work that runs while the person reads or types; its result is applied when it arrives.
    pub(super) fn spawn_bg(&mut self, label: &str, f: impl FnOnce() -> JobOut + Send + 'static) {
        let (tx, rx) = channel();
        let t0 = std::time::Instant::now();
        let lbl = label.to_string();
        std::thread::spawn(move || {
            let out = f();
            let _ = tx.send(JobOut::Timed(lbl, t0.elapsed().as_secs_f64(), Box::new(out)));
        });
        self.resolve_ui.bg.push((label.to_string(), rx));
    }

    /// Seconds the current foreground wait has lasted.
    pub fn wait_secs(&self) -> f64 {
        if self.resolve_ui.rx.is_some() { self.now - self.resolve_ui.busy_since } else { 0.0 }
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

    /// "Start something new with Resolve": the full-screen intake; Adopt creates a new workspace
    /// (named after the project) with the adopted project as its first project.
    pub fn start_something_new(&mut self) {
        let mut i = Intake::new();
        i.new_workspace = true;
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
        if fresh {
            self.start_intake();
            self.back.clear();
        }
    }

    /// Apply finished background work. Called every frame.
    pub fn resolve_poll(&mut self, ctx: &egui::Context) {
        // background work first: apply whatever has finished
        if !self.resolve_ui.bg.is_empty() {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
            let mut done = vec![];
            let mut keep = vec![];
            for (label, rx) in std::mem::take(&mut self.resolve_ui.bg) {
                match rx.try_recv() {
                    Ok(o) => done.push(o),
                    Err(std::sync::mpsc::TryRecvError::Empty) => keep.push((label, rx)),
                    Err(_) => {}
                }
            }
            self.resolve_ui.bg = keep;
            for o in done {
                self.apply_job(o);
            }
        }
        let Some(rx) = &self.resolve_ui.rx else { return };
        ctx.request_repaint_after(std::time::Duration::from_millis(150));
        let out = match rx.try_recv() {
            Ok(o) => o,
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                // no wait passes 15 s: the next question falls back to the fixed set
                if let Some((iid, "next")) = self.resolve_ui.fg.clone() {
                    if self.wait_secs() > 15.0 {
                        self.resolve_ui.rx = None;
                        self.resolve_ui.busy.clear();
                        self.resolve_ui.fg = None;
                        self.resolve_ui.timings.push(("next question (timed out)".into(), 15.0));
                        if let Some(i) = self.intake_mut(&iid) {
                            let out = crate::resolve::journey::AnswerOut { next: crate::resolve::journey::offline_next(i), note: "The AI took longer than 15 seconds, so Resolve asked this from its fixed question set.".into(), ..Default::default() };
                            let none_left = out.next.is_none();
                            crate::resolve::journey::apply_answer(i, out);
                            if none_left {
                                i.suggest_stop = true;
                            }
                        }
                        self.resolve_ui.focus_qa = None;
                        self.touch();
                    }
                }
                return;
            }
            Err(_) => {
                self.resolve_ui.rx = None;
                self.resolve_ui.busy.clear();
                self.resolve_ui.error = "The AI request stopped unexpectedly".into();
                return;
            }
        };
        let label = std::mem::take(&mut self.resolve_ui.busy);
        let secs = self.wait_secs().max(self.now - self.resolve_ui.busy_since);
        self.resolve_ui.rx = None;
        self.resolve_ui.fg = None;
        self.resolve_ui.timings.push((label, secs));
        self.apply_job(out);
    }

    fn apply_job(&mut self, out: JobOut) {
        match out {
            JobOut::Timed(label, secs, inner) => {
                self.resolve_ui.timings.push((format!("{label} (background)"), secs));
                self.apply_job(*inner);
            }
            JobOut::Answer(id, out) => {
                if let Some(i) = self.intake_mut(&id) {
                    crate::resolve::journey::apply_answer(i, out);
                    self.resolve_ui.focus_qa = None;
                }
                self.touch();
            }
            JobOut::Understood(id, n, r, calls) => {
                self.temp.remove("wait:understanding");
                if self.resolve_ui.bg_understand.as_ref().map_or(false, |(x, _)| x == &id) {
                    self.resolve_ui.bg_understand = None;
                }
                if let Some(i) = self.intake_mut(&id) {
                    crate::resolve::journey::keep_trace(i, calls);
                    // never overwrite cards the person is editing
                    if !i.editing {
                        match r {
                            Ok(u) => {
                                i.understanding = Some(u);
                                i.understood_n = n;
                                i.last_error.clear();
                            }
                            Err(e) => i.last_error = e,
                        }
                    }
                    i.touch();
                }
                self.touch();
            }
            JobOut::Proposed(id, section, key, r, calls) => {
                self.temp.remove("wait:proposal");
                if self.resolve_ui.bg_propose.as_ref().map_or(false, |(x, _)| x == &id) {
                    self.resolve_ui.bg_propose = None;
                }
                if let Some(i) = self.intake_mut(&id) {
                    crate::resolve::journey::keep_trace(i, calls);
                    match r {
                        Ok(fresh) => {
                            match (section.as_deref(), i.proposal.as_mut()) {
                                (Some(s), Some(p)) if s != "all" => crate::resolve::journey::merge_section(p, fresh, s),
                                _ => {
                                    i.proposal = Some(fresh);
                                    i.proposed_from = key;
                                    i.ending = 0;
                                    i.theme_ix = 0;
                                }
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
