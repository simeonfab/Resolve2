//! Deterministic project assessment ("What matters now?"): what is decided, uncertain, blocked
//! and what should happen next — computed from the real project data, no model call.
//! Rules (see docs/RESOLVE_SPEC.md §Orchestration):
//! * The win / next move is kept until it's done, then re-picked by priority, then roadmap
//!   position (Now → Next → Later), then due date (ResolvePM `win` rule).
//! * A missing/unconfirmed outcome is surfaced first (ResolvePM Gate 1: the one legitimate stop).
//! * Blockers before everything else; open decisions and untested hypotheses are surfaced, but
//!   nothing is forced into a fixed sequence — the user can work on any item in any order.

use super::{Kind, ProjectContext, Record};
use crate::model::*;

#[derive(Default, Debug, Clone)]
pub struct Assessment {
    pub win: Option<Id>,
    pub done: usize,
    pub total: usize,
    pub overdue: Vec<Id>,
    pub decided: Vec<Id>,
    pub open_decisions: Vec<Id>,
    pub uncertain: Vec<Id>,
    pub blocked: Vec<Id>,
    pub risks: Vec<Id>,
    pub experiments: Vec<Id>,
    /// (reason, target id) in the order they should be looked at.
    pub next: Vec<(String, Option<Id>)>,
}

fn section_rank(s: &str) -> i32 {
    match s {
        "Now" => 0,
        "Next" => 1,
        "Later" => 2,
        "" => 3,
        _ => 3,
    }
}

/// The ResolvePM win rule. Keeps the current win while it's open.
pub fn pick_win(d: &WorkspaceData, ctx: &ProjectContext) -> Option<Id> {
    if let Some(w) = &ctx.win_task_id {
        if d.task(w).map_or(false, |t| !t.completed) {
            return Some(w.clone());
        }
    }
    let mut open: Vec<&Task> = d.tasks.iter().filter(|t| !t.completed && t.parent_id.is_none() && t.project_id.as_deref() == Some(&ctx.project_id)).collect();
    open.sort_by_key(|t| (t.priority, section_rank(&t.section), t.due.unwrap_or(chrono::NaiveDate::MAX), t.order));
    open.first().map(|t| t.id.clone())
}

pub fn records_for<'a>(d: &'a WorkspaceData, project_id: &str) -> Vec<&'a Record> {
    d.records.iter().filter(|r| r.project_id.as_deref() == Some(project_id)).collect()
}

pub fn assess(d: &WorkspaceData, ctx: &ProjectContext) -> Assessment {
    let mut a = Assessment::default();
    let pid = ctx.project_id.as_str();
    let tasks: Vec<&Task> = d.tasks.iter().filter(|t| t.project_id.as_deref() == Some(pid) && t.parent_id.is_none()).collect();
    a.total = tasks.len();
    a.done = tasks.iter().filter(|t| t.completed).count();
    a.overdue = tasks.iter().filter(|t| t.is_overdue()).map(|t| t.id.clone()).collect();
    a.win = pick_win(d, ctx);
    let recs = records_for(d, pid);
    for r in &recs {
        match r.kind {
            Kind::Decision if r.status == "Decided" => a.decided.push(r.id.clone()),
            Kind::Decision if r.is_open() => a.open_decisions.push(r.id.clone()),
            Kind::Question | Kind::Hypothesis | Kind::Assumption if r.is_open() => a.uncertain.push(r.id.clone()),
            Kind::Blocker | Kind::Dependency if r.is_open() => a.blocked.push(r.id.clone()),
            Kind::Risk if r.is_open() => a.risks.push(r.id.clone()),
            Kind::Experiment if r.is_open() => a.experiments.push(r.id.clone()),
            _ => {}
        }
    }
    let title = |id: &Id| recs.iter().find(|r| &r.id == id).map(|r| r.title.clone()).unwrap_or_default();
    // 1. blockers
    for b in &a.blocked {
        a.next.push((format!("Unblock: {}", title(b)), Some(b.clone())));
    }
    // 2. outcome
    if ctx.outcome.trim().is_empty() {
        a.next.push(("Say what changes, and for whom, if this works (the project has no outcome yet)".into(), None));
    } else if !ctx.outcome_confirmed {
        a.next.push(("Confirm the outcome: who is better off, and how would you know?".into(), None));
    }
    // 3. the win
    if let Some(w) = &a.win {
        if let Some(t) = d.task(w) {
            a.next.push((format!("Do this next: {}", t.title), Some(w.clone())));
        }
    } else if a.total == 0 {
        a.next.push(("Plan the first piece of work (there are no tasks yet)".into(), None));
    }
    // 4. decisions
    for x in &a.open_decisions {
        a.next.push((format!("Decide: {}", title(x)), Some(x.clone())));
    }
    // 5. running experiments need results; planned ones need starting
    for x in &a.experiments {
        let r = recs.iter().find(|r| &r.id == x).unwrap();
        let verb = if r.status == "Running" { "Record what you observed" } else { "Run" };
        a.next.push((format!("{verb}: {}", r.title), Some(x.clone())));
    }
    // 6. unresolved hypotheses without any experiment (E0 can branch from any node)
    for r in recs.iter().filter(|r| r.kind == Kind::Hypothesis && r.is_open()) {
        let tested = recs.iter().any(|e| e.kind == Kind::Experiment && e.parent_id.as_deref() == Some(&r.id));
        if !tested {
            a.next.push((format!("Design a test for: {}", r.title), Some(r.id.clone())));
        }
    }
    // 7. overdue
    for x in &a.overdue {
        if Some(x) != a.win.as_ref() {
            if let Some(t) = d.task(x) {
                a.next.push((format!("Overdue: {}", t.title), Some(x.clone())));
            }
        }
    }
    a
}

/// Plain-text summary used by MCP and as context for roles.
pub fn summary_text(d: &WorkspaceData, ctx: &ProjectContext) -> String {
    let a = assess(d, ctx);
    let mut s = format!("Progress: {} of {} tasks done.\n", a.done, a.total);
    if !a.next.is_empty() {
        s.push_str("What matters now:\n");
        for (i, (r, _)) in a.next.iter().take(8).enumerate() {
            s.push_str(&format!("{}. {r}\n", i + 1));
        }
    }
    s.push_str(&format!("Decided: {} · open decisions: {} · uncertain: {} · blocked: {} · open risks: {} · experiments in flight: {}\n", a.decided.len(), a.open_decisions.len(), a.uncertain.len(), a.blocked.len(), a.risks.len(), a.experiments.len()));
    s
}

/// Hypothesis tree in display order: (depth, record id), recursive, cycle-safe.
pub fn hypothesis_tree(d: &WorkspaceData, project_id: &str) -> Vec<(usize, Id)> {
    let recs: Vec<&Record> = records_for(d, project_id).into_iter().filter(|r| matches!(r.kind, Kind::Hypothesis | Kind::Experiment | Kind::Evidence)).collect();
    let ids: std::collections::HashSet<&str> = recs.iter().map(|r| r.id.as_str()).collect();
    let mut out = vec![];
    let mut seen = std::collections::HashSet::new();
    fn walk<'a>(recs: &[&'a Record], parent: Option<&str>, depth: usize, out: &mut Vec<(usize, Id)>, seen: &mut std::collections::HashSet<String>, ids: &std::collections::HashSet<&str>) {
        if depth > 12 {
            return;
        }
        let mut kids: Vec<&&Record> = recs
            .iter()
            .filter(|r| match parent {
                None => r.parent_id.as_deref().map_or(true, |p| !ids.contains(p)) && r.kind == Kind::Hypothesis,
                Some(p) => r.parent_id.as_deref() == Some(p),
            })
            .collect();
        kids.sort_by_key(|r| (r.kind != Kind::Hypothesis, r.kind != Kind::Experiment, r.created));
        for r in kids {
            if !seen.insert(r.id.clone()) {
                continue;
            }
            out.push((depth, r.id.clone()));
            walk(recs, Some(&r.id), depth + 1, out, seen, ids);
        }
    }
    walk(&recs, None, 0, &mut out, &mut seen, &ids);
    out
}
