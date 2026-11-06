# Resolve 2: design brief for the new front end

6 November 2026. From Simeon. Status: draft for review. Nothing gets built from this until Simeon approves it.

## 1. What this is

Resolve 2 works. The engine is right: the intake logic, Adopt, projects, tasks, calendar, records, sync and the ~40 functions behind MCP. This brief does not change any of that.

What changes is what people see and click. Today the screens are drawn with egui, which cannot reach the quality we want: smooth motion, rich type, the feel of the Response Studios site. So every screen gets redesigned from the ground up, then rebuilt as web screens that call the same engine.

Rules:

1. Design follows the engine. Every button maps to something the engine already does (section 5). If a design needs something new from the engine, list it as a backend ticket. Never let it slip in silently.
2. The original web Resolve is the reference, not the template. Reuse its animations, assets and journey where they fit. Change anything that does not.
3. One design system for desktop and phone.
4. Screens follow the agreed sections and journey in `docs/reference/original-resolve/README.md` and the Part 4 navigation (`docs/reports/2026-11-06-part4.md`).

## 2. References

| Source | Use it for |
|---|---|
| github.com/simeonfab/resolve-docs, `mockups/` (33 `ResolvePM_*_dc.html` files) | The original screen designs. Start every screen from its mockup (table in 4) |
| resolve-docs `mockups/Resolve_Flow_Spec.html` and `MOTION-SYSTEM-HANDOFF.md` | Every animation, with exact durations, easings and staggers. Ship the "match-seam" character |
| `docs/reference/original-resolve/01` to `20` (and the PDF) | Journey, content, tone and look, screen by screen |
| github.com/simeonfab/ResolveOS-intake-frontend, `public/index.html` | The live web intake: its animations and transitions (below) |
| github.com/simeonfab/resolve-docs, `mockups/` | The original phone mockups the pictures came from |
| response.games home page | Quality bar for motion and polish. Match its level, not its design |
| `docs/reports/2026-11-06-part4/` screenshots | What each screen holds today |

Animations to carry over from the web intake (`public/index.html`):

| Name | What it does today | Where it goes |
|---|---|---|
| `bridge-centerout`, `bridge-glowpulse` | The "building your project" bridge | Intake bridge, step 9 |
| `bbrand`, `bfog`, `bparkin`, `bpin` | Bridge brand reveal, fog, items parking and pinning into place | Bridge and reveal, steps 9 to 10 |
| `pulse-mic` | Voice input pulse | Intake free-text step |
| `loader-spin`, `mini-spin` | Waits | Every AI wait (see 4.3) |

## 3. Visual direction

1. Colours: the Resolve palette from the mockups and the web intake.

| Token | Hex | Use |
|---|---|---|
| `--surface` | #F2EDE4 | Page background |
| `--card` | #FFFFFF | Cards |
| `--ink` | #1A1A18 | Main text |
| `--ink-soft` | #4A4A44 | Secondary text |
| `--ink-ghost` | #9A9A92 | Hints, disabled |
| `--border` | #DDD8CE | Lines and borders |
| `--amber` | #C8A040 | Accent, the through-line colour |
| `--amber-light` | #FBF5E6 | Accent fills |
| `--terracotta` | #8B4A2A | Warnings, emphasis |
| `--green` | #5E7D4F | Done, success |
| `--nav-bg` | #1E1E1A | Dark sign-in and intake |

2. Type: DM Serif Display for headlines, Inter 400 / 500 / 600 for everything else.
3. Motion: take every value from `Resolve_Flow_Spec.html`. Easings: `--e-spring` cubic-bezier(.34,1.56,.64,1) for arrivals and sheets, `--e-out` cubic-bezier(.22,.61,.36,1) for reveals, `--e-std` cubic-bezier(.4,0,.2,1) for dims and presses, `--e-in` cubic-bezier(.55,0,.85,.35) for exits. Every motion respects the reduce-motion setting.
4. Dark for sign-in and the intake, light for the app, joined by the dark-to-light reveal (Flow Spec surfaces A to D). Today's desktop app is dark throughout (`src/ui/theme.rs`); the new screens follow the mockups.

## 4. Every screen

Desktop is 1280 px and up. Phone is 390 px wide. Every screen needs four states: empty, loading, error and full. Nothing overflows or overlaps at any width from 1280 px up.

### 4.0 Existing mockups (resolve-docs `mockups/`)

The mockups are phone-first. Desktop layouts are new work; content, look and motion come from the mockup.

| Area | Mockups |
|---|---|
| Sign-in and intake | `Sign-in`, `Onboarding`, `Bring_It_In`, `Confirm_Understanding`, `Floor_Refusal`, `Next_Move`, `Intake_Transition`, `Roadmap_Cards`, `Todo_List`, `Roadmap_Reveal`, `OS_Reveal`, `Daily_Limit` |
| Shell and Home | `App_Shell`, `Global_Bar`, `Global_Bar_Contextual`, `Dashboard`, `Dashboard_Content`, `Dashboard_Motion`, `Portfolio`, `Settings` |
| Project sections | `Roadmap_Tab`, `Roadmap_Motion`, `Work_Tab`, `Work_Motion`, `Work_Packet_Patterns`, `Knowledge_Base_Tab`, `Knowledge_Motion`, `Team_Tab`, `Agent_Chat` |
| Cross-cutting | `Feedback_Motion` (toast and undo) |

No mockup yet, so these are fully new designs: Calendar, page editor, databases, the Hypothesis Zero tree editor, Handoffs, Search, Help, and the "See and edit everything" proposal editor.

### 4.1 Pack A: intake journey (design first)

| # | Screen | Original picture |
|---|---|---|
| 1 | Name and welcome ("What's your name?") | 01 |
| 2 | Bring it in: situation and free text, with voice | 02 |
| 3 | Questions, with "What Resolve has heard" panel | 02 |
| 4 | Confirm understanding: two actions only ("This looks right, show me the proposal" / "Something's wrong, let me change it"), cards grow to fit | 03 |
| 5 | No clear outcome yet | 04 |
| 6 | Your next move, with why | 05 |
| 7 | Roadmap themes, one card at a time ("Yes, add to my roadmap" / "Not quite") | 07 |
| 8 | Here's where to begin: first week in order | 08 |
| 9 | Whole picture, Now / Next / Later, then Adopt | 09 |
| 10 | Building your project (bridge animation) | 06 |
| 11 | Reveal, "Open my project" | 10 |
| 12 | "See and edit everything" proposal editor (H0, Hx/NHx, E0, uncertainties, knowledge pages). Later: not in Pack A. It keeps its current look until Pack C | none, new |

Waits: show what is happening and a seconds counter. No screen sits more than about 5 s without visible progress.

### 4.2 Pack B: app shell and daily screens

| Screen | Holds |
|---|---|
| Sidebar | Workspace switcher, Undo / Redo, Add task, Ask AI; Home, Projects, Calendar; project picker plus Overview, Roadmap, Tasks, Frameworks, Knowledge; AI Team, Handoffs; Search, Settings, Help, save status; unfinished intakes |
| Ask Resolve bar | Contextual input, placeholder and three suggestions per screen (picture 20) |
| Home | Dashboard across projects (pictures 12, 13) |
| Projects | All projects, new project opens the intake |
| Tasks | Project list and board; with "All projects": Captured, Today, Upcoming, Filters & labels (picture 14) |
| Calendar | Calendars, events, agenda |
| Add task | Quick capture with the natural-language line ("Pay rent every month 9am p1 #Home") |
| Search | Everything in the workspace |

### 4.3 Pack C: project sections

| Screen | Holds |
|---|---|
| Overview | Do this next, progress, ResolveOS actions, what matters now, project context, plan page, tasks board |
| Roadmap | Now / Next / Later by phase or by theme, sequence, gates, milestones, dependencies, coming up (picture 15) |
| Frameworks | Registry with Hypothesis Zero first, H0 tree editor |
| Knowledge | Tabs: Brief & documents, Research, Cut list, Evidence & Records, Advanced (databases) (picture 18) |
| Pages | Page editor, templates |
| Databases | Table views, columns, rows, CSV import |
| AI Team | General assistant, Product Director, Validation Lead, Growth Lead, Tech Lead; conversations (pictures 16, 17) |
| Handoffs | Create a brief for someone, Sent / Done, reopen |
| Settings | AI & MCP, provider, prices and budget, Sync & phone, workspaces (picture 19) |
| Help | |

### 4.4 Pack D: phone

The phone page today offers Today, Upcoming, Inbox, Calendar agenda, Search, quick add and complete. Redesign those in the same system, then add the intake on phone (the original mockups were phone-first).

## 5. Function map

The engine functions each screen calls. All exist today unless marked new.

| Screen | Engine functions |
|---|---|
| Intake 1 to 9 | Internal Resolve journey (`src/resolve`, `ui/intake*.rs`): understand, judge, recommend, plan. Not exposed over MCP today |
| Intake 10 to 11 (Adopt) | Internal handoff: creates project, sections, tasks, plan page, context, records |
| Home | `workspace_overview`, `get_agenda`, `list_projects`, `list_tasks`, `plan_my_day` |
| Projects | `list_projects`, `resolve_projects`, `create_project`, `update_project` |
| Tasks | `list_tasks`, `create_task`, `update_task`, `complete_task`, `delete_item`, `organise_todos`, `convert_todos_to_tasks`, `import_tasks` |
| Calendar | `list_calendars`, `create_calendar`, `list_events`, `create_event`, `update_event`, `get_agenda`, `import_ics` |
| Overview, Roadmap, Frameworks | `resolve_project`, `list_records`, `add_record`, `update_record`, `update_project` |
| Knowledge, Pages | `list_pages`, `read_page`, `create_page`, `update_page`, `list_page_todos`, `import_pages` |
| Databases | `list_databases`, `create_database`, `query_database`, `add_database_column`, `add_database_row`, `update_database_row`, `import_csv` |
| AI Team, Ask Resolve | Internal role router and assistant (`resolve/roles.rs`), using the tools above |
| Handoffs | Internal brief generator, Handoff records via `add_record` / `update_record` |
| Search | `search` |
| Weekly review | `weekly_review` |
| Undo / Redo, Settings, workspaces | Internal app functions. Not exposed over HTTP today |

Backend tickets this creates (for the engineer, not the designer):

1. Expose the intake journey, Adopt, the AI Team router, Undo / Redo and Settings to the web screens. Today they run inside the desktop app only.
2. Serve the new web screens from the engine, the way `/phone` is served today.

## 6. Build order

1. Pack A: the intake journey. Most reference material, most visible.
2. Pack B: shell and daily screens.
3. Pack C: project sections.
4. Pack D: phone.

Each pack is designed, approved, then built and checked against the designs (pixel comparison, as on Response) before the next starts. The egui screens stay as the fallback until each web screen replaces them.

## 7. What each design must deliver

1. Desktop and phone frames for every screen in the pack, in all four states.
2. Every animation named, with timing and easing.
3. Exact copy, in second person ("you", never "the user").
4. Tokens: colours, type sizes, spacing, radii, shadows.
5. A note on any screen element with no engine function behind it.

## 8. Decisions

1. Colours and fonts: the Resolve palette, DM Serif Display and Inter (section 3).
2. Motion: the Resolve Flow Spec, "match-seam" character.
3. The "See and edit everything" proposal editor waits for Pack C.
4. Quality bar: the response.games home page, for its level of polish, not its design.
