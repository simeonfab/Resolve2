# Data schema and migration strategy

Storage is plain JSON you own, one folder per workspace:

```
<root>/workspaces.json                 workspaces [{id, name, color, icon, created, owner}] + app settings (AI provider, keys, prices, budget)
<root>/ai-usage.jsonl                  one line per model call: {ts, provider, model, input, output, purpose}
<root>/workspaces/<id>/data.json       WorkspaceData (below)
<root>/workspaces/<id>/settings.json   per-workspace: sync folder, backups, MCP access, hub, tokens
<root>/workspaces/<id>/backups/*.json  daily snapshots
```
`<root>` = `%APPDATA%\Resolve2` (Windows), `~/.resolve2` elsewhere, or `--data-dir` / `RESOLVE2_DATA`.

## WorkspaceData
Every item has `id` and `updated` (ms). Two copies merge per item: newest `updated` wins; deletions
are tombstones `{id, deleted}` kept 60 days.

| Collection | Type | Key fields |
|---|---|---|
| `pages` | Page | title, icon, parent, blocks[{kind, text, checked, task_id}], favorite, tags, template |
| `projects` | Project | name, color, sections[], page_id, archived, favorite, order |
| `tasks` | Task | title, notes, project_id, section, parent_id, priority 1–4, due, due_time, duration_min, recurrence, reminder_min, labels, page_id, completed(_at) |
| `calendars`, `events` | Calendar, Event | name/color/visible/source_url; start, end, all_day, recurrence, page_id |
| `databases` | Database | columns[{name, kind, options, target}], rows[{cells, page_id}], view, filters |
| `intakes` | resolve::Intake | name, situation, step, input, understanding, tools, gap_answers, judged, reasked, outcome_confirmed, unverified, recommendation, plan, manual, project_id, status, last_error |
| `contexts` | resolve::ProjectContext | project_id, situation, intake_id, input, understanding{project, goal, state, uncertainty, outcome, …}, outcome, outcome_confirmed, gap_answers, tools, recommendation{recommended_action, why, milestone, open_decision, roadmap[3], top_priorities, do_not_yet}, plan{readiness, risks, …}, win_task_id, page_id, notes |
| `records` | resolve::Record | kind, project_id, parent_id, title, body, status, fields{…}, links[ids] |

### Record kinds, statuses and fields (`src/resolve/records.rs` is the source of truth)
| Kind | Statuses | Fields |
|---|---|---|
| hypothesis | Unresolved · Supported · Weakened · Refuted | negation, alternatives, type, confidence, uncertainty, decision, revisit |
| experiment | Planned · Running · Done · Stopped | decision, prediction🔒, test_type, method, baseline, support_if🔒, refute_if🔒, inconclusive_if🔒, limits, owner, date, sources, observations, limitations, result, update, next |
| evidence | Recorded · Superseded | nature, direction, quality, source, date |
| decision | Proposed · Decided · Revisit · Superseded | options, rationale, uncertainty, revisit, date, owner |
| risk | Open · Mitigating · Closed · Occurred | impact, likelihood, mitigation, owner |
| assumption | Untested · Holding · Broken | why, test |
| requirement | Draft · Agreed · Done · Dropped | acceptance, source, priority |
| dependency | Waiting · Met · At risk | on, needed_by |
| question | Open · Answered | answer |
| blocker | Blocking · Resolved | unblock, owner |
| milestone | Planned · Reached · Missed | date, criteria |
| update, guidance | Recorded | date / role, action, provider |
🔒 = locked once the experiment leaves Planned.

## Migration rule
- Additive only: new fields get `#[serde(default)]`; unknown record fields are preserved; old files load unchanged.
- Never rename or repurpose a field; add a new one and read both.
- Destructive changes require a one-off migration in `store.rs` that backs up first (`backup_now`) and bumps `WorkspaceData.version`.

## Towards multi-user (not built)
Each workspace already has an `owner` ("local"). A hosted version would map owner → account, store each
workspace as a row/blob keyed by (owner, workspace id), and keep the same merge rule for sync — no data
reshaping needed. The headless hub (`--serve`) is the starting point.
