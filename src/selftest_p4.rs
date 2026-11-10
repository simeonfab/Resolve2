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
}
