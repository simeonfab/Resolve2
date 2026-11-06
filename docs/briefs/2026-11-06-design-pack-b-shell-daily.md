Design Pack B: Resolve 2 app shell and daily screens (pack 2 of 4)

WHAT YOU ARE DESIGNING
The frame the whole app lives in, and the screens a user opens every day. Resolve 2 is a desktop app (Windows first). The original Resolve mockups are phone-first. Bring them to a desktop window at the same quality.

SOURCES (read first)
1. Screen designs: github.com/simeonfab/resolve-docs, folder mockups/, files ResolvePM_<name>_dc.html. Open them in a browser.
2. Motion: mockups/Resolve_Flow_Spec.html and mockups/MOTION-SYSTEM-HANDOFF.md. Use the values exactly. Ship the "match-seam" character.
3. Reference pictures 01 to 20: github.com/simeonfab/Resolve2, docs/reference/original-resolve/
4. Today's working screens (content and what each holds): Resolve2 docs/reports/2026-11-06-part4.md and its screenshots.
5. Pack A (intake) output from Design, for consistency.
6. Quality bar: the response.games home page. Match its level of polish and motion, not its design.

LOOK (same as Pack A)
Colours: nav-bg #1E1E1A, surface #F2EDE4, card #FFFFFF, ink #1A1A18, ink-soft #4A4A44, ink-ghost #9A9A92, border #DDD8CE, amber (accent) #C8A040, amber-light #FBF5E6, terracotta #8B4A2A, green #5E7D4F.
Type: DM Serif Display for headlines, Inter 400/500/600 for everything else.
Easings: e-spring cubic-bezier(.34,1.56,.64,1) arrivals and sheets. e-out cubic-bezier(.22,.61,.36,1) reveals. e-std cubic-bezier(.4,0,.2,1) dims and presses. e-in cubic-bezier(.55,0,.85,.35) exits.
The app is light. Reuse Pack A's tokens sheet. Do not invent new colours.

RULES
1. The logic is built and works. Design what people see, read and touch, not what the app does.
2. Every button maps to an engine function already listed below. If a design needs something new, list it as an engineering ticket. Never let it slip in silently.
3. Every screen in four states: empty, loading, error, full. Nothing overflows or overlaps at 1280 x 800 or 1920 x 1080.
4. Copy: second person, "you", never "the user". Every sentence starts with a capital letter. Plain, calm, short.

SCREENS (start from the mockup named; "new" means no mockup exists)
1. Sidebar (App_Shell, pic 11). Grouped: workspace switcher, Undo / Redo, Add task, Ask AI. Workspace: Home, Projects, Calendar. Product management, with a project picker: Overview, Roadmap, Tasks, Frameworks, Knowledge. Collaboration: AI Team, Handoffs. Footer: Search, Settings, Help, save status, unfinished intakes ("Continue setting up…"). Collapsed state too.
2. Ask Resolve bar (Global_Bar, Global_Bar_Contextual, pic 20). Changes placeholder and three suggestions per screen. Opens the chat sheet (Flow Spec E4).
3. Home (Dashboard, Dashboard_Content, Dashboard_Motion, pics 12 and 13). Across all projects: what to do today, agenda, progress, wins. Entrance and card motion from Flow Spec E1 to E8.
4. Projects (Portfolio). All projects as cards. "New project" opens the Pack A intake.
5. Tasks (Work_Tab, Work_Motion, pic 14). One project: list and board. "All projects": Captured, Today, Upcoming, Filters and labels. The tick completes, the row opens the editor (Flow Spec G2, G3). Progress bar fill (G1).
6. Add task (Work_Motion G4). Quick capture with the one-line parser, for example "Pay rent every month 9am p1 #Home". Show what was understood before saving.
7. Calendar (new). Calendars list, month, week and agenda views, event editor. Design it in the same system as Home.
8. Search (new). Everything in the workspace, grouped by type, keyboard first.
9. Toast and undo (Feedback_Motion, Flow Spec I1 to I3). One toast at a time, 4 s undo timer.

ENGINE FUNCTIONS BEHIND THESE SCREENS
Home: workspace_overview, get_agenda, list_projects, list_tasks, plan_my_day. Projects: list_projects, create_project, update_project. Tasks: list_tasks, create_task, update_task, complete_task, delete_item, organise_todos. Calendar: list_calendars, create_calendar, list_events, create_event, update_event, get_agenda. Search: search. Undo / Redo and workspaces are internal app functions.

DELIVER
1. Desktop 1280 x 800 for every screen and state. Check each at 1920 x 1080.
2. Sidebar expanded and collapsed, on every screen.
3. Every animation named, with its Flow Spec reference, or exact values for new desktop transitions.
4. Additions to Pack A's tokens sheet, if any.
5. A list of anything designed that the engine does not do, as engineering tickets.

NOT IN THIS PACK: the project sections (Pack C), phone frames (Pack D).
