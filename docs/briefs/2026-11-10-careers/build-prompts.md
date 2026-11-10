# Resolve Careers: engineer prompts, one per step

10 November 2026. For the Craft Anything builder. Paste one prompt per step. Wait for the step report and approve it before the next. Start Step 1 only after redesign Phase 5 is approved and merged.

Reference pack: this folder, `docs/briefs/2026-11-10-careers/`, pushed on branch `docs/careers-reference`. Step 1 merges it.

- `build-brief.pdf`: the full build brief with every screen.
- `screens/`: 46 PNG screenshots at 1280 wide, one per canvas artboard. File name = artboard name.
- `canvas-source/`: the HTML source of each artboard plus `canvas.json`, for exact copy, sizes and colours.
- Live versions: brief https://claude.ai/artifact/EzKD4RkhK5J6iYogLo1LeF, canvas https://claude.ai/artifact/FaYcRtkUkeDWbUqsWMFMDS.

```
RESOLVE CAREERS, STEP 1 OF 8: MODES
Branch: careers/s1-modes. Start only after redesign Phase 5 is approved and merged.

STANDING RULES (every step)
Repo simeonfab/resolve2. Read CLAUDE.md first. Brief: docs/briefs/2026-11-10-careers/build-brief.pdf. The live doc https://claude.ai/artifact/EzKD4RkhK5J6iYogLo1LeF wins if they differ.
SCREENS ARE THE REFERENCE. Before building any view, open its screenshot in docs/briefs/2026-11-10-careers/screens/ (file name = artboard name, listed in each step). For exact copy, spacing and colours, read the matching file in docs/briefs/2026-11-10-careers/canvas-source/. The live canvas is https://claude.ai/artifact/FaYcRtkUkeDWbUqsWMFMDS (read it with your artifact tool if you can; if not, say so and use the folder). Match the screens. Where your build differs from a screen, say where and why in the report. Screen content is sample data: never ship it as real data.
1. Pull before you start. Work on the branch named below. Commit after every working step. Merge to main only when cargo build passes and --selftest ends "N passed, 0 failed". Push at the end of every session.
2. Colours, fonts, sizes, radii and spacing only in src/ui/theme.rs. Component styling only in src/ui/widgets.rs. No literals in views. Navigation only in the tables in src/ui/nav.rs. Careers adds no new tokens: if a screen seems to need one, ask me.
3. Copy in the brief and on the screens is final. Use it exactly. No em dashes anywhere in the UI.
4. Every Careers view in four states (full, empty, loading, error), screenshotted at 1280 x 800 and 1920 x 1080.
5. Every complete, delete, move and AI write goes through history.rs with an Undo toast.
6. PM mode must not change. Every existing workspace loads and behaves exactly as before.
7. Never invent data, AI output or test results. Mark anything unchecked "unverified".
8. Unknowns: check each one listed for this step in the code and report what you found BEFORE building on it. Never work around one silently.
9. If the brief is unclear or conflicts with the code, stop and ask. Do not guess.

END OF STEP: stop and send me (1) what you built, item by item; (2) screenshots of every new or changed view in full, empty, loading, error at 1280 x 800 and 1920 x 1080, each next to the matching reference screen; (3) the --selftest result; (4) the brief's Done when list, ticked or explained; (5) anything you could not do, and why. Then wait for my go-ahead.

FIRST, BEFORE ANY CODE
1. The reference folder docs/briefs/2026-11-10-careers/ is on branch docs/careers-reference (docs only, no code). Fetch it and merge it into this branch first, so it reaches main with Step 1. Confirm you can open the PNGs and the canvas-source files.
2. List every place that reads NAV, FRAMEWORKS, KNOWLEDGE_TABS, AskCopy, GUIDE, TARGETS, RAIL_STEPS and the AI Team roles directly. Report the list and how you will route each through the open workspace's mode.

BUILD (brief section: Modes)
- Add mode to WorkspaceData in model.rs: PM, Careers, YGO (slot only). Serde default PM. No migration.
- Turn each fixed table into a per-mode table picked by the open workspace's mode. Careers tables can hold placeholders for now, filled in later steps. No view is copied per mode.
- Workspace switcher shows a small mode label beside each name (screen: Switcher).
- Self-test: load every existing test workspace and prove it is PM and unchanged. Add a test that a Careers workspace saves and reloads as Careers.

DONE WHEN
A PM workspace looks and works exactly as on main. A Careers workspace made by a test opens with Careers tables. Plus the brief's Done when list.
```

```
RESOLVE CAREERS, STEP 2 OF 8: THE CAREERS INTAKE
Branch: careers/s2-intake. Start only after Step 1 is approved and merged.

STANDING RULES (every step)
Repo simeonfab/resolve2. Read CLAUDE.md first. Brief: docs/briefs/2026-11-10-careers/build-brief.pdf. The live doc https://claude.ai/artifact/EzKD4RkhK5J6iYogLo1LeF wins if they differ.
SCREENS ARE THE REFERENCE. Before building any view, open its screenshot in docs/briefs/2026-11-10-careers/screens/ (file name = artboard name, listed in each step). For exact copy, spacing and colours, read the matching file in docs/briefs/2026-11-10-careers/canvas-source/. The live canvas is https://claude.ai/artifact/FaYcRtkUkeDWbUqsWMFMDS (read it with your artifact tool if you can; if not, say so and use the folder). Match the screens. Where your build differs from a screen, say where and why in the report. Screen content is sample data: never ship it as real data.
1. Pull before you start. Work on the branch named below. Commit after every working step. Merge to main only when cargo build passes and --selftest ends "N passed, 0 failed". Push at the end of every session.
2. Colours, fonts, sizes, radii and spacing only in src/ui/theme.rs. Component styling only in src/ui/widgets.rs. No literals in views. Navigation only in the tables in src/ui/nav.rs. Careers adds no new tokens: if a screen seems to need one, ask me.
3. Copy in the brief and on the screens is final. Use it exactly. No em dashes anywhere in the UI.
4. Every Careers view in four states (full, empty, loading, error), screenshotted at 1280 x 800 and 1920 x 1080.
5. Every complete, delete, move and AI write goes through history.rs with an Undo toast.
6. PM mode must not change. Every existing workspace loads and behaves exactly as before.
7. Never invent data, AI output or test results. Mark anything unchecked "unverified".
8. Unknowns: check each one listed for this step in the code and report what you found BEFORE building on it. Never work around one silently.
9. If the brief is unclear or conflicts with the code, stop and ask. Do not guess.

END OF STEP: stop and send me (1) what you built, item by item; (2) screenshots of every new or changed view in full, empty, loading, error at 1280 x 800 and 1920 x 1080, each next to the matching reference screen; (3) the --selftest result; (4) the brief's Done when list, ticked or explained; (5) anything you could not do, and why. Then wait for my go-ahead.

SCREENS: Intake01, Intake02a, Intake02b, Intake02c, Intake03, Intake04, Intake04b, Intake05, Intake06, Intake07, Intake08, Intake09, ProposalEditor, Intake10, Intake11, Intake12.

CHECK AND REPORT FIRST
- How adopt works today (adopt_step and adopt_all in resolve/journey.rs). Careers Adopt must create a NEW workspace in Careers mode, not a project in the current one. Report what that takes.
- TARGETS is a 9-entry table. Report whether Careers targets fit its shape or need new fields (Hard gate, Preference, Question labels).
- Attach: which of .pdf, .docx, .md, .txt the app can read today, and whether a LinkedIn PDF export reads cleanly. Ask before adding a crate.
- Intake 12 AI limit copy: confirm the PM screen's wording and use it. The canvas copy is a placeholder.

BUILD (brief section: Careers intake). The Pack A frame stays as built in Phase 3. Only content changes.
- 02a: fourth path card "I'm looking for a job", live, New tag.
- 02b: careers title, the eight template headings, Try an example, attach CVs and LinkedIn. 02c: the careers pull-in prompt.
- 03: Careers TARGETS, cap 8, skip what files or 02b answered, one push on a broad answer, Heard panel labels Hard gate, Preference, Question.
- 04 and 04b: the five cards, including Your CV and LinkedIn with Strengths, Gaps, What to fix and Rebuild my CV.
- 05: the three-months question. 06: Do this first plus the Get your kit ready / Start applying / Prepare for interviews strip.
- 07: gates one at a time with Your gates list. 08: five first-week tasks. 09: the kit, search name, six blocks, Adopt.
- Proposal editor with the Careers left nav. 10: the six bridge lines bound to real progress. 11: reveal with three counts, Open my job search goes to Home.
- Adopt creates the Careers workspace (default name "Job search") with gates, templates, evidence bank, LinkedIn fixes, weekly target and first tasks.
- Inside a Careers workspace, + New project opens this intake for a second search.
- One model call = one job. Tests use the labelled stand-in server.

DONE WHEN
A tester can run the Careers intake end to end on the stand-in server and land on Home in a new Careers workspace. Every screen above matches its reference. Plus the brief's Done when list.
```

```
RESOLVE CAREERS, STEP 3 OF 8: SHELL AND DAILY SCREENS
Branch: careers/s3-daily. Start only after Step 2 is approved and merged.

STANDING RULES (every step)
Repo simeonfab/resolve2. Read CLAUDE.md first. Brief: docs/briefs/2026-11-10-careers/build-brief.pdf. The live doc https://claude.ai/artifact/EzKD4RkhK5J6iYogLo1LeF wins if they differ.
SCREENS ARE THE REFERENCE. Before building any view, open its screenshot in docs/briefs/2026-11-10-careers/screens/ (file name = artboard name, listed in each step). For exact copy, spacing and colours, read the matching file in docs/briefs/2026-11-10-careers/canvas-source/. The live canvas is https://claude.ai/artifact/FaYcRtkUkeDWbUqsWMFMDS (read it with your artifact tool if you can; if not, say so and use the folder). Match the screens. Where your build differs from a screen, say where and why in the report. Screen content is sample data: never ship it as real data.
1. Pull before you start. Work on the branch named below. Commit after every working step. Merge to main only when cargo build passes and --selftest ends "N passed, 0 failed". Push at the end of every session.
2. Colours, fonts, sizes, radii and spacing only in src/ui/theme.rs. Component styling only in src/ui/widgets.rs. No literals in views. Navigation only in the tables in src/ui/nav.rs. Careers adds no new tokens: if a screen seems to need one, ask me.
3. Copy in the brief and on the screens is final. Use it exactly. No em dashes anywhere in the UI.
4. Every Careers view in four states (full, empty, loading, error), screenshotted at 1280 x 800 and 1920 x 1080.
5. Every complete, delete, move and AI write goes through history.rs with an Undo toast.
6. PM mode must not change. Every existing workspace loads and behaves exactly as before.
7. Never invent data, AI output or test results. Mark anything unchecked "unverified".
8. Unknowns: check each one listed for this step in the code and report what you found BEFORE building on it. Never work around one silently.
9. If the brief is unclear or conflicts with the code, stop and ask. Do not guess.

END OF STEP: stop and send me (1) what you built, item by item; (2) screenshots of every new or changed view in full, empty, loading, error at 1280 x 800 and 1920 x 1080, each next to the matching reference screen; (3) the --selftest result; (4) the brief's Done when list, ticked or explained; (5) anything you could not do, and why. Then wait for my go-ahead.

SCREENS: Main, Switcher, Projects, Tasks, Calendar, StateHomeEmpty, StateHomeLoading, StateHomeError.

CHECK AND REPORT FIRST
- Can a new workspace be created with starter calendars (Calendar in model.rs)? Report what adding the four Careers calendars takes.
- Does a task link to a record today? Task templates must link to a Pipeline record (built in Step 4). Report how you will leave the link ready.

BUILD (brief sections: Shell, Home, Projects, Tasks and Calendar)
- Sidebar YOUR SEARCH: Pipeline, My CV, Stats, Tasks, Interview recorder, Frameworks, Knowledge base, with the search picker. Items for later steps open an "unverified, coming in step N" empty state, not a fake screen.
- Ask Resolve placeholder and suggestions per screen, from the brief's table.
- Home: counts line, Add a job box with verdict in place, hero, Today with calendar colours, THIS WEEK, WAITING TO HEAR BACK, PIPELINE, NEW FROM SCOUT, NEEDS A CHECK. Cards that need later steps show their empty state until then.
- Projects: search cards and "+ New search".
- Tasks: Now, Waiting to hear back, Upcoming. "From a template" in + Add task with the seven templates.
- Calendar: four starter calendars with colours and toggles. "Added from Gmail" tag ready in the event editor.

DONE WHEN
Home, Projects, Tasks and Calendar match their screens in a Careers workspace and are unchanged in PM. Plus the brief's Done when list.
```

```
RESOLVE CAREERS, STEP 4 OF 8: PIPELINE, RECORD AND TRACKER
Branch: careers/s4-pipeline. Start only after Step 3 is approved and merged.

STANDING RULES (every step)
Repo simeonfab/resolve2. Read CLAUDE.md first. Brief: docs/briefs/2026-11-10-careers/build-brief.pdf. The live doc https://claude.ai/artifact/EzKD4RkhK5J6iYogLo1LeF wins if they differ.
SCREENS ARE THE REFERENCE. Before building any view, open its screenshot in docs/briefs/2026-11-10-careers/screens/ (file name = artboard name, listed in each step). For exact copy, spacing and colours, read the matching file in docs/briefs/2026-11-10-careers/canvas-source/. The live canvas is https://claude.ai/artifact/FaYcRtkUkeDWbUqsWMFMDS (read it with your artifact tool if you can; if not, say so and use the folder). Match the screens. Where your build differs from a screen, say where and why in the report. Screen content is sample data: never ship it as real data.
1. Pull before you start. Work on the branch named below. Commit after every working step. Merge to main only when cargo build passes and --selftest ends "N passed, 0 failed". Push at the end of every session.
2. Colours, fonts, sizes, radii and spacing only in src/ui/theme.rs. Component styling only in src/ui/widgets.rs. No literals in views. Navigation only in the tables in src/ui/nav.rs. Careers adds no new tokens: if a screen seems to need one, ask me.
3. Copy in the brief and on the screens is final. Use it exactly. No em dashes anywhere in the UI.
4. Every Careers view in four states (full, empty, loading, error), screenshotted at 1280 x 800 and 1920 x 1080.
5. Every complete, delete, move and AI write goes through history.rs with an Undo toast.
6. PM mode must not change. Every existing workspace loads and behaves exactly as before.
7. Never invent data, AI output or test results. Mark anything unchecked "unverified".
8. Unknowns: check each one listed for this step in the code and report what you found BEFORE building on it. Never work around one silently.
9. If the brief is unclear or conflicts with the code, stop and ask. Do not guess.

END OF STEP: stop and send me (1) what you built, item by item; (2) screenshots of every new or changed view in full, empty, loading, error at 1280 x 800 and 1920 x 1080, each next to the matching reference screen; (3) the --selftest result; (4) the brief's Done when list, ticked or explained; (5) anything you could not do, and why. Then wait for my go-ahead.

SCREENS: Pipeline, PipelineList, PipelineClosed, Record, StatePipelineEmpty.

CHECK AND REPORT FIRST (do not build until I answer)
- Pipeline storage: the existing CRM database (records linked to pages and tasks) or a new model type. Recommend one with reasons. Rule 4 in CLAUDE.md: one data system.
- Gmail for the tracker: how a desktop app can read it, with setup steps for me, privacy and cost. Propose one approach and wait.
- Fetching job links: test a few common job sites and report what blocks. Fallback is paste text or a screenshot.
- Screenshot of an ad: what reading text from an image needs with the current providers.

BUILD (brief section: Pipeline)
- Board on the built Tasks board: seven columns, Open and Closed tabs, drag with Undo, cards as drawn. List view. Closed tab with filters; a rejection keeps its stage.
- + Add a job (shared with Home): Reviewer runs the gates and saves to Sourced or Triaged with the verdict.
- Record page with right rail, every section in the brief's order. Saved ad kept offline.
- Tracker, only after I approve the Gmail approach: logs on apply, reads replies, moves stages, adds interviews to Calendar, every change in the timeline with source and Undo, unsure changes to NEEDS A CHECK.
- Wire Home's PIPELINE, WAITING TO HEAR BACK and NEEDS A CHECK cards and the task template links.

DONE WHEN
A job added on Home appears on the board with its verdict, opens as a record, moves with Undo, and shows on Home. Plus the brief's Done when list.
```

```
RESOLVE CAREERS, STEP 5 OF 8: MY CV
Branch: careers/s5-mycv. Start only after Step 4 is approved and merged.

STANDING RULES (every step)
Repo simeonfab/resolve2. Read CLAUDE.md first. Brief: docs/briefs/2026-11-10-careers/build-brief.pdf. The live doc https://claude.ai/artifact/EzKD4RkhK5J6iYogLo1LeF wins if they differ.
SCREENS ARE THE REFERENCE. Before building any view, open its screenshot in docs/briefs/2026-11-10-careers/screens/ (file name = artboard name, listed in each step). For exact copy, spacing and colours, read the matching file in docs/briefs/2026-11-10-careers/canvas-source/. The live canvas is https://claude.ai/artifact/FaYcRtkUkeDWbUqsWMFMDS (read it with your artifact tool if you can; if not, say so and use the folder). Match the screens. Where your build differs from a screen, say where and why in the report. Screen content is sample data: never ship it as real data.
1. Pull before you start. Work on the branch named below. Commit after every working step. Merge to main only when cargo build passes and --selftest ends "N passed, 0 failed". Push at the end of every session.
2. Colours, fonts, sizes, radii and spacing only in src/ui/theme.rs. Component styling only in src/ui/widgets.rs. No literals in views. Navigation only in the tables in src/ui/nav.rs. Careers adds no new tokens: if a screen seems to need one, ask me.
3. Copy in the brief and on the screens is final. Use it exactly. No em dashes anywhere in the UI.
4. Every Careers view in four states (full, empty, loading, error), screenshotted at 1280 x 800 and 1920 x 1080.
5. Every complete, delete, move and AI write goes through history.rs with an Undo toast.
6. PM mode must not change. Every existing workspace loads and behaves exactly as before.
7. Never invent data, AI output or test results. Mark anything unchecked "unverified".
8. Unknowns: check each one listed for this step in the code and report what you found BEFORE building on it. Never work around one silently.
9. If the brief is unclear or conflicts with the code, stop and ask. Do not guess.

END OF STEP: stop and send me (1) what you built, item by item; (2) screenshots of every new or changed view in full, empty, loading, error at 1280 x 800 and 1920 x 1080, each next to the matching reference screen; (3) the --selftest result; (4) the brief's Done when list, ticked or explained; (5) anything you could not do, and why. Then wait for my go-ahead.

SCREENS: MyCV, StateCVEmpty.

CHECK AND REPORT FIRST
- How to show a page preview of a .docx or .pdf CV inside egui. Options and cost. Ask before adding a crate.
- Whether the app can write a .docx today for Tailor's output. If not, what it takes.

BUILD (brief section: My CV)
- My CV view with tabs Overview, Templates, Sent versions, LinkedIn.
- Overview: master CV preview with numbered fix markers, CV check dark card, What to fix list with actions, Templates card, LinkedIn card, Sent versions linked to Pipeline records.
- CV check re-runs when the master CV or the target changes. Fix now opens Tailor with that fix.
- Templates are the same data as Frameworks' CV templates and the Knowledge base CVs tab. One store, three views.
- File names follow Name_Role_Company_YYYY-MM.

DONE WHEN
Uploading a CV fills My CV, the check runs, and a Tailor version appears under Sent versions linked to its record. Plus the brief's Done when list.
```

```
RESOLVE CAREERS, STEP 6 OF 8: STATS
Branch: careers/s6-stats. Start only after Step 5 is approved and merged.

STANDING RULES (every step)
Repo simeonfab/resolve2. Read CLAUDE.md first. Brief: docs/briefs/2026-11-10-careers/build-brief.pdf. The live doc https://claude.ai/artifact/EzKD4RkhK5J6iYogLo1LeF wins if they differ.
SCREENS ARE THE REFERENCE. Before building any view, open its screenshot in docs/briefs/2026-11-10-careers/screens/ (file name = artboard name, listed in each step). For exact copy, spacing and colours, read the matching file in docs/briefs/2026-11-10-careers/canvas-source/. The live canvas is https://claude.ai/artifact/FaYcRtkUkeDWbUqsWMFMDS (read it with your artifact tool if you can; if not, say so and use the folder). Match the screens. Where your build differs from a screen, say where and why in the report. Screen content is sample data: never ship it as real data.
1. Pull before you start. Work on the branch named below. Commit after every working step. Merge to main only when cargo build passes and --selftest ends "N passed, 0 failed". Push at the end of every session.
2. Colours, fonts, sizes, radii and spacing only in src/ui/theme.rs. Component styling only in src/ui/widgets.rs. No literals in views. Navigation only in the tables in src/ui/nav.rs. Careers adds no new tokens: if a screen seems to need one, ask me.
3. Copy in the brief and on the screens is final. Use it exactly. No em dashes anywhere in the UI.
4. Every Careers view in four states (full, empty, loading, error), screenshotted at 1280 x 800 and 1920 x 1080.
5. Every complete, delete, move and AI write goes through history.rs with an Undo toast.
6. PM mode must not change. Every existing workspace loads and behaves exactly as before.
7. Never invent data, AI output or test results. Mark anything unchecked "unverified".
8. Unknowns: check each one listed for this step in the code and report what you found BEFORE building on it. Never work around one silently.
9. If the brief is unclear or conflicts with the code, stop and ask. Do not guess.

END OF STEP: stop and send me (1) what you built, item by item; (2) screenshots of every new or changed view in full, empty, loading, error at 1280 x 800 and 1920 x 1080, each next to the matching reference screen; (3) the --selftest result; (4) the brief's Done when list, ticked or explained; (5) anything you could not do, and why. Then wait for my go-ahead.

SCREENS: Stats, StateStatsEmpty, StateStatsError.

BUILD (brief section: Stats)
- Pack C Overview layout with the Stats content: weakest stage dark card with "Add the fix to Tasks" assigned to the owning role, this-week progress, Funnel, Rejections by stage, Response rate, Results by source, Results by verdict and CV template, Interview scores over time (empty until Step 8).
- Every number computed from Pipeline data. Fewer than 5 roles in a stage shows "Too few roles to judge yet". Keep the threshold in one constant.
- Only the weakest-stage reasoning uses AI. If it fails, that card shows its error and the rest load.

DONE WHEN
Stats numbers match a hand count on the test data. Plus the brief's Done when list.
```

```
RESOLVE CAREERS, STEP 7 OF 8: FRAMEWORKS, KNOWLEDGE BASE, AI TEAM, SETTINGS, HELP
Branch: careers/s7-sections. Start only after Step 6 is approved and merged.

STANDING RULES (every step)
Repo simeonfab/resolve2. Read CLAUDE.md first. Brief: docs/briefs/2026-11-10-careers/build-brief.pdf. The live doc https://claude.ai/artifact/EzKD4RkhK5J6iYogLo1LeF wins if they differ.
SCREENS ARE THE REFERENCE. Before building any view, open its screenshot in docs/briefs/2026-11-10-careers/screens/ (file name = artboard name, listed in each step). For exact copy, spacing and colours, read the matching file in docs/briefs/2026-11-10-careers/canvas-source/. The live canvas is https://claude.ai/artifact/FaYcRtkUkeDWbUqsWMFMDS (read it with your artifact tool if you can; if not, say so and use the folder). Match the screens. Where your build differs from a screen, say where and why in the report. Screen content is sample data: never ship it as real data.
1. Pull before you start. Work on the branch named below. Commit after every working step. Merge to main only when cargo build passes and --selftest ends "N passed, 0 failed". Push at the end of every session.
2. Colours, fonts, sizes, radii and spacing only in src/ui/theme.rs. Component styling only in src/ui/widgets.rs. No literals in views. Navigation only in the tables in src/ui/nav.rs. Careers adds no new tokens: if a screen seems to need one, ask me.
3. Copy in the brief and on the screens is final. Use it exactly. No em dashes anywhere in the UI.
4. Every Careers view in four states (full, empty, loading, error), screenshotted at 1280 x 800 and 1920 x 1080.
5. Every complete, delete, move and AI write goes through history.rs with an Undo toast.
6. PM mode must not change. Every existing workspace loads and behaves exactly as before.
7. Never invent data, AI output or test results. Mark anything unchecked "unverified".
8. Unknowns: check each one listed for this step in the code and report what you found BEFORE building on it. Never work around one silently.
9. If the brief is unclear or conflicts with the code, stop and ask. Do not guess.

END OF STEP: stop and send me (1) what you built, item by item; (2) screenshots of every new or changed view in full, empty, loading, error at 1280 x 800 and 1920 x 1080, each next to the matching reference screen; (3) the --selftest result; (4) the brief's Done when list, ticked or explained; (5) anything you could not do, and why. Then wait for my go-ahead.

SCREENS: Frameworks, Knowledge, Team, SettingsAI, SettingsWorkspaces, Help.

CHECK AND REPORT FIRST
- Scout's job source: job board feeds, saved searches or AI search. Propose one with cost and reliability, and wait. Build everything else meanwhile.
- Settings tab names: the screens use AI and MCP and Workspaces. Confirm against what Phase 5 built and ask me if they differ.

BUILD (brief sections: Frameworks and Knowledge base, AI Team and Handoffs, Settings and Help)
- Frameworks: the seven Careers methods in order, gate list and 400 wide editor, More methods later card.
- Knowledge base: the seven Careers tabs, Evidence bank with theme chips, Needs a number tag, grounding bar with real counts, automatic sorting into these tabs.
- AI Team: General assistant, Scout, Reviewer, Tailor, Coach, Negotiator with their actions and the chat header line. Roles propose, people confirm.
- Settings: Gmail card on AI and MCP; mode tags on Workspaces.
- Help: the Careers GUIDE, one section per screen, including My CV.
- Handoffs unchanged.

DONE WHEN
Each view matches its screen in Careers and is unchanged in PM. Plus the brief's Done when list.
```

```
RESOLVE CAREERS, STEP 8 OF 8: INTERVIEW RECORDER
Branch: careers/s8-recorder. Start only after Step 7 is approved and merged.

STANDING RULES (every step)
Repo simeonfab/resolve2. Read CLAUDE.md first. Brief: docs/briefs/2026-11-10-careers/build-brief.pdf. The live doc https://claude.ai/artifact/EzKD4RkhK5J6iYogLo1LeF wins if they differ.
SCREENS ARE THE REFERENCE. Before building any view, open its screenshot in docs/briefs/2026-11-10-careers/screens/ (file name = artboard name, listed in each step). For exact copy, spacing and colours, read the matching file in docs/briefs/2026-11-10-careers/canvas-source/. The live canvas is https://claude.ai/artifact/FaYcRtkUkeDWbUqsWMFMDS (read it with your artifact tool if you can; if not, say so and use the folder). Match the screens. Where your build differs from a screen, say where and why in the report. Screen content is sample data: never ship it as real data.
1. Pull before you start. Work on the branch named below. Commit after every working step. Merge to main only when cargo build passes and --selftest ends "N passed, 0 failed". Push at the end of every session.
2. Colours, fonts, sizes, radii and spacing only in src/ui/theme.rs. Component styling only in src/ui/widgets.rs. No literals in views. Navigation only in the tables in src/ui/nav.rs. Careers adds no new tokens: if a screen seems to need one, ask me.
3. Copy in the brief and on the screens is final. Use it exactly. No em dashes anywhere in the UI.
4. Every Careers view in four states (full, empty, loading, error), screenshotted at 1280 x 800 and 1920 x 1080.
5. Every complete, delete, move and AI write goes through history.rs with an Undo toast.
6. PM mode must not change. Every existing workspace loads and behaves exactly as before.
7. Never invent data, AI output or test results. Mark anything unchecked "unverified".
8. Unknowns: check each one listed for this step in the code and report what you found BEFORE building on it. Never work around one silently.
9. If the brief is unclear or conflicts with the code, stop and ask. Do not guess.

END OF STEP: stop and send me (1) what you built, item by item; (2) screenshots of every new or changed view in full, empty, loading, error at 1280 x 800 and 1920 x 1080, each next to the matching reference screen; (3) the --selftest result; (4) the brief's Done when list, ticked or explained; (5) anything you could not do, and why. Then wait for my go-ahead.

SCREENS: RecorderPrep, RecorderRecording, Recorder, StateRecorderEmpty, StateRecorderLoading, StateRecorderError.

CHECK AND REPORT FIRST (do not build recording or transcription until I answer)
- Recording microphone and system audio on Windows, and transcription with speaker split. Propose one approach with cost, download size and privacy. Reuse the Phase 0 voice capture choice if it fits.
- Measure transcription time per 10 minutes of audio and put the real number in the loading copy.

BUILD (brief section: Interview recorder)
- Left column 280: upcoming from Calendar, past by role and round with score.
- Prep tab: Cockpit sheet from Frameworks, 90-day and six-month plans, Coach drills, Drill me now.
- Recording tab: consent tick first (no tick, no recording), three ways in, record button with timer.
- Analysis tab: score tiles, questions with timestamps and scores, Not prepared for, Strongest moment, full transcript.
- Results: drills to Coach, proof points offered to the evidence bank one by one (the user confirms each), summary saved to the Pipeline record, scores into Stats.

DONE WHEN
An uploaded test recording produces an analysis, and its results land in Coach, the evidence bank (after confirmation), the record and Stats. Plus the brief's Done when list.
```
