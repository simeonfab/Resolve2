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
}
