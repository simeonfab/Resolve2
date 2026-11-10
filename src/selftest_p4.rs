//! Phase 4 checks (project sections), on the canvases' café sample.

use crate::selftest::T;

pub fn run(t: &mut T) {
    println!("\n-- Phase 4: project sections --");
    let mut d = crate::model::WorkspaceData::default();
    crate::seed::seed_canvas(&mut d);
    let pid = d.projects.iter().find(|p| p.name == "Café flat-fee bookkeeping").map(|p| p.id.clone()).unwrap_or_default();
    let m = crate::ui::overview::overview_model(&d, &pid);
    // C-E1: the next move says what it tests and the gate it opens, only from links in the data
    t.check("C-E1: the next move names the hypothesis it tests and the gate it opens", m.move_line == "Tests H1, monthly workload is predictable. Its result opens the gate on Price to absorb spikes.", &m.move_line);
    t.check("C-E1: progress counts tasks, roadmap phases and hypotheses (H0 not counted)", m.total > 0 && (m.now, m.next, m.later) == (2, 2, 2) && m.hyp_total == 4 && m.hyp_resolved == 0 && m.exp_planned == 3, format!("{} {} {} {} {} {}", m.total, m.now, m.next, m.later, m.hyp_total, m.exp_planned));
    t.check("C-E1: what matters puts the waiting task first, in terracotta", m.matters.first().map_or(false, |x| x.0.starts_with("Hours export from The Corner Café's till system has been waiting") && x.1), format!("{:?}", m.matters.first()));
    t.check("C-E1: the board shows each phase's open tasks with a count", m.board.len() == 3 && m.board.iter().all(|(_, n, cards)| cards.len() == (*n).min(3)), format!("{:?}", m.board.iter().map(|b| b.1).collect::<Vec<_>>()));
    // without the links, nothing is guessed: the line falls back to the intake's own reason
    let mut d2 = d.clone();
    for r in d2.records.iter_mut() {
        r.links.retain(|_| false);
        r.fields.remove("gate");
    }
    let m2 = crate::ui::overview::overview_model(&d2, &pid);
    t.check("C-E1: with no links, the next move's line is the intake's reason, not a guess", !m2.move_line.starts_with("Tests") && m2.move_line.starts_with("Whether a flat £150"), &m2.move_line);
    // empty: a project with no tasks and no roadmap has no plan
    let mut d3 = crate::model::WorkspaceData::default();
    d3.projects.push(crate::model::Project { id: "bare".into(), name: "Bare".into(), color: [0, 0, 0], sections: vec![], page_id: None, archived: false, favorite: false, order: 1, updated: 0 });
    t.check("C-E1: a project with no tasks and no roadmap shows the empty state", !crate::ui::overview::overview_model(&d3, "bare").has_plan, "");

    // ---- Roadmap (C-E4 to C-E8)
    use crate::ui::roadmap::{move_theme, roadmap_model};
    let today = crate::model::today();
    let r = roadmap_model(&d, &pid, today, crate::model::now_ms());
    let lanes: Vec<usize> = ["Now", "Next", "Later"].iter().map(|l| r.themes.iter().filter(|t| t.lane == *l).count()).collect();
    t.check("C-E4: six themes, two in each phase, each with its tag and task count", r.themes.len() == 6 && lanes == vec![2, 2, 2] && r.themes.iter().all(|x| !x.basis.is_empty()), format!("{lanes:?}"));
    let price = r.themes.iter().find(|x| x.title == "Price to absorb spikes");
    t.check("C-E5: the gate chip reads \"H1 result\" and is closed while H1 is unresolved", price.and_then(|x| x.gate.as_ref()).map_or(false, |g| g.1 == "H1 result" && !g.2), format!("{:?}", price.map(|x| &x.gate)));
    // the gate opens when the hypothesis is Supported
    let mut d4 = d.clone();
    if let Some(h) = d4.records.iter_mut().find(|x| x.title.starts_with("H1:") && x.project_id.as_deref() == Some(pid.as_str())) {
        h.status = "Supported".into();
    }
    let r4 = roadmap_model(&d4, &pid, today, crate::model::now_ms());
    let open = r4.themes.iter().find(|x| x.title == "Price to absorb spikes").and_then(|x| x.gate.as_ref()).map_or(false, |g| g.2);
    t.check("C-E5: a Supported hypothesis opens its gate and the wait leaves the rail", open && !r4.waits.iter().any(|w| w.starts_with("Price to absorb spikes")), format!("{:?}", r4.waits));
    let measure = r.themes.iter().find(|x| x.title == "Measure true monthly hours");
    let h1 = r.hyps.iter().find(|h| h.1 == "H1").map(|h| h.0.clone());
    t.check("C-E6: the whole picture puts a theme on the row of the hypothesis its experiment tests", measure.map_or(false, |x| x.hyp == h1 && x.exp.as_ref().map_or(false, |e| e.1 == "E0")), format!("{:?}", measure.map(|x| (&x.hyp, &x.exp))));
    // C-E7: a move through history can be undone; a move of a theme that is gone changes nothing
    let mut d5 = d.clone();
    let pid5 = price.map(|x| x.id.clone()).unwrap_or_default();
    let cs = d5.record_change("Moved \"Price to absorb spikes\" to Later", "edit", |dd| {
        let _ = move_theme(dd, &pid5, "Later");
    });
    let moved = d5.record(&pid5).map_or(false, |x| x.field("phase") == "Later");
    let undone = cs.as_ref().map_or(false, |c| d5.undo_change(c).is_ok()) && d5.record(&pid5).map_or(false, |x| x.field("phase") == "Next");
    t.check("C-E7: a phase move is one undoable change", moved && undone, format!("moved {moved}, undone {undone}"));
    let before = serde_json::to_string(&d5.records).unwrap_or_default();
    let gone = move_theme(&mut d5, "no-such-theme", "Later");
    t.check("C-E7: a move that can't be made changes nothing and says why", gone.is_err() && serde_json::to_string(&d5.records).unwrap_or_default() == before, format!("{gone:?}"));
    t.check("C-E8: milestones are listed by date with their dates", r.milestones.len() == 3 && r.milestones[0].0 == "12 months of hours in hand" && !r.milestones[0].1.is_empty(), format!("{:?}", r.milestones));
    t.check("C-E8: the rail shows the waiting task against its theme, and what waits on a result", r.blocked.iter().any(|b| b.starts_with("Hours export") && b.ends_with("Measure true monthly hours")) && r.waits.iter().any(|w| w == "Price to absorb spikes waits on the E0 result."), format!("{:?} {:?}", r.blocked, r.waits));
    t.check("C-E8: coming up lists dated tasks in the next two weeks, then work behind a closed gate", r.coming.first().map_or(false, |c| c.0 == "Mark the busiest and quietest months") && r.coming.iter().any(|c| c.1 == "after the gate"), format!("{:?}", r.coming));

    // ---- Frameworks (C-E9, C-E10)
    use crate::ui::frameworks::{apply_result, draft_of, frameworks_model, save_node};
    let f = frameworks_model(&d, &pid);
    let labels: Vec<&str> = f.nodes.iter().map(|n| n.label.as_str()).collect();
    t.check("C-E9: the tree reads H0, H1, E0, H2, E1, H3, H4, E2 with H0 as its root", labels == vec!["H0", "H1", "E0", "H2", "E1", "H3", "H4", "E2"] && f.nodes[0].root && (f.hyps, f.exps, f.resolved) == (4, 3, 0), format!("{labels:?} {} {} {}", f.hyps, f.exps, f.resolved));
    let h2 = f.nodes.iter().find(|n| n.label == "H2");
    let e1 = f.nodes.iter().find(|n| n.label == "E1");
    t.check("C-E9: each hypothesis shows its negation and each experiment what it holds on", h2.map_or(false, |n| n.sub.starts_with("NH2:")) && e1.map_or(false, |n| n.sub == "Holds if at least one signs within two weeks"), format!("{:?} / {:?}", h2.map(|n| &n.sub), e1.map(|n| &n.sub)));
    // C-E10: result → status → gate → toast
    let mut d6 = d.clone();
    let e1_id = e1.map(|n| n.id.clone()).unwrap_or_default();
    let h2_id = h2.map(|n| n.id.clone()).unwrap_or_default();
    let mut toast = String::new();
    let cs = d6.record_change("result", "edit", |dd| toast = apply_result(dd, &e1_id, "supports").unwrap_or_default());
    let st = d6.record(&h2_id).map(|x| x.status.clone()).unwrap_or_default();
    let gate_open = roadmap_model(&d6, &pid, today, crate::model::now_ms()).themes.iter().find(|x| x.title == "Convert the two willing clients").and_then(|x| x.gate.as_ref()).map_or(false, |g| g.2);
    t.check("C-E10: a supporting result sets the experiment Done, H2 Supported, opens its gate, and the toast says so in that order", d6.record(&e1_id).map_or(false, |x| x.status == "Done" && x.field("result") == "supports") && st == "Supported" && gate_open && toast == "E1 result recorded: supports. H2 is now Supported. The gate on Convert the two willing clients is open.", &toast);
    let undone = cs.as_ref().map_or(false, |c| d6.undo_change(c).is_ok()) && d6.record(&h2_id).map_or(false, |x| x.status == "Unresolved");
    t.check("C-E10: the whole result is one undoable change", undone, "");
    let mut d7 = d.clone();
    let msg = apply_result(&mut d7, &e1_id, "inconclusive").unwrap_or_default();
    t.check("C-E10: an inconclusive result leaves the hypothesis and its gate as they were", d7.record(&h2_id).map_or(false, |x| x.status == "Unresolved") && msg.ends_with("The gate on Convert the two willing clients stays closed."), &msg);
    // a save that can't be made writes nothing
    let mut d8 = d.clone();
    let mut dr = d8.record(&h2_id).map(draft_of).unwrap_or_default();
    dr.status = "Maybe".into();
    let before = serde_json::to_string(&d8.records).unwrap_or_default();
    let bad = save_node(&mut d8, &dr);
    t.check("C-E9: a node save that can't be made writes nothing and says why", bad.is_err() && serde_json::to_string(&d8.records).unwrap_or_default() == before, format!("{bad:?}"));
    dr.status = "Supported".into();
    dr.statement = "Café clients want a flat monthly fee over hourly billing".into();
    let ok = save_node(&mut d8, &dr).unwrap_or_default();
    // ---- Knowledge base (C-E12 to C-E15)
    use crate::ui::kb::{kb_counts, kb_items, KbTarget};
    let docs = kb_items(&d, &pid, 0, today);
    t.check("C-E12: Brief and documents lists the project brief, pinned, then the pages under it", docs.iter().any(|i| i.title == "Project brief" && i.meta == "Pinned") && docs.iter().any(|i| i.title == "Plan" && i.kind == "Page"), format!("{:?}", docs.iter().map(|i| (&i.title, &i.meta)).collect::<Vec<_>>()));
    let recs = kb_items(&d, &pid, 3, today);
    let hyps: Vec<&crate::ui::kb::KbItem> = recs.iter().filter(|i| i.record_kind == Some(crate::resolve::Kind::Hypothesis)).collect();
    t.check("C-E12: Evidence and Records counts each kind and names hypotheses with their tests", hyps.len() == 5 && hyps.iter().any(|i| i.title == "H0 · A fixed-price package works" && i.snippet == "Unresolved · 4 under it") && hyps.iter().any(|i| i.title.starts_with("H3 ·") && i.snippet.ends_with("no test yet")), format!("{:?}", hyps.iter().map(|i| (&i.title, &i.snippet)).collect::<Vec<_>>()));
    let (np, nr) = kb_counts(&d, &pid);
    t.check("C-E12: the grounding bar's numbers are the project's pages and records", np == docs.len() + kb_items(&d, &pid, 1, today).len() && nr == recs.len(), format!("{np} {nr}"));
    t.check("C-E12: every card opens something real", docs.iter().all(|i| matches!(&i.target, KbTarget::Page(x) if d.page(x).is_some())) && recs.iter().all(|i| matches!(&i.target, KbTarget::Record(x) if d.record(x).is_some())), "");
    {
        let mut reg = crate::store::Registry::load();
        let ws = reg.create("Phase 4 checks", [200, 160, 64]);
        let mut s = crate::store::Store::open(ws.clone());
        // a scratch workspace: taken off the list again so the later checks see the list as before
        reg.workspaces.retain(|w| w.id != ws.id);
        reg.save();
        let out = crate::mcp::call(&mut s, "import_pages", &serde_json::json!({"pages": [{"title": "Parent page"}, {"title": "Child", "parent_title": "Parent page"}, {"title": "Stray", "parent_title": "No such page"}]})).unwrap_or_default();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap_or_default();
        let stray = v["imported_pages"].as_array().and_then(|a| a.iter().find(|x| x["title"] == "Stray")).cloned().unwrap_or_default();
        let child_ok = s.data.pages.iter().find(|x| x.title == "Child").and_then(|x| x.parent.clone()).is_some();
        t.check("C-E15: import_pages reports a parent it can't find for that page and files the rest", child_ok && stray["note"].as_str().map_or(false, |n| n.contains("\"No such page\" not found")) && v["pages_with_notes"] == 1, &out);
    }

    // ---- Databases: the CSV import dialog (C-E18)
    {
        use crate::import::{csv_append, csv_check, csv_new_database, csv_plan};
        use crate::model::ColKind;
        let mut text = String::from("date,staff,hours,task\n");
        for k in 0..36 {
            let h = match k {
                12 => "six".to_string(),
                13 => "n/a".to_string(),
                _ => format!("{}", 2 + k % 5),
            };
            text.push_str(&format!("2025-11-{:02},Ana,{h},Receipts\n", k % 28 + 1));
        }
        text.push_str("2025-12-01,Sam,3,VAT,extra cell\n");
        let plan = csv_plan(&text).unwrap_or_default();
        t.check("C-E18: the dialog reads 37 rows and 4 columns and guesses date, number", plan.rows.len() == 37 && plan.header.len() == 4 && plan.kinds[0] == ColKind::Date && plan.kinds[2] == ColKind::Number, format!("{} {:?}", plan.rows.len(), plan.kinds));
        let (ok, failed) = csv_check(&plan, &plan.kinds);
        t.check("C-E18: rows with text in a number column, or more cells than columns, are listed with their line and why", ok.len() == 34 && failed.len() == 3 && failed[0] == (14, "\"six\" in hours isn't a number".to_string()) && failed[2].1.starts_with("it has 5 cells"), format!("{} {:?}", ok.len(), failed));
        let db = csv_new_database("Hours", &plan, &plan.kinds, &ok, false);
        t.check("C-E18: the rest become a new database with those types", db.rows.len() == 34 && db.columns.iter().map(|c| c.kind).collect::<Vec<_>>() == plan.kinds, db.rows.len());
        let mut db2 = db.clone();
        let idx: Vec<usize> = failed.iter().map(|(n, _)| n - 2).collect();
        let n = csv_append(&mut db2, &plan, &plan.kinds, &idx, true);
        let hours = db2.columns[2].id.clone();
        t.check("C-E18: Import them as text keeps the values as they were written", n == 3 && db2.rows.iter().any(|r| r.cells.get(&hours).map_or(false, |v| v == "six")), n);
        let mut text_kinds = plan.kinds.clone();
        text_kinds[2] = ColKind::Text;
        let (ok2, failed2) = csv_check(&plan, &text_kinds);
        t.check("C-E18: changing a column's type in the dialog changes which rows fail", ok2.len() == 36 && failed2.len() == 1, format!("{} {}", ok2.len(), failed2.len()));
    }

    // ---- AI Team (C-E19)
    {
        use crate::resolve::team::{apply_proposal, find_parent, mark_proposal, split_proposals};
        let msg = "No. Ask both what they pay now.\nPROPOSED: question | H2 | What does each café pay you now per year?\nPROPOSED: nonsense line";
        let (text, props) = split_proposals(msg);
        t.check("C-E19: a role's PROPOSED line becomes a card; the text keeps the rest", props.len() == 1 && props[0].on == "H2" && props[0].kind == "question" && text.contains("No. Ask both") && !text.contains("café pay") && text.contains("nonsense"), format!("{props:?}"));
        let mut d9 = d.clone();
        let parent = find_parent(&d9, &pid, "H2");
        let before = d9.records.len();
        let cs = d9.record_change("Added", "add", |dd| apply_proposal(dd, Some(&pid), &props[0], parent.as_deref()));
        let added = d9.records.iter().find(|r| r.title == "What does each café pay you now per year?").cloned();
        t.check("C-E19: Add it puts the question under H2, as one undoable change", parent.is_some() && added.as_ref().map_or(false, |r| r.kind == crate::resolve::Kind::Question && r.parent_id == parent) && cs.as_ref().map_or(false, |c| d9.undo_change(c).is_ok()) && d9.records.len() == before, "");
        let marked = mark_proposal(msg, 0, "ADDED");
        let (_, p2) = split_proposals(&marked);
        t.check("C-E19: the message remembers that it was added", p2.len() == 1 && p2[0].decided == "ADDED", &marked);
        // a page proposal (Weekly review) saves a page under the project's brief
        let (_, pp) = split_proposals("Here it is.\nPROPOSED: page | Weekly review 6 Nov | Done: three tasks. / Next: the price call.");
        let mut d10 = d.clone();
        if let Some(x) = pp.first() {
            apply_proposal(&mut d10, Some(&pid), x, None);
        }
        let saved = d10.pages.iter().find(|x| x.title == "Weekly review 6 Nov").cloned();
        let brief = crate::ui::kb::brief_of(&d10, &pid);
        t.check("C-E19: a proposed page is saved under the project brief, one paragraph per line", saved.as_ref().map_or(false, |x| x.blocks.len() == 2 && x.parent == brief), format!("{:?}", saved.map(|x| x.blocks.len())));
        t.check("C-E19: every role is told to propose, and no role's instructions say it can change things", crate::resolve::team::PROPOSE_RULE.contains("you propose") && !crate::resolve::team::role_system(&d, "tech-lead", Some(&pid)).contains("read and change"), "");
        t.check("C-E19: the budget message is recognised in the chat", crate::ai::is_limit("This month's AI budget ($10) is used up. Raise it in Settings, AI and MCP, or carry on by hand."), "");
    }
    // ---- Handoffs (C-E20)
    {
        let ctx = d.context_for(&pid).cloned().unwrap_or_default();
        let b = crate::resolve::roles::handoff_for(&d, &ctx, "Maya", "Pull 12 months of hours for each café from the till exports");
        let parts: Vec<String> = crate::resolve::roles::brief_sections(&b).into_iter().map(|x| x.0).collect();
        t.check("C-E20: the brief has five parts: The job, What already exists, Why it matters, Off limits, Done when", parts == vec!["The job", "What already exists", "Why it matters", "Off limits", "Done when"], format!("{parts:?}"));
        t.check("C-E20: the cut list is in Off limits", b.contains("Not yet: Payroll add-on"), "");
    }

    // ---- approval follow-ups (10 Nov): cut list Undo, Overview wording
    {
        let mut d11 = d.clone();
        let cid = d11.context_for(&pid).map(|c| c.id.clone()).unwrap_or_default();
        let n = d11.context_for(&pid).map(|c| c.recommendation.do_not_yet.len()).unwrap_or(0);
        let cs = d11.record_change("Removed", "delete", |dd| crate::ui::kb::cut_remove(dd, &cid, 0));
        let gone = d11.context_for(&pid).map(|c| c.recommendation.do_not_yet.len()).unwrap_or(0) == n.saturating_sub(1);
        let back = cs.as_ref().map_or(false, |c| d11.undo_change(c).is_ok()) && d11.context_for(&pid).map(|c| c.recommendation.do_not_yet.len()).unwrap_or(0) == n;
        t.check("removing an item from the cut list can be undone", n > 0 && gone && back, format!("{n} {gone} {back}"));
        let lines: Vec<String> = m.matters.iter().map(|x| x.0.clone()).collect();
        t.check("What matters now has no \"Run:\" prefix; a planned experiment says it hasn't started", !lines.iter().any(|l| l.starts_with("Run:")) && lines.iter().any(|l| l.starts_with("E0 hasn't started yet: reconstruct 12 months")), format!("{lines:?}"));
    }

    // ---- Phase 3 fixes (10 Nov): a theme without a reason is asked for again
    {
        use crate::resolve::journey::validate_proposal;
        let i = crate::resolve::intake::Intake::new();
        let u = crate::resolve::intake::Understanding::default();
        let v = serde_json::json!({"name": "Café flat fee", "summary": "s", "objective": "o", "h0": {"title": "A fixed fee works"}, "next_move": "Pull the hours you logged for each café", "why": "You already have the numbers in your records.", "roadmap": [{"title": "Measure true monthly hours", "detail": "", "phase": "Now"}], "tasks": [{"title": "Export the till hours", "detail": "d", "phase": "Now", "parent": 0}], "pages": [{"title": "Project brief", "detail": "## Café flat fee"}]});
        let r = validate_proposal(&v, &i, &u).and_then(|p| crate::resolve::journey::quality_check(&v, &p));
        t.check("screen 07: a roadmap theme with no reason line is sent back to the AI", r.as_ref().err().map_or(false, |e| e.contains("needs a \"detail\"")), format!("{:?}", r.err()));
    }

    // ---- Proposal editor (C-E11)
    let full = crate::seed::cafe_proposal();
    let thin = crate::resolve::journey::Proposal { name: full.name.clone(), outcome: full.outcome.clone(), ..Default::default() };
    t.check("C-E11: the editor shows its empty state only when the proposal has nothing to edit", !crate::ui::proposal_editor::proposal_is_thin(&full) && crate::ui::proposal_editor::proposal_is_thin(&thin), "");
    t.check("C-E9: saving a node keeps its label and reports the gate it opens", d8.record(&h2_id).map_or(false, |x| x.title == "H2: Café clients want a flat monthly fee over hourly billing") && ok == "H2 is now Supported. The gate on Convert the two willing clients is open.", &ok);
}
