# Resolve 2 — briefing for an AI assistant (ChatGPT, Claude…)

Paste this whole file into ChatGPT or Claude and ask for recommendations ("Which local AI model should I use on my PC?", "How should I set up remote access?", "What should I add next?"). It describes exactly what the app is, how it's built and what can be configured.

## What Resolve 2 is

A single Windows desktop app that replaces Notion + Todoist + Google Calendar for one person, local-first:

- **Pages**: unlimited nesting; block editor (paragraph, H1–H3, bulleted/numbered lists, to-dos, quote, code, callout, divider, page link, sub-page, embedded database); inline **bold**, *italic*, ~~strike~~, `code`; `[[Page]]` links with backlinks; favourites; built-in and user-made templates.
- **Databases**: table and kanban board views; property types Text, Number, Select, Checkbox, Date, URL, Email, Phone, Linked page, Relation (to rows of another database, projects, tasks or pages); saved filters and sort; row pages with properties + notes; CRM / project-tracker / content-calendar templates.
- **Tasks**: Inbox, Today, Upcoming, Filters & Labels; projects with sections, list and board layouts; subtasks; priorities P1–P4; dates, times, durations; repeats (daily, weekdays, weekly, every N days/weeks, monthly, yearly; completing rolls to the next date); reminders; labels; natural-language quick capture ("Pay rent every month 9am p1 #Home remind 1h").
- **Calendar**: multiple calendars (name, colour, show/hide); day/week/month; recurring events; drag to move; dated tasks appear automatically; iCal subscriptions (Google/Outlook/Apple secret links, refreshed every 30 min, read-only).
- **Integration**: tasks link to pages and projects; events link to pages (meeting notes); database relations link CRM contacts ↔ companies ↔ opportunities ↔ projects ↔ tasks; one search across everything (Ctrl+K); Home dashboard combining today's tasks, schedule, next 7 days, recent pages and project progress.
- **Import/export**: Notion "Markdown & CSV" export, Todoist backup zip/CSV, Google Calendar .ics/zip; export to Markdown, CSV, .ics and JSON.
- **Data**: plain JSON per workspace in `%APPDATA%\Resolve2\workspaces\<id>\` (`data.json`, `settings.json`, `backups\`); automatic daily backups with restore; every item has an id + last-edited time so copies merge (newest edit per item wins; deletions kept as tombstones).
- **Workspaces**: fully isolated (own folder, backups, sync file, phone token, AI access switch, read-only option).

## AI features

1. **MCP server** (Model Context Protocol), 27 tools: workspace_overview, search, get_agenda, list/read pages, list tasks/projects/events/calendars/databases, query_database, create/update page, create/update/complete task, create project, create/update event, create database, add/update database row, delete item, and bulk import_pages / import_tasks / import_csv / import_ics.
   - stdio: `resolve2.exe --mcp --workspace <name or id>`, for Claude Desktop, Claude Code, Codex and any local MCP client (Open WebUI, LM Studio…).
   - HTTP: `POST http://<host>:<port>/mcp?token=<token>`, for ChatGPT connectors; needs a public HTTPS address (a tunnel such as cloudflared or Tailscale Funnel).
2. **In-app assistant ("✨ Ask AI")**, like Notion AI. It can answer and also act, using the same tools in a function-calling loop (up to 16 steps). Providers:
   - **Claude subscription** (no extra cost, the default when Claude Code is installed): runs the Claude Code CLI (`claude -p`, signed in with the user's Pro/Max account) with this app as its MCP server.
   - **ChatGPT subscription** (no extra cost): runs the Codex CLI (`codex exec`, signed in with ChatGPT) with this app as its MCP server.
   - **Claude API**: Anthropic Messages API, the user's own API key, default model `claude-opus-5-5`.
   - **ChatGPT**: OpenAI Chat Completions, the user's own API key, any model name.
   - **Ollama** (local): OpenAI-compatible endpoint at `http://localhost:11434`; tool calling needs a model that supports tools.
   - **Other local servers**: LM Studio, llama.cpp server, Jan (OpenAI-compatible, e.g. `http://localhost:1234`).
   - If a local model can't call tools, it falls back to plain answers.

## How it's built (for technical advice)

- Rust (stable, GNU toolchain on Windows), GUI with eframe/egui 0.29 (OpenGL). One binary: `resolve2.exe`.
- HTTP server: tiny_http (phone web app, MCP over HTTP, `/sync` hub endpoint). Outgoing HTTPS goes through Windows' built-in `curl.exe`; there is no TLS library in the app.
- Build: `cargo build` in the `app` folder; the program is `app\target\debug\resolve2.exe` (or `target\release` after `cargo build --release`).
- Command-line modes: `--mcp`, `--serve PORT --workspace NAME --token SECRET --data-dir PATH` (headless hub; runs on any always-on machine, including a Linux cloud VM), `--selftest` (48 end-to-end checks), `--screenshot file.png`, `--view <name>`, `--list-workspaces`.

## Devices

- Phone: a mobile web app served by the PC (Today, Upcoming, Inbox, agenda, search, add/complete tasks); it can be added to the Android home screen. Remote access: Tailscale on PC and phone. There is no native Android app yet, and the phone needs the hub to be reachable.
- Hub design: today the PC is the hub; later `resolve2 --serve` on a cloud VM, with the PC syncing to it every minute (full offline copy on the PC), with no rebuild or migration.
- Optional sync folder (Syncthing, OneDrive, Google Drive) for keeping other PCs in step.

## Questions worth asking an assistant

- "My PC has <CPU>, <RAM> GB RAM and <GPU / VRAM>. Which Ollama model should I use for Resolve 2's assistant? It needs reliable tool calling and decent writing. Small, medium and best options?"
- "Should I use Claude or ChatGPT's API for the in-app assistant, roughly what would it cost me a month for light daily use, and how do I keep the cost down?"
- "Walk me through Tailscale on Windows and Android for Resolve 2's phone page."
- "Which cheap always-on server would suit `resolve2 --serve` later, and how do I add HTTPS?"
- "Given these features, what workflow would you suggest for my CRM / business plans / product docs?"

## Hardware hints for local models (for the assistant to refine)

- Under 8 GB RAM, no GPU: a ~1–3B model (e.g. `llama3.2`) is fine for summaries; tool use will be weak.
- 16 GB RAM or a 6–8 GB GPU: 7–8B tool-capable models (e.g. `qwen3`, `llama3.1`) are a good balance.
- 32 GB+ RAM or 12 GB+ GPU: 14B+ models give more reliable multi-step actions.
