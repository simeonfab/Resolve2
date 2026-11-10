# CLAUDE.md — working on Resolve 2 (same content as AGENTS.md; keep both in step)

Resolve 2 is one Rust desktop app (eframe/egui) that combines a local-first workspace (pages, databases,
projects/tasks, calendars, CRM) with the Resolve product layer (Intake → ResolveOS guidance → execution
→ Hypothesis Zero validation). Goal: **build Resolve, not another general-purpose productivity app.**

## Stack
Rust stable (`x86_64-pc-windows-gnu`), eframe/egui 0.29, serde/serde_json, chrono, tiny_http (phone
page, MCP over HTTP, `/sync`), ureq + Windows `curl.exe` for outgoing HTTP. Storage: JSON files. No web stack.

## Folder map
| Path | What |
|---|---|
| `src/main.rs` | Entry point and command-line modes (`--selftest`, `--test-ai`, `--mcp`, `--serve`, `--ask`, `--screenshot`) |
| `src/model.rs` | Shared data model + merge (newest edit per item wins, tombstones) |
| `src/store.rs` | Data folder, workspaces, atomic saves, backups, sync, app settings |
| `src/resolve/` | Resolve logic: `journey.rs` (the intake: questions, heard, proposal, adopt), `testlab.rs` (Test Intake), `intake.rs` (original intake prompts, reused), `context.rs`, `records.rs` (incl. Hypothesis Zero), `assess.rs`, `roles.rs`, `team.rs` (AI Team roles) |
| `src/history.rs` | Undo / soft delete (change sets) for every delete, complete, adopt and AI write |
| `src/ai.rs` | Provider layer (Anthropic, OpenAI, Ollama/OpenAI-compatible, Claude Code/Codex CLI), usage + budget |
| `src/mcp.rs` | MCP server (stdio + HTTP) |
| `src/ui/nav.rs` | **Navigation config**: the sidebar groups and items, the project picker, Undo/Redo, the Frameworks registry, Knowledge tabs and the Ask Resolve wording per section. Edit the tables to change navigation. |
| `src/ui/sections.rs`, `knowledge.rs`, `team.rs` | Sections: Projects, Overview, Roadmap · Frameworks, Knowledge · AI Team, Handoffs (ported from Control Centre, adapted to this data model) |
| `src/ui/` | Views. `mod.rs` (App, View enum, routing), `intake.rs` + `intake_steps.rs` + `intake_review.rs` (full-screen journey), `testlab.rs`, `cockpit.rs`, `resolve.rs` (jobs), `dashboard.rs`, `tasks.rs`, `pages.rs`, `databases.rs`, `calendar.rs`, `settings.rs`, `assistant.rs` |
| `src/ui/theme.rs` | **Design tokens** (colours, fonts, text styles, radii, spacing) — the one tokens file |
| `src/ui/widgets.rs` | Shared component styling (cards, pills, buttons, nav items, chips) |
| `resolveos/` | Original ResolveOS role/governance prompt files, `h0.md` (the H0 method + question-set slot), `intake-scenarios.json` |
| `assets/fonts/` | Inter + DM Serif Display (OFL, licences alongside) |
| `src/selftest.rs`, `src/selftest_resolve.rs` | End-to-end checks |
| `docs/` | STATUS, RECONCILIATION, RESOLVE_SPEC, DATA_SCHEMA, LICENSING, THIRD_PARTY_CRATES |

## Commands
```
cargo build                                    # target\debug\resolve2.exe
target\debug\resolve2.exe --selftest           # must end "N passed, 0 failed"
target\debug\resolve2.exe --data-dir shots\x --test-ai --provider ollama --model llama3.2 --url http://localhost:11434
target\debug\resolve2.exe --data-dir shots\x --view resolve --screenshot shots\x\r.png
target\debug\resolve2.exe --testlab --data-dir shots\lab --lab-run "Service business" --provider ollama --model llama3.2 --url http://localhost:11434
target\debug\resolve2.exe --testlab --data-dir shots\lab --view intake-proposal --screenshot shots\lab\p.png
```
If the build says "Access is denied" on `resolve2.exe`, the app is open: rename the exe (it keeps
running) — never kill the user's window.

## Ownership split (two agents)
| Owner | Owns | Must not |
|---|---|---|
| **Craft Anything builder** | data models, business logic, AI and intake logic, routing, view *structure* (`src/model.rs`, `store.rs`, `resolve/`, `ai.rs`, `mcp.rs`, view files' layout and behaviour) and, **for the 2026 redesign (build brief 7 Nov 2026, phases 0 to 6), also `src/ui/theme.rs` and `src/ui/widgets.rs`** | put colours, fonts, sizes, radii or spacing anywhere but `theme.rs`, or component styling anywhere but `widgets.rs` |
| **Claude (visual layer, from GitHub)** | not active during the redesign; `theme.rs` and `widgets.rs` are the builder's until Simeon says otherwise | change data, logic or routing |

No hard-coded colours, fonts, sizes, spacing or radii in view code: use `theme.rs` tokens. (Legacy
hard-coded values still exist in views — see docs/RECONCILIATION.md §2.6 Part 6; don't add new ones.)

## Rules (from the owner's brief, Part 0)
1. Work only in `build-me-a-single-productivity-copy`. **Never touch `build-me-a-single-productivity`** (the original app, "Control Centre") or `%APPDATA%\Weave`.
2. Isolation stays proven: data `%APPDATA%\Resolve2`, MCP `resolve2_<ws>` / `resolve-2`, port 8797, shortcuts "Resolve 2".
3. Evolve, don't rebuild. Keep every working feature.
4. One data system: tasks, pages, databases, records, contexts, intakes in `WorkspaceData`. Never a second store for the same kind of thing.
5. Prove, don't claim: every "done" needs a screenshot, test output or commit. Mark unchecked things "unverified".
6. Cosmetic bugs are out of scope unless they block use, navigation, understanding or testing.
7. Unclear requirement → ask the owner. Don't guess.
8. **Scripts and tests never open the app on the real data folder** (`%APPDATA%\Resolve2`). Screenshots and tests use a sandbox folder only, and a script stops if it can't confirm that. The app enforces it too: `--screenshot` with no `--data-dir` gets a fresh sandbox folder under `%TEMP%`; pointed at the real folder it refuses and exits 2, and so does `--selftest` outside its own temp folder. `scripts/pm-compare.ps1` is the sandboxed PM comparison.
Also: one model call = one job (generation and judgement never share a call); no fake AI output
(tests use the labelled stand-in server); roles propose, people confirm; data changes are additive
(`#[serde(default)]`); never commit data, keys or tokens.

## Branching
- `main` always builds and passes `--selftest`.
- Work on feature branches: `intake/...`, `nav/...`, `ai/...`, `docs/...`, `style/...`.
- Commit after every working step; merge to `main` only when build + self-test pass; push at the end of every session; pull before each session.
