//! End-to-end checks for the Resolve layer, run by `resolve2 --selftest`.
//!
//! The model is a scripted STAND-IN server on 127.0.0.1 (OpenAI-compatible), so these checks
//! exercise the real prompt → JSON → validation → workspace code paths without any API key.
//! The stand-in's answers are fixed test data; they are not, and never appear as, real AI output.

use crate::model::*;
use crate::resolve::{assess, intake, roles, Kind, Record};
use crate::selftest::T;
use crate::store::{Store, WorkspaceMeta};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

fn reply_for(system: &str, bad_budget: &AtomicUsize) -> Value {
    let content: Value = if system.starts_with("You are operating as Resolve's project-plan handoff writer") {
        json!({
            "longTermRoadmap": [{"phase": "Grow", "direction": "Subscriptions", "note": "extrapolation"}],
            "readiness": [{"area": "Direction is clear", "status": "Ready", "note": "n"}, {"area": "Planning is usable", "status": "Partially ready", "note": "n"},
                          {"area": "Ready to act", "status": "Ready", "note": "n"}, {"area": "Evidence is strong enough", "status": "Not ready", "note": "n"}],
            "risks": [{"risk": "Shipping breakage", "whyItMatters": "Refunds eat margin", "sourceSignal": "glass jars"}],
            "sharingParagraph": "A candle maker testing online sales.",
            "howToUseThis": "Copy the next action into ChatGPT or Claude to draft the email."
        })
    } else if system.starts_with("You are operating as the ResolveOS Business Analyst role") {
        if system.contains("BADSHAPE") && bad_budget.fetch_add(1, Ordering::SeqCst) < 1 {
            json!({"project": ""}) // wrong shape once → the app must retry
        } else if system.contains("ALWAYSBAD") {
            json!({"nope": true})
        } else {
            json!({
                "project": "A handmade candle shop moving from market stalls to online sales.",
                "goal": "Sell candles online without depending on weekend markets.",
                "state": "An Etsy listing exists but orders are rare; notes are scattered across Notion and old ChatGPT chats.",
                "uncertainty": "Whether online buyers will pay £30 for a candle.",
                "outcome": "Probably: repeat customers order online without being chased.",
                "outcomeStated": false,
                "outcomeQuestion": "If the online shop works, who is better off, and how would you know?",
                "gapQuestions": ["Is there a deadline driving this?", "What have you already tried?"],
                "assumptions": ["Market customers would buy online"],
                "mentionedTools": ["Shopify"]
            })
        }
    } else if system.contains("strict validation check") {
        if system.contains("they would be happier") {
            json!({"hasBeneficiary": true, "hasObservableChange": false})
        } else {
            json!({"hasBeneficiary": true, "hasObservableChange": true})
        }
    } else if system.starts_with("You are operating as the ResolveOS Strategic Product Director role") {
        json!({
            "recommendedAction": "Email the 40 market regulars a link to the online shop with a £5 first-order code",
            "why": "They already buy at £30 in person; it's the cheapest test of whether they will buy online.",
            "milestone": "At least 5 regulars place an online order within two weeks",
            "openDecision": "Keep Etsy or move to an own-domain shop",
            "roadmap": [{"phase": "Now", "action": "Email the regulars", "output": "First online orders"},
                        {"phase": "Define", "action": "Pick the shop platform", "output": "A decision"},
                        {"phase": "Launch", "action": "Run a small paid ad test", "output": "Cost per order"}],
            "topPriorities": ["Test online demand with existing customers"],
            "doNotYet": [{"item": "Redesign the brand", "reason": "Demand isn't proven"}],
            "topCardType": "action",
            "winSummary": "A messy candle shop move — turned into a focused path forward. That's the hard part done."
        })
    } else if system.contains("project-plan handoff writer") {
        json!({
            "longTermRoadmap": [{"phase": "Grow", "direction": "Subscriptions", "note": "extrapolation"}],
            "readiness": [{"area": "Direction is clear", "status": "Ready", "note": "n"}, {"area": "Planning is usable", "status": "Partially ready", "note": "n"},
                          {"area": "Ready to act", "status": "Ready", "note": "n"}, {"area": "Evidence is strong enough", "status": "Not ready", "note": "n"}],
            "risks": [{"risk": "Shipping breakage", "whyItMatters": "Refunds eat margin", "sourceSignal": "glass jars"}],
            "sharingParagraph": "A candle maker testing online sales.",
            "howToUseThis": "Copy the next action into ChatGPT or Claude to draft the email."
        })
    } else if system.contains("orchestration router") {
        json!({"responders": ["growth-lead", "product-director", "not-a-role"], "reason": "pricing + direction"})
    } else if system.contains("speaking directly to the founder") {
        json!({"reply": "Lead the email with the code; regulars respond to familiarity."})
    } else if system.contains("Create a validation plan") {
        json!({"summary": "Two decision-sensitive assumptions, one test.", "items": [
            {"kind": "hypothesis", "title": "Regulars will buy online at £30", "fields": {"negation": "Regulars will not buy online at £30"}},
            {"kind": "experiment", "title": "Email test to 40 regulars", "parent": 0, "fields": {"prediction": ">=5 orders in 14 days", "support_if": ">=5 orders", "refute_if": "<2 orders"}},
            {"kind": "spaceship", "title": "invalid kind is dropped"}
        ]})
    } else {
        json!({"error": "unexpected prompt"})
    };
    json!({"choices": [{"message": {"role": "assistant", "content": content.to_string()}}], "usage": {"prompt_tokens": 1000, "completion_tokens": 200}})
}

fn fake_server(port: u16) -> (Arc<AtomicBool>, std::thread::JoinHandle<()>) {
    let stop = Arc::new(AtomicBool::new(false));
    let s2 = stop.clone();
    let server = tiny_http::Server::http(("127.0.0.1", port)).expect("fake model server");
    let h = std::thread::spawn(move || {
        let bad = AtomicUsize::new(0);
        while !s2.load(Ordering::SeqCst) {
            if let Ok(Some(mut req)) = server.recv_timeout(std::time::Duration::from_millis(100)) {
                let mut body = String::new();
                let _ = std::io::Read::read_to_string(req.as_reader(), &mut body);
                let v: Value = serde_json::from_str(&body).unwrap_or(json!({}));
                let system = v["messages"][0]["content"].as_str().unwrap_or("").to_string();
                let out = reply_for(&system, &bad);
                let _ = req.respond(tiny_http::Response::from_string(out.to_string()).with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()));
            }
        }
    });
    (stop, h)
}

pub fn run(t: &mut T, meta: WorkspaceMeta, port: u16) {
    println!("\n-- Resolve layer --");
    let (stop, handle) = fake_server(port);
    let cfg = crate::ai::AiConfig { provider: "openai".into(), url: format!("http://127.0.0.1:{port}"), model: "stand-in".into(), key: String::new() };

    // ---- pure rules
    t.check("tool detection is deterministic and word-bounded", intake::detect_tools("We use notion, GitHub and Claude; not Notional") == vec!["Notion", "GitHub", "Claude"], format!("{:?}", intake::detect_tools("We use notion, GitHub and Claude; not Notional")));
    let u = intake::validate_understanding(&json!({"project": "p", "goal": "g", "state": "s", "uncertainty": "u", "outcome": "o", "outcomeStated": false, "outcomeQuestion": "Who is better off?", "gapQuestions": ["A?", "B?"], "assumptions": []}), "");
    t.check("outcome question is always gap question 1, max 2 questions", u.as_ref().map_or(false, |u| u.gap_questions == vec!["Who is better off?".to_string(), "A?".to_string()]), format!("{u:?}"));
    let mut it = intake::Intake::new();
    let s1 = intake::apply_verdict(&mut it, "they'd be happier", Some((true, false)));
    let s2 = intake::apply_verdict(&mut it, "still vague", Some((true, false)));
    t.check("partial outcome → one re-ask, then proceed unconfirmed", matches!(s1, intake::OutcomeStep::Reask(ref m) if m.contains("how would you know")) && s2 == intake::OutcomeStep::Proceed && !it.outcome_confirmed, format!("{s1:?} {s2:?}"));
    let mut it2 = intake::Intake::new();
    t.check("judge failure never blocks (proceeds, marked unverified)", intake::apply_verdict(&mut it2, "x", None) == intake::OutcomeStep::Proceed && it2.unverified && !it2.outcome_confirmed, "blocked");

    // ---- full AI intake through the stand-in
    let mut s = Store::open(meta.clone());
    let mut i = intake::Intake::new();
    i.name = "Simeon".into();
    i.input = "I make candles and sell at markets. BADSHAPE. I keep notes in Notion and ChatGPT.".into();
    i.step = 1;
    let iid = i.id.clone();
    s.data.intakes.push(i);
    let _ = s.save();
    let und = intake::run_understand(&cfg, "I make candles and sell at markets. BADSHAPE. I keep notes in Notion and ChatGPT.");
    t.check("understand: a malformed answer is retried once, then validated", und.as_ref().map_or(false, |u| u.gap_questions.len() == 2 && u.gap_questions[0].contains("who is better off") && u.mentioned_tools == vec!["Notion", "ChatGPT"]), format!("{und:?}"));
    let bad = intake::run_understand(&cfg, "ALWAYSBAD");
    t.check("understand: a persistently malformed answer becomes a clear error (nothing invented)", bad.as_ref().err().map_or(false, |e| e.contains("right shape")), format!("{bad:?}"));
    let u = und.unwrap_or_default();
    {
        let x = s.data.intakes.iter_mut().find(|x| x.id == iid).unwrap();
        x.understanding = Some(u.clone());
        x.tools = u.mentioned_tools.clone();
        x.sync_gap_answers();
        x.gap_answers[0].answer = "My market regulars — they'd reorder online without me reminding them".into();
        x.step = 2;
    }
    let _ = s.save();
    // interrupted session: reopen from disk
    let reopened = Store::open(meta.clone());
    let ri = reopened.data.intakes.iter().find(|x| x.id == iid);
    t.check("an interrupted intake is saved mid-way and resumes at the same step with answers", ri.map_or(false, |x| x.step == 2 && x.outcome_answer().contains("regulars")), "lost");
    let x = s.data.intakes.iter().find(|x| x.id == iid).cloned().unwrap();
    let verdict = intake::run_judge(&cfg, &x.outcome_question(), &x.outcome_answer()).ok();
    let step = intake::apply_verdict(s.data.intakes.iter_mut().find(|x| x.id == iid).unwrap(), &x.outcome_answer(), verdict);
    t.check("judge (isolated call, temperature 0) confirms a beneficiary + observable change", step == intake::OutcomeStep::Proceed && s.data.intakes.iter().find(|x| x.id == iid).unwrap().outcome_confirmed, format!("{verdict:?}"));
    let rec = intake::run_recommend(&cfg, &u, &x.gap_answers, &x.tools);
    t.check("recommend returns one move, 3 phases, 1-3 priorities and not-yet items", rec.as_ref().map_or(false, |r| r.roadmap.len() == 3 && r.top_priorities.len() == 1 && r.do_not_yet.len() == 1), format!("{rec:?}"));
    let rec = rec.unwrap_or_default();
    let plan = intake::run_plan(&cfg, &u, &rec, &x.gap_answers, &x.tools);
    t.check("plan returns readiness (4 areas) and grounded risks", plan.as_ref().map_or(false, |p| p.readiness.len() == 4 && p.risks.len() == 1), format!("{plan:?}"));
    {
        let x = s.data.intakes.iter_mut().find(|x| x.id == iid).unwrap();
        x.recommendation = Some(rec.clone());
        x.plan = plan.ok();
        x.step = 4;
    }
    let pid = intake::handoff(&mut s.data, &iid);
    t.check("handoff creates a real project", pid.as_ref().map_or(false, |p| s.data.project(p).is_some()), format!("{pid:?}"));
    let pid = pid.unwrap_or_default();
    let again = intake::handoff(&mut s.data, &iid);
    t.check("handing off twice never duplicates the project", again.as_ref().ok() == Some(&pid) && s.data.projects.iter().filter(|p| p.name == s.data.project(&pid).unwrap().name).count() == 1, format!("{again:?}"));
    let proj = s.data.project(&pid).cloned().unwrap_or_default();
    let tasks: Vec<&Task> = s.data.tasks.iter().filter(|x| x.project_id.as_deref() == Some(&pid)).collect();
    let first = tasks.iter().find(|x| x.labels.contains(&"first-move".to_string()));
    t.check("project gets Now/Next/Later sections, a P1 first move and roadmap tasks with derived dates", proj.sections == vec!["Now", "Next", "Later"] && tasks.len() == 4 && first.map_or(false, |f| f.priority == 1 && f.due.is_some() && f.section == "Now"), format!("{} tasks", tasks.len()));
    t.check("project document holds the 9-section report", proj.page_id.as_ref().and_then(|p| s.data.page(p)).map_or(false, |pg| pg.plain_text().contains("Highest-leverage next action") && pg.plain_text().contains("How to use this")), "no report");
    let kinds: Vec<Kind> = s.data.records.iter().filter(|r| r.project_id.as_deref() == Some(&pid)).map(|r| r.kind).collect();
    t.check("intake knowledge becomes records (assumption, question, decision, milestone, risk, root hypothesis)", [Kind::Assumption, Kind::Question, Kind::Decision, Kind::Milestone, Kind::Risk, Kind::Hypothesis].iter().all(|k| kinds.contains(k)), format!("{kinds:?}"));
    let ctx = s.data.context_for(&pid).cloned().unwrap_or_default();
    t.check("context keeps the original words, answers, tools and confirmed outcome (never re-asked)", ctx.input.contains("candles") && ctx.outcome.contains("regulars") && ctx.outcome_confirmed && ctx.tools.len() == 2 && ctx.gap_answers.len() == 2, format!("{:?}", ctx.outcome));
    let _ = s.save();

    // ---- continuity across sessions + assessment
    let mut s = Store::open(meta.clone());
    let ctx = s.data.context_for(&pid).cloned().unwrap_or_default();
    let a = assess::assess(&s.data, &ctx);
    t.check("after reopening, the pinned next move is the intake's first move", a.win == ctx.win_task_id && a.win.is_some(), format!("{:?}", a.win));
    t.check("assessment lists the open decision and uncertainties", a.open_decisions.len() == 1 && !a.uncertain.is_empty() && a.next.iter().any(|n| n.0.starts_with("Decide:")), format!("{:?}", a.next));
    if let Some(w) = a.win.clone() {
        s.data.task_mut(&w).unwrap().complete();
    }
    let next = assess::pick_win(&s.data, &ctx);
    let nt = next.as_ref().and_then(|x| s.data.task(x)).cloned().unwrap_or_default();
    t.check("when the move is done, the next is picked by priority then Now/Next/Later", nt.section == "Now" && nt.priority == 2, format!("{} {}", nt.title, nt.section));
    let mut blocker = Record::new(Kind::Blocker, Some(pid.clone()), "Waiting for jar supplier");
    blocker.status = "Blocking".into();
    s.data.records.push(blocker);
    let a = assess::assess(&s.data, &ctx);
    t.check("a blocker goes to the top of 'what matters now'", a.next.first().map_or(false, |n| n.0.contains("Unblock")), format!("{:?}", a.next.first()));
    let _ = s.save();

    // ---- MCP: Hypothesis Zero through the tools
    let call = |s: &mut Store, tool: &str, args: Value| -> Result<Value, String> { crate::mcp::call(s, tool, &args).and_then(|x| serde_json::from_str(&x).map_err(|e| e.to_string())) };
    let v = call(&mut s, "resolve_project", json!({"project": proj.name}));
    t.check("MCP resolve_project returns context, assessment and records", v.as_ref().map_or(false, |v| v["outcome_confirmed"] == true && v["what_matters_now"].as_array().map_or(false, |a| !a.is_empty()) && v["records"].as_array().map_or(false, |a| a.len() >= 6)), format!("{:?}", v.as_ref().err()));
    let h = call(&mut s, "add_record", json!({"project": pid, "kind": "hypothesis", "title": "Gift buyers drive December sales", "fields": {"negation": "Gift buyers do not drive December sales"}}));
    let hid = h.as_ref().map(|v| v["id"].as_str().unwrap_or("").to_string()).unwrap_or_default();
    let sub = call(&mut s, "add_record", json!({"project": pid, "kind": "hypothesis", "title": "They buy bundles", "parent_id": hid}));
    let e = call(&mut s, "add_record", json!({"project": pid, "kind": "experiment", "title": "Bundle landing page", "parent_id": hid, "fields": {"prediction": ">3% add-to-cart", "support_if": ">=3%", "refute_if": "<1%"}}));
    let eid = e.as_ref().map(|v| v["id"].as_str().unwrap_or("").to_string()).unwrap_or_default();
    t.check("hypotheses nest recursively and experiments branch from any node", sub.is_ok() && e.is_ok() && assess::hypothesis_tree(&s.data, &pid).iter().any(|(d, id)| *d == 1 && id == &eid), format!("{sub:?} {e:?}"));
    let started = call(&mut s, "update_record", json!({"id": eid, "status": "Running"}));
    let tamper = call(&mut s, "update_record", json!({"id": eid, "fields": {"support_if": ">=1%"}}));
    let obs = call(&mut s, "update_record", json!({"id": eid, "fields": {"observations": "2.1% add-to-cart", "result": "inconclusive"}, "status": "Done"}));
    t.check("a running experiment's thresholds are locked; results can still be recorded", started.is_ok() && tamper.as_ref().err().map_or(false, |e| e.contains("fixed when the experiment started")) && obs.is_ok() && s.data.record(&eid).map_or(false, |r| r.field("support_if") == ">=3%"), format!("{tamper:?}"));
    let ev = call(&mut s, "add_record", json!({"project": pid, "kind": "evidence", "title": "Last December 60% of market sales were gifts", "parent_id": hid, "fields": {"nature": "fact", "direction": "supports", "quality": "own till records"}}));
    let bad_status = call(&mut s, "update_record", json!({"id": hid, "status": "Proven"}));
    t.check("evidence links to its hypothesis; only the method's evidential states are accepted", ev.is_ok() && bad_status.as_ref().err().map_or(false, |e| e.contains("Unresolved")), format!("{bad_status:?}"));
    let open = call(&mut s, "list_records", json!({"project": pid, "kind": "experiment", "open_only": true}));
    t.check("list_records filters by kind and open state", open.as_ref().map_or(false, |v| v.as_array().map_or(false, |a| a.is_empty())), format!("{open:?}"));
    s.data.delete_record(&hid);
    t.check("deleting a hypothesis keeps its children (they move up the tree)", s.data.record(&eid).map_or(false, |r| r.parent_id.is_none()), "orphaned");

    // ---- ResolveOS roles & actions
    let ctx = s.data.context_for(&pid).cloned().unwrap_or_default();
    let fenced = ctx.fenced_text(&s.data);
    let routed = roles::route(&cfg, &fenced, "How should I price the bundles?");
    t.check("router (isolated call) picks only valid leads", routed.as_ref().map_or(false, |r| r == &vec!["growth-lead", "product-director"]), format!("{routed:?}"));
    let rep = roles::reply(&cfg, "growth-lead", &fenced, &[], "How should I word the email?");
    t.check("a lead replies grounded in the shared fenced context", rep.as_ref().map_or(false, |r| r.contains("code")) && fenced.contains("FENCED PROJECT CONTEXT") && fenced.contains("Email the 40 market regulars"), format!("{rep:?}"));
    let prop = roles::propose(&cfg, "validation-plan", &ctx, &s.data);
    t.check("validation plan comes back as proposals (invalid kinds dropped)", prop.as_ref().map_or(false, |(_, items)| items.len() == 2 && items[1].parent == Some(0)), format!("{prop:?}"));
    let (_, items) = prop.unwrap_or_default();
    let before = s.data.records.len();
    let ids = roles::apply_proposals(&mut s.data, &pid, &items, &[true, true], None);
    let child = ids.get(1).cloned().flatten().and_then(|i| s.data.record(&i).cloned());
    t.check("confirmed proposals become linked records (experiment under its hypothesis)", s.data.records.len() == before + 2 && child.map_or(false, |c| c.parent_id == ids[0]), "not linked");
    let none = roles::apply_proposals(&mut s.data, &pid, &items, &[false, false], None);
    t.check("unconfirmed proposals write nothing", none.iter().all(|x| x.is_none()), "wrote");
    let brief = roles::handoff_brief(&s.data, &ctx);
    t.check("Codex/Claude handoff brief carries next move, decisions and the not-yet list", brief.contains("## Do this first") && brief.contains("Do NOT do yet") && brief.contains("Redesign the brand"), &brief[..brief.len().min(200)]);
    let _ = s.save();

    // ---- manual intake (no AI) and incomplete data
    let mut m = intake::Intake::new();
    m.manual = true;
    m.input = "x".into();
    m.understanding = Some(intake::Understanding { project: "Garden shed".into(), goal: "Build it".into(), state: "Plans drawn".into(), outcome: "My kids play outside more".into(), ..Default::default() });
    m.recommendation = Some(intake::Recommendation { recommended_action: "Buy timber".into(), ..Default::default() });
    let mid = m.id.clone();
    s.data.intakes.push(m);
    let refused = intake::handoff(&mut s.data, &mid);
    t.check("an incomplete session is refused with a clear reason (no hollow project)", refused.as_ref().err().map_or(false, |e| e.contains("incomplete")), format!("{refused:?}"));
    {
        let m = s.data.intakes.iter_mut().find(|x| x.id == mid).unwrap();
        let r = m.recommendation.as_mut().unwrap();
        r.top_priorities = vec!["Get the base level".into()];
        r.do_not_yet = vec![intake::NotYet { item: "Paint it".into(), reason: "Later".into() }];
    }
    let ok = intake::handoff(&mut s.data, &mid);
    let mctx = ok.as_ref().ok().and_then(|p| s.data.context_for(p)).cloned();
    t.check("a manual (no-AI) intake becomes a project with its outcome marked unconfirmed", mctx.map_or(false, |c| !c.outcome_confirmed && c.outcome.contains("kids")), format!("{ok:?}"));

    // ---- sync, isolation of deletes, usage
    let mut other = s.data.clone();
    if let Some(c) = other.contexts.iter_mut().find(|c| c.project_id == pid) {
        c.notes = "Edited on the phone".into();
        c.updated = now_ms() + 5000;
    }
    s.data.merge(&other);
    t.check("Resolve context merges between devices (newest edit wins)", s.data.context_for(&pid).map_or(false, |c| c.notes == "Edited on the phone"), "not merged");
    let n_before = s.data.records.iter().filter(|r| r.project_id.as_deref() == Some(&pid)).count();
    s.data.delete_project(&pid);
    t.check("deleting a project removes its context and records", n_before > 0 && s.data.context_for(&pid).is_none() && !s.data.records.iter().any(|r| r.project_id.as_deref() == Some(&pid)), "left behind");
    let _ = s.save();
    let u = crate::ai::usage_since(0);
    t.check("every stand-in model call was logged for cost tracking", u.calls >= 8 && u.input >= 8000, format!("{u:?}"));
    let mut reg = crate::store::Registry::load();
    reg.settings.price_in_per_m = 1000.0;
    reg.settings.monthly_budget = 0.01;
    reg.save();
    let blocked = intake::run_understand(&cfg, "anything");
    t.check("a reached monthly budget stops further AI calls with a clear message", blocked.as_ref().err().map_or(false, |e| e.contains("budget")), format!("{blocked:?}"));
    reg.settings.monthly_budget = 0.0;
    reg.save();

    stop.store(true, Ordering::SeqCst);
    let _ = handle.join();
}
