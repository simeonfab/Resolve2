# Resolve 2 — reconciliation (Parts 0–2 of the 5 Nov evolution brief)

Status: **report only. Parts 3–4 wait for Simeon's approval.**
Evidence lives in this repo (`--selftest` output, commits) and in the test transcripts quoted below.
"Unverified" means I did not run it.

## Part 0 — isolation from the original (Control Centre, `build-me-a-single-productivity`)

Checked read-only on 5 Nov 2026, ~21:45. Nothing in the original folder or its data was modified
(`%APPDATA%\Weave`: 11 files, identical size + timestamps before/after the earlier session).

| Resource | Original (Control Centre) | Resolve 2 | Shared? |
|---|---|---|---|
| Code folder | `build-me-a-single-productivity\app` | `build-me-a-single-productivity-copy\app` | No |
| Data folder | `%APPDATA%\Weave` (code: `store.rs:30`, `WEAVE_DATA`) | `%APPDATA%\Resolve2` (`store.rs:30`, `RESOLVE2_DATA`) | No |
| Tutorial's data folder | Original's TUTORIAL.md lines 12 and 209 say **`%APPDATA%\Weave`** | Copy's TUTORIAL.md says `%APPDATA%\Resolve2` | No — the brief's worry is unfounded: `Resolve2` appears only in the copy's own docs |
| Backups | `%APPDATA%\Weave\workspaces\<id>\backups` | `%APPDATA%\Resolve2\workspaces\<id>\backups` | No |
| Workspace ids | Personal `8dd271ce…`, Sample `308482e2…` | Personal `c7d962ed…`, Sample `4903584e…`, Response `14c27ebc…` | No (same *display names* "Personal" / "Sample (demo data)", separate registries and folders) |
| MCP server name | `control_centre_<ws>`; serverInfo `control-centre` | `resolve2_<ws>`; serverInfo `resolve-2` | No |
| MCP registrations on this PC | `control_centre_personal` → original `weave.exe` in Claude Desktop (both config locations) and Claude Code | **None** for Resolve 2 | No |
| App identity / window | `run_native("Control Centre")`, title "Control Centre", binary `weave.exe` | "Resolve 2", `resolve2.exe` | No |
| Phone page / hub port | 8787 | 8797 | No |
| Shortcuts | Desktop + Start menu "Control Centre.lnk" → original `weave.exe` | "Resolve 2.lnk" → copy `resolve2.exe` | No |
| Sync-folder file prefix | `weave-<id>.json` | `resolve2-<id>.json` | No |

One problem found and fixed: the desktop "Resolve 2" shortcut had been retargeted by Windows to a
renamed old build (`resolve2.running-214349.exe`) after I moved a locked file during a build. It now
points at `target\debug\resolve2.exe` again (both shortcuts re-created with `--install-shortcuts`).

No data migration was needed: Resolve 2 already had its own folder, so nothing was moved.

## Part 1 — GitHub

| Step | Status | Evidence |
|---|---|---|
| Git initialised in `app\` | Done | `git log`: dcc63e6, 3c962dc, 961cee3 |
| .gitignore (target, node_modules, dist, data.json, workspaces.json, settings.json, backups, *.jsonl, .env*, keys) | Done | `.gitignore` |
| Secret scan before first push | Done — 0 hits | regex for sk-/sk-ant-/gh*_/AKIA/xox/private keys/JWT/hex tokens over all tracked + untracked files |
| Baseline commit + tag `v0-baseline` | Done locally | tag on 961cee375691c62cf9d4913645bc6fe62994389e |
| **Private repo github.com/simeonfab/resolve2 + push** | **Not done** | This environment's auto mode refuses to create repos or push ("never publishes or pushes anything without the person"), even with the brief's instruction. Run the commands in "Your steps" below, or approve it interactively. |
| README / AGENTS.md / CLAUDE.md with stack, map, commands, ownership split, Part 0 rules | Done (this branch) | files |
| Tokens file | `src/ui/theme.rs` is the single tokens file (Rust, not CSS). **38 hard-coded colours and most font sizes still live in view files** — moving them is Part 6; listed in the plan. |

## Part 2 — inspection

### 2.1 Architecture

| Aspect | Today |
|---|---|
| Frontend | Rust + eframe/egui 0.29 (immediate mode, OpenGL). One binary. |
| Backend | Same binary. Modes: desktop app, `--mcp` (stdio), `--serve` (HTTP hub: phone page, MCP-over-HTTP, `/sync`), `--selftest`, `--test-ai`, `--ask`. |
| Storage | JSON per workspace (`data.json`, `settings.json`, `backups\`) under `%APPDATA%\Resolve2`; atomic writes; item-level merge by `updated` + tombstones. `ai-usage.jsonl` for cost. |
| AI providers | `src/ai.rs`: Anthropic Messages API, OpenAI Chat Completions, Ollama / any OpenAI-compatible server (LM Studio, llama.cpp, Jan), Claude Code CLI and Codex CLI (personal subscriptions). Tool-calling agent loop (`run_agent`), JSON completions (`complete_json`) with one retry, usage log and budget. |
| How views are registered | `enum View` in `src/ui/mod.rs`; `App::central()` matches on it; each view is an `impl App` method in its own file. |
| How navigation is defined | Hard-coded in `App::sidebar()` (`src/ui/mod.rs`): nav items, Resolve, My Projects, Favorites, Pages tree, Databases, Settings, Help. Project "tabs" are pills in `src/ui/cockpit.rs`. **Not config-driven yet.** |

### 2.2 Feature inventory (tutorial sections)

Status key: **V** verified working (self-test or a real run in this session) · **P** partly working · **C** claimed only (not run) · **B** broken.

| Tutorial feature | Status | Evidence |
|---|---|---|
| Open app, shortcuts, data location | V | `--install-shortcuts` run; shortcut targets read back; Settings "Where your files are" |
| Quick capture (dates, repeats, time, priority, #project, reminders) | V | self-test "quick capture parses…" |
| Recurring tasks, subtasks, task↔page links, move between projects | V | self-test |
| Inbox / Today / Upcoming / Filters & Labels views | V (render) | screenshots; filter logic partly via tests |
| Reminders (desktop notifications) | C | not exercised |
| Projects with sections, list/board | V/C | sections tested; board view only screenshot-level |
| Pages: blocks, nesting, [[links]], backlinks, templates | V | self-test (templates: C) |
| Databases: table/board, relations, filters, CRM template | V | self-test |
| Calendar: multiple calendars, recurring, move, tasks on calendar, iCal subscriptions | V | self-test |
| Search (Ctrl+K) | V | self-test "search finds…" |
| Import Notion / Todoist / Google Calendar; export | V | self-test |
| Backups + restore | V | self-test |
| Phone page, hub sync, tokens | V | self-test (HTTP on localhost; Tailscale/phone hardware: C) |
| Voice typing (Windows+H) | C | not exercised |
| Organise page to-dos with AI | V (stand-in model) | self-test; not run with a real model |
| Resolve: intake, project handoff, cockpit, roles, H0, records, handoff brief | V (stand-in) | 38 self-test checks; real-model run of intake step 1 only (below) |

**AI routes — tested for real on 5 Nov (sandbox data folders under `app\shots\aitest-*`):**

| Route | Status | What happened |
|---|---|---|
| Ollama (llama3.2, installed at `%LOCALAPPDATA%\Programs\Ollama`, server running) — intake "understand" | **V** | 6.9 s, valid JSON. Quality issues: project field is a sentence, outcome question not in "who is better off" shape (= your known failures 1–2). |
| Ollama — ✨ Ask AI agent (tool calling) | **B in practice** | Asked to add a task: the model replied "I created a new task…", but its tool call failed (`✗ update task`) and **no task exists** (checked data.json). The app's action list told the truth; the model's prose did not. |
| Claude subscription via Claude Code CLI (`claude` found) — intake "understand" | **V** | 9.7 s, good quality. Assumptions phrased "I am assuming…" (= known failure 5). Counts against your Claude plan. |
| ChatGPT via Codex CLI | **C** | Codex is not installed on this PC (`codex` not found). |
| Anthropic API | **C** | No API key configured in Resolve 2 or Control Centre. Same code path as the stand-in tests. |
| OpenAI API | **C** | No key configured. |
| LM Studio | **C** | Not installed; nothing on :1234. Uses the same OpenAI-compatible path that Ollama exercises. |
| MCP stdio (as Claude Desktop calls it) | **V** | initialize → serverInfo `resolve-2`; tools/list → 37 tools; tools/call works. Found and fixed: a byte-order mark from PowerShell pipes broke parsing (now stripped). |
| MCP into Claude Desktop itself | **C** | Resolve 2 is **not registered** in Claude Desktop (only `control_centre_personal`, which points at the original). Registering it edits Claude's config — needs your OK. |
| MCP over HTTP (ChatGPT connector) | V (local) | self-test with token; public tunnel: C |

**Safety claims, checked in code:**

| Claim | Verdict | Where |
|---|---|---|
| The assistant lists what it changed after each answer | **True** (API/local: per tool call with ✓/✗; CLI: before/after counts) | `ai.rs` `finish()` and `run_cli` count diff; real Ollama run showed the ✗ line |
| Nothing is deleted without asking | **Partly — prompt only.** `AGENT_SYSTEM` says "Ask before deleting anything", but the `delete_item` tool runs immediately for the in-app agent and for any MCP client. Resolve roles only *propose* (confirm-gated). | `ai.rs` AGENT_SYSTEM; `mcp.rs` `delete_item` |
| AI changes are covered by backups | **Partly.** Automatic backup is daily and only when the app opens or switches workspace (`store.rs auto_backup_if_due`); there is no backup before an AI write, and MCP edits made while the app is closed wait for the next open. Restore + tombstones work (self-test). | `store.rs`, `ui/mod.rs` |

### 2.3 The earlier changes prompt

**Not found.** Searched: the copy (all folders except build output), the original project folder
(read-only, 0 matches for "undo"), the five original repos, the Notion export of the Resolve tree, and
the audits. There is no separate "changes prompt for Resolve 2". Undo appears only as original Resolve
design:
- `resolve-docs/briefs/ALPHA-BUILD-TICKETS.md` T2.16 "Toasts, confirmations & undo": reversible actions show a toast with Undo (4 s depletion), Undo reverses the action and cancels pending follow-through; plain toasts have no Undo.
- Notion Active Priorities: "delete = soft via `cancelled` + undo toast (ruled default)".

| Known item | Classification |
|---|---|
| Undo | **Still required.** Resolve 2 has no undo anywhere (no matches in `src`). Adaptation: desktop toasts with an Undo action for complete / delete / adopt / AI-applied changes; deletes become soft (tombstone kept, restorable) — fits the existing tombstone model. |

Nothing else was reconstructed.

### 2.4 Original Resolve sections vs Resolve 2

| Original Resolve (resolve-docs Alpha, N1 locked order) | Resolve 2 today | After Part 4 (proposed) |
|---|---|---|
| Intake (6 screens) → bridge → sign-in | Intake view, multi-field screens (`ui/intake.rs`) | Typeform-style intake + proposal review + Test Intake lab |
| Dashboard (win for today, waiting on, today, worth a look) | Home (greeting, capture, Resolve panel, stats, today, schedule, pages) | Home: across-projects dashboard + Captured + Today blocks |
| Roadmap tab (theme × Now/Next/Later, provisional) | Project sections Now/Next/Later inside the task board | Project → Roadmap |
| Work tab (Now, waiting on, captured, upcoming) | Inbox, Today, Upcoming, Filters & Labels, project task views | Project → Tasks (+ Captured filter) |
| Team tab (3 leads + stand-up) | Cockpit → Team pills | Project → Team & Handoffs; global AI Team |
| Knowledge tab (library, cut list) | Pages tree + Databases in sidebar; plan page | Knowledge (global + project) |
| Settings (from avatar) | Settings, sync & AI | Settings |
| Global bar (ask/find/capture/create, contextual captions) | ✨ Ask AI side panel + Search + Add task | Top bar: Search, Ask AI (contextual), Add task, New project |
| — (H0 lived in Notion) | Cockpit → Validation | Project → Frameworks (H0 first) |
| Knowledge-base decision/assumption items, work-item risk/decision types | Cockpit → Records | Project → Evidence & Records |
| — | Calendar | Calendar (kept) |
| OS reveal, sign-in/OTP, daily limit | — | Not planned (desktop, single user) |

### 2.5 Reuse map (what powers each Part 4 section)

| New section | Existing code reused | New code needed |
|---|---|---|
| Nav config | `nav_item`, `section_label` (widgets) | `src/ui/nav.rs` (array of {id, label, icon, route, visible}) + sidebar renders from it |
| Top bar | `palette_window` (search), `open_quick` (add task), `ai_panel` (Ask AI), `start_intake` | `App::global_bar()` |
| Home | `dashboard.rs` (today, schedule, Resolve panel), Inbox query | Captured + Today blocks as functions over the same task data |
| Projects list | sidebar project list, `assess::assess` per project | `projects_view` |
| Overview | `cockpit_overview` unchanged | — |
| Roadmap | tasks with section Now/Next/Later, milestone/dependency/hypothesis records, `assess` | `roadmap_view` (columns; derived dates only) |
| Tasks | `tasks_view` for `View::Project` incl. board, filters, quick add | tab wrapper; Captured filter |
| Frameworks | `cockpit_validation`, `record_editor`, `hypothesis_tree`, `roles::propose("validation-plan")` | `src/resolve/frameworks.rs` registry (H0 entry) |
| Evidence & Records | `cockpit_records`, `record_editor`, records store | decision ↔ evidence link picker; history (record revisions) |
| Knowledge | Pages (`page_view`, page tree), databases (Advanced) | project-scoped page list (pages under the project page) |
| Team & Handoffs / AI Team / Ask AI | `cockpit_team`, `roles::route/reply`, `cockpit_handoff`, `ai::run_agent`, Guidance records | one conversation store (Guidance records with optional project), specialists list, general-assistant mode = `run_agent` |
| Calendar, Settings | unchanged | Ollama detection + per-workflow provider (Part 5) |

### 2.6 Plan for Parts 3 and 4 (after approval)

**Part 3 — intake (branch `intake/typeform`)**
1. `src/resolve/intake.rs` → session model v2: ordered answers `{question, target, why_asked, answer, depends_on, needs_review}`, coverage targets (goal, problem, who, why, outcome+signal, evidence, assumptions, constraints, uncertainties), "heard" items `{text, kind: evidence|assumption|question, from_answer}`. Old sessions still load.
2. `src/resolve/intake_ai.rs` (new): next-question call (H0-guided, returns question + target + why + `enough`), classification call (separate, rule N1), proposal generation per section, regeneration per section. Offline fixed question set covering the same targets.
3. `src/resolve/proposal.rs` (new): proposal model (name, summary, objective, outcome, H0 + Hx/NHx, uncertainties, E0…, Now/Next/Later, tasks, knowledge pages), every item tagged evidence/assumption/uncertain; `adopt()` reuses `intake::handoff` logic to write into the existing stores, plus transcript + decision-trail page.
4. Fixes for the five known failures: short confirmed title; outcome in the user's own words, confirmed; short task titles with detail in notes (validator); no generic roadmap words (validator + handoff brief uses real items); assumptions stored as the claim itself (normaliser strips "The user is making the assumption that…", "I am assuming…").
5. `src/ui/intake.rs` → one question per screen, progress, Back/Next, Enter / Shift+Enter / Esc, edit earlier answers with review flags, "What Resolve has heard" side panel, "Show me the proposal".
6. Test Intake lab: `src/resolve/testlab.rs` + `src/ui/testlab.rs`: sandbox workspace (hidden, never MCP-exposed), banner, 5 scenarios (`resolveos/intake-scenarios.json`) + own answers, restart/jump/branch, side-by-side compare, inspector (question, H0 reasoning, classification, provider/model, raw request/response — needs a capture hook in `ai::complete_json_once`), per-run provider, save/replay, export Markdown + JSON, "Promote to real project".
7. Tests: self-test checks for each rule above using the stand-in server; real runs on Ollama + Claude Code recorded in the run exports.

**Part 4 — navigation (branch `nav/sections`)**
1. `src/ui/nav.rs` config; `src/ui/mod.rs` sidebar reads it; `View` gains `Projects`, `ProjectTab(id, Tab)`, `AiTeam`, `Knowledge`; old views (`Inbox`, `Today`, `Upcoming`, `Filters`) become blocks/filters, not sidebar items.
2. Top bar on every screen (`App::global_bar`), Ask AI leaves the sidebar, contextual captions per section (from 14-N8).
3. Project tabs in `src/ui/cockpit.rs` (renamed `project.rs`): Overview, Roadmap (new), Tasks (existing), Frameworks, Evidence & Records, Knowledge, Team & Handoffs.
4. `src/resolve/frameworks.rs` registry with H0.
5. One chat engine: `src/resolve/chat.rs` over Guidance records; Ask AI = general-assistant mode of it.
6. Part 5 items: Ollama detection/recommendation/pull in `ui/settings.rs`; per-workflow provider map in `store.rs` `AppSettings`.
7. Part 6: move remaining hard-coded colours/sizes into `theme.rs` tokens; Resolve light theme default, current dark as dark theme; Georgia stays as the runtime serif until DM Serif Display/Inter are bundled (OFL — needs your OK).

## Questions that block Part 3 (please answer)
1. **"The five-question method as already implemented"** — I can't find it. Resolve 2 has no five-question method, and neither the H0 page nor any repo defines one (the only "five questions" is ResolveOS's file-writing standard). Where is it written down?
2. Undo scope: undo for complete/delete/adopt/AI changes as above — OK?
3. Deletion safety: should AI deletes (agent + MCP) require confirmation in the app, or become soft deletes with Undo?
4. Sandbox for Test Intake: a hidden workspace inside `%APPDATA%\Resolve2` (simplest), or a separate data folder?
5. Bundle Inter + DM Serif Display (OFL) in the binary, or keep system fonts until the visual pass?

## Your steps
1. **Push to GitHub** (from `build-me-a-single-productivity-copy\app` in a terminal):
   ```
   gh repo create simeonfab/resolve2 --private --source . --remote origin
   git push -u origin main --tags
   ```
2. Check isolation yourself: Settings → General → "Where your files are" shows `%APPDATA%\Resolve2`; open the original Control Centre and confirm its data is unchanged.
3. Try a real model: `target\debug\resolve2.exe --data-dir %TEMP%\r2test --test-ai --provider ollama --model llama3.2 --url http://localhost:11434` (or `--provider claude-code`).
4. Run the tests: `target\debug\resolve2.exe --selftest` → expect "92 passed, 0 failed".
