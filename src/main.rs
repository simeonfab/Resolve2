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
    if let Some(d) = arg_value(&args, "--data-dir") {
        let p = PathBuf::from(d);
        let p = if p.is_absolute() { p } else { std::env::current_dir().map(|c| c.join(&p)).unwrap_or(p) };
        store::set_root(p);
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

    let opts = ui::StartOpts {
        screenshot: arg_value(&args, "--screenshot").map(PathBuf::from),
        workspace: workspace.clone(),
        view: arg_value(&args, "--view"),
    };
    let native = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Resolve 2")
            .with_inner_size([1600.0, 900.0])
            .with_min_inner_size([960.0, 600.0]),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native("Resolve 2", native, Box::new(move |cc| Ok(Box::new(ui::App::new(cc, opts))))) {
        eprintln!("Couldn't start the window: {e}");
        std::process::exit(1);
    }
}
