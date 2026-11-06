Design Pack D: Resolve 2 on the phone (pack 4 of 4)

WHAT YOU ARE DESIGNING
Resolve 2 on a phone. The desktop app serves a phone page over the local network today. The original Resolve mockups were phone-first, so this pack comes closest to them.

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

SCREENS
1. The intake (Pack A screens 1 to 12) at phone size. Start from the original mockups, which are already phone-sized, and match Pack A's content and motion.
2. Today, Upcoming and Inbox task views (Work_Tab, Work_Motion). Tick completes, row opens.
3. Quick add (Work_Motion G4) with the one-line parser.
4. Calendar agenda (new).
5. Search (new).
6. Navigation: a bottom bar or menu that reaches Home, Tasks, Calendar, Projects and Ask Resolve. Choose one and justify it in one line.
7. Ask Resolve chat sheet (Agent_Chat, Flow Spec E4).
8. Not connected: the phone cannot reach the desktop app (desktop asleep or off the network). Say what to do.

ENGINE FUNCTIONS BEHIND THESE SCREENS
Tasks: list_tasks, create_task, update_task, complete_task. Calendar: list_events, get_agenda. Search: search. Home: workspace_overview, plan_my_day. The intake runs on the desktop engine.

DELIVER
1. Phone frames at 390 x 844 for every screen and state. Check each at 360 x 780 for overflow.
2. Touch targets at least 44 x 44.
3. Every animation named, with its Flow Spec reference.
4. A list of anything designed that the engine does not do, as engineering tickets. Expect at least one: the intake is not served to the phone today.

NOT IN THIS PACK: desktop frames (Packs A to C).
