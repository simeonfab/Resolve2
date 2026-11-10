//! Careers Step 1 checks: the modes foundation. Every existing workspace loads as PM with
//! nothing changed; a Careers workspace keeps its mode; each fixed table is picked by the mode.

use crate::model::*;
use crate::selftest::T;
use std::path::{Path, PathBuf};

/// Every `workspaces/<id>/data.json` under `root`, a few folders down.
fn workspace_files(root: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(root) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().map_or(false, |n| n == "workspaces") {
                if let Ok(ws) = std::fs::read_dir(&p) {
                    for w in ws.flatten() {
                        let f = w.path().join("data.json");
                        if f.is_file() {
                            out.push(f);
                        }
                    }
                }
            } else if depth > 0 {
                workspace_files(&p, depth - 1, out);
            }
        }
    }
}

/// Does this JSON (a workspace) carry a "mode" anywhere a mode lives: the top, or an intake?
fn writes_mode(v: &serde_json::Value) -> bool {
    v.get("mode").is_some() || v.get("intakes").and_then(|x| x.as_array()).map_or(false, |a| a.iter().any(|i| i.get("mode").is_some()))
}

/// Every sidebar label with its Ask wording, to compare two tables.
fn labels(g: &[crate::ui::nav::NavGroup]) -> Vec<String> {
    g.iter().flat_map(|g| std::iter::once(g.label.to_string()).chain(g.items.iter().map(|i| format!("{} | {} | {:?}", i.label, i.ask.placeholder, i.ask.suggestions)))).collect()
}

pub fn run(t: &mut T) {
    use crate::resolve::journey::{self, MAX_QUESTIONS};
    use crate::resolve::team;
    use crate::ui::{dashboard, help, nav};
    println!("\n-- Careers Step 1: modes --");

    // 1. every existing workspace loads as PM, with nothing changed
    let old = "{\"version\": 3, \"projects\": [], \"intakes\": [{\"id\": \"i1\", \"name\": \"Sam\", \"created\": 1, \"updated\": 1}]}";
    let d: WorkspaceData = serde_json::from_str(old).unwrap_or_default();
    t.check("a workspace saved before modes loads as PM, and its intakes too", d.mode == Mode::Pm && d.intakes.iter().all(|i| i.mode == Mode::Pm) && d.intakes.len() == 1, format!("{:?}", d.mode));
    let back = serde_json::to_value(&d).unwrap_or_default();
    t.check("a PM workspace never writes a mode, so its file stays as it was", !writes_mode(&back), "");

    let mut files = vec![];
    let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("shots");
    workspace_files(&here, 4, &mut files);
    let fixtures = files.len();
    if let Ok(app) = std::env::var("APPDATA") {
        // the real data folder, read only: nothing is written there
        workspace_files(&Path::new(&app).join("Resolve2"), 0, &mut files);
    }
    let (mut loaded, mut pm, mut silent, mut same_bytes, mut failed) = (0, 0, 0, 0, vec![]);
    for f in &files {
        let Ok(text) = std::fs::read_to_string(f) else { continue };
        match serde_json::from_str::<WorkspaceData>(&text) {
            Ok(d) => {
                loaded += 1;
                if d.mode == Mode::Pm && d.intakes.iter().all(|i| i.mode == Mode::Pm) {
                    pm += 1;
                }
                let out = serde_json::to_string_pretty(&d).unwrap_or_default();
                if !writes_mode(&serde_json::from_str(&out).unwrap_or_default()) {
                    silent += 1;
                }
                // a file this version wrote comes back byte for byte
                if out == text {
                    same_bytes += 1;
                }
            }
            Err(e) => failed.push(format!("{}: {e}", f.display())),
        }
    }
    t.check(
        "every existing workspace (test data and this PC's own, read only) loads, as PM, and saves without a mode",
        !files.is_empty() && failed.is_empty() && pm == loaded && silent == loaded,
        format!("{} files ({fixtures} test data, {} on this PC): {loaded} loaded, {pm} PM, {silent} save without a mode, {same_bytes} round-trip byte for byte; failed: {failed:?}", files.len(), files.len() - fixtures),
    );
    println!("      {} files ({fixtures} test data, {} on this PC): {loaded} loaded, {pm} PM, {silent} save without a mode, {same_bytes} round-trip byte for byte", files.len(), files.len() - fixtures);

    // 2. a Careers workspace keeps its mode
    let mut c = WorkspaceData::default();
    c.mode = Mode::Careers;
    let mut i = crate::resolve::Intake::default();
    i.id = "i2".into();
    i.mode = Mode::Careers;
    c.intakes.push(i);
    let text = serde_json::to_string_pretty(&c).unwrap_or_default();
    let again: WorkspaceData = serde_json::from_str(&text).unwrap_or_default();
    t.check("a Careers workspace saves its mode and reloads as Careers, with its intake", text.contains("\"mode\": \"careers\"") && again.mode == Mode::Careers && again.intakes.first().map_or(false, |i| i.mode == Mode::Careers), "");
    // a Careers workspace made by the test, written and opened from disk, gets Careers tables
    let dir = std::env::temp_dir().join(format!("resolve2-modes-{}", std::process::id()));
    let path = dir.join("data.json");
    let wrote = crate::store::write_json_atomic(&path, &c).is_ok();
    let opened = crate::store::read_data(&path);
    let _ = std::fs::remove_dir_all(&dir);
    let opened_mode = opened.as_ref().map(|d| d.mode).unwrap_or_default();
    let mid: Vec<&str> = nav::nav(opened_mode).get(1).map(|g| g.items.iter().map(|i| i.label).collect()).unwrap_or_default();
    t.check("a Careers workspace written to disk opens with Careers tables (YOUR SEARCH, Careers team)", wrote && opened_mode == Mode::Careers && mid.first() == Some(&"Pipeline") && team::team(opened_mode)[1].name == "Scout", format!("{opened_mode:?} {mid:?}"));
    let mut merged = WorkspaceData::default();
    merged.mode = Mode::Careers;
    merged.merge(&WorkspaceData::default());
    t.check("syncing with another copy never changes a workspace's mode", merged.mode == Mode::Careers, "");
    t.check("mode labels read PM and Careers", Mode::Pm.label() == "PM" && Mode::Careers.label() == "Careers", "");

    // 3. PM mode picks exactly today's tables
    t.check(
        "PM mode picks today's tables: sidebar, Frameworks, Knowledge tabs, Help guide, intake targets and rail, AI Team",
        labels(nav::nav(Mode::Pm)) == labels(nav::NAV)
            && nav::frameworks(Mode::Pm).iter().map(|f| f.name).eq(nav::FRAMEWORKS.iter().map(|f| f.name))
            && nav::knowledge_tabs(Mode::Pm) == nav::KNOWLEDGE_TABS
            && journey::targets(Mode::Pm).len() == journey::TARGETS.len()
            && journey::targets(Mode::Pm)[0].0 == "goal"
            && dashboard::rail_steps(Mode::Pm)[3].0 == "Your next move"
            && help::guide(Mode::Pm)[0].0 == "Welcome"
            && team::team(Mode::Pm).len() == 5
            && team::default_role(Mode::Pm) == "validation"
            && team::team(Mode::Pm).iter().all(|r| r.action.is_none()),
        format!("{:?}", labels(nav::nav(Mode::Pm))),
    );
    let pm_mid: Vec<&str> = nav::nav(Mode::Pm)[1].items.iter().map(|i| i.label).collect();
    t.check("PM's middle group is still YOUR PROJECT: Overview, Roadmap, Tasks, Frameworks, Knowledge base", nav::nav(Mode::Pm)[1].label == "Your project" && pm_mid == ["Overview", "Roadmap", "Tasks", "Frameworks", "Knowledge base"], format!("{pm_mid:?}"));

    // 4. the Careers entries (Careers brief, "Modes")
    let groups: Vec<&str> = nav::nav(Mode::Careers).iter().map(|g| g.label).collect();
    let mid: Vec<&str> = nav::nav(Mode::Careers)[1].items.iter().map(|i| i.label).collect();
    t.check("Careers sidebar: WORKSPACE, YOUR SEARCH (with the picker), COLLABORATION", groups == ["Workspace", "Your search", "Collaboration"] && nav::nav(Mode::Careers)[1].picker, format!("{groups:?}"));
    t.check("YOUR SEARCH: Pipeline, My CV, Stats, Tasks, Interview recorder, Frameworks, Knowledge base", mid == ["Pipeline", "My CV", "Stats", "Tasks", "Interview recorder", "Frameworks", "Knowledge base"], format!("{mid:?}"));
    let ask = |label: &str| nav::nav(Mode::Careers).iter().flat_map(|g| g.items.iter()).find(|i| i.label == label).map(|i| (i.ask.placeholder, i.ask.suggestions.to_vec())).unwrap_or_default();
    t.check("Careers Ask wording: Home and Pipeline as the brief writes them", ask("Home") == ("Add a job, or ask what's next", vec!["Review this job ad", "What should I apply to today?", "Start my daily hunt"]) && ask("Pipeline").0 == "Ask about your applications", format!("{:?}", ask("Home")));
    // Projects and Handoffs keep PM's wording (Handoffs: "Same as PM mode")
    t.check("every other Careers screen with a bar has its own placeholder and three suggestions; AI Team has no bar", nav::nav(Mode::Careers).iter().flat_map(|g| g.items.iter()).filter(|i| nav::shows_ask_bar(Some(i.target)) && !["Projects", "Handoffs"].contains(&i.label)).all(|i| !i.ask.placeholder.is_empty() && i.ask.suggestions.len() == 3) && !nav::shows_ask_bar(Some(nav::Target::Sec(nav::Section::AiTeam))), "");
    let fw: Vec<&str> = nav::frameworks(Mode::Careers).iter().map(|f| f.name).collect();
    t.check("Careers Frameworks: Gates, CV templates, Cover letter, Interview prep, Offers and negotiation, Improvement playbook, Advice", fw == ["Gates", "CV templates", "Cover letter", "Interview prep", "Offers and negotiation", "Improvement playbook", "Advice"] && nav::frameworks(Mode::Careers)[0].read_by == "Reviewer", format!("{fw:?}"));
    let kt: Vec<&str> = nav::knowledge_tabs(Mode::Careers).iter().map(|t| t.0).collect();
    t.check("Careers Knowledge tabs: CVs and cover letters, Evidence bank, LinkedIn, Job ads, Interview notes, Offers, Advanced: databases", kt == ["CVs and cover letters", "Evidence bank", "LinkedIn", "Job ads", "Interview notes", "Offers", "Advanced: databases"], format!("{kt:?}"));
    let tg: Vec<&str> = journey::targets(Mode::Careers).iter().map(|t| t.1).collect();
    t.check("Careers intake targets: eight, within the question cap, each heard as a gate, a preference or a question", tg.len() == 8 && tg.len() <= MAX_QUESTIONS && tg[0] == "Role and level" && journey::targets(Mode::Careers).iter().all(|t| ["gate", "preference", "question"].contains(&t.3)), format!("{tg:?}"));
    let mut ci = crate::resolve::Intake::default();
    ci.mode = Mode::Careers;
    t.check("a Careers intake asks Careers questions without AI", journey::offline_next(&ci).map_or(false, |q| q.target == "role"), "");
    let rail: Vec<&str> = dashboard::rail_steps(Mode::Careers).iter().map(|s| s.0).collect();
    t.check("Careers rail: Get started, Bring it in, Confirm understanding, Your kit", rail == ["Get started", "Bring it in", "Confirm understanding", "Your kit"], format!("{rail:?}"));
    let roles: Vec<(&str, Option<&str>)> = team::team(Mode::Careers).iter().map(|r| (r.name, r.action)).collect();
    t.check(
        "Careers AI Team: General assistant, Scout, Reviewer, Tailor, Coach, Negotiator, each with its action",
        roles == [("General assistant", None), ("Scout", Some("Start my daily hunt")), ("Reviewer", Some("Review a job")), ("Tailor", Some("Tailor CV and cover letter")), ("Coach", Some("Prep me for an interview")), ("Negotiator", Some("Compare offers"))],
        format!("{roles:?}"),
    );
    let gd: Vec<&str> = help::guide(Mode::Careers).iter().map(|g| g.0).collect();
    t.check("Careers guide: one section per Careers screen, Pipeline written out", gd == ["Start a job search", "Home", "Pipeline", "My CV", "Stats", "Tasks", "Interview recorder", "Frameworks", "Knowledge base", "AI Team", "Calendar", "Handoffs"] && help::guide(Mode::Careers)[2].1.contains("### The tracker"), format!("{gd:?}"));

    // 5. the YGO slot has no content
    t.check("the YGO slot is there with no content", nav::nav(Mode::Ygo).is_empty() && nav::frameworks(Mode::Ygo).is_empty() && nav::knowledge_tabs(Mode::Ygo).is_empty() && help::guide(Mode::Ygo).is_empty() && journey::targets(Mode::Ygo).is_empty() && dashboard::rail_steps(Mode::Ygo).is_empty() && team::team(Mode::Ygo).is_empty(), "");

    // 6. no em dash in the new copy
    let mut copy: Vec<&str> = vec![];
    for g in nav::nav(Mode::Careers) {
        copy.push(g.label);
        for i in g.items {
            copy.extend([i.label, i.hint, i.ask.placeholder]);
            copy.extend(i.ask.suggestions.iter().copied());
        }
    }
    copy.extend(nav::frameworks(Mode::Careers).iter().flat_map(|f| [f.name, f.about, f.read_by]));
    copy.extend(nav::knowledge_tabs(Mode::Careers).iter().map(|t| t.0));
    copy.extend(journey::targets(Mode::Careers).iter().flat_map(|t| [t.1, t.2]));
    copy.extend(team::team(Mode::Careers).iter().flat_map(|r| [r.name, r.tag, r.prompt, r.action.unwrap_or("")]));
    copy.extend(help::guide(Mode::Careers).iter().flat_map(|g| [g.0, g.1]));
    let dashed: Vec<&&str> = copy.iter().filter(|s| s.contains('—')).collect();
    t.check("no em dash in any Careers entry", dashed.is_empty(), format!("{dashed:?}"));
}
