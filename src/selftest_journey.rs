//! Checks for the intake journey (v2). The model is a scripted STAND-IN server (test data only).

use crate::model::*;
use crate::resolve::intake::Intake;
use crate::resolve::journey::{self, *};
use crate::resolve::Kind;
use crate::selftest::T;
use crate::store::{Store, WorkspaceMeta};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

fn reply(system: &str, proposals: &AtomicUsize) -> Value {
    let c: Value = if system.starts_with("You are Resolve's intake interviewer") {
        let answered = system.matches("\nA: ").count();
        match answered {
            0 => json!({"done": false, "covered": ["goal"], "target": "outcome", "question": "If the events work, who is better off, and how would you know?", "why": "H0 is 'companies want Doom Clock events'; the outcome tests whether anyone is actually better off.", "builds_on": ["input"]}),
            1 => json!({"done": false, "covered": ["goal", "outcome"], "target": "evidence", "question": "What have companies told you so far?", "why": "Separates evidence from assumption for H1 (HR teams will pay).", "builds_on": ["input"]}),
            _ => json!({"done": true, "covered": ["goal", "outcome", "evidence"], "target": "followup", "question": "", "why": "", "builds_on": []}),
        }
    } else if system.starts_with("You sort what a founder said") {
        json!({"items": [
            {"text": "Two HR managers asked for a quote", "kind": "evidence"},
            {"text": "The user is making the assumption that companies will pay £2,000 per event", "kind": "assumption"},
            {"text": "Whether bookings repeat", "kind": "question"}]})
    } else if system.starts_with("You are operating as the ResolveOS Business Analyst role") {
        json!({"project": "Resolve is a project to create and deliver Doom Clock escape-room style events for corporate team building.", "goal": "Sell Doom Clock events to companies", "state": "Two pilots run", "uncertainty": "Repeat bookings",
               "outcome": "The user appears to be better off when teams bond", "outcomeStated": true, "outcomeQuestion": "", "gapQuestions": [], "assumptions": ["I am assuming HR budgets exist"]})
    } else if system.starts_with("You are operating as the ResolveOS Strategic Product Director, writing a project proposal") {
        if proposals.fetch_add(1, Ordering::SeqCst) == 0 {
            json!({"name": "Resolve is a project to create and deliver Doom Clock events", "summary": "s", "objective": "o", "h0": {"title": "Companies want it"}, "roadmap": [{"title": "Validate", "phase": "Now"}], "tasks": [], "pages": [{"title": "Project brief", "detail": "# Brief"}]})
        } else {
            json!({"name": "Doom Clock corporate events", "summary": "Sell escape-room style team events to companies.", "objective": "Book repeat corporate events",
                "next_move": "Send both HR contacts a quote", "why": "The user already has two warm leads asking for a price.",
                "h0": {"title": "Companies want Doom Clock team events", "negation": "Companies do not want Doom Clock team events", "basis": "uncertain"},
                "hypotheses": [{"title": "The user is making the assumption that HR teams will pay £2,000", "negation": "HR teams won't pay £2,000", "basis": "evidence", "sources": []}],
                "uncertainties": [{"title": "Whether bookings repeat"}],
                "experiments": [{"title": "E0: quote three HR teams", "parent": 0, "fields": {"decision": "Price point", "prediction": "2 of 3 accept", "support_if": ">=2 accept", "refute_if": "0 accept", "inconclusive_if": "1 accepts"}}],
                "roadmap": [{"title": "Price the corporate package", "detail": "Fixed price per team of 12", "phase": "Now", "basis": "assumption"},
                            {"title": "Run three paid pilots", "detail": "With the two HR leads plus one more", "phase": "Next", "basis": "evidence", "sources": ["missing-id"]},
                            {"title": "Repeat-booking offer", "detail": "A quarterly rebooking deal, because you don't know yet whether bookings repeat", "phase": "Later", "basis": "uncertain"}],
                "tasks": [{"title": "Email both HR managers a quote for a 12-person session with venue and timing options", "detail": "", "phase": "Now", "parent": 0, "basis": "assumption"},
                          {"title": "List three venues", "detail": "Within 20 minutes of the office", "phase": "Now", "parent": 1, "basis": "uncertain"}],
                "pages": [{"title": "Project brief", "detail": "## Doom Clock corporate events\n\nTeam events built on the Doom Clock game."}]})
        }
    } else {
        json!({"error": "unexpected"})
    };
    json!({"choices": [{"message": {"role": "assistant", "content": c.to_string()}}], "usage": {"prompt_tokens": 500, "completion_tokens": 100}})
}

pub fn run(t: &mut T, meta: WorkspaceMeta, port: u16) {
    println!("\n-- Intake journey --");
    // ---- the five known failures, as rules
    t.check("assumptions are stored as the claim itself", clean_claim("The user is making the assumption that offices pay monthly") == "Offices pay monthly" && clean_claim("I am assuming HR budgets exist") == "HR budgets exist", clean_claim("The user is making the assumption that offices pay monthly"));
    let n = clean_name("Resolve is a project to create and deliver Doom Clock corporate events for teams");
    t.check("project names are short and never the app's name", !n.to_lowercase().starts_with("resolve") && n.split_whitespace().count() <= 5, &n);
    t.check("outcomes drop the third-person framing", clean_outcome("The user appears to be better off when teams bond") == "Teams bond", clean_outcome("The user appears to be better off when teams bond"));
    let answer = "Two people who played at a games night asked if I could run it at their offices.";
    let items = vec![
        Heard { id: "a".into(), text: "Two people asked if I could run it at their offices".into(), kind: "evidence".into(), from: "q".into(), corrected: false },
        Heard { id: "b".into(), text: "It matters because it can improve team cohesion".into(), kind: "assumption".into(), from: "q".into(), corrected: false },
        Heard { id: "c".into(), text: "and the".into(), kind: "evidence".into(), from: "q".into(), corrected: false },
    ];
    let kept = filter_heard(items, "why", "Why does this matter?", &[], answer);
    t.check("'What Resolve has heard' keeps only the user's own words (invented items and fragments dropped)", kept.len() == 1 && kept[0].kind == "evidence", format!("{kept:?}"));
    t.check("generic filler is recognised", is_generic("Validate") && is_generic("Refine") && is_generic("Iterate and improve") && !is_generic("Book three venues"), "generic check");

    // ---- offline fallback: fixed questions cover every target, nothing invented
    let mut off = Intake::new();
    off.input = "A bakery wants office bread subscriptions.".into();
    let mut asked = 0;
    while let Some(mut q) = offline_next(&off) {
        q.answer = format!("answer {}", q.target);
        off.qas.push(q);
        asked += 1;
        if asked > 20 {
            break;
        }
    }
    t.check("without AI, the fixed question set asks up to the 8-question cap", asked == crate::resolve::journey::MAX_QUESTIONS && off.coverage() > 0.85, asked);
    let ev = off.qas.iter().find(|q| q.target == "evidence").cloned().unwrap_or_default();
    t.check("without AI, answers are sorted by their target (evidence stays evidence)", offline_classify(&ev).first().map_or(false, |h| h.kind == "evidence"), "kind");
    for q in off.qas.clone() {
        off.heard.extend(offline_classify(&q));
    }
    off.understanding = Some(offline_understanding(&off));
    let op = offline_proposal(&off, off.understanding.as_ref().unwrap());
    t.check("without AI, the proposal uses only the user's words (no invented roadmap or tasks)", op.offline && op.roadmap.is_empty() && op.tasks.is_empty() && op.pages.len() == 1 && op.outcome == "Answer outcome" && op.next_move.is_empty(), format!("{op:?}").chars().take(200).collect::<String>());

    // ---- with the stand-in model
    let stop = Arc::new(AtomicBool::new(false));
    let s2 = stop.clone();
    let server = tiny_http::Server::http(("127.0.0.1", port)).expect("stand-in server");
    let h = std::thread::spawn(move || {
        let proposals = AtomicUsize::new(0);
        while !s2.load(Ordering::SeqCst) {
            if let Ok(Some(mut req)) = server.recv_timeout(std::time::Duration::from_millis(100)) {
                let mut body = String::new();
                let _ = std::io::Read::read_to_string(req.as_reader(), &mut body);
                let v: Value = serde_json::from_str(&body).unwrap_or(json!({}));
                let out = reply(v["messages"][0]["content"].as_str().unwrap_or(""), &proposals);
                let _ = req.respond(tiny_http::Response::from_string(out.to_string()).with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()));
            }
        }
    });
    let mut i = Intake::new();
    i.provider = format!("openai|stand-in|http://127.0.0.1:{port}");
    i.input = "Doom Clock is a game; I want to sell it as corporate team-building events.".into();
    let cfg = journey::cfg_for(&i);
    let out = run_answer(cfg.clone(), &i, None, true);
    apply_answer(&mut i, out);
    let q1 = i.qas.first().cloned().unwrap_or_default();
    t.check("the model chooses the first question and says why (decision trail)", q1.target == "outcome" && q1.why.contains("H0") && q1.depends_on == vec!["input".to_string()] && !q1.calls.is_empty(), format!("{q1:?}").chars().take(200).collect::<String>());
    i.qas[0].answer = "HR teams — they'd rebook every quarter. Two HR managers asked for a quote; companies will pay £2,000 per event; not sure whether bookings repeat.".into();
    let out = run_answer(cfg.clone(), &i, Some(&q1.id), true);
    apply_answer(&mut i, out);
    t.check("an answer is sorted into evidence / assumption / open question, assumptions cleaned", i.heard.iter().any(|h| h.kind == "evidence") && i.heard.iter().any(|h| h.kind == "assumption" && h.text == "Companies will pay £2,000 per event") && i.heard.iter().any(|h| h.kind == "question"), format!("{:?}", i.heard));
    t.check("the next question builds on the answers so far", i.qas.len() == 2 && i.cursor == 1 && i.qas[1].target == "evidence", i.qas.len());
    // user corrects a heard item: never overwritten
    if let Some(hh) = i.heard.iter_mut().find(|h| h.kind == "question") {
        hh.kind = "assumption".into();
        hh.corrected = true;
    }
    i.qas[1].answer = "Two HR managers asked for a quote".into();
    let out = run_answer(cfg.clone(), &i, Some(&i.qas[1].id.clone()), true);
    apply_answer(&mut i, out);
    t.check("the model can say coverage is good enough to stop", i.suggest_stop, "no stop suggestion");
    t.check("a user's correction survives re-classification", i.heard.iter().any(|h| h.corrected && h.kind == "assumption"), "lost correction");
    // editing an earlier answer flags later answers built on it
    i.qas[1].depends_on = vec![q1.id.clone()];
    i.flag_dependents(&q1.id);
    t.check("changing an earlier answer flags later answers built on it", i.qas[1].needs_review, "not flagged");
    // understanding: outcome stays in the user's words
    let (u, _) = run_understanding(cfg.clone(), &i);
    let u = u.unwrap_or_default();
    t.check("confirm cards keep the outcome in the user's own words; an assumption they never stated is dropped", u.assumptions.is_empty() && u.outcome =="HR teams — they'd rebook every quarter. Two HR managers asked for a quote; companies will pay £2,000 per event; not sure whether bookings repeat.", format!("{:?} {:?}", u.outcome, u.assumptions));
    i.understanding = Some(u);
    i.outcome_confirmed = true;
    let (p, calls) = run_proposal(cfg.clone(), &i, None);
    t.check("the proposal is written in two parallel calls (plan / validation); a part breaking the rules is retried", p.is_ok() && calls.len() == 3 && calls.iter().any(|c| c.purpose == "intake:proposal:plan") && calls.iter().any(|c| c.purpose == "intake:proposal:validation"), format!("{} calls, {:?}", calls.len(), p.as_ref().err()));
    t.check("the next move and why come back in second person", p.as_ref().map_or(false, |x| x.next_move == "Send both HR contacts a quote" && x.why.starts_with("You already have")), format!("{:?}", p.as_ref().map(|x| x.why.clone())));
    t.check("card text never says 'the user' and starts with a capital", journey::humanise("the user has sketched some screens but hasn't built anything") == "You have sketched some screens but haven't built anything", journey::humanise("the user has sketched some screens but hasn't built anything"));
    let p = p.unwrap_or_default();
    t.check("proposal name is short and real", p.name == "Doom Clock corporate events", &p.name);
    t.check("long task titles are shortened, the detail kept", p.tasks.first().map_or(false, |x| x.title.split_whitespace().count() <= 8 && x.detail.contains("12-person")), format!("{:?}", p.tasks.first()));
    t.check("'evidence' without cited heard evidence is downgraded to assumption", p.roadmap.get(1).map_or(false, |r| r.basis == "assumption") && p.hypotheses.first().map_or(false, |h| h.basis == "assumption" && h.title == "HR teams will pay £2,000"), format!("{:?}", p.roadmap.get(1)));
    // regenerate one section only
    let mut p2 = p.clone();
    p2.roadmap.clear();
    let (fresh, _) = run_proposal(cfg.clone(), &i, Some("roadmap"));
    merge_section(&mut p2, fresh.unwrap_or_default(), "roadmap");
    t.check("one section can be regenerated without touching the others", p2.roadmap.len() == 3 && p2.tasks == p.tasks, p2.roadmap.len());
    i.proposal = Some(p.clone());
    stop.store(true, Ordering::SeqCst);
    let _ = h.join();

    // fallback when the AI is unreachable
    let mut dead = Intake::new();
    dead.provider = "openai|x|http://127.0.0.1:9".into();
    dead.input = "x".into();
    let out = run_answer(journey::cfg_for(&dead), &dead, None, true);
    apply_answer(&mut dead, out);
    t.check("if the AI is unreachable the intake continues with the fixed question set, and says so", dead.qas.first().map_or(false, |q| q.source == "fixed question set") && dead.last_error.contains("fixed question set"), &dead.last_error);

    // ---- adopt into the existing stores
    let mut s = Store::open(meta.clone());
    let iid = i.id.clone();
    s.data.intakes.push(i.clone());
    let cs = s.data.record_change("Adopted", "adopt", |d| {
        let _ = journey::adopt_all(d, &iid);
    });
    let pid = s.data.intakes.iter().find(|x| x.id == iid).and_then(|x| x.project_id.clone()).unwrap_or_default();
    let d = &s.data;
    t.check("Adopt creates the project with the confirmed short name", d.project(&pid).map_or(false, |p| p.name == "Doom Clock corporate events" && p.sections == vec!["Now", "Next", "Later"]), &pid);
    let rm: Vec<_> = d.records.iter().filter(|r| r.project_id.as_deref() == Some(&pid) && r.kind == Kind::Roadmap).collect();
    t.check("roadmap items are records in Now/Next/Later with their basis", rm.len() == 3 && rm.iter().any(|r| r.field("phase") == "Later") && rm.iter().all(|r| !r.field("basis").is_empty()), rm.len());
    let tasks: Vec<_> = d.tasks.iter().filter(|x| x.project_id.as_deref() == Some(&pid)).collect();
    t.check("tasks have short titles, detail in notes, basis as a label, and link to their roadmap item", tasks.len() == 2 && tasks.iter().all(|x| x.title.split_whitespace().count() <= 8) && tasks.iter().any(|x| x.notes.contains("12-person")) && tasks.iter().all(|x| x.labels.len() == 1) && rm.iter().any(|r| !r.links.is_empty()), tasks.len());
    let h0 = d.records.iter().find(|r| r.project_id.as_deref() == Some(&pid) && r.kind == Kind::Hypothesis && r.field("type") == "H0");
    let hx = d.records.iter().find(|r| r.project_id.as_deref() == Some(&pid) && r.kind == Kind::Hypothesis && r.parent_id.is_some());
    let e0 = d.records.iter().find(|r| r.project_id.as_deref() == Some(&pid) && r.kind == Kind::Experiment);
    t.check("H0 with NH0, Hx with NHx under it, E0 under its hypothesis with thresholds set", h0.map_or(false, |h| !h.field("negation").is_empty()) && hx.map_or(false, |h| h.parent_id.as_ref() == h0.map(|x| &x.id) && !h.field("negation").is_empty()) && e0.map_or(false, |e| e.parent_id.as_ref() == hx.map(|x| &x.id) && e.field("support_if") == ">=2 accept"), "tree");
    t.check("heard evidence and assumptions stay visible after adoption, never merged", d.records.iter().any(|r| r.project_id.as_deref() == Some(&pid) && r.kind == Kind::Evidence) && d.records.iter().any(|r| r.project_id.as_deref() == Some(&pid) && r.kind == Kind::Assumption && r.status == "Untested"), "missing");
    let brief = d.project(&pid).and_then(|p| p.page_id.clone()).and_then(|x| d.page(&x).cloned());
    let transcript = d.pages.iter().find(|p| p.title.starts_with("Intake transcript"));
    t.check("the brief is the project's page; the transcript keeps why each question was asked", brief.as_ref().map_or(false, |b| b.title == "Project brief") && transcript.map_or(false, |p| p.plain_text().contains("Why asked") && p.parent == brief.as_ref().map(|b| b.id.clone())), "pages");
    let ctx = d.context_for(&pid).cloned().unwrap_or_default();
    t.check("the project context carries the confirmed outcome; the reveal hasn't been shown yet", ctx.outcome_confirmed && ctx.outcome.contains("rebook") && !ctx.reveal_seen && ctx.win_task_id.is_some(), format!("{:?}", ctx.outcome));
    let r = s.data.undo_change(cs.as_deref().unwrap_or(""));
    t.check("Undo after Adopt removes everything Adopt wrote", r.is_ok() && s.data.project(&pid).is_none() && !s.data.records.iter().any(|x| x.project_id.as_deref() == Some(&pid)), format!("{r:?}"));
    let _ = s.save();
    let md = journey::export_markdown(&i);
    t.check("a run exports as Markdown (questions, why, heard, proposal, AI calls)", md.contains("Why:") && md.contains("## Proposal: Doom Clock corporate events") && md.contains("intake:next-question"), md.len());
}
