# Resolve 2

AI-guided product and project management, built on a local-first workspace (pages, databases, CRM,
projects and tasks, calendars). Arrive with an idea, a problem or a messy project; Resolve turns it into
a connected project with a clear first move, keeps the context, guides execution with ResolveOS roles,
and validates outcomes with Hypothesis Zero.

Separate from **Resolve Beta** (the personal workspace it was copied from): its own code, binary,
data folder (`%APPDATA%\Resolve2`) and MCP names.

## Setup
1. Install Rust (stable, `x86_64-pc-windows-gnu`): https://rustup.rs
2. `cargo build` (or `cargo build --release`)
3. Run `target\debug\resolve2.exe`, or double-click `Open Resolve 2.cmd`.
4. Optional AI: Settings → AI & MCP. Choose an API key (Anthropic/OpenAI), a local model (Ollama,
   LM Studio…), or — for personal use only — the Claude Code / Codex CLI. Enter your provider's prices
   and an optional monthly budget to track cost. Without AI, every Resolve step can be filled in by hand.

## Use
- **✦ Start with Resolve** (sidebar or Home) → intake → your project opens in its Resolve cockpit.
- Any existing project: open it and press **✦ Open in Resolve**.
- Claude / Codex / ChatGPT can read and update everything over MCP (Settings → AI & MCP).

## Test
- `target\debug\resolve2.exe --selftest` — ~90 end-to-end checks; no network or keys (AI via a labelled stand-in server).
- `target\debug\resolve2.exe --data-dir shots\aitest --test-ai --provider ollama --model llama3.2 --url http://localhost:11434` — one real intake call through a real model (also `--provider claude-code`, `anthropic`, `openai-cloud`, `openai`).
- `--data-dir <folder>` keeps any test away from your real data.

## Develop
- `target\debug\resolve2.exe --selftest` runs ~90 end-to-end checks (no network or keys needed).
- Read `AGENTS.md` before changing code. Docs: `ARCHITECTURE.md`, `docs/`.
- Command-line modes: see `resolve2.exe --help`.

## Docs
| File | |
|---|---|
| `docs/STATUS.md` | What works, what's tested, what's next |
| `docs/RESOLVE_SPEC.md` | The original Resolve, reconstructed; comparison, conflicts, open questions |
| `docs/DATA_SCHEMA.md` | Data model + migration rule |
| `docs/LICENSING.md` | IP and licence review; what to fix before commercial release |
| `docs/THIRD_PARTY_CRATES.md` | Dependency licences (regenerate with `scripts/licenses.ps1`) |
| `TUTORIAL.md`, `FOR_AI_ASSISTANTS.md` | Workspace guide (inherited) |
