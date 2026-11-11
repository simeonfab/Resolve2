//! Test Intake: sample scenarios, saved answer sets, branching, export and promotion.
//! The lab runs as its own process with its own data folder (%APPDATA%\Resolve2-TestLab, never
//! served over MCP); "Promote to real project" hands one run to the real data folder through a
//! separate `resolve2 --promote` process.

use super::intake::Intake;
use crate::model::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

const BUILT_IN: &str = include_str!("../../resolveos/intake-scenarios.json");

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Scenario {
    pub name: String,
    pub opening: String,
    /// target key → answer ("followup" answers any follow-up question).
    pub answers: BTreeMap<String, String>,
    #[serde(default)]
    pub saved: bool,
    /// Careers scenarios run the Careers intake.
    #[serde(default)]
    pub mode: Mode,
    /// A test CV attached on 02b (Careers).
    #[serde(default)]
    pub cv: String,
}

pub fn default_root() -> PathBuf {
    std::env::var("APPDATA").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir()).join("Resolve2-TestLab")
}

fn saved_path() -> PathBuf {
    crate::store::root().join("saved-scenarios.json")
}

pub fn scenarios() -> Vec<Scenario> {
    let mut v: Vec<Scenario> = serde_json::from_str(BUILT_IN).unwrap_or_default();
    let saved: Vec<Scenario> = std::fs::read_to_string(saved_path()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
    v.extend(saved);
    v
}

pub fn scenario(name: &str) -> Option<Scenario> {
    scenarios().into_iter().find(|s| s.name == name)
}

/// Save a run's answers as a reusable scenario (for replay).
pub fn save_answers(i: &Intake, name: &str) -> Result<(), String> {
    let mut answers = BTreeMap::new();
    for q in &i.qas {
        if !q.answer.trim().is_empty() {
            answers.entry(q.target.clone()).or_insert_with(|| q.answer.trim().to_string());
        }
    }
    let mut saved: Vec<Scenario> = std::fs::read_to_string(saved_path()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
    saved.retain(|s| s.name != name);
    saved.push(Scenario { name: name.to_string(), opening: i.input.clone(), answers, saved: true, mode: i.mode, cv: i.attachments.first().map(|a| a.text.clone()).unwrap_or_default() });
    crate::store::write_json_atomic(&saved_path(), &saved).map_err(|e| e.to_string())
}

/// A new run. `provider` is "provider|model|url", "offline", or "" for the app default.
pub fn new_run(scn: Option<&Scenario>, provider: &str) -> Intake {
    let mut i = Intake::new();
    i.name = "Tester".into();
    i.stage = "bring".into();
    i.provider = provider.to_string();
    if let Some(s) = scn {
        i.scenario = s.name.clone();
        i.input = s.opening.clone();
        if s.mode == Mode::Careers {
            i.mode = Mode::Careers;
            i.situation = "job_search".into();
            if !s.cv.trim().is_empty() {
                i.attachments.push(super::intake::Attachment { name: "test-cv.md".into(), text: s.cv.clone(), place: "cv".into(), how: "you".into(), why: String::new(), unreadable: false });
            }
            i.ai_covered = super::careers::covered_by_bring(&i.input);
        }
    }
    i
}

/// The scenario's answer for a question (by target; follow-ups use "followup").
pub fn answer_for(scn: &Scenario, target: &str) -> Option<String> {
    scn.answers.get(target).or_else(|| scn.answers.get("followup")).cloned()
}

/// A copy of `i` up to and including question `k`, to try another path from there.
pub fn branch(i: &Intake, k: usize) -> Intake {
    let mut b = i.clone();
    b.id = new_id();
    b.qas.truncate(k + 1);
    let keep: Vec<Id> = b.qas.iter().map(|q| q.id.clone()).collect();
    b.heard.retain(|h| h.from == "input" || keep.contains(&h.from));
    b.cursor = k;
    b.stage = "questions".into();
    b.status = "in_progress".into();
    b.understanding = None;
    b.outcome_confirmed = false;
    b.proposal = None;
    b.project_id = None;
    b.suggest_stop = false;
    b.branched_from = Some((i.id.clone(), k));
    b.created = now_ms();
    b.updated = now_ms();
    b
}

/// Write `<root>/exports/run-<id>.md` and `.json`. Returns the two paths.
pub fn export(i: &Intake) -> Result<(PathBuf, PathBuf), String> {
    let dir = crate::store::root().join("exports");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let stem = format!("run-{}-{}", crate::store::sanitize(&i.title()).replace(' ', "-").chars().take(40).collect::<String>(), &i.id[..6.min(i.id.len())]);
    let md = dir.join(format!("{stem}.md"));
    let js = dir.join(format!("{stem}.json"));
    std::fs::write(&md, super::journey::export_markdown(i)).map_err(|e| e.to_string())?;
    crate::store::write_json_atomic(&js, i).map_err(|e| e.to_string())?;
    Ok((md, js))
}

/// Drive a whole run without the window (used by `resolve2 --testlab --lab-run NAME`): answer each
/// question from the scenario, stop when Resolve says coverage is enough (or after `max_q`), confirm
/// the understanding, generate the proposal, and optionally adopt. Same code paths as the screens.
pub fn headless_run(scn: &Scenario, provider: &str, max_q: usize, adopt: bool, log: &mut dyn FnMut(&str)) -> Intake {
    use super::journey;
    let mut i = new_run(Some(scn), provider);
    i.stage = "questions".into();
    let cfg = journey::cfg_for(&i);
    log(&format!("provider: {}", cfg.as_ref().map(|c| format!("{} / {}", c.provider, c.model)).unwrap_or_else(|| "no AI (fixed questions)".into())));
    let out = journey::run_answer(cfg.clone(), &i, None, true);
    journey::apply_answer(&mut i, out);
    for _ in 0..max_q {
        let Some(q) = i.qas.get(i.cursor).cloned() else { break };
        let a = answer_for(scn, &q.target).unwrap_or_default();
        log(&format!("Q{} [{}] {}\n    why: {}\n    A: {}", i.cursor + 1, q.target, q.question, q.why, a));
        i.qas[i.cursor].answer = a;
        let before = i.qas.len();
        let (t0, n0) = (std::time::Instant::now(), i.trace.len());
        let out = journey::run_answer(cfg.clone(), &i, Some(&q.id), true);
        journey::apply_answer(&mut i, out);
        log_calls(log, &i, n0, t0);
        if !i.last_error.is_empty() {
            log(&format!("    note: {}", i.last_error));
        }
        if i.suggest_stop || i.qas.len() == before {
            log("Resolve suggests stopping: coverage is good enough.");
            break;
        }
    }
    for h in &i.heard {
        log(&format!("heard ({}) {}", h.kind, h.text));
    }
    if i.mode == Mode::Careers {
        return careers_rest(i, cfg, log);
    }
    if std::env::args().any(|a| a == "questions") {
        // --stop-at questions: leave the run on its last question
        i.stage = "questions".into();
        i.touch();
        return i;
    }
    i.stage = "confirm".into();
    i.last_error.clear();
    let (t0, n0) = (std::time::Instant::now(), i.trace.len());
    let (u, calls) = journey::run_understanding(cfg.clone(), &i);
    journey::keep_trace(&mut i, calls);
    log_calls(log, &i, n0, t0);
    i.understanding = u.ok();
    // same freshness key the app uses, so opening the run doesn't redo the understanding
    i.understood_n = i.qas.iter().filter(|q| !q.answer.trim().is_empty()).count() + usize::from(!i.input.trim().is_empty());
    i.outcome_confirmed = true;
    if let Some(u) = &i.understanding {
        log(&format!("understood: project={} | outcome={}", u.project, u.outcome));
    }
    if std::env::args().any(|a| a == "confirm") {
        i.outcome_confirmed = false;
        i.touch();
        return i;
    }
    i.stage = "proposal".into();
    let (t0, n0) = (std::time::Instant::now(), i.trace.len());
    let (p, calls) = journey::run_proposal(cfg, &i, None);
    journey::keep_trace(&mut i, calls);
    log_calls(log, &i, n0, t0);
    match p {
        Ok(mut p) => {
            journey::sanitise(&mut p, &i.clone());
            log(&format!("proposal: {} ({})", p.name, p.summary));
            i.proposal = Some(p);
        }
        Err(e) => {
            log(&format!("proposal failed: {e}"));
            i.last_error = e;
        }
    }
    let _ = adopt;
    i.touch();
    i
}

/// The Careers run after the questions: the 04 cards, the kit, Rebuild my CV (the same calls the
/// screens make).
fn careers_rest(mut i: Intake, cfg: Option<crate::ai::AiConfig>, log: &mut dyn FnMut(&str)) -> Intake {
    use super::{careers, journey};
    log(&format!("skipped (02b answered them): {}", careers::skipped(&i)));
    i.stage = "confirm".into();
    let (t0, n0) = (std::time::Instant::now(), i.trace.len());
    let (c, calls) = careers::run_cards(cfg.clone(), &i);
    journey::keep_trace(&mut i, calls);
    log_calls(log, &i, n0, t0);
    let c = match c {
        Ok(c) => c,
        Err(e) => {
            log(&format!("cards failed: {e}"));
            i.last_error = e;
            return i;
        }
    };
    log(&format!("04 The job you want: {}
04 Your hard gates: {}
04 Your preferences: {}
04 What a win looks like: {}
04 Strengths: {}
04 Gaps: {}
04 What to fix: {}
04 broad: {}", c.job, c.hard_gates, c.preferences, c.win, c.strengths, c.gaps, c.fix, c.broad));
    let mut car = careers::Career { cards: Some(c), ..Default::default() };
    car.cards_n = i.qas.iter().filter(|q| !q.answer.trim().is_empty()).count() + 1;
    i.career = Some(car);
    i.stage = "proposal".into();
    let (t0, n0) = (std::time::Instant::now(), i.trace.len());
    let (k, calls) = careers::run_kit(cfg.clone(), &i);
    journey::keep_trace(&mut i, calls);
    log_calls(log, &i, n0, t0);
    match k {
        Ok(k) => {
            log(&format!("06 Do this first: {} (why: {})", k.do_first, k.why));
            for g in &k.gates {
                log(&format!("07 {}: {} (why: {})", if g.hard { "Hard gate" } else { "Preference" }, g.rule, g.why));
            }
            for w in &k.week {
                log(&format!("08 {} · {}", w.title, w.day));
            }
            log(&format!("09 Search name: {}", k.search_name));
            for (t, l) in careers::kit_lines(&k) {
                log(&format!("09 {t}: {l}"));
            }
            for t in &k.templates {
                log(&format!("   template {}{}: suits {} | headline {}", t.name, if t.default { " (default)" } else { "" }, t.suits, t.headline));
            }
            for e in &k.evidence {
                log(&format!("   proof: {} [{}] {} | {}", e.point, e.theme, if e.number.is_empty() { "needs a number".to_string() } else { e.number.clone() }, e.role));
            }
            for l in &k.linkedin {
                log(&format!("   LinkedIn {}: {}", l.area, l.fix));
            }
            let key = i.career.as_ref().map(|c| c.cards_key()).unwrap_or_default();
            if let Some(c) = i.career.as_mut() {
                c.kit = Some(k);
                c.kit_from = key;
            }
        }
        Err(e) => {
            log(&format!("kit failed: {e}"));
            i.last_error = e;
            return i;
        }
    }
    if let Some(c) = cfg.as_ref() {
        let t0 = std::time::Instant::now();
        let n0 = i.trace.len();
        let _ = crate::ai::take_trace();
        let r = careers::rebuild_cv(c, &i);
        journey::keep_trace(&mut i, crate::ai::take_trace());
        log_calls(log, &i, n0, t0);
        match r {
            Ok(cv) => {
                log(&format!("Rebuild my CV:\n{cv}"));
                if let Some(car) = i.career.as_mut() {
                    car.cv = cv;
                }
            }
            Err(e) => log(&format!("Rebuild my CV failed: {e}")),
        }
    }
    i.ending = 3;
    i.touch();
    i
}

/// Timing line for one step: wall time plus each model call it made (parallel calls overlap).
fn log_calls(log: &mut dyn FnMut(&str), i: &Intake, from: usize, t0: std::time::Instant) {
    let calls: Vec<String> = i.trace.iter().skip(from).map(|t| format!("{} {:.1}s{}", t.purpose, t.ms as f64 / 1000.0, if t.ok { "" } else { " (failed)" })).collect();
    log(&format!("    time: {:.1}s [{}]", t0.elapsed().as_secs_f64(), calls.join(", ")));
}

/// Run by `resolve2 --promote FILE [--workspace NAME]` in the REAL data folder: adopt the run's
/// proposal into a real workspace. Returns (workspace name, project id).
pub fn promote_file(path: &str, workspace: Option<&str>) -> Result<(String, Id), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("Can't read {path}: {e}"))?;
    let mut i: Intake = serde_json::from_str(&text).map_err(|e| format!("Not an intake run: {e}"))?;
    if i.proposal.is_none() {
        return Err("This run has no proposal yet".into());
    }
    let reg = crate::store::Registry::load();
    let meta = workspace
        .and_then(|w| reg.find(w).cloned())
        .or_else(|| reg.settings.last_workspace.as_deref().and_then(|w| reg.find(w).cloned()))
        .or_else(|| reg.find("Personal").cloned())
        .ok_or("No workspace to promote into")?;
    let mut store = crate::store::Store::open(meta.clone());
    // a promoted run is a fresh intake in the real workspace, marked as coming from the lab
    i.id = new_id();
    i.project_id = None;
    i.status = "in_progress".into();
    i.stage = "bridge".into();
    i.scenario = format!("promoted from Test Intake{}", if i.scenario.is_empty() { String::new() } else { format!(" ({})", i.scenario) });
    let iid = i.id.clone();
    store.data.intakes.push(i);
    let mut pid = String::new();
    let name = store.data.intakes.last().and_then(|x| x.proposal.as_ref()).map(|p| p.name.clone()).unwrap_or_default();
    let cs = store.data.record_change(&format!("Promoted \"{name}\" from Test Intake"), "adopt", |d| {
        if let Ok(p) = super::journey::adopt_all(d, &iid) {
            pid = p;
        }
    });
    if pid.is_empty() || cs.is_none() {
        return Err("Couldn't create the project".into());
    }
    store.save()?;
    Ok((meta.name, pid))
}
