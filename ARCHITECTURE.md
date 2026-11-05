# Resolve 2 — how it fits together

One program, three ways to run it:

| Command | What it is |
|---|---|
| `resolve2` | The desktop app (pages, databases, tasks, calendars). |
| `resolve2 --mcp --workspace NAME` | MCP over stdio for Claude Desktop, Claude Code and Codex. |
| `resolve2 --serve PORT --workspace NAME [--token SECRET] [--data-dir PATH]` | Headless **hub**: phone web app, MCP over HTTP (ChatGPT), and `/sync`. |

## Data

* Each workspace is a folder: `data.json` (everything), `settings.json`, `backups/`.
  Default root: `%APPDATA%\Resolve2` (`--data-dir` or `RESOLVE2_DATA` to change it).
* Every item (page, task, event, database, project, calendar) has an id and an `updated` time;
  deletions leave a tombstone. Any two copies merge: newest edit per item wins, deletions are respected.
* Writes are atomic and merge with what's on disk, so the app, MCP and the hub can share a folder.

## Devices and the hub

```
 Android (phone web app) ──┐
 ChatGPT (MCP over HTTPS) ─┼──►  HUB  (`resolve2 --serve`)  ◄── /sync ──  Windows app (full offline copy)
 Claude / Codex (stdio) ───┘      this PC today, a cloud VM later
```

* **Today:** the Windows app itself is the hub (Settings → Sync & phone → serve). Tailscale on the PC
  and phone makes it reachable anywhere, encrypted, without opening ports or paying for hosting.
* **Later:** run `resolve2 --serve 8797 --data-dir /srv/resolve2 --workspace Personal --token SECRET` on any
  always-on server (same code; Tailscale or a reverse proxy such as Caddy for HTTPS). In the Windows app set
  *Hub address* + *Hub token* and press *Sync now*: the full workspace uploads and both stay merged every minute.
  Point the phone and ChatGPT at the server. No rebuild, no migration step, no data loss — every device
  keeps a complete copy, and backups keep running on the PC.
* A shared folder (Syncthing, OneDrive…) is a second, optional way to keep PCs in step.

## The Resolve layer (`src/resolve`)

```
Intake (ui/intake.rs) ──handoff──► Project + sections + tasks + plan page   (existing workspace types)
   │  AI: understand / judge / recommend / plan          + ProjectContext + Records   (WorkspaceData.contexts / .records)
   ▼
Cockpit (ui/cockpit.rs) ── assess.rs (deterministic: win, what matters now)
   ├─ roles.rs: router + Product Director / Growth Lead / Tech Lead (role .md = system prompt)
   ├─ actions → Proposals → user confirms → Records / Tasks
   ├─ Hypothesis Zero tree (records: hypothesis → experiment / evidence)
   └─ Handoff brief (Codex / Claude Code / Cursor)          MCP: resolve_* / *_record tools
```
* All model calls: `ai::complete_json` → provider (Anthropic, OpenAI, Ollama/OpenAI-compatible, or CLI) →
  shape validation → one retry → usage log (`ai-usage.jsonl`) → monthly budget check.
* Calls run on a background thread (`ui/resolve.rs` `spawn_job`); results are applied in `resolve_poll`.
* Nothing in the Resolve layer has its own storage: it is saved, merged, synced, backed up and exposed over
  MCP with the rest of the workspace.

## Isolation

Workspaces never share files, tokens, sync files or MCP access. Each MCP server or hub opens exactly one
workspace; AI access can be turned off or made read-only per workspace.
