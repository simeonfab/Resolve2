# Status (5 Nov 2026, v2.0.0-alpha.1)

Legend: **Tested** = covered by `--selftest` (real code paths; AI via a scripted stand-in server) ·
**Built** = works in the app, checked by screenshot/manual use, no automated test · **Partial** · **Missing**.

## Isolation from Resolve Beta
| Item | State |
|---|---|
| Separate codebase (`build-me-a-single-productivity-copy/app`), binary `resolve2.exe`, package `resolve2` | Done |
| Data `%APPDATA%\Resolve2` (env `RESOLVE2_DATA`), fresh Personal + Sample workspaces, no copied data | Done — verified Beta's `%APPDATA%\Weave` files unchanged (sizes + timestamps) |
| MCP names `resolve2_<workspace>`, server id `resolve-2`, hub port 8797, sync file prefix `resolve2-`, shortcut "Resolve 2" | Done |
| GitHub repo `Resolve-2` (private) | **Not created** — needs your approval to create/push (local git repo is ready) |
| Deployment | None (desktop app) |

## Journey
| Capability | State |
|---|---|
| Start intake from sidebar/Home; name → situation → free text → confirm → next move → project | Built + Tested (stages, rules, handoff) |
| Outcome question first + required; isolated judge; one re-ask; never blocks on judge failure | Tested |
| Manual (no-AI) intake | Tested (handoff) / Built (UI) |
| Interrupted intake resumes at the same step | Tested |
| Project created with Now/Next/Later, P1 first move, roadmap tasks, plan page, context, records | Tested |
| Context carried forward, editable, used by all roles and handoffs | Tested |
| Cockpit: next move (stable win rule), progress, what matters now | Tested (assessment) / Built (UI) |
| Runtime actions: What matters now, Challenge assumptions, Validation plan, Prepare for implementation, Codex/Claude handoff | Tested (proposals, apply, brief) / Built (UI) |
| Team chat with router + 3 leads; conversation saved | Tested (route, reply) / Built (UI) |
| Records (decisions, risks, assumptions, requirements, dependencies, questions, blockers, milestones, updates) | Tested via MCP / Built (UI) |
| Hypothesis Zero tree, E0 experiments from any node, threshold lock, evidence, re-parenting | Tested |
| MCP tools: resolve_projects, resolve_project, list_records, add_record, update_record | Tested |
| Merge/sync of Resolve data between devices; project delete cleans up | Tested |
| AI usage log, user-priced cost, monthly budget stop | Tested |
| Existing projects adopted into Resolve ("✦ Open in Resolve") | Built |
| Real AI providers (Anthropic / OpenAI / Ollama / CLI) for Resolve calls | Built — **not run against a live model in this session** (no key configured); the code path is the one the stand-in tests |
| Roadmap themes pipeline (floor gate, referee, theme cards, definition of done) | Missing |
| Routes for new idea / stuck with their own questions; Screen 0 router | Missing (not specified in sources) |
| ResolveOS 5-state readiness; decision-log ADR fields; updates/briefings | Missing |
| Phone page / hub for Resolve screens | Missing (workspace data syncs; no Resolve UI on phone) |
| Accounts, billing, teams | Not planned yet (by request) |

## Build sequence (next increments, each end-to-end + tests)
1. **Live-model pass**: run one real intake with your chosen provider; tune prompts only where shapes fail.
2. **Roadmap themes** (ResolvePM pipeline): floor gate → themes → referee → definition of done → assembly into sections/tasks; provisional flags.
3. **Readiness + decision log**: ResolveOS readiness states per project; decision records with supersedes/review trigger.
4. **Routes**: once you specify questions for new idea / stuck (or approve Screen 0).
5. **Briefing**: daily "what matters now" across all Resolve projects on Home, from the assessment (no AI needed).
6. Commercial prep (only when wanted): licence/NOTICES, icons/logo, account model on the hub.
