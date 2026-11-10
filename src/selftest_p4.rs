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

    // ---- Proposal editor (C-E11)
    let full = crate::seed::cafe_proposal();
    let thin = crate::resolve::journey::Proposal { name: full.name.clone(), outcome: full.outcome.clone(), ..Default::default() };
    t.check("C-E11: the editor shows its empty state only when the proposal has nothing to edit", !crate::ui::proposal_editor::proposal_is_thin(&full) && crate::ui::proposal_editor::proposal_is_thin(&thin), "");
    t.check("C-E9: saving a node keeps its label and reports the gate it opens", d8.record(&h2_id).map_or(false, |x| x.title == "H2: Café clients want a flat monthly fee over hourly billing") && ok == "H2 is now Supported. The gate on Convert the two willing clients is open.", &ok);
}
