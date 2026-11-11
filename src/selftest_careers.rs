//! Careers Step 2 checks: the Careers intake end to end, on the labelled STAND-IN server (test
//! data only), and the same run with no AI. Nothing here is real AI output.

use crate::model::*;
use crate::resolve::careers::{self, *};
use crate::resolve::intake::{Attachment, Intake};
use crate::resolve::journey::{self, run_answer, apply_answer};
use crate::resolve::Kind;
use crate::selftest::T;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// The stand-in's scripted replies, chosen by which job the system prompt asks for.
fn reply(system: &str) -> Value {
    let c: Value = if system.starts_with("You are Resolve's job-search interviewer") {
        match system.matches("\nA: ").count() {
            0 => json!({"done": false, "covered": ["role", "location", "company", "timeline"], "target": "pay", "question": "What's the lowest base pay you'd accept?", "why": "Sets your pay gate: Reviewer fails any job below it.", "builds_on": ["input"]}),
            _ => json!({"done": true, "covered": ["role", "location", "company", "timeline", "pay", "contract"], "target": "followup", "question": "", "why": "", "builds_on": []}),
        }
    } else if system.starts_with("You sort what someone said about the job") {
        json!({"items": [{"text": "Base pay of at least 95,000", "kind": "gate"}, {"text": "Scale-ups over agencies", "kind": "preference"}, {"text": "Not sure about contract roles", "kind": "question"}]})
    } else if system.starts_with("You read a job-search intake") {
        json!({"job": "Head of Product, in identity or platform.", "hard_gates": "Head of Product or Director · 2 office days a week at most · base pay of at least 95,000", "preferences": "Scale-ups over agencies", "win": "A Head of Product role at 95,000 or more by March", "strengths": "Platform depth", "gaps": "Headline says Senior Product Owner", "fix": "Headline first", "broad": false})
    } else if system.starts_with("You are Resolve's job-search coach") && system.contains("SEARCH half") {
        json!({"search_name": "Head of Product search", "target": "Head of Product, identity or platform", "do_first": "Fix your CV headline", "why": "It says Senior Product Owner, and every role you want says Head of Product.",
               "gates": [{"rule": "No more than 2 office days a week", "hard": true, "why": "The commute from Birmingham"}, {"rule": "Base pay of at least 95,000", "hard": true, "why": "Your minimum"}, {"rule": "Scale-up or enterprise", "hard": false, "why": "You prefer them to agencies"}],
               "weekly_target": 10, "daily_found": 20})
    } else if system.starts_with("You are Resolve's job-search coach") {
        json!({"templates": [{"name": "Platform", "suits": "platform, identity", "headline": "Product leader for platforms", "default": true}, {"name": "General", "suits": "anything else", "headline": "", "default": true}],
               "letter_structure": "Hook, proof, fit, close", "letter": "Dear [NAME], ...",
               "evidence": [{"point": "Ran the managed services portfolio", "theme": "Platform scale", "role": "Senior PM, Acme", "number": "", "source": "from CV"}, {"point": "Grew partner revenue", "theme": "Partners", "role": "Partner Manager", "number": "40%", "source": "from CV"}, {"point": "Shipped a sign-in journey", "theme": "Identity", "role": "Senior PO", "number": "3 million users", "source": "from CV"}],
               "linkedin": [{"area": "Headline", "fix": "Say Head of Product"}, {"area": "About", "fix": "Lead with platforms"}]})
    } else if system.starts_with("You rebuild someone's CV") {
        json!({"cv": "# Head of Product\n\nProduct leader for platforms and identity. Senior PM at Acme, where I ran the managed services portfolio and grew partner revenue by 40%. Senior PO before that, shipping a sign-in journey."})
    } else {
        json!({"error": "unexpected"})
    };
    json!({"choices": [{"message": {"role": "assistant", "content": c.to_string()}}], "usage": {"prompt_tokens": 500, "completion_tokens": 100}})
}

fn count(d: &WorkspaceData, pid: &str, k: Kind) -> usize {
    d.records.iter().filter(|r| r.project_id.as_deref() == Some(pid) && r.kind == k).count()
}

pub fn run(t: &mut T, port: u16) {
    println!("\n-- Careers Step 2: the Careers intake --");
    // ---- the shape of the content
    t.check("Careers TARGETS fit the table's shape: key, label, fixed question, and what's heard (gate, preference, question)", journey::TARGETS_CAREERS.len() == 8 && journey::TARGETS_CAREERS.iter().all(|x| ["gate", "preference", "question"].contains(&x.3)), "");
    t.check("02b: the eight template headings, in order", TEMPLATE.len() == 8 && TEMPLATE[0] == "The job I want" && TEMPLATE[7] == "When I need it by", "");
    t.check("02c: the careers pull-in prompt asks for the eight headings, first person, Not covered", pull_prompt().contains("Write in the first person") && pull_prompt().contains("Not covered") && TEMPLATE.iter().all(|h| pull_prompt().contains(&format!("{h}:"))), "");
    let covered = covered_by_bring(EXAMPLE);
    t.check("03 skips what 02b answered (the example answers role, location, travel, company, timeline; its pay is a placeholder)", covered.contains(&"role".to_string()) && covered.contains(&"location".to_string()) && !covered.contains(&"pay".to_string()), format!("{covered:?}"));
    t.check("the Careers record kinds never appear in PM's lists", Kind::CAREERS.iter().all(|k| !Kind::ALL.contains(k)) && Kind::parse("gate") == Some(Kind::Gate), "");

    // ---- PDFs (CVs and LinkedIn exports): read with the PDF library, or refused, never garbage
    use crate::resolve::files;
    let here = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let brief = files::extract(&here.join("docs/briefs/2026-11-10-careers/build-brief.pdf"));
    t.check("a Google Docs PDF (the Careers brief) reads as clean text", brief.as_ref().map_or(false, |(_, x)| files::readable(x) && x.contains("Resolve Careers is a second mode")), brief.as_ref().map(|(_, x)| x.chars().take(80).collect::<String>()).unwrap_or_else(|e| e.clone()));
    let junk = std::env::temp_dir().join(format!("resolve2-junk-{}.pdf", std::process::id()));
    let _ = std::fs::write(&junk, b"%PDF-1.4\n1 0 obj << /Length 40 /Filter /DCTDecode >> stream\n\xff\xd8\xff\xe0 not text at all \x01\x02\nendstream endobj\n%%EOF");
    let r = files::extract(&junk);
    let _ = std::fs::remove_file(&junk);
    t.check("a PDF with no readable text says \"Resolve couldn't read this PDF. Paste the text instead.\"", r.as_ref().err().map(|e| e.as_str()) == Some(files::UNREADABLE_PDF), format!("{r:?}").chars().take(120).collect::<String>());
    t.check("text made of stray bytes (a colour profile read as text) doesn't count as readable", !files::readable("gTRC (bTRC (cprt <mluc enUS X G o o g l e / S k i a / 7 C 5 F A 2 1 5 1 3 9 7 4 òíäòíäòíäòíä 93 93 93 93 93 93 93 93 93 93 ÈÈ@È@È@") && files::readable(&"I led a team of six and grew partner revenue by forty percent in two years. ".repeat(3)), "");
    // your own LinkedIn export, if it's in docs/test-files/ (never committed)
    let linkedin: Vec<std::path::PathBuf> = std::fs::read_dir(here.join("docs/test-files")).map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x.eq_ignore_ascii_case("pdf"))).collect()).unwrap_or_default();
    if linkedin.is_empty() {
        println!("      (no PDF in docs/test-files yet: the LinkedIn export check is unverified)");
    }
    for f in &linkedin {
        let r = files::extract(f);
        let name = f.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        t.check(&format!("your PDF {name} reads as clean text, or is refused with the paste message"), r.as_ref().map_or_else(|e| e == files::UNREADABLE_PDF, |(_, x)| files::readable(x)), match &r { Ok((_, x)) => format!("read: {} words", x.split_whitespace().count()), Err(e) => e.clone() });
        println!("      {name}: {}", match &r { Ok((_, x)) => format!("read, {} words", x.split_whitespace().count()), Err(e) => e.clone() });
    }

    // ---- no AI: the fixed Careers question set, then cards and kit from their own words
    let mut off = Intake::new();
    off.mode = Mode::Careers;
    off.provider = "offline".into();
    off.input = EXAMPLE.to_string();
    off.ai_covered = covered.clone();
    let out = run_answer(None, &off, None, true);
    apply_answer(&mut off, out);
    let first = off.qas.first().cloned().unwrap_or_default();
    t.check("with no AI, the fixed Careers set asks the first target 02b didn't answer", first.target == "pay" && first.source == "fixed question set", format!("{} / {}", first.target, first.question));
    t.check("03 says how many it skipped", careers::skipped(&off) == covered.len(), careers::skipped(&off));
    let mut asked = 1;
    while asked < 20 {
        let cur = off.cursor;
        off.qas[cur].answer = match off.qas[cur].target.as_str() {
            "pay" => "At least 95,000 base".into(),
            "contract" => "Permanent only".into(),
            "disciplines" => "Product ops or platform strategy".into(),
            _ => "Not sure yet".into(),
        };
        let qid = off.qas[cur].id.clone();
        let before = off.qas.len();
        let out = run_answer(None, &off, Some(&qid), true);
        apply_answer(&mut off, out);
        if off.qas.len() == before {
            break;
        }
        asked += 1;
    }
    t.check("with no AI, the run stops within the cap of 8 and never asks a skipped target", off.qas.len() <= journey::MAX_QUESTIONS && off.qas.iter().all(|q| !covered.contains(&q.target)), off.qas.len());
    t.check("with no AI, answers are heard as their target's kind (pay is a hard gate)", off.heard.iter().any(|h| h.kind == "gate" && h.text.contains("95,000")), format!("{:?}", off.heard));
    let cards = offline_cards(&off);
    t.check("with no AI, the cards use only their own words (job from 02b, gates from what was heard)", cards.job.contains("Head of Product") && cards.hard_gates.contains("95,000") && cards.fix.is_empty(), format!("{cards:?}"));
    off.career = Some(Career { cards: Some(cards.clone()), ..Default::default() });
    let kit = offline_kit(&off, &cards);
    t.check("with no AI, the kit's gates are what they said, with nothing invented", kit.offline && !kit.gates.is_empty() && kit.gates.iter().all(|g| off.heard.iter().any(|h| h.text == g.rule)) && kit.gates.iter().all(|g| g.why.is_empty()), format!("{:?}", kit.gates));
    t.check("08: five first-week tasks with a day each, as the brief sets them", kit.week.len() == 5 && kit.week[0].title == "Fix your LinkedIn headline" && kit.week[2].title == "Set your weekly target: 10 applications" && kit.week.iter().all(|w| !w.day.is_empty()), format!("{:?}", kit.week));

    // ---- one push on a broad answer
    let mut b = Intake::new();
    b.mode = Mode::Careers;
    b.qas.push(journey::Qa { id: "q1".into(), target: "role".into(), question: "What role?".into(), answer: "Head of Product, Director of Product, Principal PM or Product Ops".into(), ..Default::default() });
    let push = push_for(&b, &b.qas[0].clone());
    t.check("03: a broad answer gets one push: \"That covers a lot. Which one first?\"", push.as_ref().map_or(false, |q| q.question == PUSH && q.target == "role"), format!("{push:?}"));
    b.qas.push(push.unwrap_or_default());
    t.check("03: only one push per target", push_for(&b, &b.qas[0].clone()).is_none(), "");

    // ---- with the stand-in model
    let stop = Arc::new(AtomicBool::new(false));
    let s2 = stop.clone();
    let server = tiny_http::Server::http(("127.0.0.1", port)).expect("stand-in server");
    let h = std::thread::spawn(move || {
        while !s2.load(Ordering::SeqCst) {
            if let Ok(Some(mut req)) = server.recv_timeout(std::time::Duration::from_millis(100)) {
                let mut body = String::new();
                let _ = std::io::Read::read_to_string(req.as_reader(), &mut body);
                let v: Value = serde_json::from_str(&body).unwrap_or(json!({}));
                let out = reply(v["messages"][0]["content"].as_str().unwrap_or(""));
                let _ = req.respond(tiny_http::Response::from_string(out.to_string()).with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()));
            }
        }
    });
    let mut i = Intake::new();
    i.mode = Mode::Careers;
    i.name = "Tester".into();
    i.provider = format!("openai|stand-in|http://127.0.0.1:{port}");
    i.input = EXAMPLE.to_string();
    i.attachments.push(Attachment { name: "cv.md".into(), text: "Senior PM at Acme. Ran the managed services portfolio. Grew partner revenue by 40%. Senior PO: shipped a sign-in journey.".into(), ..Default::default() });
    i.ai_covered = covered_by_bring(&i.input);
    let cfg = journey::cfg_for(&i);
    let out = run_answer(cfg.clone(), &i, None, true);
    apply_answer(&mut i, out);
    let q1 = i.qas.first().cloned().unwrap_or_default();
    t.check("the model asks for the gate still missing, and says which gate it sharpens", q1.target == "pay" && q1.why.contains("gate") && !q1.calls.is_empty(), format!("{q1:?}").chars().take(200).collect::<String>());
    i.qas[0].answer = "At least 95,000 base. Scale-ups over agencies. Not sure about contract roles.".into();
    let out = run_answer(cfg.clone(), &i, Some(&q1.id), true);
    apply_answer(&mut i, out);
    t.check("an answer is heard as hard gate, preference and question", ["gate", "preference", "question"].iter().all(|k| i.heard.iter().any(|h| &h.kind == k)), format!("{:?}", i.heard));
    t.check("the model can say the gates are clear enough to build the kit", i.suggest_stop, "");
    let (c, _) = run_cards(cfg.clone(), &i);
    let c = c.unwrap_or_default();
    t.check("04: the five cards come back, the CV card from the attached CV", c.job.starts_with("Head of Product") && !c.hard_gates.is_empty() && !c.strengths.is_empty() && !c.broad, format!("{c:?}"));
    i.career = Some(Career { cards: Some(c), ..Default::default() });
    let (k, calls) = run_kit(cfg.clone(), &i);
    let k = k.unwrap_or_default();
    t.check("the kit is two calls (one job each), written side by side", calls.iter().filter(|c| c.purpose.starts_with("careers:kit")).count() == 2, calls.iter().map(|c| c.purpose.clone()).collect::<Vec<_>>().join(", "));
    t.check("the kit has gates, templates (one default), cover letter, evidence bank, LinkedIn fixes, weekly target and five tasks", k.gates.len() == 3 && k.templates.iter().filter(|t| t.default).count() == 1 && k.letter_structure == "Hook, proof, fit, close" && k.evidence.len() == 3 && k.linkedin.len() == 2 && k.weekly_target == 10 && k.week.len() == 5, format!("{k:?}").chars().take(240).collect::<String>());
    t.check("a number stays only if their own words or files give it (40% stays; \"3 million users\" is dropped)", k.evidence.iter().any(|e| e.number == "40%") && k.evidence.iter().any(|e| e.point.starts_with("Shipped") && e.number.is_empty()), format!("{:?}", k.evidence));
    let cv = rebuild_cv(cfg.as_ref().unwrap(), &i);
    t.check("Rebuild my CV returns a CV built from the attached file", cv.as_ref().map_or(false, |s| s.starts_with("# Head of Product")), format!("{cv:?}").chars().take(120).collect::<String>());
    let mut no_cv = i.clone();
    no_cv.attachments.clear();
    t.check("Rebuild my CV refuses without a CV to rebuild from (nothing invented)", rebuild_cv(cfg.as_ref().unwrap(), &no_cv).is_err(), "");
    stop.store(true, Ordering::SeqCst);
    let _ = h.join();
    let mut car = i.career.clone().unwrap_or_default();
    car.kit = Some(k.clone());
    car.cv = cv.unwrap_or_default();
    i.career = Some(car);
    // gate 3 left out on 07
    if let Some(kk) = i.career.as_mut().and_then(|c| c.kit.as_mut()) {
        kk.gates[2].check = "not quite".into();
    }
    i.stage = "bridge".into();

    // ---- Adopt into a new workspace in Careers mode
    let mut d = WorkspaceData { mode: Mode::Careers, ..Default::default() };
    d.intakes.push(i.clone());
    let pid = journey::adopt_all(&mut d, &i.id).unwrap_or_default();
    let p = d.project(&pid).cloned().unwrap_or_default();
    t.check("Adopt creates the search (the project) with its name", p.name == "Head of Product search", &p.name);
    t.check("Adopt saves the kept gates (the one left out on 07 isn't), each Hard gate or Preference with why", count(&d, &pid, Kind::Gate) == 2 && d.records.iter().any(|r| r.kind == Kind::Gate && r.status == "Hard gate" && r.fields.get("why").map_or(false, |w| !w.is_empty())), count(&d, &pid, Kind::Gate));
    t.check("Adopt builds the CV templates (default marked) and the cover letter template", count(&d, &pid, Kind::CvTemplate) == 2 && d.records.iter().any(|r| r.kind == Kind::CvTemplate && r.status == "Default") && count(&d, &pid, Kind::CoverLetter) == 1, "");
    t.check("the rebuilt CV saves into My CV (the master CV)", d.records.iter().any(|r| r.kind == Kind::Cv && r.body.starts_with("# Head of Product") && r.fields.get("source").map(|s| s.as_str()) == Some("Rebuilt in the intake")), "");
    t.check("Adopt starts the evidence bank (numbers kept, the rest marked Needs a number) and the LinkedIn fixes", count(&d, &pid, Kind::Proof) == 3 && d.records.iter().any(|r| r.kind == Kind::Proof && r.status == "Needs a number") && count(&d, &pid, Kind::LinkedinFix) == 2, "");
    t.check("Adopt saves the weekly target", d.records.iter().any(|r| r.kind == Kind::SearchTarget && r.fields.get("weekly_target").map(|s| s.as_str()) == Some("10")), "");
    let tasks: Vec<&Task> = d.tasks.iter().filter(|x| x.project_id.as_deref() == Some(pid.as_str())).collect();
    t.check("Adopt adds the five first-week tasks, each dated to its day", tasks.len() == 5 && tasks.iter().all(|x| x.due.is_some()), tasks.len());
    t.check("Adopt saves the intake transcript and the attached CV as pages, and the intake moves to the reveal", d.pages.iter().any(|x| x.title.starts_with("Intake transcript")) && d.pages.iter().any(|x| x.title == "cv.md") && d.intakes.iter().any(|x| x.id == i.id && x.stage == "reveal" && x.status == "done"), "");
    t.check("11: the reveal's three counts are gates, templates and proof points", reveal_counts(&d, &pid) == (2, 2, 3), format!("{:?}", reveal_counts(&d, &pid)));
    t.check("the workspace stays in Careers mode, so it opens with Careers tables", d.mode == Mode::Careers && crate::ui::nav::nav(d.mode)[1].label == "Your search", "");
    // a saved Careers workspace reloads with its search
    let text = serde_json::to_string(&d).unwrap_or_default();
    let back: WorkspaceData = serde_json::from_str(&text).unwrap_or_default();
    t.check("the adopted Careers workspace saves and reloads with its search, gates and kit", back.mode == Mode::Careers && back.project(&pid).is_some() && back.records.iter().filter(|r| r.kind == Kind::Gate).count() == 2, "");
    // a second search in the same workspace
    let mut i2 = off.clone();
    i2.id = new_id();
    i2.career = Some(Career { cards: Some(cards.clone()), kit: Some(kit.clone()), here: true, ..Default::default() });
    d.intakes.push(i2.clone());
    let pid2 = journey::adopt_all(&mut d, &i2.id).unwrap_or_default();
    t.check("+ New project in a Careers workspace adds a second search with its own gates", pid2 != pid && d.projects.len() == 2 && count(&d, &pid2, Kind::Gate) == kit.gates.len(), d.projects.len());
    // PM's adopt is untouched
    t.check("a PM intake still adopts the PM way", journey::ADOPT_STEPS[0] == "Creating the project" && careers::ADOPT_STEPS[0] == "Creating your job search", "");
    let dashed: Vec<&str> = careers::ADOPT_STEPS.iter().chain(TEMPLATE.iter()).copied().chain([PUSH, THREE_MONTHS, EXAMPLE]).filter(|s| s.contains('—')).collect();
    t.check("no em dash in the Careers intake's copy", dashed.is_empty(), format!("{dashed:?}"));
}
