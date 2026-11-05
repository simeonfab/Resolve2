# AGENTS.md — working on Resolve 2 (Codex, Claude Code, other agents)

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
| `src/resolve/` | Resolve logic: `intake.rs`, `context.rs`, `records.rs` (incl. Hypothesis Zero), `assess.rs`, `roles.rs` |
| `src/ai.rs` | Provider layer (Anthropic, OpenAI, Ollama/OpenAI-compatible, Claude Code/Codex CLI), usage + budget |
| `src/mcp.rs` | MCP server (stdio + HTTP) |
| `src/ui/` | Views. `mod.rs` (App, View enum, sidebar, routing), `intake.rs`, `cockpit.rs`, `resolve.rs` (jobs), `dashboard.rs`, `tasks.rs`, `pages.rs`, `databases.rs`, `calendar.rs`, `settings.rs`, `assistant.rs` |
| `src/ui/theme.rs` | **Design tokens** (colours, fonts, text styles, radii, spacing) — the one tokens file |
| `src/ui/widgets.rs` | Shared component styling (cards, pills, buttons, nav items, chips) |
| `resolveos/` | Original ResolveOS role/governance prompt files (owner's material) |
| `src/selftest.rs`, `src/selftest_resolve.rs` | End-to-end checks |
| `docs/` | STATUS, RECONCILIATION, RESOLVE_SPEC, DATA_SCHEMA, LICENSING, THIRD_PARTY_CRATES |

## Commands
```
cargo build                                    # target\debug\resolve2.exe
target\debug\resolve2.exe --selftest           # must end "N passed, 0 failed"
target\debug\resolve2.exe --data-dir shots\x --test-ai --provider ollama --model llama3.2 --url http://localhost:11434
target\debug\resolve2.exe --data-dir shots\x --view resolve --screenshot shots\x\r.png
```
If the build says "Access is denied" on `resolve2.exe`, the app is open: rename the exe (it keeps
running) — never kill the user's window.

## Ownership split (two agents)
| Owner | Owns | Must not |
|---|---|---|
| **Codex / Craft Anything builder** | data models, business logic, AI and intake logic, routing, view *structure* (`src/model.rs`, `store.rs`, `resolve/`, `ai.rs`, `mcp.rs`, view files' layout and behaviour) | change token values or component styling except when unavoidable — then say so in the commit message |
| **Claude (visual layer, from GitHub)** | `src/ui/theme.rs` (tokens) and `src/ui/widgets.rs` (component styles) | change data, logic or routing |

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
Also: one model call = one job (generation and judgement never share a call); no fake AI output
(tests use the labelled stand-in server); roles propose, people confirm; data changes are additive
(`#[serde(default)]`); never commit data, keys or tokens.

## Branching
- `main` always builds and passes `--selftest`.
- Work on feature branches: `intake/...`, `nav/...`, `ai/...`, `docs/...`, `style/...`.
- Commit after every working step; merge to `main` only when build + self-test pass; push at the end of every session; pull before each session.
