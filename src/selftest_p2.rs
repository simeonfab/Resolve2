//! Phase 2 checks (daily screens). Home's data rules on the canvases' sample, with the clock
//! pinned to the canvases' Friday 6 November, 14:00.

use crate::selftest::T;

pub fn run(t: &mut T) {
    println!("\n-- Phase 2: daily screens --");
    let at = chrono::NaiveDate::from_ymd_opt(2026, 11, 6).and_then(|d| d.and_hms_opt(14, 0, 0)).unwrap();
    crate::model::set_clock(at);
    let mut d = crate::model::WorkspaceData::default();
    crate::seed::seed_canvas(&mut d);
    let m = crate::ui::dashboard::home_model(&d, crate::model::now_local(), crate::model::now_ms(), None);
    // back to the real clock for every later check
    crate::model::set_clock(chrono::Local::now().naive_local());

    // found timing the proposal (Phase 3, D5): a short word with an accent crashed the check
    t.check("a heard item with \"café\" in it is checked against the answer without crashing", crate::resolve::journey::grounded("cafés pay", "small cafés pay a fixed price"), "");
    // B-E8 All projects: Today by project in sidebar order, then not in a project; overdue apart
    let all = crate::ui::tasks::all_tasks_model(&d, at.date());
    let groups: Vec<String> = all.today.iter().map(|(p, ids)| format!("{}:{}", p.as_ref().and_then(|x| d.project(x)).map_or("none".to_string(), |x| x.name.clone()), ids.len())).collect();
    t.check("B-E8: All projects counts Captured 2 and Today 3, Today grouped Café, Website, then not in a project", all.captured.len() == 2 && all.today_count() == 3 && groups == ["Café flat-fee bookkeeping:1", "Website refresh:1", "none:1"] && all.overdue.len() == 1, format!("{groups:?} overdue {}", all.overdue.len()));
    let inv = d.tasks.iter().find(|x| x.title == "Send October invoices").cloned().unwrap_or_else(|| crate::model::Task::new(""));
    t.check("B-E8: a timed, repeating task due today reads \"Today 16:00 · every month\"", crate::ui::tasks::all_when(&inv, at.date(), true).0 == "Today 16:00 · every month", crate::ui::tasks::all_when(&inv, at.date(), true).0);
    t.check("sample data: the canvases' café project and Website refresh, adopted through the real Adopt", d.projects.iter().any(|p| p.name == "Café flat-fee bookkeeping") && d.projects.iter().any(|p| p.name == "Website refresh") && d.contexts.len() == 2, format!("{} projects", d.projects.len()));
    t.check("Home: greeting by time of day with the person's name", m.greeting == "Good afternoon, Simeon", &m.greeting);
    t.check("Home: subline with the date, tasks due today and events", m.subline == "Friday 6 November · 3 due today · 1 event", &m.subline);
    let h = m.hero.clone();
    t.check(
        "B-E4: the hero is the project's first move, its reason and its focus theme",
        h.as_ref().map_or(false, |h| h.project == "Café flat-fee bookkeeping" && h.title.starts_with("Pull the actual hours") && h.why.starts_with("Whether a flat £150") && h.theme == "Measure true monthly hours" && h.task.is_some()),
        format!("{h:?}"),
    );
    let times: Vec<&str> = m.agenda.iter().map(|a| a.time.as_str()).collect();
    t.check("Home: Today lists events and tasks due today by time; untimed tasks sit at now", times.first() == Some(&"10:30") && times.last() == Some(&"16:00") && times.iter().filter(|x| **x == "Today").count() == 2, format!("{times:?}"));
    t.check("B-E4: Since Monday lists what was finished this week", m.wins == ["Confirm your roadmap", "List the three cafés and their contacts", "Set a target fee of £150"], format!("{:?}", m.wins));
    t.check("B-E4: Waiting on shows what still waits, and for how long", m.waiting == [("Hours export from The Corner Café's till system".to_string(), "3 days".to_string())], format!("{:?}", m.waiting));
    t.check("B-E4: Projects with done counts, and captured tasks counted", m.projects.len() == 2 && m.captured == 2, format!("{:?} / {}", m.projects, m.captured));
    let mut empty = crate::model::WorkspaceData::default();
    crate::seed::seed_empty(&mut empty);
    let e = crate::ui::dashboard::home_model(&empty, at, crate::model::now_ms(), None);
    t.check("Home, empty: no hero, no projects, and \"nothing scheduled yet\"", e.hero.is_none() && e.projects.is_empty() && e.subline.ends_with("nothing scheduled yet"), &e.subline);

    // B-E7: a project card's outcome, next move, phase counts and open questions
    let cafe = d.projects.iter().find(|p| p.name == "Café flat-fee bookkeeping").map(|p| p.id.clone()).unwrap_or_default();
    let c = crate::ui::sections::project_card_model(&d, &cafe, crate::model::now_ms(), at);
    t.check(
        "B-E7: project card with outcome, next move, progress, Now / Next / Later counts and open questions",
        c.outcome.starts_with("Café owners know their monthly cost") && c.next.starts_with("Pull the actual hours") && c.now == "2 Now" && c.next_n == "2 Next" && c.later == "2 Later" && c.open.ends_with("open questions") && c.updated == "Updated today" && c.done.ends_with("tasks"),
        format!("{} | {} | {} {} {} {} | {}", c.done, c.next, c.now, c.next_n, c.later, c.open, c.updated),
    );

    // B-E21: the plan's request lists today's real items; a reply is matched back to them, and
    // anything not on today's list is dropped. (The reply below is written test input for the
    // parser, not AI output.)
    let prompt = crate::ui::plan::plan_prompt(&d, at, "I can't start before 10");
    let reply = r#"Here you go: {"rows":[{"start":"10:30","end":"11:00","item":"Call with The Corner Café","why":"Fixed."},{"start":"11:00","end":"12:30","item":"Reconstruct 12 months of hours per café","why":"Highest value."},{"start":"13:00","item":"Water the garden","why":"Not on the list."}],"wont_fit":[{"item":"Chase the till export","why":"No room."}],"clashes":1}"#;
    let plan = crate::ui::plan::parse_plan(reply, &d, at);
    t.check(
        "B-E21: Plan my day asks with today's real items and the person's change, and keeps only items from today's list",
        prompt.contains("Call with The Corner Café") && prompt.contains("Send October invoices") && prompt.contains("I can't start before 10")
            && plan.as_ref().map_or(false, |p| p.rows.len() == 2 && p.rows[1].task.is_some() && p.rows[0].tag == "Event" && p.rows[0].time == "10:30 – 11:00" && p.wont_fit.len() == 1 && p.wont_fit[0].2.is_some() && p.clashes == 1),
        format!("{plan:?}"),
    );
    t.check("B-E21: a reply naming none of today's items is an error (the rest of Home still loads)", crate::ui::plan::parse_plan(r#"{"rows":[{"item":"Something else"}]}"#, &d, at).is_err(), "");
    let dashed = crate::ui::plan::parse_plan(r#"{"rows":[{"item":"Send October invoices","start":"16:00","end":"16:30","why":"Due at 16:00 — sent on time."}]}"#, &d, at);
    t.check("B-E21: an em dash in the AI's reason never reaches the screen", dashed.as_ref().map_or(false, |p| p.rows.first().map_or(false, |r| !r.why.contains('—') && r.why == "Due at 16:00, sent on time.")), format!("{:?}", dashed.as_ref().map(|p| p.rows.first().map(|r| r.why.clone()))));
    t.check("B-E21: Claude Code only gets a Claude model name (a local one in settings is left out)", crate::ai::claude_model_name("sonnet") && crate::ai::claude_model_name("claude-opus-5-5") && !crate::ai::claude_model_name("qwen3") && !crate::ai::claude_model_name("llama3.2"), "");
    t.check("B-E23: Move to Monday goes to the next Monday", crate::ui::plan::next_monday(at.date()) == chrono::NaiveDate::from_ymd_opt(2026, 11, 9).unwrap(), "");

    // B-E12: the Agenda lists each day with something on it: events with their calendar, dated
    // tasks with their project
    let ag = crate::ui::calendar::agenda_days(&d, at.date(), 21, true, at.date());
    let first = ag.first().map(|(day, l)| (*day, l.first().map(|x| (x.time.clone(), x.title.clone(), x.right.clone()))));
    t.check(
        "B-E12: Agenda by day; today first with the 10:30–11:00 call (Work); Bean & Co. on the 19th",
        first == Some((at.date(), Some(("10:30–11:00".to_string(), "Call with The Corner Café".to_string(), "Work".to_string())))) && ag.iter().any(|(day, l)| day.format("%-d").to_string() == "19" && l.iter().any(|x| x.title.contains("Bean"))) && ag.iter().any(|(_, l)| l.iter().any(|x| x.time == "Due today" && x.right == "Café flat-fee bookkeeping")),
        format!("{} days; first {first:?}", ag.len()),
    );

    // B-E11: Add task highlights what it recognised, and its chips carry no emoji
    use crate::nlp::Recognised as R;
    let text = "Pay rent every month 9am p1 #Home";
    let spans: Vec<(&str, R)> = crate::nlp::recognised(text, at.date()).into_iter().map(|(r, k)| (&text[r], k)).collect();
    let chips: Vec<String> = crate::nlp::parse(text, at.date()).plain_chips().into_iter().map(|c| c.0).collect();
    t.check(
        "B-E11: recognised words are found (repeat and time amber, p1 terracotta, #Home green) and the chips have no emoji",
        spans == [("every", R::When), ("month", R::When), ("9am", R::When), ("p1", R::Priority), ("#Home", R::Project)] && chips.iter().all(|c| c.chars().all(|ch| (ch as u32) < 0x2190 || ch == '↻')) && chips.contains(&"Priority 1".to_string()) && chips.contains(&"Project: Home".to_string()),
        format!("{spans:?} {chips:?}"),
    );

    // B-E5: waiting since; old data loads unchanged, and marking it can be undone
    let old: Result<crate::model::Task, _> = serde_json::from_str(r#"{"id":"t1","title":"Old task","priority":4,"created":1,"updated":1}"#);
    let saved = old.as_ref().map(|x| serde_json::to_string(x).unwrap_or_default()).unwrap_or_default();
    t.check("B-E5: a task saved before waiting existed loads unchanged, and saves without the new fields", old.as_ref().map_or(false, |x| x.waiting_since.is_none() && x.planned_start.is_none()) && !saved.contains("waiting_since") && !saved.contains("planned_start"), &saved);
    let mut w = crate::model::WorkspaceData::default();
    let wt = crate::model::Task::new("Ask for the till export".to_string());
    let wid = wt.id.clone();
    w.tasks.push(wt);
    let cs = w.record_change("Waiting on \"Ask for the till export\"", "edit", |d| {
        if let Some(x) = d.task_mut(&wid) {
            x.waiting_since = Some(1);
        }
    });
    let marked = w.task(&wid).map_or(false, |x| x.waiting_since.is_some());
    let undone = cs.as_deref().map_or(false, |c| w.undo_recorded(c).is_ok()) && w.task(&wid).map_or(false, |x| x.waiting_since.is_none());
    t.check("B-E5: marking a task waiting is a change set Undo reverses", marked && undone, "");
    let wt_model = crate::ui::tasks::project_tasks_model(&d, &cafe, crate::model::now_ms(), at);
    t.check("B-E5: a waiting task sits in Waiting on (with its days), not in Now; Home's card lists it", wt_model.waiting.iter().any(|w| w.4 && w.1.starts_with("Hours export") && w.2 == "Waiting · 3 days") && !wt_model.now.iter().any(|(id, _)| d.task(id).map_or(false, |x| x.title.starts_with("Hours export"))) && m.waiting.iter().any(|(w, a)| w.starts_with("Hours export") && a == "3 days"), format!("{:?}", wt_model.waiting));

    // B-E22: Use this plan writes only the planned start, replaces earlier ones, and the
    // calendar's Agenda shows the task at that time
    let mut pd = d.clone();
    let rec = pd.tasks.iter().find(|x| x.title == "Reconstruct 12 months of hours per café").cloned().unwrap();
    let before = (rec.due, rec.due_time, rec.reminder_min);
    let plan1 = crate::ui::plan::Plan { rows: vec![crate::ui::plan::PlanRow { time: "09:30 – 11:00".into(), item: rec.title.clone(), why: String::new(), tag: "Task".into(), task: Some(rec.id.clone()) }], ..Default::default() };
    let starts = crate::ui::plan::plan_starts(&plan1, at.date());
    crate::ui::plan::apply_plan(&mut pd, &starts, at.date());
    let after1 = pd.task(&rec.id).cloned().unwrap();
    let agenda = crate::ui::calendar::agenda_days(&pd, at.date(), 1, true, at.date());
    let shows = agenda.first().map_or(false, |(_, l)| l.iter().any(|x| x.title == rec.title && x.time == "Planned 09:30"));
    let plan2 = crate::ui::plan::Plan { rows: vec![crate::ui::plan::PlanRow { time: "14:00".into(), item: "Send October invoices".into(), why: String::new(), tag: "Task".into(), task: pd.tasks.iter().find(|x| x.title == "Send October invoices").map(|x| x.id.clone()) }], ..Default::default() };
    let s2 = crate::ui::plan::plan_starts(&plan2, at.date());
    crate::ui::plan::apply_plan(&mut pd, &s2, at.date());
    t.check(
        "B-E22: Use this plan sets only the planned start (due date, time and reminder untouched), the Agenda shows it, and a new plan replaces it",
        after1.planned_start.map(|x| x.format("%H:%M").to_string()) == Some("09:30".into()) && (after1.due, after1.due_time, after1.reminder_min) == before && shows && pd.task(&rec.id).map_or(false, |x| x.planned_start.is_none()),
        format!("{:?} {shows}", after1.planned_start),
    );

    // B-E11: #Name with no project: nothing is created quietly; a close name is offered
    let names = vec!["Café flat-fee bookkeeping".to_string(), "Website refresh".to_string(), "Home".to_string()];
    t.check(
        "B-E11: a close project name is offered for a #Name that matches none (\"Did you mean …?\"), and choosing it removes the #word",
        crate::nlp::closest_project("Websit", &names).map(|s| s.as_str()) == Some("Website refresh") && crate::nlp::closest_project("Hom", &names).map(|s| s.as_str()) == Some("Home") && crate::nlp::closest_project("Garden", &names).is_none() && crate::nlp::without_tag("Fix gate #Hom tomorrow", "Hom") == "Fix gate tomorrow",
        "",
    );

    // B-E8: one project's tasks in Now, Waiting on and Upcoming from roadmap, with their themes
    let tm = crate::ui::tasks::project_tasks_model(&d, &cafe, crate::model::now_ms(), at);
    let first_now = tm.now.first().and_then(|(id, th)| d.task(id).map(|t| (t.title.clone(), th.clone())));
    t.check(
        "B-E8: Now, Waiting on and Upcoming from roadmap, each task with its roadmap theme; done this week counted",
        first_now == Some(("Reconstruct 12 months of hours per café".to_string(), "Measure true monthly hours".to_string()))
            && tm.waiting.len() == 1
            && tm.waiting[0].2 == "Waiting · 3 days"
            && tm.waiting[0].3 == "Measure true monthly hours"
            && tm.upcoming.first().map(|u| u.2.as_str()) == Some("Next")
            && tm.upcoming.last().map(|u| u.2.as_str()) == Some("Later")
            && tm.done_week == 3,
        format!("now {} waiting {:?} upcoming {} done {}", tm.now.len(), tm.waiting, tm.upcoming.len(), tm.done_week),
    );
}
