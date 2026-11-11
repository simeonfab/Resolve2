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

/// A new Careers workspace's four calendars (Careers brief, "Calendar"): each does the job of a
/// tag, with its own colour from the tokens and its own toggle.
pub fn careers_calendars(d: &mut WorkspaceData) {
    use crate::ui::theme::tok;
    let rgb = |c: eframe::egui::Color32| [c.r(), c.g(), c.b()];
    let t = now_ms();
    d.calendars = [("Interviews", tok::TERRACOTTA), ("Closing dates", tok::AMBER), ("Follow-ups", tok::GREEN), ("Offer deadlines", tok::INK_SOFT)]
        .into_iter()
        .map(|(name, c)| Calendar { id: new_id(), name: name.into(), color: rgb(c), visible: true, updated: t, ..Default::default() })
        .collect();
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
            calendar_id: cal.to_string(), record_id: None, source: String::new(),
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

/// The café proposal from the canvases (Pack A to C), shared by the sample project and the
/// intake screenshots.
pub fn cafe_proposal() -> crate::resolve::journey::Proposal {
    use crate::resolve::journey::Proposal;
    Proposal {
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
    }
}

/// The outcome line the canvases draw for a sample project (empty for any other name).
pub fn canvas_outcome(name: &str) -> String {
    if name == "Café flat-fee bookkeeping" {
        "Café owners know their monthly cost up front and stop sending shoeboxes of receipts at year end.".into()
    } else {
        String::new()
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
    let cafe = cafe_proposal();
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
    // the canvas's links (Pack C Overview, Roadmap): E0 tests the hours task, and H1's result
    // opens the gate on "Price to absorb spikes"
    {
        use crate::resolve::Kind;
        let task = d.tasks.iter().find(|x| x.title == "Reconstruct 12 months of hours per café" && x.project_id == cafe_id).map(|x| x.id.clone());
        let h1 = d.records.iter().find(|r| r.kind == Kind::Hypothesis && r.title.starts_with("H1:") && r.project_id == cafe_id).map(|r| r.id.clone());
        if let (Some(t), Some(e)) = (task, d.records.iter_mut().find(|r| r.kind == Kind::Experiment && r.title.starts_with("Reconstruct 12 months"))) {
            e.links.push(t);
        }
        if let (Some(h), Some(g)) = (h1, d.records.iter_mut().find(|r| r.kind == Kind::Roadmap && r.title == "Price to absorb spikes")) {
            g.fields.insert("gate".into(), h);
        }
        // Pack C Roadmap: H2 and H3 gate their themes; E1 and E2 test the willing-client call and
        // the receipt handover; three milestones with dates
        let hyp = |d: &WorkspaceData, n: &str| d.records.iter().find(|r| r.kind == Kind::Hypothesis && r.title.starts_with(n) && r.project_id == cafe_id).map(|r| r.id.clone());
        for (theme, h) in [("Convert the two willing clients", "H2:"), ("Reach cafés beyond word of mouth", "H3:")] {
            if let Some(hid) = hyp(d, h) {
                if let Some(g) = d.records.iter_mut().find(|r| r.kind == Kind::Roadmap && r.title == theme && r.project_id == cafe_id) {
                    g.fields.insert("gate".into(), hid);
                }
            }
        }
        for (exp, task) in [("Offer the fixed fee", "Book a call with each willing client"), ("Run a 3-month flat-fee trial", "Define the monthly receipt handover")] {
            let tid = d.tasks.iter().find(|x| x.title == task && x.project_id == cafe_id).map(|x| x.id.clone());
            if let (Some(t), Some(e)) = (tid, d.records.iter_mut().find(|r| r.kind == Kind::Experiment && r.title.starts_with(exp) && r.project_id == cafe_id)) {
                e.links.push(t);
            }
        }
        if let Some(g) = d.records.iter_mut().find(|r| r.kind == Kind::Roadmap && r.title == "Measure true monthly hours" && r.project_id == cafe_id) {
            g.body = "Count the hours you actually worked for each café, month by month. If the busiest month is under twice the quietest, H1 holds.".into();
        }
        for (title, days) in [("12 months of hours in hand", 14), ("First flat-fee client signed", 40), ("Trial review", 140)] {
            let mut r = crate::resolve::Record::new(Kind::Milestone, cafe_id.clone(), title);
            r.fields.insert("date".into(), (today + Duration::days(days)).format("%Y-%m-%d").to_string());
            d.records.push(r);
        }
    }
    // Pack C Databases: the café clients table and an hours log
    {
        let col = |name: &str, kind: ColKind, options: &[&str], width: f32| Column { id: new_id(), name: name.into(), kind, options: options.iter().map(|s| s.to_string()).collect(), width, target: String::new() };
        let cols = vec![
            col("Café", ColKind::Text, &[], 150.0),
            col("Billing", ColKind::Select, &["Hourly", "Flat fee"], 80.0),
            col("Avg hours / month", ColKind::Text, &[], 140.0),
            col("Busiest month", ColKind::Text, &[], 120.0),
            col("Flat fee?", ColKind::Select, &["Said yes", "Not asked", "Said no"], 110.0),
            col("Next step", ColKind::Text, &[], 150.0),
        ];
        let rows = [
            ["The Corner Café", "Hourly", "waiting on export", "", "Said yes", "Chase till export"],
            ["Bean and Co.", "Hourly", "14", "January", "Said yes", "Book the price call"],
            ["Harbour Kitchen", "Hourly", "9", "April", "Not asked", "Ask after E1"],
        ];
        let mut db = Database { id: new_id(), name: "Café clients".into(), icon: "▦".into(), columns: cols.clone(), rows: vec![], view: DbView::Table, group_by: None, updated: t, ..Default::default() };
        for r in rows {
            let cells = cols.iter().zip(r.iter()).map(|(c, v)| (c.id.clone(), v.to_string())).collect();
            db.rows.push(Row { id: new_id(), cells, page_id: None });
        }
        d.databases.push(db);
        let hcols = vec![col("Date", ColKind::Date, &[], 120.0), col("Staff", ColKind::Text, &[], 100.0), col("Hours", ColKind::Number, &[], 80.0), col("Task", ColKind::Select, &["Receipts", "Payroll", "VAT"], 110.0)];
        let mut hdb = Database { id: new_id(), name: "Hours log".into(), icon: "▦".into(), columns: hcols.clone(), rows: vec![], view: DbView::Table, group_by: None, updated: t, ..Default::default() };
        for k in 0..36 {
            let day = today - Duration::days(36 - k as i64);
            let vals = [day.format("%Y-%m-%d").to_string(), ["Ana", "Sam", "Jo"][k % 3].to_string(), format!("{}", 2 + (k * 7) % 6), ["Receipts", "Payroll", "VAT"][k % 3].to_string()];
            let cells = hcols.iter().zip(vals.iter()).map(|(c, v)| (c.id.clone(), v.clone())).collect();
            hdb.rows.push(Row { id: new_id(), cells, page_id: None });
        }
        d.databases.push(hdb);
    }
    // Pack C AI Team: the canvas's own sample chat with the Validation Lead (the design's text,
    // in the screenshot sample only; no model wrote it), and two earlier chats
    if let Some(pid) = cafe_id.clone() {
        let msg = |role: &str, text: &str, at: i64| ChatMessage { role: role.into(), text: text.into(), at, ..Default::default() };
        let day = 24 * 3600 * 1000;
        d.conversations.push(Conversation { id: new_id(), project_id: Some(pid.clone()), agent: "validation".into(), title: "Validation Lead".into(), messages: vec![msg("user", "Threshold for E0", t - 4 * day), msg("validation", "Sample answer from the canvas.", t - 4 * day)], updated: t - 4 * day });
        d.conversations.push(Conversation { id: new_id(), project_id: Some(pid.clone()), agent: "validation".into(), title: "Validation Lead".into(), messages: vec![msg("user", "Is two yes answers enough for H2?", t - 3 * day), msg("validation", "Sample answer from the canvas.", t - 3 * day)], updated: t - 3 * day });
        d.conversations.push(Conversation {
            id: new_id(),
            project_id: Some(pid),
            agent: "validation".into(),
            title: "Validation Lead".into(),
            messages: vec![
                msg("user", "Two clients said yes in principle. Is that enough to call H2 supported?", t - 60_000),
                msg("validation", "No. A yes in principle costs them nothing. E1 tests it properly: a written price, and a deadline to sign. Your threshold says at least one signs within two weeks. I'd add a second check: ask both what they pay you now per year, so the price lands against a real number.\nPROPOSED: question | H2 | What does each café pay you now per year?", t - 30_000),
            ],
            updated: t,
        });
    }
    // Pack C Handoffs: three briefs written by Resolve's own generator (no AI), two sent, one done
    if let Some(ctx) = cafe_id.as_ref().and_then(|p| d.context_for(p)).cloned() {
        let day = 24 * 3600 * 1000;
        for (to, focus, status, ago) in [("Codex", "Draft the fixed-fee offer letter", "Done", 9), ("Claude Code", "Build the flat-fee calculator sheet", "Sent", 3), ("Maya, who does my admin", "Pull 12 months of hours for each café from the till exports", "Sent", 0)] {
            let mut r = crate::resolve::Record::new(crate::resolve::Kind::Handoff, cafe_id.clone(), focus);
            r.body = crate::resolve::roles::handoff_for(d, &ctx, to, focus);
            r.fields.insert("to".into(), to.into());
            r.fields.insert("focus".into(), focus.into());
            r.fields.insert("writer".into(), "Resolve (no AI connected)".into());
            r.status = status.into();
            r.created = t - ago * day;
            r.updated = t - ago * day;
            d.records.push(r);
        }
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
        Event { id: new_id(), calendar_id: cal.to_string(), record_id: None, source: String::new(), title: title.into(), start, end: start + Duration::minutes(mins), all_day: false, location: String::new(), notes: notes.into(), recurrence: rec, page_id: None, reminder_min: None, updated: t }
    };
    let first = today.with_day(1).unwrap_or(today);
    d.events.push(ev(&wc, "Call with The Corner Café", today, 10, 30, 30, Recurrence::None, "Bring the till export so we can check the hours"));
    d.events.push(ev(&pc, "Pay rent", first, 9, 0, 15, Recurrence::Monthly, ""));
    d.events.push(ev(&pc, "Gym", monday + Duration::days(1), 8, 0, 60, Recurrence::None, ""));
    d.events.push(ev(&wc, "Accountant check-in", monday + Duration::days(2), 14, 0, 60, Recurrence::None, ""));
    d.events.push(ev(&wc, "Intro call with Bean & Co.", today + Duration::days(13), 11, 0, 30, Recurrence::None, ""));
}

/// The café intake part-way through, as the Pack A canvases draw it: seven questions (the last
/// one open), what Resolve heard, the understanding cards and the proposal. Screenshot data only
/// (`--view intake-<step>` with `--state full`); in memory, never saved.
pub fn intake_sample(d: &mut WorkspaceData) -> Id {
    use crate::resolve::intake::{Intake, Understanding};
    use crate::resolve::journey::{Heard, Qa};
    let mut i = Intake::new();
    i.name = "Simeon".into();
    i.greeted = true;
    i.input = crate::resolve::intake::EXAMPLE_INPUT.into();
    i.stage = "questions".into();
    let qa = |target: &str, q: &str, a: &str, why: &str| Qa { id: new_id(), target: target.into(), question: q.into(), answer: a.into(), classified: a.into(), why: why.into(), source: "sample".into(), ..Default::default() };
    i.qas = vec![
        qa("goal", "What would a fixed monthly fee change for you, beyond steadier income?", "Clients would know their cost up front and I'd stop chasing hours.", ""),
        qa("problem", "What makes hourly billing hard for your café clients today?", "Hourly billing makes clients nervous about costs.", ""),
        qa("who", "Which of your cafés would try a flat fee first?", "Two of my three existing café clients said they'd prefer a fixed fee.", ""),
        qa("evidence", "How many hours does each café take you in a normal month?", "I spend roughly 5 hours a month on each café.", ""),
        qa("assumptions", "What price per month do you have in mind?", "About £150 a month. I think cafés will pay that.", ""),
        qa("uncertainties", "Does the work stay the same month to month?", "I think the work per café is predictable, but year end is a rush.", ""),
        qa("constraints", "Besides the three cafés you already do, roughly how many more could you take on each month before you simply run out of hours yourself?", "", "This tests whether a flat fee scales with the hours you actually have."),
    ];
    i.cursor = 6;
    let heard = |kind: &str, text: &str| Heard { id: new_id(), kind: kind.into(), text: text.into(), ..Default::default() };
    i.heard = vec![
        heard("evidence", "Two of my three existing café clients said they'd prefer a fixed fee"),
        heard("evidence", "I spend roughly 5 hours a month on each café"),
        heard("assumption", "Hourly billing makes clients nervous about costs"),
        heard("assumption", "Cafés will pay about £150 a month"),
        heard("assumption", "The work per café is predictable"),
    ];
    i.understanding = Some(Understanding {
        project: "You are a freelance bookkeeper turning your hourly café bookkeeping work into a fixed-price monthly package (around £150 a month) aimed at small independent cafés.".into(),
        goal: "To move your café clients from hourly billing to a flat monthly fee so your income is steadier and clients have cost certainty, then grow that packaged service beyond your current three clients.".into(),
        state: "You have three café clients billed hourly, two of whom said they'd prefer a fixed fee. Each café takes roughly 5 hours a month. Price (about £150) and how the work varies month to month are not yet confirmed.".into(),
        uncertainty: "Whether the per-café workload is predictable enough to price as a flat fee. The year-end \"shoebox\" rush suggests the hours may spike in ways a fixed £150 can't absorb.".into(),
        outcome: "Café owners know their monthly cost up front and stop sending me shoeboxes of receipts at year end. I'd see it in on-time monthly records.".into(),
        ..Default::default()
    });
    i.outcome_confirmed = true;
    i.understood_n = i.qas.iter().filter(|q| !q.answer.trim().is_empty()).count() + 1;
    let mut p = cafe_proposal();
    // the first week as the canvas draws it
    p.tasks.retain(|t| !["Hours export from The Corner Café's till system", "Book a call with each willing client", "Ask what they pay now per year", "List the three cafés and their contacts", "Confirm your roadmap", "Set a target fee of £150", "Set the spike buffer"].contains(&t.title.as_str()));
    for r in p.roadmap.iter_mut() {
        r.confirmed = false;
    }
    i.proposal = Some(p);
    i.proposed_from = crate::ui::intake_key(&i);
    let id = i.id.clone();
    d.intakes.push(i);
    id
}

/// The Careers intake part-way through, as the Careers canvases draw it (Intake 03 to 11):
/// three questions (the third answered), what Resolve heard, the 04 cards and the kit. Screenshot
/// data only (`--view careers-<step>`); in memory, never saved. Copy is the canvas's sample.
pub fn careers_sample(d: &mut WorkspaceData) -> Id {
    use crate::resolve::careers::{self, Cards, Career, CvTemplate, Gate, Kit, LinkedinFix, Proof};
    use crate::resolve::intake::{Attachment, Intake};
    use crate::resolve::journey::{Heard, Qa};
    let mut i = Intake::new();
    i.mode = Mode::Careers;
    i.situation = "job_search".into();
    i.name = "Simeon".into();
    i.greeted = true;
    i.input = careers::EXAMPLE.into();
    i.attachments = vec![
        Attachment { name: "Simeon_CV_2026.pdf".into(), text: "Senior Product Owner, Capgemini. Senior PM, OneAdvanced: owned the managed services portfolio end to end. Partner Manager, OneAdvanced: achieved both Microsoft Solution Partner designations, in both years.".into(), place: "cv".into(), how: "you".into(), why: String::new(), unreadable: false },
        Attachment { name: "LinkedIn.pdf".into(), text: "Headline: Senior Product Owner.".into(), place: "cv".into(), how: "you".into(), why: String::new(), unreadable: false },
    ];
    i.stage = "questions".into();
    i.ai_covered = vec!["role".into(), "company".into()];
    let qa = |target: &str, q: &str, a: &str, why: &str| Qa { id: new_id(), target: target.into(), question: q.into(), answer: a.into(), classified: a.into(), why: why.into(), source: "sample".into(), ..Default::default() };
    i.qas = vec![
        qa("pay", "What's the lowest base pay you'd accept?", "I'd rather keep that private here, but I know my number.", ""),
        qa("contract", "Permanent, contract, or either?", "Permanent roles only.", ""),
        qa("location", "How many days a week can you be in an office, and where?", "Two a week at most. London is fine, Birmingham is better.", "Sets your office-days gate: Reviewer fails any job that asks for more."),
    ];
    i.cursor = 2;
    let heard = |kind: &str, text: &str| Heard { id: new_id(), kind: kind.into(), text: text.into(), ..Default::default() };
    i.heard = vec![heard("gate", "Head of Product or Director level"), heard("gate", "Permanent roles only"), heard("preference", "Identity or platform products"), heard("question", "Is travel to London once a fortnight fine?")];
    let cards = Cards {
        job: "Head of Product or Director of Product, in identity, platform or customer access.".into(),
        hard_gates: "Head of Product or Director · 2 office days a week at most · minimum base pay · permanent · identity or platform".into(),
        preferences: "Under 2 hours from Birmingham · scale-up or enterprise, 200+ people".into(),
        win: "A Head of Product role, at or above your minimum, signed by March".into(),
        strengths: "Platform and managed services depth. Partner programmes with results.".into(),
        gaps: "Headline says Senior Product Owner. Few numbers on outcomes.".into(),
        fix: "Headline, LinkedIn About, add a number to 6 proof points.".into(),
        broad: false,
    };
    let gate = |rule: &str, hard: bool, why: &str, check: &str| Gate { rule: rule.into(), hard, why: why.into(), check: check.into() };
    let mut kit = Kit {
        search_name: "Head of Product search".into(),
        target: "Head of Product or Director of Product, identity, platform or customer access".into(),
        do_first: "Fix your CV headline".into(),
        why: "It says Senior Product Owner, and every role you want says Head of Product. Reviewers read the headline first.".into(),
        gates: vec![
            gate("Head of Product or Director level", true, "The level you're aiming for.", "kept"),
            gate("Minimum base pay met", true, "Your minimum, kept private.", "kept"),
            gate("Permanent contract", true, "You said permanent roles only.", "kept"),
            gate("No more than 2 office days a week", true, "You said family time and the commute from Birmingham come first, and two London days a month is comfortable.", ""),
            gate("Identity, platform or customer access", true, "Where your proof points are strongest.", ""),
            gate("Under 2 hours from Birmingham", false, "Birmingham is better than London.", ""),
            gate("Scale-up or enterprise, 200+ people", false, "No agencies.", ""),
        ],
        templates: vec![
            CvTemplate { name: "Platform".into(), suits: "platform, identity, customer access, managed services".into(), headline: "Product leader for platforms and customer access at national scale".into(), default: true },
            CvTemplate { name: "Identity".into(), suits: "identity, verification, account security".into(), headline: String::new(), default: false },
            CvTemplate { name: "General".into(), suits: "anything outside the two above".into(), headline: String::new(), default: false },
        ],
        letter_structure: "Hook, proof, fit, close".into(),
        letter: String::new(),
        evidence: vec![
            Proof { point: "Achieved both Microsoft Solution Partner designations, in both years".into(), theme: "Partners and channels".into(), role: "OneAdvanced, Partner Manager".into(), number: "2 of 2".into(), source: "from CV, 2025".into() },
            Proof { point: "Owned the managed services portfolio end to end".into(), theme: "Platform scale".into(), role: "OneAdvanced, Senior PM".into(), number: String::new(), source: "from Interview debrief".into() },
            Proof { point: "Shipped a sign-in journey for a national identity service".into(), theme: "Identity".into(), role: "Capgemini, Senior PO".into(), number: String::new(), source: "from CV, 2026".into() },
            Proof { point: "Rebuilt the cloud services catalogue for enterprise buyers".into(), theme: "Platform scale".into(), role: "CDW".into(), number: String::new(), source: "from Evidence session".into() },
        ],
        linkedin: vec![
            LinkedinFix { area: "Headline".into(), fix: "Say Head of Product, not Senior Product Owner".into() },
            LinkedinFix { area: "About".into(), fix: "Lead with platforms and identity".into() },
            LinkedinFix { area: "OneAdvanced, Senior PM".into(), fix: "Add a number to the managed services work".into() },
            LinkedinFix { area: "Capgemini, Senior PO".into(), fix: "Name the national identity service".into() },
        ],
        weekly_target: 10,
        daily_found: 20,
        week: vec![],
        offline: false,
    };
    careers::finish(&mut kit);
    let mut c = Career { cards: Some(cards), kit: Some(kit), ..Default::default() };
    c.cards_n = i.qas.iter().filter(|q| !q.answer.trim().is_empty()).count() + 1;
    c.kit_from = c.cards_key();
    i.career = Some(c);
    i.outcome_confirmed = true;
    i.theme_ix = 3;
    let id = i.id.clone();
    d.intakes.push(i);
    id
}
