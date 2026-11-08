//! Phase 3 checks (the intake redesign): AI limits, progress events, and the intake's own rules.

use crate::selftest::T;

pub fn run(t: &mut T) {
    println!("\n-- Phase 3: the intake --");
    use crate::ai::{is_budget, is_limit};
    // E28: what counts as an AI limit (screen 12) and what is an ordinary failure (Try again)
    let budget = "This month's AI budget ($5.00) has been reached (≈ $5.10). Raise it in Settings → AI, or continue manually.";
    t.check("E28: the monthly budget stop is an AI limit", is_limit(budget) && is_budget(budget), "");
    t.check(
        "E28: API rate limits and quota are AI limits",
        is_limit("The AI service said (rate_limit_error): Number of request tokens has exceeded your per-minute rate limit") && is_limit("The AI service said (insufficient_quota): You exceeded your current quota") && is_limit("The AI server answered 429: slow down"),
        "",
    );
    t.check("E28: Claude Code's own cap wording is an AI limit", is_limit("Claude Code: You've hit your session limit · resets 3pm") && is_limit("Claude Code: You've reached your Fable limit."), "");
    t.check(
        "E28: a broken answer or an overloaded server is not a limit",
        !is_limit("The AI's answer didn't come back in the right shape, even after one retry (The model didn't return JSON).") && !is_limit("The AI service said (overloaded_error): Overloaded") && !is_budget("The AI service said (rate_limit_error): slow down"),
        "",
    );
    // E5: progress events are per intake, noted once, and cleared for the next wait
    use crate::resolve::journey::{clear_progress, has_progress, note_progress};
    note_progress("selftest-a", "sorted");
    note_progress("selftest-a", "sorted");
    let a = has_progress("selftest-a", "sorted") && !has_progress("selftest-b", "sorted") && !has_progress("selftest-a", "chosen");
    clear_progress("selftest-a", &["sorted"]);
    t.check("E5: progress events belong to one intake and clear for the next wait", a && !has_progress("selftest-a", "sorted"), "");

    // E27: the first move is task 1 of week 1, and it sits in the first Now theme
    use crate::resolve::journey::{adopt_all, first_week_ix};
    let mut d = crate::model::WorkspaceData::default();
    let iid = crate::seed::intake_sample(&mut d);
    let p = d.intakes.iter().find(|i| i.id == iid).and_then(|i| i.proposal.clone()).unwrap_or_default();
    let week = first_week_ix(&p);
    let first_now = p.roadmap.iter().position(|r| r.phase == "Now");
    let pid = adopt_all(&mut d, &iid).unwrap_or_default();
    let pinned = d.contexts.iter().find(|c| c.project_id == pid).and_then(|c| c.win_task_id.clone()).and_then(|t| d.task(&t).map(|x| x.title.clone())).unwrap_or_default();
    let task1 = week.first().and_then(|&k| p.tasks.get(k)).map(|t| t.title.clone()).unwrap_or_default();
    t.check(
        "E27: the first move Adopt pins is task 1 of the first week, in the first Now theme",
        !pinned.is_empty() && pinned == task1 && week.first().and_then(|&k| p.tasks[k].parent) == first_now,
        format!("pinned {pinned:?}, week 1 task 1 {task1:?}"),
    );
    // listed out of order, a Next task first: week 1 still starts in the first Now theme
    let mut q = p.clone();
    let first_next = q.tasks.iter().position(|t| t.phase == "Next").unwrap_or(0);
    q.tasks.rotate_left(first_next);
    let k = first_week_ix(&q).first().copied();
    t.check("E27: a Next task listed first doesn't become the first move", k.and_then(|k| q.tasks[k].parent) == first_now && k.map_or(false, |k| q.tasks[k].phase == "Now"), format!("{:?}", k.map(|k| q.tasks[k].title.clone())));
    // E22: the pull-in prompt asks 11 questions, the last six covering the remaining targets
    let prompt = crate::resolve::intake::IMPORT_PROMPT;
    t.check("E22: the pull-in prompt has 11 numbered questions and says \"Not covered\" instead of guessing", (1..=11).all(|n| prompt.contains(&format!("\n{n}. "))) && prompt.contains("\"Not covered\"") && !prompt.contains('—'), "");
}
