# Resolve 2 — product spec reconstructed from the original Resolve

Audited 5 Nov 2026, read-only. Full audit notes (with file/line citations) were produced from:

| Source | Status found | Used for |
|---|---|---|
| Notion "🚀 Resolve" (390d8b53…) — read via the copy imported into the Resolve Beta workspace (the live Notion URL needs a login) | Current per its own root page ("Notion = product truth, GitHub = code", corrected 12 Jul 2026). Several sub-pages are archived by their own banners (Current Context, Active Priorities after 4 Aug). Newest page: Tech Lead Home, 8 Sep. | Layers, roles, runtime actions, roadmap, design rules |
| Notion "Hypothesis Zero (H0) — E0 Experiment Branching" (30 Oct 2026) | The only written definition of H0 found anywhere | Validation model |
| `ResolveOS-intake-frontend` (public/index.html + api/resolve.js, Jul 2026) | Current intake implementation | Intake flow, prompts, schemas, branching |
| `cap-pm-cockpit-alaria` (ResolvePM "Alaria" v0.1.0) | Current Alpha app; mixes two eras | Orchestration rules (win, gates, roles, team router), schema |
| `resolve-docs` (to 3 Aug 2026) | `briefs/ALPHA-BUILD-TICKETS.md` is "the authority"; mockups `*_dc.html` | Journey, brand tokens, conflicts |
| `ResolveOS` (v1.0.0, 22 Jun 2026; ResolveOS-beta is byte-identical) | Framework; most files v0.1 drafts | Role doctrine, governance, readiness, decision log |
| `Resolve-Planning` (24 Jun 2026) | Superseded: Notion says it "does not exist"; its own README later points back to Notion | History only |

**Precedence used:** your instruction (Nov 2026) > Notion current pages > ALPHA-BUILD-TICKETS / shipped code > resolve-docs strategy > ResolveOS drafts > Resolve-Planning.
Nothing marked "not specified" below was filled in by guesswork.

## 1. The journey (as implemented in Resolve 2)

```
Intake ──► connected workspace project ──► ResolveOS cockpit ──► execution in the workspace ──► validation
 name, situation,     project + Now/Next/Later      next move, what matters now,      tasks, calendar,     hypotheses (H0),
 free-text dump,      sections + P1 first move +    roles (PD/GL/TL), actions,        pages, CRM           experiments (E0),
 confirm 4 cards,     plan page (9-section report)  context editing, handoff                               evidence, decisions
 outcome Q + judge,   + context + records
 next move, plan
```
Nothing is a locked sequence: every record, task and context field can be revisited at any time; the
assessment recomputes from the current state; unfinished intakes resume where they stopped.

## 2. Layer mapping

| Original layer | Original definition | In Resolve 2 |
|---|---|---|
| Resolve Intake | Entry + routing; outcome capture | `resolve/intake.rs`, `ui/intake.rs` — ported flow and prompts |
| ResolveOS | Roles, routing, readiness, validation logic | `resolve/roles.rs` (role prompts = the role `.md` files), `resolve/assess.rs`, `ai.rs` |
| ResolveOS Runtime Surface | Where the user keeps talking to ResolveOS after intake (handoff-first) | Project cockpit: Overview actions, Team chat, Handoff tab (`ui/cockpit.rs`) |
| ResolvePM Light / Full | Workspace: roadmap, work, knowledge, briefing | The existing workspace (projects/sections/tasks/pages/calendar/databases) + Resolve records |
| External execution tools | Cursor/Codex/Claude via handoffs | Handoff brief + the existing MCP server (Claude, Codex, ChatGPT) |
| AI Project OS Sprint (service offer) | Manual validation offer | Not software — out of scope |

## 3. Intake (ported)
- Steps: name → "What do you need help with?" (📁 messy project / 💡 new idea / 🧭 stuck) → "Tell Resolve what's going on." (starter chips, import-from-AI-chat prompt) → confirm 4 cards (Project / What you want / Where it is / Biggest uncertainty) + tools + 0–2 gap questions + assumptions → "Here's what to do next." (do this first, why, first milestone, This week / Next / Later, priorities, not yet) → "You just cleared the fog" → project.
- Branching (verbatim rules): outcome question is always gap question 1 and required unless the outcome was stated; an isolated judge (temperature 0) checks beneficiary + observable change; one re-ask with the three original messages; still partial → proceed `outcome_confirmed=false`; judge failure → proceed unverified (never blocks); an answer is never judged twice.
- AI stages: understand (Business Analyst file), validate-outcome (judge), recommend (Strategic Product Director + project-readiness files), plan (sections 5–9). Shapes are validated; one retry; otherwise a clear error.
- Handoff validation = original hard-fail rules (no hollow project).

**Deliberate deviations (reasons):**
1. *Output templates (15) dropped from the recommend prompt* — the template grid was removed from the original UI ("Output grid was removed in the I2 revision") and the generators were unreachable.
2. *Call 3 QA self-check dropped* — documented as condemned/superseded by the referee (audit F27).
3. *New idea / stuck enabled using the messy-project flow* — the originals were "Coming soon" with no questions specified anywhere; they are labelled in the UI as using the messy-project flow.
4. *Manual path* ("Fill it in myself") so the journey works with no AI; outcome is then recorded as not confirmed.
5. *Sign-in/OTP, bridge animation, Microsoft Clarity analytics* — not applicable to a single-user desktop app; not built.
6. *Phases* use Now/Next/Later sections (cockpit convention); dates and priorities derive from position (ResolvePM assembly rule: Now +7 days, Next +21, Later undated; first move P1).

## 4. ResolveOS (implemented subset)
- **Roles (current Notion set):** chat roles Product Director ("Roadmap & sequencing"), Growth Lead ("Customers & positioning"), Tech Lead ("Build & migration") — their ResolvePM `.md` files are the system prompts; identical fenced project context for all. Implementation Engineer is not a chat role (handoff instead). Business Analyst / Strategic Product Director are retired as roles but still drive Intake ("do NOT swap the role files").
- **Stand-up router:** isolated call choosing 1–3 leads; invalid ids dropped.
- **Runtime actions:** What matters now? (deterministic), Challenge assumptions, Create validation plan, Prepare for implementation (AI → proposals the user confirms), Generate Codex/Claude handoff (deterministic).
- **Win / next move:** kept until done, then re-picked by priority → Now/Next/Later → due date.
- **Assessment order:** blockers → missing/unconfirmed outcome (Gate 1 analogue: the one thing worth stopping for) → next move → open decisions → experiments in flight → hypotheses with no test → overdue.

## 5. Hypothesis Zero (from the H0 page)
Implemented as records, not a checklist: hypotheses (Hx) nest to any depth; each holds explicit negation NHx, alternatives, type, confidence, material uncertainty, decision at stake, revisit conditions; evidential state Unresolved / Supported / Weakened / Refuted (unresolved is valid). Experiments E(Hx) attach to **any** node and carry the page's minimum record (decision, prediction differing between Hx and NHx, method/sample, baseline/biases, predefined support/refute/inconclusive thresholds, limits, owner/date/sources, observations, limitations, result, update, next). Thresholds lock once an experiment is Running ("do not retroactively tune"). Evidence records fact/assumption/inference/evidence, direction (incl. contradicting — kept), quality. Decisions keep provenance links and revisit conditions. Deleting a node re-parents its children.

## 6. Feature comparison

| Area | Resolve Beta (foundation) | Original Resolve | Resolve 2 now |
|---|---|---|---|
| Pages, databases, CRM, tasks, calendar, search, sync, backups, phone page, MCP | Working | Partly (KB, work tracker, no calendar) | **Reused unchanged** (renamed, isolated) |
| Intake | — | Messy-project flow live | **Ported**, + manual path |
| Project context carried forward | — | `project_sessions` row | **ProjectContext**, editable, fenced into every role call |
| Roadmap themes pipeline (floor gate, referee, theme cards, definition of done) | — | Alpha pipeline (~8 isolated calls) | **Not built** — next increment (see STATUS) |
| Team roles + router | — | Built (history not persisted) | **Built**, history persisted as Guidance records |
| Global orchestrator bar (ask/find/capture/create) | ✨ Ask AI (tool-calling assistant) | Built | **Reused** ✨ Ask AI (now also sees Resolve tools) |
| Decisions / risks / assumptions / dependencies / requirements / blockers / milestones | Generic databases | Work-item types, KB types, ResolveOS templates | **Records** with per-kind fields and statuses |
| Hypothesis Zero | — | Notion method only | **Built** (tree, E0 experiments, locks, evidence) |
| Readiness | — | Intake plan table; ResolveOS 5-state model | Intake readiness table shown; 5-state model **not built** |
| AI providers / cost | Claude/ChatGPT subscriptions via CLI, API keys, Ollama | OpenAI gpt-4o only; usage ledger never enforced | Provider-independent; **usage log + user-priced cost + monthly budget enforced**; CLI subscriptions marked personal/dev only |
| Accounts / multi-user | Local workspaces | Supabase + OTP | Workspace `owner` field only (by design) |

## 7. Conflicts (resolved how)
- Notion vs Resolve-Planning as source of truth → Notion (root page, July) and your instruction.
- Role set 3 vs 4 vs 6 → chat roles PD/GL/TL; IE as handoff (Notion 3 Aug ruling also mentions IE in Team — **open**).
- Gate 2 halts vs never halts → newer docs: never halts (not built yet).
- Outcome question wording (3 versions) → the intake's "If this works, who is better off, and how would you know?" form, phrased per project.
- Phase labels Now/Define/Launch vs This week/Next/Later → model prompt keeps Now/Define/Launch, UI shows This week/Next/Later (as shipped).
- "Never a dark theme throughout" (brand ref) vs your preference for the current dark design → dark default kept, light (cream) theme available; **open** for you.
- "Never say Intake to users" vs mockups using it → Resolve 2 UI avoids it except the screen names in this doc (**open**: confirm wording).

## 8. Open questions (need your decision; nothing was invented)
1. Questions/branches for the "new idea" and "stuck" routes, and whether Screen 0 (the 8×10×13 router draft of 24 Jun) should be built.
2. Whether to build the roadmap-themes pipeline (floor gate + referee) next, and the lever criterion (explicitly your positioning call).
3. Readiness: intake's 4 areas vs ResolveOS's 5 states (Adoption/Discovery/Planning/Implementation/Validation).
4. Should Implementation Engineer appear in Team chat?
5. Final logo (brand ref: "Logo TBC"; compass motif proposed) and whether to bundle Inter + DM Serif Display.
6. Updates / briefings / attention queue / risk register schemas (not specified anywhere).
7. H0 confidence scale — the page names "confidence" without a scale; Resolve 2 stores your free-text judgement.
