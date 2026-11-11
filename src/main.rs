//! Resolve 2 — pages, databases, tasks and calendars in one local-first workspace.
//!
//!   resolve2                                  open the app
//!   resolve2 --screenshot shot.png            open, draw ~120 frames, save a PNG, exit
//!   resolve2 --view calendar|today|inbox|page|database|settings   start on a view
//!   resolve2 --mcp [--workspace NAME]         MCP server on stdio (Claude Desktop, Claude Code, Codex)
//!   resolve2 --serve [PORT] [--workspace NAME] [--token SECRET]
//!                                          headless hub: phone page, MCP over HTTP, /sync
//!                                          (run this on an always-on PC or a cloud server)
//!   resolve2 --data-dir PATH                  use another data folder (e.g. on a server)
//!   resolve2 --selftest                       run the end-to-end checks
//!   resolve2 --list-workspaces
//!   resolve2 --install-shortcuts            add Resolve 2 to the desktop and Start menu
//!   resolve2 --test-connection claude-code|ollama|anthropic   one real test reply (Connect your AI)
//!   resolve2 --view gallery [--state full|empty|loading|error] [--theme light|dark] [--reduced-motion]
//!            [--size 1920x1080] --screenshot shot.png      the component gallery (and any view's states)
//!   resolve2 --view <screen> [--collapsed] [--open ask|search[:words]] ...   the shell: sidebar collapsed, Ask sheet or search open

// the release build (what testers get) opens as a normal windowed app, with no console window
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai;
mod history;
mod import;
mod mcp;
mod model;
mod nlp;
mod resolve;
mod search;
mod seed;
mod selftest;
mod selftest_resolve;
mod selftest_safety;
mod selftest_journey;
mod selftest_alpha;
mod selftest_p0;
mod selftest_p1;
mod selftest_p2;
mod selftest_p3;
mod selftest_p4;
mod selftest_p5;
mod selftest_modes;
mod selftest_careers;
mod server;
mod store;
mod templates;
mod ui;

use std::path::PathBuf;

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).filter(|v| !v.starts_with("--")).cloned()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let testlab = args.iter().any(|a| a == "--testlab");
    // a preview build: its own data folder, and never the real one
    if store::PREVIEW.is_some() {
        match arg_value(&args, "--data-dir") {
            Some(d) if store::is_real_root(std::path::Path::new(&d)) => {
                eprintln!("Refused: Resolve 2 Preview never opens the real data folder ({d}).");
                std::process::exit(2);
            }
            Some(_) => {}
            None => store::set_root(store::preview_root()),
        }
    }
    if let Some(d) = arg_value(&args, "--data-dir") {
        let p = PathBuf::from(d);
        let p = if p.is_absolute() { p } else { std::env::current_dir().map(|c| c.join(&p)).unwrap_or(p) };
        store::set_root(p);
    } else if testlab {
        // Test Intake never touches the real data folder
        store::set_root(resolve::testlab::default_root());
    }
    if store::PREVIEW.is_some() && store::is_real_root(&store::root()) {
        eprintln!("Refused: Resolve 2 Preview never opens the real data folder ({}).", store::root().display());
        std::process::exit(2);
    }
    // resolve2 --seed-sample: add the café sample workspace to this data folder (the preview
    // script uses it); refused in the real data folder
    if args.iter().any(|a| a == "--seed-sample") {
        if store::is_real_root(&store::root()) {
            eprintln!("Refused: the sample goes in a preview or sandbox folder, never the real one.");
            std::process::exit(2);
        }
        let mut reg = store::Registry::load();
        let meta = match reg.find("Café sample").cloned() {
            Some(m) => m,
            None => reg.create("Café sample", [200, 160, 64]),
        };
        let mut st = store::Store::open(meta);
        let mut d = model::WorkspaceData::default();
        seed::seed_canvas(&mut d);
        st.data = d;
        let _ = st.save();
        // and a Careers one: the sample job search, as Adopt leaves it plus today's events
        let job = match reg.find("Job search sample").cloned() {
            Some(m) => m,
            None => reg.create("Job search sample", [94, 125, 79]),
        };
        if let Some(w) = reg.workspaces.iter_mut().find(|w| w.id == job.id) {
            w.mode = model::Mode::Careers;
        }
        reg.save();
        let mut js = store::Store::open(job);
        js.data = seed::careers_workspace(true);
        match js.save() {
            Ok(_) => println!("café and job search samples ready in {}", store::root().display()),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
        return;
    }
    // standing rule: screenshots run only in a sandbox folder, never the real data folder. This
    // runs before anything opens a workspace. With no --data-dir, a screenshot gets a fresh sandbox
    // folder of its own under %TEMP% (Craft Anything's build check runs it that way); pointed at
    // the real folder, it's refused.
    if args.iter().any(|a| a == "--screenshot") {
        // a preview's screenshot needs its own sandbox: its data folder holds a copy of yours
        if store::PREVIEW.is_some() && arg_value(&args, "--data-dir").is_none() {
            eprintln!("Refused: a preview screenshot needs --data-dir with a sandbox folder.");
            std::process::exit(2);
        }
        if arg_value(&args, "--data-dir").is_none() && !testlab {
            store::set_root(std::env::temp_dir().join(format!("resolve2-shot-{}", std::process::id())).join("data"));
            // the sandbox holds the café sample workspace, set up and past Connect your AI, so
            // the build check's picture is a workspace, not the first-run screen
            let mut reg = store::Registry::load();
            reg.settings.ai_setup_done = true;
            reg.save();
            let mut st = store::Store::open(reg.workspaces[0].clone());
            let mut d = model::WorkspaceData::default();
            seed::seed_canvas(&mut d);
            st.data = d;
            let _ = st.save();
        }
        if store::is_real_root(&store::root()) {
            eprintln!("Refused: a screenshot runs only in a sandbox folder, never the real data folder ({}).", store::root().display());
            std::process::exit(2);
        }
    }
    // resolve2 --test-connection PROVIDER: the same real round trip as Connect your AI's Test button
    if let Some(which) = arg_value(&args, "--test-connection") {
        let s = store::Registry::load().settings;
        let cfg = match which.as_str() {
            "claude-code" => ai::AiConfig { provider: which.clone(), url: String::new(), model: String::new(), key: String::new() },
            "anthropic" => ai::AiConfig { provider: which.clone(), url: String::new(), model: ai::CLAUDE_MODEL.into(), key: s.anthropic_key },
            _ => {
                let models = ai::list_models("ollama", ui::connect::OLLAMA_URL).unwrap_or_default();
                let m = models.iter().find(|m| m.starts_with("qwen3")).or(models.first()).cloned().unwrap_or_else(|| "qwen3".into());
                ai::AiConfig { provider: "ollama".into(), url: ui::connect::OLLAMA_URL.into(), model: m, key: String::new() }
            }
        };
        let t0 = std::time::Instant::now();
        match ai::test_connection(&cfg) {
            Ok(r) => println!("✓ {} / {} replied in {:.1}s: {r}", cfg.provider, if cfg.model.is_empty() { "default" } else { &cfg.model }, t0.elapsed().as_secs_f32()),
            Err(e) => {
                println!("✗ {}: {e}", cfg.provider);
                std::process::exit(1);
            }
        }
        return;
    }
    // resolve2 --import FILE... [--no-ai]: read each file, sort it, file it in Knowledge (newest
    // guided project), and say where it went — the same steps as Knowledge → Add files
    if args.iter().any(|a| a == "--import") {
        let paths: Vec<PathBuf> = args.iter().skip_while(|a| *a != "--import").skip(1).take_while(|a| !a.starts_with("--")).map(PathBuf::from).collect();
        let reg = store::Registry::load();
        let meta = reg.find("Personal").cloned().unwrap_or_else(|| reg.workspaces[0].clone());
        let mut st = store::Store::open(meta);
        let pid = st.data.contexts.iter().filter(|c| st.data.project(&c.project_id).is_some()).max_by_key(|c| c.created).map(|c| c.project_id.clone());
        let cfg = ai::current_config();
        let use_ai = cfg.provider != "offline" && !args.iter().any(|a| a == "--no-ai");
        for p in paths {
            let t0 = std::time::Instant::now();
            match resolve::files::extract(&p) {
                Ok((title, text)) => {
                    let (place, why, how) = if use_ai {
                        match resolve::files::sort_with_ai(&cfg, &title, &text) {
                            Ok((pl, w)) => (pl, format!("sorted by the AI: {w}"), "ai"),
                            Err(e) => {
                                let (pl, w) = resolve::files::guess(&title, &text);
                                (pl, format!("{w} (AI failed: {e})"), "guess")
                            }
                        }
                    } else {
                        let (pl, w) = resolve::files::guess(&title, &text);
                        (pl, w, "guess")
                    };
                    resolve::files::place(&mut st.data, pid.as_deref(), &title, &text, &place, how);
                    println!("{} → {} ({} words, {:.1}s) — {why}", p.display(), resolve::files::label(&place), text.split_whitespace().count(), t0.elapsed().as_secs_f32());
                }
                Err(e) => println!("{} → not imported: {e}", p.display()),
            }
        }
        let _ = st.save();
        return;
    }
    // resolve2 --handoff "REQUEST" [--to WHO] [--ai]: print the handoff brief Resolve writes for
    // the newest guided project (deterministic; with --ai, rewritten by the AI in Settings)
    if let Some(focus) = arg_value(&args, "--handoff") {
        let reg = store::Registry::load();
        let meta = reg.find("Personal").cloned().unwrap_or_else(|| reg.workspaces[0].clone());
        let st = store::Store::open(meta);
        let Some(ctx) = st.data.contexts.iter().filter(|c| st.data.project(&c.project_id).is_some()).max_by_key(|c| c.created).cloned() else {
            println!("No guided project in this data folder.");
            std::process::exit(1);
        };
        let to = arg_value(&args, "--to").unwrap_or_default();
        if args.iter().any(|a| a == "--ai") {
            let t0 = std::time::Instant::now();
            match resolve::roles::handoff_ai(&ai::current_config(), &st.data, &ctx, &to, &focus) {
                Ok(b) => println!("{b}\n\n(written by {} in {:.1}s)", ai::current_config().provider, t0.elapsed().as_secs_f32()),
                Err(e) => println!("AI failed: {e}"),
            }
        } else {
            println!("{}", resolve::roles::handoff_for(&st.data, &ctx, &to, &focus));
        }
        return;
    }
    // resolve2 --promote RUN.json [--workspace NAME]: adopt a Test Intake run into the real data folder
    if let Some(f) = arg_value(&args, "--promote") {
        match resolve::testlab::promote_file(&f, arg_value(&args, "--workspace").as_deref()) {
            Ok((ws, pid)) => println!("Created the project in your \"{ws}\" workspace (id {pid}). Open Resolve 2 to see it."),
            Err(e) => {
                println!("{e}");
                std::process::exit(1);
            }
        }
        return;
    }
    // resolve2 --testlab --speed-compare "SCENARIO" --out FILE [--max-q N] (the D5 speed fix):
    // the questions and the understanding once on Claude Code, then the proposal's validation half
    // twice from that same understanding (the default model, then the faster one), timed, both
    // written to FILE as Markdown. Real AI calls; nothing is invented.
    if let Some(name) = arg_value(&args, "--speed-compare") {
        let Some(scn) = resolve::testlab::scenario(&name) else {
            println!("No scenario called '{name}'.");
            std::process::exit(2);
        };
        let max_q = arg_value(&args, "--max-q").and_then(|n| n.parse().ok()).unwrap_or(3);
        let out = arg_value(&args, "--out").unwrap_or_else(|| "speed.md".into());
        // "confirm" makes the headless run stop once the understanding is written
        let mut i = resolve::testlab::headless_run(&scn, "claude-code||", max_q, false, &mut |s| println!("{s}"));
        let cfg = ai::AiConfig { provider: "claude-code".into(), url: String::new(), model: String::new(), key: String::new() };
        let u = i.understanding.clone().unwrap_or_default();
        i.outcome_confirmed = true;
        let mut md = format!("## {name}\n\n**Project, as understood:** {}\n\n**Outcome:** {}\n\n", u.project, u.outcome);
        let mut row = vec![name.clone()];
        for (label, c) in [("Default model (Claude Code's own default)", cfg.clone()), ("Faster model (Sonnet)", ai::fast_cfg(&cfg).unwrap_or(cfg.clone()))] {
            let t0 = std::time::Instant::now();
            let r = resolve::journey::propose_half(&c, &i, &u, "validation");
            let secs = t0.elapsed().as_secs_f32();
            row.push(format!("{secs:.1} s{}", if r.is_err() { " (failed)" } else { "" }));
            md.push_str(&format!("### {label}: {secs:.1} s\n\n"));
            match r {
                Ok(p) => {
                    let it = |x: &resolve::journey::Item| format!("{} ({})", x.title, if x.basis.is_empty() { "uncertain" } else { &x.basis });
                    md.push_str(&format!("- **H0:** {}{}\n", it(&p.h0), if p.h0.negation.is_empty() { String::new() } else { format!(". Against it: {}", p.h0.negation) }));
                    for h in &p.hypotheses {
                        md.push_str(&format!("- **Hypothesis:** {}{}\n", it(h), if h.negation.is_empty() { String::new() } else { format!(". {}", h.negation) }));
                    }
                    for e in &p.experiments {
                        let s = e.fields.get("support_if").cloned().unwrap_or_default();
                        md.push_str(&format!("- **Experiment:** {}{}\n", e.title, if s.is_empty() { String::new() } else { format!(". Supports it if {s}") }));
                    }
                    for x in &p.uncertainties {
                        md.push_str(&format!("- **Uncertainty:** {}\n", x.title));
                    }
                    for x in &p.pages {
                        md.push_str(&format!("- **Page:** {}\n", x.title));
                    }
                }
                Err(e) => md.push_str(&format!("Failed: {e}\n")),
            }
            md.push('\n');
        }
        let _ = std::fs::write(&out, md);
        println!("ROW|{}", row.join("|"));
        return;
    }
    // resolve2 --testlab --lab-run "SCENARIO" [--provider P] [--model M] [--url U] [--max-q N] [--adopt]
    // drives a whole intake headlessly in the Test Intake data folder and prints what happened
    if let Some(name) = arg_value(&args, "--lab-run") {
        let Some(scn) = resolve::testlab::scenario(&name) else {
            println!("No scenario called '{name}'. Scenarios: {}", resolve::testlab::scenarios().iter().map(|s| s.name.clone()).collect::<Vec<_>>().join(", "));
            std::process::exit(2);
        };
        let provider = match arg_value(&args, "--provider") {
            Some(p) if p == "offline" => p,
            Some(p) => format!("{p}|{}|{}", arg_value(&args, "--model").unwrap_or_default(), arg_value(&args, "--url").unwrap_or_default()),
            None => String::new(),
        };
        let max_q = arg_value(&args, "--max-q").and_then(|n| n.parse().ok()).unwrap_or(9);
        let t0 = std::time::Instant::now();
        let mut run = resolve::testlab::headless_run(&scn, &provider, max_q, false, &mut |s| println!("{s}"));
        let mut reg = store::Registry::load();
        let adopt = args.iter().any(|a| a == "--adopt");
        // a Careers run adopts into a new workspace in Careers mode, as the app does
        let careers_new = adopt && run.mode == model::Mode::Careers && run.career.as_ref().map_or(false, |c| c.kit.is_some());
        let search = run.career.as_ref().and_then(|c| c.kit.as_ref()).map(|k| k.search_name.trim().to_string()).filter(|n| !n.is_empty()).unwrap_or_else(|| "Job search".into());
        let meta = if careers_new { reg.create(&search, [94, 125, 79]) } else { reg.find("Personal").cloned().unwrap_or_else(|| reg.workspaces[0].clone()) };
        let mut st = store::Store::open(meta);
        if careers_new {
            st.data.mode = model::Mode::Careers;
            seed::careers_calendars(&mut st.data);
            if let Some(w) = reg.workspaces.iter_mut().find(|w| w.id == st.meta.id) {
                w.mode = model::Mode::Careers;
            }
            reg.save();
        }
        let rid = run.id.clone();
        if adopt && (run.proposal.is_some() || careers_new) {
            run.stage = "bridge".into();
        }
        st.data.intakes.push(run);
        if adopt {
            match resolve::journey::adopt_all(&mut st.data, &rid) {
                Ok(pid) if careers_new => {
                    let (g, t, p) = resolve::careers::reveal_counts(&st.data, &pid);
                    let tasks = st.data.tasks.iter().filter(|x| x.project_id.as_deref() == Some(pid.as_str())).count();
                    println!("adopted → workspace \"{}\" (mode {}), search \"{}\": {g} gates, {t} templates, {p} proof points, {tasks} tasks", st.meta.name, st.data.mode.label(), st.data.project(&pid).map(|x| x.name.clone()).unwrap_or_default());
                }
                Ok(pid) => println!("adopted → project {pid}"),
                Err(e) => println!("adopt failed: {e}"),
            }
        }
        let _ = st.save();
        if let Some(r) = st.data.intakes.iter().find(|x| x.id == rid) {
            if let Ok((md, js)) = resolve::testlab::export(r) {
                println!("exported: {}\n          {}", md.display(), js.display());
            }
        }
        println!("seconds: {:.1}", t0.elapsed().as_secs_f32());
        return;
    }
    // resolve2 --connect-claude-desktop [--workspace NAME]: register this workspace in Claude Desktop only
    if args.iter().any(|a| a == "--connect-claude-desktop") {
        let reg = store::Registry::load();
        let meta = arg_value(&args, "--workspace").as_deref().and_then(|w| reg.find(w).cloned()).or_else(|| reg.find("Personal").cloned()).unwrap_or_else(|| reg.workspaces[0].clone());
        let exe = std::env::current_exe().map(|x| x.display().to_string()).unwrap_or_default();
        let name = format!("resolve2_{}", store::sanitize(&meta.name).to_lowercase().replace([' ', '-', '.', '(', ')'], "_"));
        match ui::settings::add_to_claude_desktop(&name, &exe, &meta.id) {
            Ok(m) => println!("{m}"),
            Err(e) => {
                println!("⚠ {e}");
                std::process::exit(1);
            }
        }
        return;
    }
    let workspace = arg_value(&args, "--workspace");
    let pick = |reg: &store::Registry| -> store::WorkspaceMeta {
        workspace
            .as_deref()
            .and_then(|w| reg.find(w).cloned())
            .or_else(|| reg.find("Personal").cloned())
            .unwrap_or_else(|| reg.workspaces[0].clone())
    };

    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{}", include_str!("main.rs").lines().take_while(|l| l.starts_with("//!")).map(|l| l.trim_start_matches("//!")).collect::<Vec<_>>().join("\n"));
        return;
    }
    if args.iter().any(|a| a == "--selftest") {
        std::process::exit(selftest::run());
    }
    // resolve2 --ask "request" [--provider claude-code|codex-cli|anthropic|openai-cloud|ollama] [--workspace NAME]
    if let Some(q) = arg_value(&args, "--ask") {
        let reg = store::Registry::load();
        let s = &reg.settings;
        let provider = arg_value(&args, "--provider").unwrap_or_else(|| s.ai_provider.clone());
        let key = match provider.as_str() {
            "anthropic" => s.anthropic_key.clone(),
            "openai-cloud" => s.openai_key.clone(),
            _ => String::new(),
        };
        let cfg = ai::AiConfig { provider, url: s.ai_url.clone(), model: s.ai_model.clone(), key };
        let prompt = format!("Today is {}.\nRequest: {q}", model::today().format("%A %-d %B %Y"));
        match ai::run_agent(&cfg, ai::AGENT_SYSTEM, &prompt, pick(&reg), true) {
            Ok(t) => println!("{t}"),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
        return;
    }
    // resolve2 --test-ai [--provider P] [--model M] [--url U]: one real intake "understand" call
    // through the same code the app uses; prints the provider, timing and the validated result or error.
    if args.iter().any(|a| a == "--test-ai") {
        let mut cfg = ai::current_config();
        if let Some(p) = arg_value(&args, "--provider") {
            cfg.provider = p;
        }
        if let Some(m) = arg_value(&args, "--model") {
            cfg.model = m;
        }
        if let Some(u) = arg_value(&args, "--url") {
            cfg.url = u;
        }
        let sample = "I run a small bakery and want to start selling bread subscriptions to offices nearby. I have two office customers who asked for it. I don't know if they'd pay monthly up front.";
        println!("provider={} model={} url={}", cfg.provider, if cfg.model.is_empty() { "(default)" } else { &cfg.model }, cfg.url);
        let t0 = std::time::Instant::now();
        let r = resolve::intake::run_understand(&cfg, sample);
        println!("seconds={:.1}", t0.elapsed().as_secs_f32());
        match r {
            Ok(u) => println!("OK\n{}", serde_json::to_string_pretty(&u).unwrap_or_default()),
            Err(e) => {
                println!("ERROR {e}");
                std::process::exit(1);
            }
        }
        return;
    }
    // resolve2 --connect-claude [--workspace NAME]: add this workspace to Claude Desktop and Claude Code
    if args.iter().any(|a| a == "--connect-claude") {
        let reg = store::Registry::load();
        let meta = pick(&reg);
        let exe = std::env::current_exe().map(|x| x.display().to_string()).unwrap_or_default();
        let name = format!("resolve2_{}", store::sanitize(&meta.name).to_lowercase().replace([' ', '-', '.', '(', ')'], "_"));
        for r in [ui::settings::add_to_claude_desktop(&name, &exe, &meta.id), ui::settings::add_to_claude_code(&name, &exe, &meta.id)] {
            match r {
                Ok(m) => println!("{m}"),
                Err(e) => println!("⚠ {e}"),
            }
        }
        return;
    }
    if args.iter().any(|a| a == "--install-shortcuts") {
        let exe = std::env::current_exe().map(|x| x.display().to_string()).unwrap_or_default();
        println!("{}", ui::settings::make_shortcuts(&exe));
        return;
    }
    if args.iter().any(|a| a == "--list-workspaces") {
        let reg = store::Registry::load();
        for w in &reg.workspaces {
            println!("{}\t{}", w.id, w.name);
        }
        return;
    }
    if args.iter().any(|a| a == "--mcp") {
        let reg = store::Registry::load();
        if let Some(w) = workspace.as_deref() {
            if reg.find(w).is_none() {
                eprintln!("No workspace named '{w}'");
                std::process::exit(2);
            }
        }
        mcp::run_stdio(store::Store::open(pick(&reg)));
        return;
    }
    if args.iter().any(|a| a == "--serve") {
        let mut reg = store::Registry::load();
        // on a fresh cloud server, `--workspace Personal` creates it; the first device sync fills it
        if let Some(w) = workspace.as_deref() {
            if reg.find(w).is_none() {
                reg.create(w, [219, 76, 63]);
            }
        }
        let port = arg_value(&args, "--serve").and_then(|p| p.parse().ok()).unwrap_or(8797);
        if let Err(e) = server::run_forever(pick(&reg), port, arg_value(&args, "--token")) {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return;
    }

    // --now 2026-11-06T14:00 (screenshots only): the app's clock reads that, so the canvases'
    // Friday 6 November sample lines up like for like
    if args.iter().any(|a| a == "--screenshot") {
        if let Some(at) = arg_value(&args, "--now").and_then(|s| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M").ok()) {
            model::set_clock(at);
        }
    }
    let opts = ui::StartOpts {
        screenshot: arg_value(&args, "--screenshot").map(PathBuf::from),
        workspace: workspace.clone(),
        view: arg_value(&args, "--view"),
        testlab,
        // screenshot helpers (in memory only; never saved): which of the four states to draw,
        // light or dark, and reduced motion
        state: arg_value(&args, "--state"),
        theme: arg_value(&args, "--theme"),
        reduced_motion: args.iter().any(|a| a == "--reduced-motion"),
        scroll: arg_value(&args, "--scroll").and_then(|s| s.parse().ok()),
        collapsed: args.iter().any(|a| a == "--collapsed"),
        careers: args.iter().any(|a| a == "--careers"),
        motion_at: arg_value(&args, "--motion-at").and_then(|v| v.parse().ok()),
        open: arg_value(&args, "--open"),
        project: arg_value(&args, "--project"),
        provider: arg_value(&args, "--provider"),
        model: arg_value(&args, "--model"),
    };
    let native = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            // the version in the title (CHANGELOG.md has what each one changed)
            .with_title(match (testlab, store::PREVIEW) {
                (true, _) => format!("Resolve 2 {}: Test Intake (sandbox)", env!("CARGO_PKG_VERSION")),
                (false, Some(b)) => format!("Resolve 2 Preview: {b}, {}", env!("CARGO_PKG_VERSION")),
                _ => format!("Resolve 2 {}", env!("CARGO_PKG_VERSION")),
            })
            // --size WxH (screenshots at a given window width)
            .with_inner_size(arg_value(&args, "--size").and_then(|s| s.split_once('x').and_then(|(w, h)| Some([w.parse::<f32>().ok()?, h.parse::<f32>().ok()?]))).unwrap_or([1600.0, 900.0]))
            .with_min_inner_size([960.0, 600.0]),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native("Resolve 2", native, Box::new(move |cc| Ok(Box::new(ui::App::new(cc, opts))))) {
        eprintln!("Couldn't start the window: {e}");
        std::process::exit(1);
    }
}
