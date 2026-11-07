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
}
