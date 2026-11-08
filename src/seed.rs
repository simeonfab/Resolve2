//! Starter content for a brand-new workspace, so every view has something to show.

use crate::model::*;
use chrono::{Datelike, Duration, NaiveTime};
use std::collections::BTreeMap;

pub fn seed_empty(d: &mut WorkspaceData) {
    let t = now_ms();
    d.version = 1;
    // a new workspace holds no content of ours, just one calendar to put events in
    d.calendars.push(Calendar { id: new_id(), name: "Personal".into(), color: crate::ui::theme::PICK_COLOURS[1], visible: true, updated: t, ..Default::default() });
}

fn at(h: u32, m: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(h, m, 0).unwrap()
}

pub fn seed(d: &mut WorkspaceData) {
    let t = now_ms();
    let today = today();
    d.version = 1;

    // ---- calendars
    let cal = |name: &str, color: [u8; 3], visible: bool| Calendar { id: new_id(), name: name.into(), color, visible, updated: t, ..Default::default() };
    let personal = cal("Personal", [63, 81, 181], true);
    let work = cal("Work", [3, 155, 229], true);
    let family = cal("Family", [11, 128, 67], true);
    let birthdays = cal("Birthdays", [142, 36, 170], false);
    let (pc, wc, fc, bc) = (personal.id.clone(), work.id.clone(), family.id.clone(), birthdays.id.clone());
    d.calendars.extend([personal, work, family, birthdays]);

    // ---- pages
    let mut welcome = Page::new("Welcome to Resolve 2");
    welcome.icon = "🧭".into();
    welcome.favorite = true;
    let mut review = Page::new("Weekly Review");
    review.icon = "🚨".into();
    review.favorite = true;
    let mut lab = Page::new("Boredom Lab: Project brief");
    lab.icon = "🧪".into();
    lab.favorite = true;
    let mut ideas = Page::new("Ideas & Notes");
    ideas.icon = "💡".into();
    let mut jobs_page = Page::new("Job search");
    jobs_page.icon = "💼".into();
    let mut lab_meeting = Page::new("Kick-off meeting notes");
    lab_meeting.icon = "📝".into();
    lab_meeting.parent = Some(lab.id.clone());

    welcome.blocks = vec![
        Block::new(BlockKind::Paragraph, "One place for your **pages**, **tasks** and **calendar**, stored on your own computer, readable by your AI assistants."),
        Block::new(BlockKind::Callout, "**Tip:** press Ctrl+Q anywhere to capture a task like *Call the garage tomorrow 9am p1 #Home*."),
        Block::new(BlockKind::H2, "What you can do"),
        Block::new(BlockKind::Bullet, "Write pages with headings, lists, to-dos, quotes, code and callouts. Type / for the block menu."),
        Block::new(BlockKind::Bullet, "Link pages with [[Weekly Review]] style links. Backlinks appear at the bottom."),
        Block::new(BlockKind::Bullet, "Build tables and kanban boards in databases, like the [[Job search]] tracker."),
        Block::new(BlockKind::Bullet, "Plan tasks with priorities, subtasks, reminders and repeating due dates."),
        Block::new(BlockKind::Bullet, "See tasks and events together on the calendar in day, week and month views."),
        Block::new(BlockKind::H2, "Try it"),
        {
            let mut b = Block::new(BlockKind::Todo, "Open the Calendar and drag an event to a new time");
            b.checked = false;
            b
        },
        Block::new(BlockKind::Todo, "Connect Claude, ChatGPT or Codex in Settings → AI & MCP"),
        Block::new(BlockKind::Todo, "Pick a sync folder so your phone sees the same workspace"),
    ];
    review.blocks = vec![
        Block::new(BlockKind::H2, "Purpose"),
        Block::new(BlockKind::Paragraph, "A consistent way to **look back, tidy up and plan ahead** each week, so nothing important slips through the cracks."),
        Block::new(BlockKind::Paragraph, "This is deliberately **v0**. For the first few weeks the goal is to collect real examples and adjust the routine from evidence."),
        Block::new(BlockKind::Callout, "**Rule:** judge a task by its effect on your week, not by how urgent it feels."),
        Block::new(BlockKind::H2, "Operating model"),
        Block::new(BlockKind::Numbered, "**Capture**: empty your head into the Inbox."),
        Block::new(BlockKind::Numbered, "**Clarify**: give every task a project, priority and date."),
        Block::new(BlockKind::Numbered, "**Schedule**: block time on the calendar for the big three."),
        Block::new(BlockKind::Numbered, "**Review**: check projects in [[Boredom Lab: Project brief]] and the job tracker."),
        Block::new(BlockKind::Numbered, "**Reflect**: note one thing to improve next week."),
        Block::new(BlockKind::Divider, ""),
        Block::new(BlockKind::Quote, "What gets scheduled gets done."),
    ];
    lab.blocks = vec![
        Block::new(BlockKind::H2, "Goal"),
        Block::new(BlockKind::Paragraph, "Design and test three small party games with friends before the end of the month."),
        Block::new(BlockKind::H2, "Milestones"),
        Block::new(BlockKind::Todo, "Prototype rules for game one"),
        Block::new(BlockKind::Todo, "First playtest with friends"),
        Block::new(BlockKind::Todo, "Write up results"),
        Block::new(BlockKind::H3, "Related"),
        Block::new(BlockKind::PageLink, lab_meeting.id.clone()),
    ];
    lab_meeting.blocks = vec![
        Block::new(BlockKind::Paragraph, "Attendees: me, Sam, Alex"),
        Block::new(BlockKind::Bullet, "Keep each game under 20 minutes"),
        Block::new(BlockKind::Bullet, "Cards first, app later"),
        Block::new(BlockKind::Code, "score = votes * 2 + bonus"),
    ];
    ideas.blocks = vec![
        Block::new(BlockKind::Paragraph, "A scratchpad. Anything here can become a task. Select a line and use the AI panel to *extract tasks*."),
        Block::new(BlockKind::Bullet, "Learn to make sourdough"),
        Block::new(BlockKind::Bullet, "Weekend trip in spring"),
    ];

    // ---- databases
    let col = |name: &str, kind: ColKind, opts: &[&str], width: f32| Column {
        id: new_id(),
        name: name.into(),
        kind,
        options: opts.iter().map(|s| s.to_string()).collect(),
        width,
        target: String::new(),
    };
    let role = col("Role", ColKind::Text, &[], 220.0);
    let company = col("Company", ColKind::Text, &[], 150.0);
    let status = col("Status", ColKind::Select, &["To apply", "Applied", "Interview", "Offer"], 130.0);
    let applied = col("Applied on", ColKind::Date, &[], 130.0);
    let link = col("Link", ColKind::Url, &[], 180.0);
    let rows_src = [
        ("Product Manager", "Northwind", "Interview", -6),
        ("Operations Lead", "Contoso", "Applied", -3),
        ("Programme Manager", "Fabrikam", "Applied", -2),
        ("Head of Delivery", "Tailspin", "To apply", 0),
        ("Service Manager", "Litware", "To apply", 0),
    ];
    let mut rows = vec![];
    for (r, c, s, days) in rows_src {
        let mut cells = BTreeMap::new();
        cells.insert(role.id.clone(), r.to_string());
        cells.insert(company.id.clone(), c.to_string());
        cells.insert(status.id.clone(), s.to_string());
        if s != "To apply" {
            cells.insert(applied.id.clone(), (today + Duration::days(days)).to_string());
        }
        cells.insert(link.id.clone(), format!("https://example.com/jobs/{}", c.to_lowercase()));
        rows.push(Row { id: new_id(), cells, page_id: None });
    }
    let jobs_db = Database {
        id: new_id(),
        name: "Job applications".into(),
        icon: "💼".into(),
        group_by: Some(status.id.clone()),
        columns: vec![role, company, status, applied, link],
        rows,
        view: DbView::Table,
        sort_by: None, filters: vec![], template: false,
        updated: t,
    };

    let title = col("Title", ColKind::Text, &[], 220.0);
    let author = col("Author", ColKind::Text, &[], 150.0);
    let rstatus = col("Status", ColKind::Select, &["Want to read", "Reading", "Finished"], 130.0);
    let rating = col("Rating", ColKind::Number, &[], 80.0);
    let done = col("Owned", ColKind::Checkbox, &[], 80.0);
    let mut rrows = vec![];
    for (a, b, s, r, o) in [
        ("Deep Work", "Cal Newport", "Finished", "5", "true"),
        ("Getting Things Done", "David Allen", "Reading", "", "true"),
        ("The Making of a Manager", "Julie Zhuo", "Want to read", "", "false"),
    ] {
        let mut cells = BTreeMap::new();
        cells.insert(title.id.clone(), a.into());
        cells.insert(author.id.clone(), b.into());
        cells.insert(rstatus.id.clone(), s.into());
        cells.insert(rating.id.clone(), r.into());
        cells.insert(done.id.clone(), o.into());
        rrows.push(Row { id: new_id(), cells, page_id: None });
    }
    let reading_db = Database {
        id: new_id(),
        name: "Reading list".into(),
        icon: "📚".into(),
        group_by: Some(rstatus.id.clone()),
        columns: vec![title, author, rstatus, rating, done],
        rows: rrows,
        view: DbView::Board,
        sort_by: None, filters: vec![], template: false,
        updated: t,
    };
    jobs_page.blocks = vec![
        Block::new(BlockKind::Paragraph, "Target: **5 applications a week**. The tracker below is a live database. Switch it to a board to drag cards between stages."),
        Block::new(BlockKind::Database, jobs_db.id.clone()),
    ];

    // ---- projects
    let proj = |name: &str, color: [u8; 3], sections: &[&str], order: i64| Project {
        id: new_id(),
        name: name.into(),
        color,
        sections: sections.iter().map(|s| s.to_string()).collect(),
        page_id: None,
        archived: false,
        favorite: false,
        order,
        updated: t,
    };
    let mut boredom = proj("Boredom Lab", [175, 56, 235], &["Ideas", "Prototyping", "Playtesting"], 1);
    boredom.page_id = Some(lab.id.clone());
    let home = proj("Home", [41, 171, 135], &[], 2);
    let career = proj("Career", [255, 153, 51], &["Applications", "Prep"], 3);
    let (bid, hid, cid) = (boredom.id.clone(), home.id.clone(), career.id.clone());
    d.projects.extend([boredom, home, career]);

    // ---- tasks
    let mut order = 0;
    let mut task = |title: &str, project: Option<&str>, section: &str, prio: u8, due: Option<i64>, time: Option<NaiveTime>| {
        order += 1;
        let mut x = Task::new(title);
        x.project_id = project.map(|s| s.to_string());
        x.section = section.into();
        x.priority = prio;
        x.due = due.map(|n| today + Duration::days(n));
        x.due_time = time;
        x.order = order;
        x
    };
    let mut road = task("Road tax and MOT for the car", None, "", 1, Some(-3), None);
    road.notes = "Check the MOT centre's opening hours first.".into();
    let mut jobs = task("Apply for 5 jobs", None, "", 4, Some(-1), None);
    jobs.recurrence = Recurrence::Weekly;
    jobs.page_id = Some(jobs_page.id.clone());
    let mut cv = task("Update CV with latest project", Some(&cid), "Prep", 2, Some(0), Some(at(11, 0)));
    cv.duration_min = Some(60);
    cv.reminder_min = Some(15);
    let mut plants = task("Water the plants", Some(&hid), "", 3, Some(0), None);
    plants.recurrence = Recurrence::EveryDays(3);
    let groceries = task("Order groceries", Some(&hid), "", 3, Some(1), Some(at(18, 0)));
    let bins = task("Put the bins out", Some(&hid), "", 4, Some(2), Some(at(20, 0)));
    let mut rules = task("Write rules for game one", Some(&bid), "Prototyping", 2, Some(1), None);
    rules.page_id = Some(lab.id.clone());
    let rules_id = rules.id.clone();
    let mut sub1 = task("Scoring system", Some(&bid), "Prototyping", 4, None, None);
    sub1.parent_id = Some(rules_id.clone());
    let mut sub2 = task("Round timer", Some(&bid), "Prototyping", 4, None, None);
    sub2.parent_id = Some(rules_id.clone());
    sub2.completed = true;
    sub2.completed_at = Some(t);
    let mut sub3 = task("Print card deck", Some(&bid), "Prototyping", 4, None, None);
    sub3.parent_id = Some(rules_id);
    let mut playtest = task("Playtest with friends", Some(&bid), "Playtesting", 1, Some(3), Some(at(19, 0)));
    playtest.duration_min = Some(120);
    playtest.reminder_min = Some(60);
    let brainstorm = task("Brainstorm three game themes", Some(&bid), "Ideas", 3, None, None);
    let mut prep = task("Prepare interview stories", Some(&cid), "Prep", 1, Some(2), Some(at(10, 0)));
    prep.duration_min = Some(90);
    prep.labels = vec!["focus".into()];
    let review_t = {
        let mut x = task("Weekly review", None, "", 2, None, Some(at(17, 0)));
        let fri = crate::nlp::next_weekday(today, chrono::Weekday::Fri, true);
        x.due = Some(fri);
        x.recurrence = Recurrence::Weekly;
        x.page_id = Some(review.id.clone());
        x.duration_min = Some(45);
        x
    };
    let mut done1 = task("Book dentist appointment", Some(&hid), "", 3, Some(-1), None);
    done1.completed = true;
    done1.completed_at = Some(t - 3600 * 1000);
    d.tasks.extend([road, jobs, cv, plants, groceries, bins, rules, sub1, sub2, sub3, playtest, brainstorm, prep, review_t, done1]);

    // ---- events for this week
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let ev = |cal: &str, title: &str, day: i64, h: u32, m: u32, mins: i64, rec: Recurrence| {
        let start = (monday + Duration::days(day)).and_time(at(h, m));
        Event {
            id: new_id(),
            calendar_id: cal.to_string(),
            title: title.into(),
            start,
            end: start + Duration::minutes(mins),
            all_day: false,
            location: String::new(),
            notes: String::new(),
            recurrence: rec,
            page_id: None,
            reminder_min: Some(10),
            updated: t,
        }
    };
    let mut standup = ev(&wc, "Daily stand-up", 0, 9, 0, 15, Recurrence::Weekdays);
    standup.location = "Video call".into();
    d.events.push(standup);
    d.events.push(ev(&wc, "Team sync", 0, 10, 0, 30, Recurrence::Weekdays));
    d.events.push(ev(&pc, "Focus time", 0, 13, 0, 60, Recurrence::Weekdays));
    let mut planning = ev(&wc, "Sprint planning", 1, 11, 0, 90, Recurrence::None);
    planning.page_id = Some(review.id.clone());
    d.events.push(planning);
    d.events.push(ev(&wc, "Stakeholder review", 1, 14, 0, 60, Recurrence::None));
    d.events.push(ev(&wc, "Refinement", 2, 10, 30, 55, Recurrence::None));
    d.events.push(ev(&pc, "Phone call with recruiter", 2, 15, 0, 30, Recurrence::None));
    let mut gg = ev(&fc, "Board game night", 2, 19, 0, 120, Recurrence::None);
    gg.location = "City centre".into();
    d.events.push(gg);
    d.events.push(ev(&wc, "Video interview", 3, 11, 0, 60, Recurrence::None));
    d.events.push(ev(&pc, "Timesheet", 3, 16, 0, 15, Recurrence::Weekly));
    let mut haircut = ev(&pc, "Haircut", 0, 18, 0, 45, Recurrence::None);
    haircut.location = "High Street".into();
    d.events.push(haircut);
    let mut lunch = ev(&fc, "Family lunch", 6, 14, 0, 240, Recurrence::None);
    lunch.location = "Grandma's".into();
    d.events.push(lunch);
    let mut bday = ev(&bc, "Sam's birthday", 5, 0, 0, 24 * 60, Recurrence::Yearly);
    bday.all_day = true;
    d.events.push(bday);
    let mut trip = ev(&fc, "Weekend away", 5, 0, 0, 24 * 60 * 2, Recurrence::None);
    trip.all_day = true;
    d.events.push(trip);

    d.pages.extend([welcome, review, lab, lab_meeting, ideas, jobs_page]);
    d.databases.extend([jobs_db, reading_db]);
    seed_resolve(d);
}

/// A demo Resolve project, made through the real (manual, no-AI) intake handoff. Sample data only.
fn seed_resolve(d: &mut WorkspaceData) {
    use crate::resolve::intake::{Intake, NotYet, Phase, Recommendation, Understanding};
    let mut i = Intake::new();
    i.name = "Sam".into();
    i.manual = true;
    i.input = "I make soy candles and sell them at two weekend markets. I've got an Etsy page nobody visits and notes all over Notion. I want online sales so I'm not stuck at markets every weekend.".into();
    i.understanding = Some(Understanding {
        project: "Moving a handmade candle business from weekend markets to online sales.".into(),
        goal: "Make online orders a reliable share of sales.".into(),
        state: "An Etsy listing exists but gets few orders; notes are scattered in Notion.".into(),
        uncertainty: "Whether market customers will buy online at £30.".into(),
        outcome: "Market regulars reorder online without being reminded, so weekends are optional.".into(),
        assumptions: vec!["Regulars would buy online if asked".into()],
        mentioned_tools: vec!["Notion".into()],
        ..Default::default()
    });
    i.tools = vec!["Notion".into()];
    i.recommendation = Some(Recommendation {
        recommended_action: "Email the market regulars a link with a first-order code".into(),
        why: "They already buy at £30 in person; it's the cheapest test of online demand.".into(),
        milestone: "Five regulars order online within two weeks".into(),
        open_decision: "Keep Etsy or move to an own-domain shop".into(),
        win_summary: String::new(),
        roadmap: vec![
            Phase { phase: "Now".into(), action: "Collect regulars' emails at Saturday's market".into(), output: "A list of 40 emails".into() },
            Phase { phase: "Next".into(), action: "Choose the shop platform".into(), output: "A decision".into() },
            Phase { phase: "Later".into(), action: "Try a small paid ad test".into(), output: "Cost per order".into() },
        ],
        top_priorities: vec!["Prove online demand with existing customers".into()],
        do_not_yet: vec![NotYet { item: "Redesign the brand".into(), reason: "Demand isn't proven yet".into() }],
        top_card_type: "action".into(),
    });
    let id = i.id.clone();
    d.intakes.push(i);
    if let Ok(pid) = crate::resolve::intake::handoff(d, &id) {
        if let Some(c) = d.context_for_mut(&pid) {
            c.notes = "Sample project, made with the manual (no-AI) intake.".into();
        }
    }
}

// ======================================================================================
// The design canvases' sample (Pack B and C, 7 Nov 2026): the café flat-fee bookkeeping
// project, Website refresh, two captured tasks and the week's events, copied from the canvas
// renders in docs/reference/canvases so screens can be compared like for like. Screenshot data
// only (--state full); never a new user's starter content. Dates are relative to today, which
// screenshots pin to the canvases' Friday 6 November with --now.
// ======================================================================================

fn item(title: &str, detail: &str, basis: &str, phase: &str, parent: Option<usize>) -> crate::resolve::journey::Item {
    crate::resolve::journey::Item { title: title.into(), detail: detail.into(), basis: basis.into(), phase: phase.into(), parent, ..Default::default() }
}

fn hyp(title: &str, basis: &str, negation: &str) -> crate::resolve::journey::Item {
    crate::resolve::journey::Item { negation: negation.into(), ..item(title, "", basis, "", None) }
}

fn exp(title: &str, parent: usize, support_if: &str) -> crate::resolve::journey::Item {
    crate::resolve::journey::Item { fields: [("support_if".to_string(), support_if.to_string())].into_iter().collect(), ..item(title, "", "", "", Some(parent)) }
}

/// Adopt a hand-written proposal through the real journey Adopt (the same path as the intake).
fn adopt_sample(d: &mut WorkspaceData, input: &str, question: Option<&str>, heard: &[(&str, &str)], p: crate::resolve::journey::Proposal) -> Option<Id> {
    use crate::resolve::intake::{Intake, Understanding};
    let mut i = Intake::new();
    i.name = "Simeon".into();
    i.input = input.into();
    i.stage = "proposal".into();
    if let Some(q) = question {
        i.qas.push(crate::resolve::journey::Qa { id: new_id(), target: "constraints".into(), question: q.into(), source: "sample".into(), ..Default::default() });
    }
    for (kind, text) in heard {
        i.heard.push(crate::resolve::journey::Heard { id: new_id(), text: text.to_string(), kind: kind.to_string(), ..Default::default() });
    }
    i.understanding = Some(Understanding { project: p.objective.clone(), outcome: p.outcome.clone(), ..Default::default() });
    i.outcome_confirmed = true;
    i.proposal = Some(p);
    let id = i.id.clone();
    d.intakes.push(i);
    crate::resolve::journey::adopt_all(d, &id).ok()
}

fn set_task(d: &mut WorkspaceData, title: &str, f: impl FnOnce(&mut Task)) {
    if let Some(x) = d.tasks.iter_mut().find(|x| x.title == title) {
        f(x);
    }
}

pub fn seed_canvas(d: &mut WorkspaceData) {
    use crate::resolve::journey::Proposal;
    use crate::ui::theme::PICK_COLOURS;
    let t = now_ms();
    let today = today();
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    d.version = 1;

    // ---- calendars: Work (terracotta), Personal (green)
    let work = Calendar { id: new_id(), name: "Work".into(), color: PICK_COLOURS[0], visible: true, updated: t, ..Default::default() };
    let personal = Calendar { id: new_id(), name: "Personal".into(), color: PICK_COLOURS[1], visible: true, updated: t, ..Default::default() };
    let (wc, pc) = (work.id.clone(), personal.id.clone());
    d.calendars.extend([work, personal]);

    // ---- the café project, through the real Adopt
    let cafe = Proposal {
        name: "Café flat-fee bookkeeping".into(),
        summary: "Prove the flat fee on two cafés before January.".into(),
        objective: "Turning my hourly café bookkeeping into a fixed-price monthly package.".into(),
        outcome: "Café owners know their monthly cost up front and stop sending shoeboxes of receipts at year end.".into(),
        h0: item("A fixed-price package works", "", "assumption", "", None),
        hypotheses: vec![
            hyp("Monthly workload is predictable", "assumption", "NH1: hours swing too much month to month to price flat"),
            hyp("Café clients want a flat fee", "evidence", "NH2: clients prefer hourly once they see a real price"),
            hyp("The segment is reachable beyond word of mouth", "assumption", "NH3: new café clients only come through referrals"),
            hyp("The package ends the shoebox rush", "assumption", "NH4: receipts still arrive in January"),
        ],
        experiments: vec![
            exp("Reconstruct 12 months of actual hours per café", 0, "the busiest month is under 2× the quietest"),
            exp("Offer the fixed fee to the two willing clients", 1, "at least one signs within two weeks"),
            exp("Run a 3-month flat-fee trial and watch when records arrive", 3, "80% of receipts land within the month"),
        ],
        uncertainties: vec![item("What does a café pay you now per year?", "Affects H2.", "uncertain", "", None)],
        roadmap: vec![
            item("Measure true monthly hours", "", "evidence", "Now", None),
            item("Ask the two willing clients what a flat fee is worth", "", "assumption", "Now", None),
            item("Price to absorb spikes", "", "assumption", "Next", None),
            item("Convert the two willing clients", "", "evidence", "Next", None),
            item("Design the no-shoebox rhythm", "", "assumption", "Later", None),
            item("Reach cafés beyond word of mouth", "", "assumption", "Later", None),
        ],
        tasks: vec![
            item("Reconstruct 12 months of hours per café", "For each of your three cafés, list the hours actually worked in each of the last 12 months.", "evidence", "Now", Some(0)),
            item("Mark the busiest and quietest months", "For each café, find the highest-hour month and the lowest, and note what drove the peak.", "assumption", "Now", Some(0)),
            item("Hours export from The Corner Café's till system", "", "evidence", "Now", Some(0)),
            item("Book a call with each willing client", "", "assumption", "Now", Some(1)),
            item("Ask what they pay now per year", "", "assumption", "Now", Some(1)),
            item("Model the flat fee against real hours", "Set an hourly rate you would accept, multiply by average monthly hours, then add a buffer for the spike months.", "assumption", "Next", Some(2)),
            item("Set the spike buffer", "", "assumption", "Next", Some(2)),
            item("Draft the fixed-fee offer for two clients", "", "evidence", "Next", Some(3)),
            item("Define the monthly receipt handover", "", "assumption", "Later", Some(4)),
            item("List the three cafés and their contacts", "", "evidence", "Now", Some(0)),
            item("Confirm your roadmap", "", "evidence", "Now", Some(0)),
            item("Set a target fee of £150", "", "assumption", "Now", Some(2)),
        ],
        pages: vec![
            item("Project brief", "What this is, who it is for, what done looks like.\n\nI spend roughly 5 hours a month on each café, so each café takes roughly 5 hours a month before the year-end spike.", "evidence", "", None),
            item("Plan", "Prove the flat fee on two cafés before January.", "assumption", "", None),
        ],
        next_move: "Pull the actual hours you logged on each café for the last 12 months.".into(),
        why: "Whether a flat £150 survives the year-end spike is the one thing that makes or breaks this package. The numbers are already in your records.".into(),
        cut: vec!["Payroll add-on".into(), "Online booking page".into()],
        ..Default::default()
    };
    let input = "Turning my hourly café bookkeeping into a fixed-price monthly package. I want it to achieve: steadier income, and clients who know their cost up front. Hourly billing makes clients nervous about costs. I spend roughly 5 hours a month on each café. Two of my three existing café clients said yes in principle.";
    let q = "Besides the three cafés you already do, roughly how many more could you take on each month before you simply run out of hours yourself?";
    let cafe_id = adopt_sample(d, input, Some(q), &[("evidence", "Two clients said yes in principle"), ("assumption", "Hourly billing makes clients nervous about costs")], cafe);

    // dates, done items and the reason, as the canvases show them
    for x in d.tasks.iter_mut().filter(|x| cafe_id.is_some() && x.project_id == cafe_id) {
        x.due = None;
    }
    set_task(d, "Reconstruct 12 months of hours per café", |x| x.due = Some(today));
    set_task(d, "Mark the busiest and quietest months", |x| x.due = Some(today + Duration::days(7)));
    for (title, n) in [("Confirm your roadmap", 0), ("List the three cafés and their contacts", 1), ("Set a target fee of £150", 2)] {
        // finished on that day at 11:00, counted back from the app's clock (which a screenshot may pin)
        let when = (monday + Duration::days(n)).and_time(at(11, 0));
        let at = Some(t - (now_local() - when).num_milliseconds());
        set_task(d, title, |x| {
            x.completed = true;
            x.completed_at = at;
        });
    }
    // waiting on someone else for three days (B-E5)
    set_task(d, "Hours export from The Corner Café's till system", |x| x.waiting_since = Some(t - 3 * 24 * 3600 * 1000));
    if let Some(c) = cafe_id.as_ref().and_then(|p| d.context_for_mut(p)) {
        c.recommendation.why = "Whether a flat £150 survives the year-end spike is the one thing that makes or breaks this package. The numbers are already in your records.".into();
    }

    // ---- Website refresh
    let web = Proposal {
        name: "Website refresh".into(),
        objective: "A website that shows the packages.".into(),
        outcome: "New clients can see the packages and book a call without emailing first.".into(),
        h0: item("Clients book once they see the packages", "", "assumption", "", None),
        roadmap: vec![item("Show the packages", "", "assumption", "Now", None)],
        tasks: vec![item("Write the three package descriptions", "", "assumption", "Now", Some(0))],
        pages: vec![item("Project brief", "New clients can see the packages and book a call without emailing first.", "assumption", "", None)],
        ..Default::default()
    };
    let web_id = adopt_sample(d, "Refresh the website so the packages are clear.", None, &[], web);
    if let Some(pr) = web_id.as_ref().and_then(|w| d.projects.iter_mut().find(|p| &p.id == w)) {
        pr.color = PICK_COLOURS[0];
    }
    set_task(d, "Write the three package descriptions", |x| x.due = Some(today));

    // ---- captured (not in a project)
    let mut inv = Task::new("Send October invoices");
    inv.due = Some(today);
    inv.due_time = Some(at(16, 0));
    inv.recurrence = Recurrence::Monthly;
    let mut chase = Task::new("Chase the till export");
    chase.due = Some(today - Duration::days(3));
    d.tasks.extend([inv, chase]);

    // ---- events
    let ev = |cal: &str, title: &str, date: chrono::NaiveDate, h: u32, m: u32, mins: i64, rec: Recurrence, notes: &str| {
        let start = date.and_time(at(h, m));
        Event { id: new_id(), calendar_id: cal.to_string(), title: title.into(), start, end: start + Duration::minutes(mins), all_day: false, location: String::new(), notes: notes.into(), recurrence: rec, page_id: None, reminder_min: None, updated: t }
    };
    let first = today.with_day(1).unwrap_or(today);
    d.events.push(ev(&wc, "Call with The Corner Café", today, 10, 30, 30, Recurrence::None, "Bring the till export so we can check the hours"));
    d.events.push(ev(&pc, "Pay rent", first, 9, 0, 15, Recurrence::Monthly, ""));
    d.events.push(ev(&pc, "Gym", monday + Duration::days(1), 8, 0, 60, Recurrence::None, ""));
    d.events.push(ev(&wc, "Accountant check-in", monday + Duration::days(2), 14, 0, 60, Recurrence::None, ""));
    d.events.push(ev(&wc, "Intro call with Bean & Co.", today + Duration::days(13), 11, 0, 30, Recurrence::None, ""));
}
