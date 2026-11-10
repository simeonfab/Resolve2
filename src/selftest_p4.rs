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
}
