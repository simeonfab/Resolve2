Design Pack C: Resolve 2 project sections (pack 3 of 4)

WHAT YOU ARE DESIGNING
Everything inside one project, plus Settings and Help. These screens sit inside the Pack B shell, with the project picker selecting which project they show.

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
1. Overview (Dashboard patterns, pic 13). Do this next, progress, ResolveOS actions, what matters now, project context, plan page, tasks board.
2. Roadmap (Roadmap_Tab, Roadmap_Motion, pic 15). Now / Next / Later, by phase or by theme, with the toggle (Flow Spec F2). Sequence, gates, milestones, dependencies, coming up. Action card expands inline (F3, F4). "Whole picture" pull-up (F5).
3. Frameworks (new). Registry with Hypothesis Zero first. The H0 tree editor: the main hypothesis, its supporting hypotheses, experiments and evidence, readable by a non-technical founder.
4. "See and edit everything" proposal editor (new). Opened from the end of the intake. Edits the whole proposal: H0, hypotheses, experiments, uncertainties and knowledge pages.
5. Knowledge (Knowledge_Base_Tab, Knowledge_Motion, pic 18). Tabs: Brief and documents, Research, Cut list, Evidence and Records, Advanced. Library grid, filter chips, add or brain-dump sheet, preview (Flow Spec H1 to H4).
6. Page editor (new). Rich text pages with templates, in the same type system.
7. Databases (new). Table view, columns, rows, CSV import. Lives under Knowledge, Advanced.
8. AI Team (Team_Tab, Agent_Chat, pics 16 and 17). General assistant, Product Director, Validation Lead, Growth Lead, Tech Lead. Role cards aligned, conversations, chat sheet.
9. Handoffs (new; check Work_Packet_Brief and Work_Packet_Patterns for brief layout). Create a brief for someone, Sent and Done lists, reopen.
10. Settings (Settings, pic 19). Four tabs: AI and MCP (provider, prices, spending limit), Sync and phone, Workspaces, About.
11. Help (new). Sectioned guide with the alpha welcome and Report a problem.

ENGINE FUNCTIONS BEHIND THESE SCREENS
Overview, Roadmap, Frameworks: resolve_project, list_records, add_record, update_record, update_project. Knowledge and pages: list_pages, read_page, create_page, update_page, list_page_todos, import_pages. Databases: list_databases, create_database, query_database, add_database_column, add_database_row, update_database_row, import_csv. Weekly review: weekly_review. AI Team, Handoffs, Settings and the proposal editor are internal app functions.

DELIVER
1. Desktop 1280 x 800 for every screen and state, inside the Pack B shell. Check each at 1920 x 1080.
2. Every animation named, with its Flow Spec reference, or exact values for new transitions.
3. Additions to the tokens sheet, if any.
4. A list of anything designed that the engine does not do, as engineering tickets.

NOT IN THIS PACK: the shell and daily screens (Pack B), phone frames (Pack D).
