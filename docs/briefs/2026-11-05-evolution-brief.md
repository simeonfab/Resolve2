# Resolve 2: evolution brief for Craft Anything

5 November 2026. From Simeon.

Read this whole brief before you change anything. Do Parts 0 to 2 first, then stop and wait for my approval before Part 3.

Resolve 2 has strong features, especially planning, Hypothesis Zero, records and handoffs. The problem: it looks and behaves like a general productivity app (task manager, notes, databases, calendar) stitched together. It is not Resolve yet. This brief turns it into Resolve without throwing away what works.

Reference order when sources disagree:
1. The original Resolve product vision and sections (this brief, plus resolve-docs).
2. Linear (linear.app) for app structure, navigation and product-management workflows. Learn from it. Do not clone its sidebar.
3. Typeform (typeform.com) for the intake experience.
4. The current Resolve 2 code as the technical foundation.

## Part 0. Ground rules

1. Work only in C:\Users\simeo\Documents\Craft Anything\build-me-a-single-productivity-copy. Never touch build-me-a-single-productivity. That is the original app, and I use it every day as Control Centre.
2. Prove isolation before anything else. Check whether Resolve 2 shares any of these with the original: data folder, workspace names, MCP server name, desktop and Start-menu shortcuts, app identifier, phone-page port, backup folder. The original's tutorial also names %APPDATA%\Resolve2 as its data folder, so check this one first. If anything is shared, stop and tell me. Then give Resolve 2 its own data folder, MCP server name and shortcuts, and move existing Resolve 2 data across with a backup first.
3. Evolve, do not rebuild. Keep every working feature. Never replace working code only to look more like Linear.
4. One data system. Reuse the existing tasks, pages, databases, records and validation stores. Never create a second store for the same kind of thing.
5. Prove, do not claim. Anything you report as done needs evidence: the screen, the test output or the commit. Mark anything unchecked as unverified.
6. Cosmetic bugs are out of scope unless they block use, navigation, understanding or testing.
7. When a requirement is unclear, ask me. Do not guess.

## Part 1. Put Resolve 2 in GitHub (first task)

Resolve 2 is not in GitHub. I checked my account on 5 November and no repo exists for it. A second agent (Claude) will work on the visual layer from GitHub while you work on structure and logic, so this comes first.

1. Create a private repo at github.com/simeonfab/resolve2 and initialise git in the copy's app folder.
2. Write a .gitignore covering target/, node_modules/, build output, data.json, backups, settings files with tokens, .env files and API keys. Never commit my workspace data or any secret. Scan for secrets before the first push.
3. Commit the current state as the baseline, tag it v0-baseline, and push.
4. Add README.md with how to build, run and test. Keep TUTORIAL.md and FOR_AI_ASSISTANTS.md in the repo and keep them current as features change.
5. Add CLAUDE.md and AGENTS.md at the root with: the stack, a folder map, build and test commands, the ownership split below, and the rules in Part 0.
6. Branching: main always builds. Work on feature branches (intake/..., nav/..., ai/...). Commit after every working step. Merge to main only when the build and tests pass. Push at the end of every session.
7. Ownership split, so two agents never collide:
   - You own data models, business logic, AI and intake logic, routing and view structure.
   - Claude owns design tokens, theme files and component styling.
   - Put every colour, font, size, spacing value and radius in one tokens file (for example src/styles/tokens.css) and keep component styles in one styles area. No hard-coded colours or sizes inside view code.
   - Pull before each session. If you must edit a styling file, say so in the commit message.
8. Report the repo URL and the baseline commit hash.

## Part 2. Inspect and reconcile (report, then stop)

Read app\docs, TUTORIAL.md, FOR_AI_ASSISTANTS.md, app\src, and the original Resolve repos in build-me-a-single-productivity-copy\sources. The canonical original Resolve design is in github.com/simeonfab/resolve-docs: RESOLVE-ALPHA-MASTER-PLAN.md, briefs/00 to 14, and the .dc.html files in mockups/. Use the mockups for intent, sections and look. They are mobile web mockups, so they are not a pixel spec for this desktop app.

Write docs/RECONCILIATION.md containing:

1. Architecture summary: frontend framework, backend, storage format, AI providers, how views are registered, how navigation is defined.
2. Feature inventory. For every feature in the tutorial, give a status: verified working, partly working, claimed only, or broken. Test each AI route for real: Claude subscription through Claude Code, ChatGPT through Codex, Anthropic API, OpenAI API, Ollama, LM Studio, and MCP to Claude Desktop. Check in the code three safety claims: the assistant lists what it changed after each answer, nothing is deleted without asking, and AI changes are covered by backups.
3. The earlier changes prompt. An earlier, larger changes prompt for Resolve exists and was never applied to this build. The one item I remember is Undo. Search sources and docs for it. If you find it, classify every requirement as already implemented, still required, requires adaptation, or no longer appropriate. If you cannot find it, say so, list Undo as the only known item, and do not reconstruct the rest.
4. A table comparing original Resolve sections with Resolve 2 sections.
5. A reuse map: which existing code will power each new section in Part 4.
6. Your plan for Parts 3 and 4, with the files each change touches.

Stop here and wait for my approval.

## Part 3. The intake (highest priority)

Goal: a dedicated intake which turns an idea into an editable, evidence-aware project proposal. Today, project generation happens straight inside the app. Keep that generation logic and put the intake in front of it.

### Experience (Typeform as reference)

1. One question per screen, large type, nothing else competing for attention.
2. A progress indicator, Back and Next, Enter to continue, Shift+Enter for a new line, Esc to leave with answers saved.
3. Any earlier answer stays editable. When an earlier answer changes, flag later answers built on it for review.
4. Conditional questions: skip what the answers already cover.

### Intelligence

1. The AI chooses the next question from the answers so far, guided by Hypothesis Zero. This is not a fixed form.
2. Coverage targets, not a script: what the user wants to achieve, the problem or opportunity, who is affected, why it matters, the outcome wanted and how they would know, the evidence they have, their assumptions, constraints, and uncertainties.
3. A side panel called "What Resolve has heard" sorts each answer into evidence, assumption or open question. The user corrects it in place.
4. Apply Hypothesis Zero faithfully, reusing the existing H0 code: H0 sets the baseline objective or hypothesis, Hx and NHx are competing hypotheses at each level where they fit, the five-question method as already implemented applies recursively, E0 and later experiments test hypotheses, and there is no fixed number of levels. Never replace this with a generic checklist.
5. Keep the decision trail: for every question, store why it was asked.
6. The user stops whenever they want with "Show me the proposal". Resolve also suggests stopping once coverage is good enough.
7. Offline fallback: with no AI connected, use a fixed question set covering the same targets, so the intake still works.

### Output: proposal review

Before anything reaches a real project, show a proposal screen with:

1. Project name and one-line summary
2. Objective and outcome
3. H0 plus first Hx and NHx
4. Open uncertainties
5. Suggested validation, starting with E0
6. First roadmap in Now, Next and Later
7. Tasks
8. Knowledge pages, starting with a project brief

Every section is editable, removable and regenerable. Every item carries a tag: evidence, assumption or uncertain. Nothing is written to the workspace until the user presses Adopt. Adopt writes into the existing stores, plus the intake transcript and decision trail. Uncertainty stays visible after adoption: a generated plan never marks an assumption as evidence.

### Known failures to fix (from my screenshots of the current build)

1. The project is titled "Resolve is a project to create and deliver Doom...". The intake named the project after the app and turned a sentence into a title. Titles must be short, name the real thing (for example "Doom Clock corporate events"), and the user confirms them.
2. The outcome reads "The user appears to be better off when...". Write outcomes in the user's own terms, and the user confirms them.
3. Tasks and roadmap items are long sentences. Keep titles short and put detail in the description.
4. The handoff's "Then" list reads "Validate (Now), Refine (Next), Iterate (Later)". That is generic filler. Every roadmap item must belong to this project.
5. Records store assumptions as "The user is making the assumption that...". Store the assumption itself.

### Testing the intake (explicit requirement)

I must be able to test the intake easily.

1. A Test Intake action, reachable from Home and from Settings.
2. Test runs happen in a sandbox workspace. Nothing touches my real workspaces. A clear "Test mode" banner shows the whole time.
3. Five sample scenarios, for example: a new consumer app idea, rescuing a stalled project, a service business, an internal work project, and Doom Clock corporate events. Doom Clock is one sample among several. Never shape the intake around it. Also offer "Use my own answers".
4. Restart at any point. Jump back to any question and change the answer. Branch from any question to try an alternative path, then compare the two proposals side by side.
5. An inspector panel showing, per question: the question chosen, the H0 reasoning behind it, the evidence and assumption classification, the provider and model used, and the raw AI request and response.
6. Preview the generated proposal. "Promote to real project" is a deliberate, separate action.
7. Pick the AI provider per test run, so I compare Ollama with Claude on the same answers.
8. Save a run's answers and replay them for a repeatable re-test.
9. Export any test run as Markdown and JSON so I share it with Claude for review.

## Part 4. Navigation and information architecture

Put the whole navigation in one config file (an array of sections with id, label, icon, route and visibility). Adding, renaming or reordering a section must not touch view code.

### Global sidebar

Home, Projects, Calendar, AI Team, Knowledge, Settings.

Top bar, on every screen: Search (Ctrl+K), Ask AI (prominent), Add task (Ctrl+Q), New project (opens the intake). Ask AI leaves the sidebar.

### Inside a project

Overview, Roadmap, Tasks, Frameworks, Evidence and Records, Knowledge, Team and Handoffs. Show these as project tabs, replacing today's Overview, Validation, Records, Team, Handoff pills and the Plan page and Tasks board buttons.

### Where today's items go

| Current item | Destination |
|---|---|
| Home | Keep, as the dashboard across projects |
| Inbox | Becomes a "Captured" block on Home plus a filter in Tasks |
| Today | Becomes a "Today" block on Home |
| Upcoming | Feeds the project Roadmap and Calendar. Remove it as a sidebar item |
| Filters and Labels | Filters inside Tasks |
| Outcomes | Overview and Roadmap |
| Calendar | Keep |
| Projects, My Projects, Favorites | Projects |
| Start with Resolve | New project, which opens the intake |
| Tasks board | The board view inside project Tasks |
| Plan page | Overview and the project brief in Knowledge |
| Hypothesis Zero / Validation | Frameworks |
| Validation records, Records | Evidence and Records |
| Team, Team Handoff | Team and Handoffs in a project, plus the global AI Team |
| Ask AI | Top bar |
| Pages | Knowledge |
| Databases | Structured data behind Evidence and Records, and an Advanced area in Knowledge |

Nothing appears in two places as two different things. Where a view repeats in two spots, it is the same component reading the same data.

### Section behaviour

1. Overview: objective, outcome with its confirmed or unconfirmed state, "Do this next", progress, open uncertainties, recent decisions. Keep today's Overview content, including the editable project context and the ResolveOS quick actions.
2. Roadmap: Now, Next and Later columns of outcomes or workstreams, with milestones, dependencies, linked hypotheses and validation gates, and progress from linked tasks. Derive dates from the sequence. Never let a model invent dates. Build it on the existing task and planning data. No second planning system.
3. Tasks: the existing task engine, list and board views, filters and quick add.
4. Frameworks: a small registry. Each framework declares an id, a name, how it creates items, how it renders, and how it links to evidence. Hypothesis Zero is the first entry, with everything I like kept: generated hypotheses, the validation structure, clicking into and acting on single hypotheses. No marketplace.
5. Evidence and Records: existing records and validation records in one place. Every decision links to the evidence behind it, and history is kept.
6. Knowledge: built on Pages. Project brief, requirements, research, documentation, reference material. The intake creates the brief.
7. Team and Handoffs: today's Team and Handoff tabs combined. The global AI Team section lists specialists (for example strategist, researcher, delivery, validation, general assistant; illustrative, not final) and their conversations across projects. Use one chat engine and one conversation store. Ask AI is the same engine in general-assistant mode, not a separate chat. Every agent reads the real project context.
8. Ask AI is contextual. Its placeholder and suggestions change with the current section, following the original Resolve global bar (resolve-docs briefs/14-N8-global-bar.md).

## Part 5. AI providers and Ollama

1. Keep every working provider and MCP integration. Do not rebuild them.
2. In Settings, AI: detect whether Ollama is installed. If not, explain why a local model helps and link to the official download. Recommend a model from the models named in the tutorial, based on the PC's memory, and offer to pull it.
3. One provider layer. Each workflow (intake, planning, framework reasoning, evidence analysis, AI Team) uses a chosen provider, with one default. Local and hosted models both stay supported.
4. Separate what the help text claims from what you tested, in RECONCILIATION.md.

## Part 6. Editability

1. Reusable components.
2. One tokens file for every visual value.
3. Navigation from config.
4. Views separate from business logic. Shared data models.
5. Visual direction for the tokens: set the Resolve look as the default light theme now, and keep the current dark look as the dark theme. Resolve tokens: ink #1A1A18, surface #F2EDE4, card #FFFFFF, border #DDD8CE, amber #C8A040, terracotta #8B4A2A. DM Serif Display for one headline per screen, Inter everywhere else. Radii 10px and 6px. Sentence case. Apply tokens only. Do not restyle screens: Claude handles the visual pass from GitHub.

## Part 7. Order and reporting

Order: Part 0 and Part 1, then Part 2 (stop for approval), then Part 3, then Part 4, then wire intake, overview, roadmap, frameworks, records, tasks, knowledge and AI Team together on the shared data. Visual polish comes last.

Every report back contains:
1. What changed, in plain English.
2. Evidence for each item.
3. Deviations from this brief and why.
4. Files changed and the commit hash on main.
5. Step-by-step instructions for me to test it myself.

Build Resolve, not another general-purpose productivity app.
