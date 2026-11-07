//! Phase 1 checks (the shell). First: no em dash in any string the app shows.
//!
//! Every string literal in src/ is read (comments are skipped). A literal holding an em dash
//! fails unless it is on the short list below of text that never reaches the screen: prompts
//! sent to an AI model, MCP descriptions read by Claude or ChatGPT, terminal output, and word
//! lists used to read what people type. Each entry says why.

use crate::selftest::T;

const EM: char = '\u{2014}';

/// (file, the literal or a piece of it, whole literal only?, why it is not on screen)
const NOT_UI: &[(&str, &str, bool, &str)] = &[
    ("ai.rs", "Organise the to-dos", false, "prompt sent to the AI model"),
    ("main.rs", "words, {:.1}s)", false, "terminal output of --import"),
    ("mcp.rs", "Fields by kind", false, "MCP tool description, read by Claude or ChatGPT"),
    ("mcp.rs", "optionally nested under a parent", false, "MCP tool description, read by Claude or ChatGPT"),
    ("mcp.rs", "sensible column types", false, "MCP tool description, read by Claude or ChatGPT"),
    ("mcp.rs", "Page not found", false, "MCP error, read by Claude or ChatGPT"),
    ("mcp.rs", "Couldn't read the start", false, "MCP error, read by Claude or ChatGPT"),
    ("mcp.rs", "Resolve 2 \u{2014} {}", true, "MCP server name, shown inside Claude or ChatGPT"),
    ("mcp.rs", "Weekly review \u{2014} <today's date>", false, "MCP prompt, read by Claude or ChatGPT"),
    ("resolve/files.rs", "[Attached file", false, "file text handed to the AI model"),
    ("resolve/context.rs", "\u{2014}", true, "project context handed to the AI model"),
    ("resolve/context.rs", "FENCED PROJECT CONTEXT", false, "project context handed to the AI model"),
    ("resolve/journey.rs", "\u{2014}", true, "words that end a title, used to shorten what people typed"),
    ("resolve/journey.rs", "Treat what they want as H0", false, "intake prompt sent to the AI model"),
    ("resolve/journey.rs", "Coverage targets", false, "intake prompt sent to the AI model"),
    ("resolve/journey.rs", "CONFIRMED UNDERSTANDING", false, "proposal prompt sent to the AI model"),
    ("resolve/journey.rs", "was already asked", false, "retry note sent back to the AI model"),
    ("resolve/journey.rs", "is generic", false, "retry note sent back to the AI model"),
    ("resolve/roles.rs", "orchestration router", false, "prompt sent to the AI model"),
    ("resolve/roles.rs", "speaking directly to the founder", false, "prompt sent to the AI model"),
    ("resolve/roles.rs", "An experiment E(Hx)", false, "prompt sent to the AI model"),
    ("resolve/roles.rs", " \u{2014} {}", true, "project context handed to the AI model"),
    ("ui/team.rs", "(id {})", false, "record list handed to the AI model"),
];

/// Every string or char literal in a Rust source holding an em dash, with its line number.
/// Skips line and block comments; reads normal, byte and raw strings (r"..", r#".."#).
pub fn em_dash_literals(text: &str) -> Vec<(usize, String)> {
    let c: Vec<char> = text.chars().collect();
    let (mut i, mut line, mut out) = (0usize, 1usize, vec![]);
    let ident = |ch: char| ch.is_alphanumeric() || ch == '_';
    while i < c.len() {
        let ch = c[i];
        if ch == '\n' {
            line += 1;
            i += 1;
        } else if ch == '/' && c.get(i + 1) == Some(&'/') {
            while i < c.len() && c[i] != '\n' {
                i += 1;
            }
        } else if ch == '/' && c.get(i + 1) == Some(&'*') {
            i += 2;
            while i < c.len() && !(c[i] == '*' && c.get(i + 1) == Some(&'/')) {
                if c[i] == '\n' {
                    line += 1;
                }
                i += 1;
            }
            i += 2;
        } else if ch == 'r' && (i == 0 || !ident(c[i - 1])) && matches!(c.get(i + 1), Some('"') | Some('#')) {
            // raw string: r, n hashes, quote ... quote, n hashes
            let mut j = i + 1;
            let mut hashes = 0;
            while c.get(j) == Some(&'#') {
                hashes += 1;
                j += 1;
            }
            if c.get(j) != Some(&'"') {
                i += 1;
                continue;
            }
            let (start, l0) = (j + 1, line);
            j += 1;
            loop {
                if j >= c.len() {
                    break;
                }
                if c[j] == '\n' {
                    line += 1;
                }
                if c[j] == '"' && (0..hashes).all(|k| c.get(j + 1 + k) == Some(&'#')) {
                    break;
                }
                j += 1;
            }
            let s: String = c[start..j.min(c.len())].iter().collect();
            if s.contains(EM) {
                out.push((l0, s));
            }
            i = j + 1 + hashes;
        } else if ch == '"' {
            let (start, l0) = (i + 1, line);
            let mut j = i + 1;
            while j < c.len() && c[j] != '"' {
                if c[j] == '\\' {
                    j += 1;
                }
                if j < c.len() && c[j] == '\n' {
                    line += 1;
                }
                j += 1;
            }
            let s: String = c[start..j.min(c.len())].iter().collect();
            if s.contains(EM) {
                out.push((l0, s));
            }
            i = j + 1;
        } else if ch == '\'' {
            // char literal ('x', '\n', '\'') or a lifetime ('a)
            if c.get(i + 1) == Some(&'\\') {
                let mut j = i + 2;
                while j < c.len() && c[j] != '\'' && c[j] != '\n' {
                    j += 1;
                }
                i = j + 1;
            } else if c.get(i + 2) == Some(&'\'') {
                if c[i + 1] == EM {
                    out.push((line, EM.to_string()));
                }
                i += 3;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    out
}

fn rs_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                rs_files(&p, out);
            } else if p.extension().map_or(false, |x| x == "rs") {
                out.push(p);
            }
        }
    }
}

pub fn run(t: &mut T) {
    println!("\n-- Phase 1: the shell --");

    // the scanner itself: finds strings, skips comments, reads raw strings and char literals
    let sample = "// a \u{2014} comment\nlet a = \"plain\"; /* b \u{2014} */ let b = \"x \u{2014} y\";\nlet c = r#\"raw \u{2014}\"#; let d = '\u{2014}'; let e = '\"'; let f = \"ok\";";
    let found = em_dash_literals(sample);
    t.check("the em dash scanner finds strings, raw strings and chars, and skips comments", found.len() == 3 && found[0] == (2, "x \u{2014} y".to_string()) && found[1].0 == 3, format!("{found:?}"));

    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = vec![];
    rs_files(&src, &mut files);
    let mut bad = vec![];
    let mut allowed = 0;
    for f in &files {
        let rel = f.strip_prefix(&src).unwrap_or(f).to_string_lossy().replace('\\', "/");
        if rel.starts_with("selftest") {
            continue; // test text: never shown
        }
        let text = std::fs::read_to_string(f).unwrap_or_default();
        for (line, s) in em_dash_literals(&text) {
            if NOT_UI.iter().any(|(file, piece, whole, _)| rel == *file && if *whole { s == *piece } else { s.contains(piece) }) {
                allowed += 1;
            } else {
                let short: String = s.chars().take(70).collect();
                bad.push(format!("{rel}:{line} \"{short}\""));
            }
        }
    }
    t.check(&format!("no em dash in any string the app shows ({} files read; {allowed} prompt or parsing strings set aside)", files.len()), bad.is_empty() && files.len() > 30, bad.join("; "));
    // the tutorial is shown inside Help, so all of it is on screen
    let tut: Vec<usize> = include_str!("../TUTORIAL.md").lines().enumerate().filter(|(_, l)| l.contains(EM)).map(|(n, _)| n + 1).collect();
    t.check("no em dash in the tutorial shown in Help", tut.is_empty(), format!("TUTORIAL.md lines {tut:?}"));

    shell(t);
}

/// The shell (B-E24, B-E1, B-E2, B-E3, B-E17, B-E14): what can be checked without a window.
fn shell(t: &mut T) {
    use crate::ui::nav::{self, Section, Target, NAV, UTILITIES};
    use crate::ui::shell::{group_hits, next_type, split_type, with_type, SEARCH_TYPES};

    // B-E24: the renamed group and item, and the footer
    let items: Vec<&str> = NAV.iter().flat_map(|g| g.items.iter().map(|i| i.label)).collect();
    t.check("B-E24: the middle group is YOUR PROJECT and Knowledge is Knowledge base", NAV[1].label.to_uppercase() == "YOUR PROJECT" && items.contains(&"Knowledge base") && !items.contains(&"Knowledge"), format!("{} / {items:?}", NAV[1].label));
    let foot: Vec<&str> = UTILITIES.iter().map(|i| i.label).collect();
    t.check("the sidebar footer reads Search (Ctrl K), Settings, Help", foot == ["Search", "Settings", "Help"] && UTILITIES[0].hint == "Ctrl K", foot.join(", "));

    // B-E3: the Ask Resolve wording per screen, exactly as the brief's table
    let ask = |t: Target| NAV.iter().flat_map(|g| g.items.iter()).find(|i| i.target == t).map(|i| (i.ask.placeholder, i.ask.suggestions.to_vec()));
    let want: [(Target, &str, &[&str]); 6] = [
        (Target::Sec(Section::Overview), "Ask about this project", &["What matters most this week?", "What's still uncertain?", "Summarise where this project is"]),
        (Target::Sec(Section::Roadmap), "Ask about your roadmap", &["What's missing from this plan?", "Move a theme into Now", "Why is this sequenced this way?"]),
        (Target::Sec(Section::Frameworks), "Ask about your hypotheses and tests", &["Which hypothesis is weakest?", "Suggest a cheap test for H0", "What would change my mind?"]),
        (Target::Sec(Section::Knowledge), "Find or capture something you know", &["What evidence supports H0?", "Summarise the project brief", "Capture a research note"]),
        (Target::Sec(Section::Handoffs), "Ask about a handoff", &["What should the next handoff cover?", "Is anything blocking a handoff?"]),
        (Target::Calendar, "Ask about your schedule", &["Plan my day", "What's overdue?", "Find time for my next task"]),
    ];
    let wrong: Vec<String> = want.iter().filter(|(tg, ph, sg)| ask(*tg) != Some((*ph, sg.to_vec()))).map(|(tg, _, _)| format!("{tg:?}")).collect();
    let help = UTILITIES.iter().find(|i| i.target == Target::Help).map(|i| (i.ask.placeholder, i.ask.suggestions.to_vec()));
    t.check("B-E3: Ask Resolve placeholder and suggestions per screen match the brief", wrong.is_empty() && help == Some(("Ask Resolve, or capture a thought", vec!["What should I focus on right now?", "Capture a quick note"])), wrong.join(", "));
    let with_bar: Vec<bool> = NAV.iter().flat_map(|g| g.items.iter()).map(|i| nav::shows_ask_bar(Some(i.target))).collect();
    t.check("B-E3: the Ask bar is on every screen but AI Team", !nav::shows_ask_bar(Some(Target::Sec(Section::AiTeam))) && with_bar.iter().filter(|b| !**b).count() == 1, format!("{with_bar:?}"));

    // B-E1: collapse timing (labels 120 e-std, then width 248 ↔ 64 in 320 e-out), reduced motion
    let (w0, l0) = nav::sidebar_frame(0.0, true, false);
    let (w1, l1) = nav::sidebar_frame(120.0, true, false);
    let (w2, l2) = nav::sidebar_frame(440.0, true, false);
    t.check("B-E1: collapsing, the labels fade first, then the width goes 248 → 64", w0 == 248.0 && l0 == 1.0 && w1 == 248.0 && l1 < 0.01 && (w2 - 64.0).abs() < 0.01 && l2 < 0.01, format!("{w0} {l0} / {w1} {l1} / {w2} {l2}"));
    let (e1, el1) = nav::sidebar_frame(320.0, false, false);
    let (e2, el2) = nav::sidebar_frame(440.0, false, false);
    t.check("B-E1: expanding, the width comes back first, then the labels", (e1 - 248.0).abs() < 0.01 && el1 < 0.01 && (e2 - 248.0).abs() < 0.01 && el2 > 0.99, format!("{e1} {el1} / {e2} {el2}"));
    let (rw, rl) = nav::sidebar_frame(60.0, false, true);
    let (_, rl2) = nav::sidebar_frame(120.0, false, true);
    t.check("B-E1: with reduced motion the width jumps and the labels fade in 120 ms or less", rw == 248.0 && rl > 0.0 && rl < 1.0 && rl2 >= 1.0, format!("{rw} {rl} {rl2}"));
    let s: crate::store::WorkspaceSettings = serde_json::from_str("{}").unwrap_or_default();
    let mut s2 = s.clone();
    s2.sidebar_collapsed = true;
    let back: crate::store::WorkspaceSettings = serde_json::from_str(&serde_json::to_string(&s2).unwrap_or_default()).unwrap_or_default();
    t.check("B-E1: collapse is remembered per workspace (in its settings, off by default)", !s.sidebar_collapsed && back.sidebar_collapsed, "");

    // B-E2: "Continue setting up…" names the rail step the intake is on
    let mut i = crate::resolve::intake::Intake::new();
    let mut steps = vec![];
    for st in ["name", "bring", "questions", "confirm", "proposal"] {
        i.stage = st.into();
        steps.push(crate::ui::resolve::rail_step(&i));
    }
    t.check("B-E2: an unfinished intake shows the rail step it is on (1 to 4)", i.is_v2() && steps == [1, 2, 3, 3, 4], format!("{steps:?}"));

    // B-E17: Undo names what changed
    let one = crate::ui::undo_message(&["Completed \"Buy milk\"".to_string()], 0);
    let many = crate::ui::undo_message(&["AI: create task".to_string(), "AI: update page".to_string()], 1);
    t.check("B-E17: the Undo toast names what was undone", one == "Undone: Completed \"Buy milk\"" && many == "Undone: AI: create task and 1 more (1 edited since and left as they are)", format!("{one} / {many}"));

    // B-E14: search types (Tab steps through them) and grouped results
    let mut q = "payroll".to_string();
    let mut seen = vec![];
    for _ in 0..SEARCH_TYPES.len() {
        q = next_type(&q);
        seen.push(split_type(&q).0);
    }
    t.check("B-E14: Tab steps through task:, page:, project:, event:, db: and back to all, keeping the words", seen == [1, 2, 3, 4, 5, 0] && q == "payroll" && with_type("payroll", 1) == "task: payroll" && split_type("Task: payroll") == (1, "payroll".into()), format!("{seen:?} {q}"));
    let mut d = crate::model::WorkspaceData::default();
    crate::seed::seed(&mut d);
    let hits = crate::search::search(&d, "weekly", 30);
    let groups = group_hits(&hits);
    let flat: Vec<usize> = groups.iter().flat_map(|g| g.1.clone()).collect();
    let kinds: Vec<_> = groups.iter().map(|g| g.0).collect();
    let mut dedup = kinds.clone();
    dedup.dedup();
    t.check("B-E14: results come grouped by type, each result once", !hits.is_empty() && flat.len() == hits.len() && dedup.len() == kinds.len() && { let mut f = flat.clone(); f.sort(); f.dedup(); f.len() == hits.len() }, format!("{} hits in {} groups", hits.len(), groups.len()));
}
