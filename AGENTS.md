# AGENTS.md — working on Resolve 2 (Claude Code, Codex, other agents)

Resolve 2 is a single Rust desktop app (eframe/egui) that combines a local-first workspace
(pages, databases, projects/tasks, calendars, CRM) with the Resolve product layer
(Intake → ResolveOS guidance → ResolvePM execution → Hypothesis Zero validation).

## Read first (in this order, stop when you have enough)
1. `docs/STATUS.md` — what is implemented, tested, partial, missing; the build sequence.
2. `ARCHITECTURE.md` — modules, data flow, run modes.
3. `docs/DATA_SCHEMA.md` — the JSON data model and migration rule.
4. `docs/RESOLVE_SPEC.md` — the product spec reconstructed from the original Resolve sources, with conflicts and open questions. Do not invent behaviour that isn't there; add an open question instead.

## Build, test, run
```
cargo build                      # target\debug\resolve2.exe
target\debug\resolve2.exe --selftest   # ~90 end-to-end checks; must stay at 0 failed
target\debug\resolve2.exe --data-dir shots\tmp --view resolve --screenshot shots\tmp\r.png
```
Windows GNU toolchain; `.cargo/config.toml` and `linklibs/` make it build without Visual Studio.

## Rules
- **Never touch Resolve Beta.** Its data is `%APPDATA%\Weave` and its code lives elsewhere. Resolve 2 uses `%APPDATA%\Resolve2` (or `--data-dir` / `RESOLVE2_DATA`). Don't add code that reads `Weave`.
- **Reuse before rebuilding.** Projects, tasks, pages, calendar, search, MCP, sync already work. The Resolve layer is data inside `WorkspaceData` (`records`, `intakes`, `contexts`) — don't add a second database.
- **One model call = one job.** Generation and judgement never share a call (original rule N1). Every AI JSON call goes through `ai::complete_json` (validated, one retry, usage logged, budget checked).
- **No fake AI.** Without a provider, flows offer a manual path. Tests use the scripted stand-in server in `selftest_resolve.rs`; never present stand-in output as real.
- **Roles propose, people confirm.** Anything a role wants to write comes back as `roles::Proposal`s.
- **New record kinds/fields:** add them to `resolve/records.rs` (`Kind`, `statuses`, `fields`). Unknown fields are preserved, so old data keeps loading.
- **Data changes:** only additive, `#[serde(default)]` fields. See `docs/DATA_SCHEMA.md`.
- **Secrets:** API keys live only in the user's `workspaces.json` under the data folder. Never commit data folders, `shots/`, keys or tokens (`.gitignore` covers them).
- Keep changes small; add a self-test check for every user-visible behaviour you add; run `--selftest` before committing.
- Don't push, deploy or create remote resources without the owner's approval.

## Map
| Path | What |
|---|---|
| `src/model.rs` | Workspace data model + merge (newest edit per item wins, tombstones) |
| `src/store.rs` | Data folder, workspaces, atomic saves, backups, sync |
| `src/resolve/` | Resolve layer: `intake.rs`, `context.rs`, `records.rs`, `assess.rs`, `roles.rs` |
| `src/ai.rs` | Provider-independent AI (Anthropic, OpenAI, Ollama/OpenAI-compatible, optional CLI), usage + budget |
| `src/mcp.rs` | MCP server (stdio + HTTP) — workspace and Resolve tools |
| `src/ui/` | egui screens; Resolve: `intake.rs`, `cockpit.rs`, `resolve.rs` (jobs/state) |
| `resolveos/` | The original ResolveOS role/governance files used as prompts (owner's material) |
| `src/selftest*.rs` | End-to-end checks |
