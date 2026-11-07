# Resolve 2 redesign: engineer prompts, one per phase

7 November 2026. Paste one prompt per phase. Wait for the phase report and approve it before the next.

```
RESOLVE 2 REDESIGN, PHASE 0 OF 6: FOUNDATIONS
Branch: style/p0-foundations

STANDING RULES (every phase)
Repo simeonfab/resolve2. Read CLAUDE.md first. Brief: docs/briefs/2026-11-07-build-brief-packs-a-d.md. The live doc https://claude.ai/artifact/1yKbJZFzfLsgBri5m9tfHi wins if they differ. Canvases (read with your artifact tool when you need a screen): A https://claude.ai/artifact/UUpPfSfTcEphWNP2nKaXHP, B https://claude.ai/artifact/K4phbfZQVnS6GJQa3NpqFg, C https://claude.ai/artifact/TcsdU9czKbts2U1hruGPSC, D https://claude.ai/artifact/91HZUemQxHXL5DfH5Jdcvv. If you cannot open them, say so and work from the brief.
1. Pull before you start. Work on the branch named below. Commit after every working step. Merge to main only when cargo build passes and --selftest ends "N passed, 0 failed". Push at the end of every session.
2. Colours, fonts, sizes, radii and spacing only in src/ui/theme.rs. Component styling only in src/ui/widgets.rs. No literals in views. Navigation only in the tables in src/ui/nav.rs.
3. Copy in the brief is final. Use it exactly. No em dashes anywhere in the UI.
4. Every view in four states (full, empty, loading, error) and at 1920 x 1080 with the content caps in Rules, item 6.
5. Every complete, delete, move and AI write goes through history.rs with an Undo toast.
6. Never invent data, AI output or test results. Mark anything unchecked "unverified".
7. Unknowns: check each one listed for this phase in the code and report what you found BEFORE building on it. Never work around one silently.
8. If the brief is unclear or conflicts with the code, stop and ask. Do not guess.

END OF PHASE: stop and send me (1) what you built, ticket by ticket; (2) screenshots of every new or changed view in full, empty, loading, error and at 1920 x 1080; (3) the --selftest result; (4) the Done when checklist from the brief, ticked or explained; (5) anything you could not do, and why. Then wait for my go-ahead.

FIRST, BEFORE ANY CODE
1. Ownership. CLAUDE.md gives theme.rs and widgets.rs to the visual layer and tells the builder not to change them. For this redesign you own both. Update the ownership table in CLAUDE.md and AGENTS.md in your first commit to say so.
2. Screenshots in four states. Check whether --view can force a state and a window size. If not, add --state full|empty|loading|error and --size WIDTHxHEIGHT to the screenshot mode, with sample data for each state. Report what you did.
3. Voice capture (E2). Check what the app can use on Windows for speech to text without sending audio anywhere the user has not agreed to. Propose one approach to me with its cost, download size and privacy, and wait for my answer before building it. Build everything else in this phase meanwhile.

BUILD (tickets from the brief)
- E18 Tokens in theme.rs: everything in the brief's Tokens section, including pane, terracotta on dark, the text-on-dark set, shadows, resized HERO, QUESTION and ANSWER, and the contrast fixes (no amber text on light; green text #3F5A33 on #E8EFE3).
- Shared components in widgets.rs that the later phases use: cards, buttons, pills, chips, inputs, skeletons, empty and error blocks.
- E16 and B-E18 Motion helper: the easings and timings in the brief's Motion section, width tween and origin-aware scale, and the app-wide reduced-motion flag (every move becomes a fade of 120 ms or less, loops hold still).
- E17 and B-E20 Glow and bloom, and hero shadows, as textures or gradient mesh.
- B-E10 Toasts: one at a time, fast swap, 4 s Undo timer, 2.6 s plain.
- E2 Voice: the mic button and recording state as a shared component. Speech to text only after I approve the approach.

DONE WHEN
A component gallery view (--view gallery) shows every token, component, toast and motion sample, in light and dark, with reduced motion on and off. Plus the brief's Done when list.
```

```
RESOLVE 2 REDESIGN, PHASE 1 OF 6: THE SHELL
Branch: nav/p1-shell. Start only after Phase 0 is approved and merged.

STANDING RULES (every phase)
Repo simeonfab/resolve2. Read CLAUDE.md first. Brief: docs/briefs/2026-11-07-build-brief-packs-a-d.md. The live doc https://claude.ai/artifact/1yKbJZFzfLsgBri5m9tfHi wins if they differ. Canvases (read with your artifact tool when you need a screen): A https://claude.ai/artifact/UUpPfSfTcEphWNP2nKaXHP, B https://claude.ai/artifact/K4phbfZQVnS6GJQa3NpqFg, C https://claude.ai/artifact/TcsdU9czKbts2U1hruGPSC, D https://claude.ai/artifact/91HZUemQxHXL5DfH5Jdcvv. If you cannot open them, say so and work from the brief.
1. Pull before you start. Work on the branch named below. Commit after every working step. Merge to main only when cargo build passes and --selftest ends "N passed, 0 failed". Push at the end of every session.
2. Colours, fonts, sizes, radii and spacing only in src/ui/theme.rs. Component styling only in src/ui/widgets.rs. No literals in views. Navigation only in the tables in src/ui/nav.rs.
3. Copy in the brief is final. Use it exactly. No em dashes anywhere in the UI.
4. Every view in four states (full, empty, loading, error) and at 1920 x 1080 with the content caps in Rules, item 6.
5. Every complete, delete, move and AI write goes through history.rs with an Undo toast.
6. Never invent data, AI output or test results. Mark anything unchecked "unverified".
7. Unknowns: check each one listed for this phase in the code and report what you found BEFORE building on it. Never work around one silently.
8. If the brief is unclear or conflicts with the code, stop and ask. Do not guess.

END OF PHASE: stop and send me (1) what you built, ticket by ticket; (2) screenshots of every new or changed view in full, empty, loading, error and at 1920 x 1080; (3) the --selftest result; (4) the Done when checklist from the brief, ticked or explained; (5) anything you could not do, and why. Then wait for my go-ahead.

BUILD (Pack B canvas; brief sections The shell and Pack B)
- B-E24 Rename the middle sidebar group to YOUR PROJECT and Knowledge to Knowledge base, in the nav.rs tables.
- B-E1 Sidebar collapse to 64 with tooltips, remembered per workspace, Ctrl \.
- B-E2 "Continue setting up…" card in the sidebar footer (the Home card comes in Phase 2).
- B-E3 Floating Ask Resolve bar on every screen, with the per-section placeholder and suggestions from nav.rs, opening the sheet (A3). Keep the Ask AI button.
- B-E17 Undo and Redo show a toast naming what changed.
- B-E14 Search palette over the current screen, using the existing search prefixes.

Leave the screens inside the shell as they are. Phase 2 restyles them.

DONE WHEN
The shell renders around every existing screen, expanded and collapsed, at 1280 x 800 and 1920 x 1080. Plus the brief's Done when list.
```

```
RESOLVE 2 REDESIGN, PHASE 2 OF 6: DAILY SCREENS
Branch: nav/p2-daily. Start only after Phase 1 is approved and merged.

STANDING RULES (every phase)
Repo simeonfab/resolve2. Read CLAUDE.md first. Brief: docs/briefs/2026-11-07-build-brief-packs-a-d.md. The live doc https://claude.ai/artifact/1yKbJZFzfLsgBri5m9tfHi wins if they differ. Canvases (read with your artifact tool when you need a screen): A https://claude.ai/artifact/UUpPfSfTcEphWNP2nKaXHP, B https://claude.ai/artifact/K4phbfZQVnS6GJQa3NpqFg, C https://claude.ai/artifact/TcsdU9czKbts2U1hruGPSC, D https://claude.ai/artifact/91HZUemQxHXL5DfH5Jdcvv. If you cannot open them, say so and work from the brief.
1. Pull before you start. Work on the branch named below. Commit after every working step. Merge to main only when cargo build passes and --selftest ends "N passed, 0 failed". Push at the end of every session.
2. Colours, fonts, sizes, radii and spacing only in src/ui/theme.rs. Component styling only in src/ui/widgets.rs. No literals in views. Navigation only in the tables in src/ui/nav.rs.
3. Copy in the brief is final. Use it exactly. No em dashes anywhere in the UI.
4. Every view in four states (full, empty, loading, error) and at 1920 x 1080 with the content caps in Rules, item 6.
5. Every complete, delete, move and AI write goes through history.rs with an Undo toast.
6. Never invent data, AI output or test results. Mark anything unchecked "unverified".
7. Unknowns: check each one listed for this phase in the code and report what you found BEFORE building on it. Never work around one silently.
8. If the brief is unclear or conflicts with the code, stop and ask. Do not guess.

END OF PHASE: stop and send me (1) what you built, ticket by ticket; (2) screenshots of every new or changed view in full, empty, loading, error and at 1920 x 1080; (3) the --selftest result; (4) the Done when checklist from the brief, ticked or explained; (5) anything you could not do, and why. Then wait for my go-ahead.

CHECK AND REPORT FIRST (unknowns)
- B-E5 Can a task be marked waiting? If not, say what adding the state would take.
- B-E13 Do events support all-day and repeats?
- B-E22 Does a task hold a start time?
- B-E6 Does a failed AI suggestion on Home stop the rest of Home loading today?
- Open decision B-E11: what #Name does in Add task when no project has that name. Ask me; build the rest of Add task meanwhile.

BUILD (Pack B canvas)
- B-E4 Home: hero from the project's first move, Since Monday, Waiting on, Projects, Captured, and the Continue setting up card. B-E6 a failed suggestion shows its error in its own card.
- B-E21 Plan my day sheet: today it is a saved AI prompt in mcp.rs that replies as chat text. Ask the AI for a structured reply (time, item, reason, won't fit) and draw rows. Fallback lists today's items by time. B-E22 Use this plan writes a start time to each task. B-E23 Change box re-plans with the typed constraint; Move to Monday sets the due date.
- B-E7 Projects: cards with outcome, next move, phase counts, open questions and the ⋯ menu. Three columns at 1920 (B-E19).
- B-E8 Tasks: Now / Waiting on / Upcoming from the roadmap, chips, View completed; list and board; All projects view. B-E9 the task editor as a 440 push panel.
- B-E11 Add task: highlight recognised words, chips without emoji, Goes to picker.
- B-E12 Calendar: Agenda view added to Month and Week; calendars list with toggles and New calendar; event editor. B-E13 all-day and repeats as the check allows.
- B-E15 "Add as a task" from a search with no results.
- B-E16 Voice on the Ask bar, Add task and task notes.
- B-E19 The 1920 caps for every screen in this phase.
```

```
RESOLVE 2 REDESIGN, PHASE 3 OF 6: THE INTAKE
Branch: intake/p3-redesign. Start only after Phase 2 is approved and merged.

STANDING RULES (every phase)
Repo simeonfab/resolve2. Read CLAUDE.md first. Brief: docs/briefs/2026-11-07-build-brief-packs-a-d.md. The live doc https://claude.ai/artifact/1yKbJZFzfLsgBri5m9tfHi wins if they differ. Canvases (read with your artifact tool when you need a screen): A https://claude.ai/artifact/UUpPfSfTcEphWNP2nKaXHP, B https://claude.ai/artifact/K4phbfZQVnS6GJQa3NpqFg, C https://claude.ai/artifact/TcsdU9czKbts2U1hruGPSC, D https://claude.ai/artifact/91HZUemQxHXL5DfH5Jdcvv. If you cannot open them, say so and work from the brief.
1. Pull before you start. Work on the branch named below. Commit after every working step. Merge to main only when cargo build passes and --selftest ends "N passed, 0 failed". Push at the end of every session.
2. Colours, fonts, sizes, radii and spacing only in src/ui/theme.rs. Component styling only in src/ui/widgets.rs. No literals in views. Navigation only in the tables in src/ui/nav.rs.
3. Copy in the brief is final. Use it exactly. No em dashes anywhere in the UI.
4. Every view in four states (full, empty, loading, error) and at 1920 x 1080 with the content caps in Rules, item 6.
5. Every complete, delete, move and AI write goes through history.rs with an Undo toast.
6. Never invent data, AI output or test results. Mark anything unchecked "unverified".
7. Unknowns: check each one listed for this phase in the code and report what you found BEFORE building on it. Never work around one silently.
8. If the brief is unclear or conflicts with the code, stop and ask. Do not guess.

END OF PHASE: stop and send me (1) what you built, ticket by ticket; (2) screenshots of every new or changed view in full, empty, loading, error and at 1920 x 1080; (3) the --selftest result; (4) the Done when checklist from the brief, ticked or explained; (5) anything you could not do, and why. Then wait for my go-ahead.

CHECK AND REPORT FIRST
- E28 How the app hears about AI limits today: the budget stop (check_budget in ai.rs), provider rate limits and the Claude Code usage cap. Report what is caught and what is not. Open decision D7: "Daily limit" copy means the monthly AI budget; confirm the copy fits what you find.
- D5 Measure how long the proposal takes today on each provider you can run, and report the numbers before proposing any speed fix.
- E14 What build progress events the adopt step emits today, so the bridge can follow real progress.

BUILD (Pack A canvas, screens 01 to 12, states ST1 to ST6). The intake logic stays as it is. Change only what it looks like and the tickets below.
- E1 Welcome with three paths; new idea and stuck disabled with Coming soon.
- E21 Use a template (8 headings) and Try an example. E22 the new 11-question pull-in prompt, with Copy and Copied ✓.
- E23 Heard panel: label menu, pencil and × on hover, inline edit (Enter, Esc), first-time hint.
- E24 Back · Skip this question · Next, and Skip the rest with the coverage bar.
- E25 Save and exit plus the ⋯ menu with Restart; Home card. E9 Resume card on launch mid-intake.
- E4 Stepper rail, 4 steps, collapsing to 72 on wide steps. E20 1920 rules: column 960 centred, heard panel 440.
- E3 No clear outcome screen with the floor reason and one question; the answer re-runs the check.
- E5 Waiting card driven by real progress events, seconds counter, 15 s bar, a change at least every 5 s. E6 the 15 s hand-off from TARGETS with "Why this question"; drop late answers. E7 long-wait note after 15 s with Leave and come back later. E8 error state per step with Try again focused plus the manual path.
- E10 AI limit screen (screen 12): catch the budget error and the provider caps found above, freeze the checklist, Continue without AI. E11 its checklist strings match the waiting checklist exactly.
- E13 Now / Next / Later strip on Next move. E12 Themes screen: On your roadmap list, segment bar, card. E26 Skip to the whole picture on 06 to 08; Edit the full proposal on 09 only; reassurance line.
- E14 Bridge beats B0 to B4 bound to live build progress, minimum 7.2 s. E15 Reveal D1 to D4 (open decision D4: project name, three counts, the first move, Open my project). E19 a click skips long reveal sequences.
- E27 New self-test: the first move equals task 1 of week 1 and sits in the first Now theme.
- E29 is parked. Do not build it.

Screenshots: every screen 01 to 12 and states ST1 to ST6, at 1280 x 800 and 1920 x 1080. Force screen 12 with the budget stop.
```

```
RESOLVE 2 REDESIGN, PHASE 4 OF 6: PROJECT SECTIONS
Branch: nav/p4-project. Start only after Phase 3 is approved and merged.

STANDING RULES (every phase)
Repo simeonfab/resolve2. Read CLAUDE.md first. Brief: docs/briefs/2026-11-07-build-brief-packs-a-d.md. The live doc https://claude.ai/artifact/1yKbJZFzfLsgBri5m9tfHi wins if they differ. Canvases (read with your artifact tool when you need a screen): A https://claude.ai/artifact/UUpPfSfTcEphWNP2nKaXHP, B https://claude.ai/artifact/K4phbfZQVnS6GJQa3NpqFg, C https://claude.ai/artifact/TcsdU9czKbts2U1hruGPSC, D https://claude.ai/artifact/91HZUemQxHXL5DfH5Jdcvv. If you cannot open them, say so and work from the brief.
1. Pull before you start. Work on the branch named below. Commit after every working step. Merge to main only when cargo build passes and --selftest ends "N passed, 0 failed". Push at the end of every session.
2. Colours, fonts, sizes, radii and spacing only in src/ui/theme.rs. Component styling only in src/ui/widgets.rs. No literals in views. Navigation only in the tables in src/ui/nav.rs.
3. Copy in the brief is final. Use it exactly. No em dashes anywhere in the UI.
4. Every view in four states (full, empty, loading, error) and at 1920 x 1080 with the content caps in Rules, item 6.
5. Every complete, delete, move and AI write goes through history.rs with an Undo toast.
6. Never invent data, AI output or test results. Mark anything unchecked "unverified".
7. Unknowns: check each one listed for this phase in the code and report what you found BEFORE building on it. Never work around one silently.
8. If the brief is unclear or conflicts with the code, stop and ask. Do not guess.

END OF PHASE: stop and send me (1) what you built, ticket by ticket; (2) screenshots of every new or changed view in full, empty, loading, error and at 1920 x 1080; (3) the --selftest result; (4) the Done when checklist from the brief, ticked or explained; (5) anything you could not do, and why. Then wait for my go-ahead.

CHECK AND REPORT FIRST (unknowns)
- C-E5 Can a record link a roadmap theme to a hypothesis?
- C-E8 Do milestones exist as records?
- C-E7 What happens today when a phase move fails?
- C-E15 Does import_pages report errors per file?
- C-E18 Is there a CSV import dialog today?
- Open decision: the one-line description of what Frameworks does is parked. Leave a slot for it and use no placeholder text that pretends to be final.

BUILD (Pack C canvas)
- C-E1 Overview layout; C-E2 Weekly review in the header; C-E3 a failed suggestion shows its error, the rest loads.
- C-E4 Roadmap theme cards with tag, gate chip, progress, expand in place (C-E26 height spring). C-E5 gates as the check allows. C-E6 Whole picture overlay. C-E7 phase move with F6; a failed move rolls back and says so. C-E8 right rail.
- C-E9 Frameworks: registry strip, tree with negations, experiments with prediction and threshold, node editor. C-E10 result, then status, then gate, then toast, in that order.
- C-E11 Proposal editor: section list, inline edits, Back keeps edits, Adopt in the top bar.
- C-E12 Knowledge base: five tabs, filter chips with counts, card library, grounding bar. C-E13 Add sheet with five options; new item lands first, outlined 2 s. C-E14 preview panel. C-E15 per-file import errors.
- C-E16 Page editor toolbar, / menu, Saved state, to-dos count, Linked chips. C-E17 New page dialog with built-in templates plus your own.
- C-E18 Databases: list, table, group and sort, CSV import dialog, failed rows listed.
- C-E19 AI Team: role cards, earlier chats, proposed-change card, budget error in the chat.
- C-E20 Handoffs: For and Who, five-part brief, Sent and Done, Reopen, plain-brief fallback.
- C-E24 Voice on Handoffs and AI Team. C-E25 the 1920 caps.
```

```
RESOLVE 2 REDESIGN, PHASE 5 OF 6: SETTINGS AND HELP
Branch: nav/p5-settings-help. Start only after Phase 4 is approved and merged.

STANDING RULES (every phase)
Repo simeonfab/resolve2. Read CLAUDE.md first. Brief: docs/briefs/2026-11-07-build-brief-packs-a-d.md. The live doc https://claude.ai/artifact/1yKbJZFzfLsgBri5m9tfHi wins if they differ. Canvases (read with your artifact tool when you need a screen): A https://claude.ai/artifact/UUpPfSfTcEphWNP2nKaXHP, B https://claude.ai/artifact/K4phbfZQVnS6GJQa3NpqFg, C https://claude.ai/artifact/TcsdU9czKbts2U1hruGPSC, D https://claude.ai/artifact/91HZUemQxHXL5DfH5Jdcvv. If you cannot open them, say so and work from the brief.
1. Pull before you start. Work on the branch named below. Commit after every working step. Merge to main only when cargo build passes and --selftest ends "N passed, 0 failed". Push at the end of every session.
2. Colours, fonts, sizes, radii and spacing only in src/ui/theme.rs. Component styling only in src/ui/widgets.rs. No literals in views. Navigation only in the tables in src/ui/nav.rs.
3. Copy in the brief is final. Use it exactly. No em dashes anywhere in the UI.
4. Every view in four states (full, empty, loading, error) and at 1920 x 1080 with the content caps in Rules, item 6.
5. Every complete, delete, move and AI write goes through history.rs with an Undo toast.
6. Never invent data, AI output or test results. Mark anything unchecked "unverified".
7. Unknowns: check each one listed for this phase in the code and report what you found BEFORE building on it. Never work around one silently.
8. If the brief is unclear or conflicts with the code, stop and ask. Do not guess.

END OF PHASE: stop and send me (1) what you built, ticket by ticket; (2) screenshots of every new or changed view in full, empty, loading, error and at 1920 x 1080; (3) the --selftest result; (4) the Done when checklist from the brief, ticked or explained; (5) anything you could not do, and why. Then wait for my go-ahead.

CHECK AND REPORT FIRST
- C-E21 Is month-to-date AI cost tracked today? The spending card needs it.
- C-E22 What the phone pairing needs in a QR code (address and token), and whether a QR crate is already in the build. Ask before adding a new crate.

BUILD (Pack C canvas)
- C-E21 Settings in four tabs, as the brief names them, with the spending card.
- C-E22 Phone pairing QR code.
- C-E23 Help: guide list, Welcome card, Report a problem dialog with the included details, clipboard fallback.
- C-E24 Voice on Report.
```

```
RESOLVE 2 REDESIGN, PHASE 6 OF 6: THE PHONE PAGE
Branch: style/p6-phone. Start only after Phase 5 is approved and merged.

STANDING RULES (every phase)
Repo simeonfab/resolve2. Read CLAUDE.md first. Brief: docs/briefs/2026-11-07-build-brief-packs-a-d.md. The live doc https://claude.ai/artifact/1yKbJZFzfLsgBri5m9tfHi wins if they differ. Canvases (read with your artifact tool when you need a screen): A https://claude.ai/artifact/UUpPfSfTcEphWNP2nKaXHP, B https://claude.ai/artifact/K4phbfZQVnS6GJQa3NpqFg, C https://claude.ai/artifact/TcsdU9czKbts2U1hruGPSC, D https://claude.ai/artifact/91HZUemQxHXL5DfH5Jdcvv. If you cannot open them, say so and work from the brief.
1. Pull before you start. Work on the branch named below. Commit after every working step. Merge to main only when cargo build passes and --selftest ends "N passed, 0 failed". Push at the end of every session.
2. Colours, fonts, sizes, radii and spacing only in src/ui/theme.rs. Component styling only in src/ui/widgets.rs. No literals in views. Navigation only in the tables in src/ui/nav.rs.
3. Copy in the brief is final. Use it exactly. No em dashes anywhere in the UI.
4. Every view in four states (full, empty, loading, error) and at 1920 x 1080 with the content caps in Rules, item 6.
5. Every complete, delete, move and AI write goes through history.rs with an Undo toast.
6. Never invent data, AI output or test results. Mark anything unchecked "unverified".
7. Unknowns: check each one listed for this phase in the code and report what you found BEFORE building on it. Never work around one silently.
8. If the brief is unclear or conflicts with the code, stop and ask. Do not guess.

END OF PHASE: stop and send me (1) what you built, ticket by ticket; (2) screenshots of every new or changed view in full, empty, loading, error and at 1920 x 1080; (3) the --selftest result; (4) the Done when checklist from the brief, ticked or explained; (5) anything you could not do, and why. Then wait for my go-ahead.

The phone page is the HTML in server.rs (PHONE_HTML), not egui. Tokens still come from theme.rs values: generate the page's CSS variables from them rather than retyping hex codes.

CHECK AND REPORT FIRST (unknowns)
- D-E10 Can a completed task be reopened?
- D-E7 Is a chat call reachable from the phone over /mcp? If not, say what adding one with the budget check takes.
- D-E1 What serving the intake to the phone needs (the journey calls need a phone route). Report the size of the job before building it.
- D-E14 Browser speech input on the testers' phones.
- Open decision D3 and D5: Ask tab and swipe to Tomorrow. Build as drawn; the phone is low priority, so tell me if either costs more than a few hours.

BUILD (Pack D canvas)
- D-E2 Restyle with Resolve tokens and bundle the fonts.
- D-E3 Light bottom bar (Tasks, Calendar, Search) and Ask pill; Today, Upcoming, Inbox segments. D-E4 Your next move card on Today.
- D-E5 Quick add with live highlighting (needs a parse-only call).
- D-E6 Offline: cache lists with a time, queue captures, send on reconnect, banner states. D-E13 tell an error from not connected.
- D-E7 Ask on the phone, per the check. D-E8 swipe left to Tomorrow. D-E10 Undo after finishing.
- D-E11 Search chips, grouped results, Add as a task. D-E12 Agenda with dated tasks beside events and calendar colours.
- D-E9 Motion as CSS transitions, honouring reduced motion. D-E14 voice. D-E15 pairing by QR (with C-E22). D-E16 nothing scrolls sideways at 360 except the chip row.
- D-E1 the intake on the phone, only after I approve the size you reported.

Screenshots at 390 x 844 and 360 x 780. Every target 44 px or more.
```
