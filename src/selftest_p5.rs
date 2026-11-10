//! Phase 5 checks (Settings and Help).

use crate::selftest::T;

pub fn run(t: &mut T) {
    println!("\n-- Phase 5: Settings and Help --");
    // C-E21: the four tabs, in the nav table
    t.check("C-E21: Settings has the four tabs, in order", crate::ui::nav::SETTINGS_TABS == ["AI and MCP", "Sync and phone", "Workspaces", "About"], format!("{:?}", crate::ui::nav::SETTINGS_TABS));
    // the Spending card's estimate: only pay-per-use providers cost anything
    let u = crate::ai::UsageTotals {
        calls: 3,
        input: 3_000_000,
        output: 300_000,
        by_model: vec![("anthropic / claude".into(), 1, 1_000_000, 100_000), ("claude-code / subscription".into(), 1, 1_000_000, 100_000), ("ollama / qwen3".into(), 1, 1_000_000, 100_000)],
    };
    let cost = crate::ai::estimated_cost(&u, 3.0, 15.0);
    t.check("C-E21: the estimate counts only API keys at your prices ($3 + $1.50 here); Claude Code and Ollama are $0", (cost - 4.5).abs() < 1e-9, cost);
    let none = crate::ai::estimated_cost(&crate::ai::UsageTotals::default(), 3.0, 15.0);
    t.check("C-E21: with nothing used the estimate is $0.00, never negative", format!("{none:.2}") == "0.00" && none.is_sign_positive(), none);
    // About shows the changelog with the same text as CHANGELOG.md
    let file = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("CHANGELOG.md")).unwrap_or_default();
    t.check("About's changelog is the same text as CHANGELOG.md, newest first", !file.is_empty() && crate::ui::settings_page::CHANGELOG == file && file.find("## 0.6.0").unwrap_or(usize::MAX) < file.find("## 0.5.1").unwrap_or(0), "");
    // C-E23: Help's Quick start, and the report's details
    let steps = crate::ui::help::quick_steps();
    t.check("C-E23: Quick start has the five numbered steps with their bold leads", steps.len() == 5 && steps[0].0 == "Connect your AI." && steps[4].1 == "Adopt builds it, then Open my project.", format!("{steps:?}"));
    let d = crate::ui::help::report_details("0.6.0", "Windows 11", "Claude Code", "save_record timeout");
    t.check("C-E23: the report's details are version, Windows, the AI and the last error, and nothing else", d == "Alpha 0.6.0 · Windows 11 · AI: Claude Code · last error: save_record timeout", &d);
    t.check("C-E23: with no error this session the report says so", crate::ui::help::report_details("0.6.0", "Windows 11", "Ollama", "").ends_with("last error: none this session"), "");
    // Open Resolve 2 with: an older settings file without the field still loads
    let s: Result<crate::store::AppSettings, _> = serde_json::from_str("{\"dark\": false}");
    t.check("Open Resolve 2 with: older settings load, starting with the last workspace used", s.map_or(false, |s| s.open_with.is_none()), "");
}
