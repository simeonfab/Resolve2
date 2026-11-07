# Resolve 2 Build Brief: Packs A to D

7 November 2026 · Simeon

Live version: [Resolve 2 Build Brief (doc)](https://claude.ai/artifact/1yKbJZFzfLsgBri5m9tfHi). This file is a copy taken on 7 November 2026; if the two differ, the doc is newer.

This brief turns the four approved design canvases into code for the Rust app (eframe/egui 0.29, repo simeonfab/Resolve2). Build in the phase order below. The canvases are the visual reference; every value you need is also written here.

## Design references

Open the canvas for the screen you are building. Each canvas has the same five pages: 1 Map, 2 Screens, 3 States and wide view, 4 Motion, 5 Tokens, decisions and tickets.

| Pack | Canvas | Covers |
| --- | --- | --- |
| A | [Resolve 2 Intake Journey](https://claude.ai/artifact/UUpPfSfTcEphWNP2nKaXHP) | The intake, front door to reveal (screens 01 to 12, states ST1 to ST6) |
| B | [Resolve 2 App Shell and Daily Screens](https://claude.ai/artifact/K4phbfZQVnS6GJQa3NpqFg) | Sidebar, Ask Resolve bar, Home, Plan my day, Projects, Tasks, Add task, Calendar, Search |
| C | [Resolve 2 Project Sections](https://claude.ai/artifact/TcsdU9czKbts2U1hruGPSC) | Overview, Roadmap, Frameworks, proposal editor, Knowledge base, page editor, databases, AI Team, Handoffs, Settings, Help |
| D | [Resolve 2 Phone](https://claude.ai/artifact/91HZUemQxHXL5DfH5Jdcvv) | The phone page: tasks, quick add, agenda, search, Ask, not connected, intake |

The canvases are private to Simeon's account. A coding agent signed in to the same account reads them with its artifact read tool. Anyone else needs them shared from each canvas's Share menu first.

How to read a frame:

- Every screen frame has Tweaks for `state` (full, empty, loading, error) and `collapsed` (sidebar). Page 3 lays out each state side by side, plus the 1920 × 1080 version.
- Frame names on page 2 match the view names in this brief.
- Sample content (the café project, names, dates, prices) is placeholder. Real screens read the user's data.
- Where a canvas and this brief disagree, this brief wins. It includes the review changes made on 7 November.

Other sources in the docs repo, `resolve-docs/mockups/`: `Resolve_Flow_Spec.html` (the motion spec every timing traces to) and the original ResolvePM mockups.

## Build order

Seven phases, 0 to 6. Each one ends with `--selftest` passing and screenshots of every new view in all four states. Do not start a phase until the one before it passes.

| Phase | What ships | Pack | Tickets |
| --- | --- | --- | --- |
| 0 Foundations | Tokens in `theme.rs`, shared components in `widgets.rs`, the motion helper, one-at-a-time toasts, voice capture | A, B | E16, E17, E18, B-E10, B-E18, B-E20, E2 |
| 1 Shell | Sidebar with groups, collapse and project picker; Ask Resolve bar and sheet; Undo and Redo toasts; Search palette | B | B-E1 to B-E3, B-E14, B-E17, B-E24 |
| 2 Daily screens | Home, Plan my day, Projects, Tasks (list, board, all projects, editor), Add task, Calendar (month, week, agenda, event editor) | B | B-E4 to B-E13, B-E15, B-E19, B-E21 to B-E23 |
| 3 Intake | Every intake step, waiting states, errors, the AI limit screen, bridge and reveal | A | E1, E3 to E15, E19 to E28 |
| 4 Project sections | Overview, Roadmap, Frameworks, proposal editor, Knowledge base, page editor, databases, AI Team, Handoffs | C | C-E1 to C-E20, C-E24 to C-E26 |
| 5 Settings and Help | Four Settings tabs, Help guide, Report a problem | C | C-E21 to C-E23 |
| 6 Phone | The phone page restyled and extended | D | D-E1 to D-E16 |

Phase 0 comes first because every later screen uses the tokens, the components and the motion helper. The shell comes before the screens because every screen sits inside it.

## Rules for every screen

1. **Ownership.** Per the repo's CLAUDE.md: colours, fonts, text styles, radii and spacing live only in `src/ui/theme.rs`; component styling lives in `src/ui/widgets.rs`. Views never hard-code a colour or size.
2. **Navigation is data.** Sidebar groups, items, the Frameworks registry, Knowledge base tabs and the Ask Resolve wording per section live in the tables in `src/ui/nav.rs`. Change the tables, not the views.
3. **Four states.** Every view draws full, empty, loading and error. Empty says what goes here and gives one action. Loading uses skeletons in the layout's shape plus one line of what is happening. Error says what failed, that nothing else was lost, and offers Try again plus a manual path.
4. **Errors never block the rest.** A failed AI call shows its error inside its own card. The rest of the screen loads.
5. **Voice.** Every multi-line field and every Ask input has a mic button: recording state, then speech to text into the field.
6. **Wide screens.** Content caps and centres at 1920 (Home, Projects, Overview, Frameworks, Knowledge base, Handoffs, Help 1240; Tasks 1100; Settings 1100; AI Team 1320; Roadmap and Calendar full width). Reading text stays 680 to 720 wide.
7. **Copy.** Plain words, short sentences, no em dashes. Buttons say what happens ("Use this plan", not "OK").
8. **Contrast.** Never amber text on light. Green text is #3F5A33 on #E8EFE3. On dark, terracotta becomes #C97A4A and the smallest text is #8A8578.
9. **Reduced motion.** One app-wide flag. When on, every move becomes a fade of 120 ms or less, loops hold still, the bridge skips to its last beat.
10. **Undo.** Every complete, delete, move and AI write goes through `history.rs` and shows a toast with Undo.

## Tokens

Add these to `theme.rs` under these names. No other hues exist in the app.

**Colour**

| Token | Value | Use |
| --- | --- | --- |
| nav-bg | #1E1E1A | Sidebar, intake top bar and rail, front door, phone "not connected" |
| ink | #1A1A18 | Text on light, dark hero cards, primary dark buttons, Ask bar |
| pane | #201F1C | Cards and fields on dark, phone intake |
| surface | #F2EDE4 | Work area |
| card | #FFFFFF | Cards, fields, light phone bar |
| border | #DDD8CE | Card and field edges |
| amber | #C8A040 | Accent, CTA fill, focus ring, active nav icon |
| amber-light | #FBF5E6 | Assumption tag, highlights |
| amber-text | #7A5B12 | Amber meaning as text, on amber-light |
| ink-soft | #4A4A44 | Secondary text |
| ink-ghost | #9A9A92 | Disabled and decoration only |
| terracotta | #8B4A2A | Eyebrows, links, errors on light, Work calendar |
| terracotta-tint | #F5E8E1 | Waiting, priority, error panels on light |
| terracotta-on-dark | #C97A4A | Errors on dark |
| green | #5E7D4F | Evidence marks, Personal calendar, saved dot |
| green-text / green-tint | #3F5A33 on #E8EFE3 | Evidence tag, Supported status |
| text-on-dark | #F5F1E8 | Text on dark |
| text-on-dark-soft | #B4AFA3 | Secondary text on dark |
| text-on-dark-muted | #8A8578 | Smallest text on dark |
| surface-raised | #F7F3EC | Sheet backgrounds, dialog footers, hovered rows |
| surface-sunk | #EAE4D9 | Board and roadmap columns |
| track | #E4DED2 | Segmented control track, progress track, disabled button |
| divider | #EEE9DF | Rows inside a card |

The last four are derived tints (Pack B decision B9). Simeon approved Pack B as a whole on 7 November; treat them as approved.

**Type.** DM Serif Display for headlines, Inter 400, 500, 600, 700 for the rest. Both are in `assets/fonts/`.

| Token | Font | Size / line height | Use |
| --- | --- | --- | --- |
| hero | DM Serif 400 | 52 / 1.05 | "Hi Simeon." only |
| display | DM Serif 400 | 44 / 1.1 | Front door, reveal, intake 05 |
| title | DM Serif 400 | 30 to 36 / 1.15 | Screen and step headlines |
| question | DM Serif 400 | 30 / 1.22 | One intake question at a time |
| card-serif | DM Serif 400 | 20 to 28 / 1.25 | Hero cards, theme cards, sheet titles |
| body | Inter 400 | 13.5 to 14.5 / 1.5 | Card text |
| answer | Inter 400 | 15 / 1.55 | Text boxes |
| button | Inter 600 | 13 to 15 | Buttons |
| label | Inter 600 or 700 | 12.5 to 14 | Eyebrows, card titles, group labels |
| small | Inter 400 or 500 | 12 to 12.5 | Hints, meta |
| tag | Inter 600 or 700 | 11 to 11.5 | Tags, pills, counts |

Today's `theme.rs` sizes: HERO 30, QUESTION 28, TITLE 22, SECTION 15, BODY 14, ANSWER 17, SMALL 12.5, TINY 11.5. Change HERO, QUESTION and ANSWER to the values above. Numbers use tabular figures.

**Spacing** (4 px base): 4, 6, 8 inside tags and button rows; 10, 12 card and list gaps; 14 to 18 card padding; 22, 24 rail padding; 26 to 40 work area padding (30 top, 40 sides; 32 on Calendar and Roadmap).

**Radii:** 999 pills, tags, Ask bar; 6 to 8 chips and small buttons; 9 to 10 buttons and fields; 12 to 14 cards; 16 dialogs and hero cards; 20 sheets (top corners).

**Shadows and rings**

| Name | Value |
| --- | --- |
| focus | 0 0 0 3px rgba(200,160,64,.22) plus border amber |
| card-lift | 0 1px 2px rgba(26,26,24,.04), 0 10px 30px rgba(26,26,24,.07) |
| dialog | 0 30px 80px rgba(26,26,24,.35) |
| sheet | 0 -18px 60px rgba(26,26,24,.25) |
| side panel | -18px 0 50px rgba(26,26,24,.14) |
| Ask bar | 0 10px 30px rgba(26,26,24,.22) |
| backdrop | rgba(26,26,24,.18) panel, .32 sheet, .38 dialog |

egui has no CSS gradients or blur. Shadows, the hero glow and the reveal bloom need pre-rendered textures or a gradient mesh (E17, B-E20).

**Status pills** (Frameworks, records, roadmap gates)

| Status | Pair |
| --- | --- |
| Unresolved | ink-soft on surface |
| Supported | green-text on green-tint |
| Weakened | amber-text on amber-light |
| Refuted | terracotta on terracotta-tint |
| Planned (experiment) | amber-text on amber-light |
| Gate | "◇ H1 result", ink-soft on surface, radius 6; open gate "✓ Gate open" in the Supported pair |

Knowledge base card kinds: Page, File, Experiment = ink on surface; Research, Evidence = green pair; Hypothesis, Assumption = amber pair; Open question, Cut = terracotta pair.

## The shell

One dark sidebar on the left, a light work area, and the floating Ask Resolve bar in every screen. Reference: Pack B, page 1.

**Sidebar** (`nav.rs`, `mod.rs`). 248 px wide, 64 collapsed, background nav-bg.

| Part | Contents |
| --- | --- |
| Top row | Workspace switcher (amber tile with initial, name, ▾), Undo ↶, Redo ↷, collapse « |
| Buttons | Add task (amber, opens Add task) and Ask AI (outlined, ✦, opens the Ask sheet). Both stay. |
| WORKSPACE | Home, Projects, Calendar |
| YOUR PROJECT | Project picker (dot + name + ▾), then Overview, Roadmap, Tasks (count), Frameworks, Knowledge base |
| COLLABORATION | AI Team, Handoffs |
| Footer | "Continue setting up…" card (only when an intake is unfinished: "New project · step 3 of 4"), Search (Ctrl K), Settings, Help, then "Saved on this PC · works offline" with a green dot |

The middle group was "Product management"; it is now YOUR PROJECT. "Knowledge" is now "Knowledge base". Frameworks stays its own item. Groups do not collapse.

Item: 32 high, radius 8, Inter 500 13.5, text #B4AFA3, icon 16 stroke 1.4. Active: background rgba(200,160,64,.16), text #F5F1E8, icon amber. Group label: Inter 600 11, letter spacing .04em, #8A8578. Dividers rgba(255,255,255,.08).

Collapsed (Ctrl \\ or «): icons only, centred, group labels become thin dividers; hovering an icon for 400 ms shows a tooltip with its name. The choice is remembered per workspace.

**Ask Resolve bar.** 560 × 48, radius 999, ink, centred on the work area, 20 px above the bottom edge. Left ✦ in amber, the placeholder for this screen, a 34 px mic button. Focusing it opens a "Try" panel of suggestions above it. Pressing Enter (or the sidebar's Ask AI) opens the Ask sheet. Wording per screen comes from `nav.rs`:

| Screen | Placeholder | Suggestions |
| --- | --- | --- |
| Overview | Ask about this project | What matters most this week? · What's still uncertain? · Summarise where this project is |
| Roadmap | Ask about your roadmap | What's missing from this plan? · Move a theme into Now · Why is this sequenced this way? |
| Frameworks | Ask about your hypotheses and tests | Which hypothesis is weakest? · Suggest a cheap test for H0 · What would change my mind? |
| Knowledge base | Find or capture something you know | What evidence supports H0? · Summarise the project brief · Capture a research note |
| Handoffs | Ask about a handoff | What should the next handoff cover? · Is anything blocking a handoff? |
| Calendar | Ask about your schedule | Plan my day · What's overdue? · Find time for my next task |
| Help, other | Ask Resolve, or capture a thought | What should I focus on right now? · Capture a quick note |

Home, Tasks, Projects and Search keep the wording already in `nav.rs`. AI Team shows no bar: its page has its own chat box (Pack C decision C2).

**Ask sheet.** Rises from the bottom edge of the work area, 720 wide, 82% of the window high, radius 20 on the top corners, background surface-raised, over a .32 backdrop. It is flush with the bottom on purpose; it never floats above it. Esc, × or a backdrop click closes it.

**Overlays**

| Kind | Size | Used by |
| --- | --- | --- |
| Dialog | radius 16; widths: Add task 640, Search 680, Event 520, templates 640, CSV import 600, Report 560, whole picture 900 | Centred, 70 to 90 px from the top |
| Sheet | 720 (Ask), 760 (Plan my day), 620 (Add to Knowledge base) | From the bottom |
| Side panel | 440 (task editor), 520 (document preview), 400 (Frameworks node editor, inline) | From the right |

**Toasts.** One at a time; a new one replaces the old with a 160 ms swap. Ink, radius 14, 460 wide, 10 px above the Ask bar. With Undo: a 3 px amber timer bar across the bottom, 4 s. Plain: 2.6 s, no timer. Undo and Redo in the sidebar show a toast naming what changed.

## Pack A: the intake

Files: `ui/intake.rs`, `intake_steps.rs`, `intake_review.rs`, `intake_ending.rs`; logic in `resolve/journey.rs` and `resolve/intake.rs`. Full window, no sidebar.

**Frame (02a to 09).** Dark top bar, 64 high (brand tile "R" + "Resolve", then "Save and exit" and a ⋯ menu on the right). Dark rail on the left, 308 wide, collapsing to 72 on wide steps (03 and 09): four steps (1 Get started, 2 Bring it in, 3 Confirm understanding, 4 Your next move), each with a one-line description; done steps show ✓; footer card "Saved as you go. Leave any time. Resolve opens on this same step when you come back." Light work pane on surface, padding 40 top, 56 sides, column max 760 (800 on 04, 960 at 1600 wide and above).

**Top bar actions.** "Save and exit" saves and goes to Home. The ⋯ menu holds "Start this intake again (keeps your name)". Home and the sidebar footer then show "Continue setting up…".

| Screen | What it shows | Actions and rules |
| --- | --- | --- |
| 01 Name | Dark front door. Eyebrow "Let's get started", display headline "What's your name?", one field "Your first name" | Continue → (disabled until a character is typed); "Press Enter to continue"; "Your answers save as you go." |
| 02a Welcome | Frame appears (rail slides in, pane washes light). "Hi Simeon." (hero), "So," then "What do you need help with?" with three path cards | "I have a messy project" is live. "I have a new idea" and "I'm stuck, not sure where to start" are greyed with a "Coming soon" tag. Enter or click continues |
| (parked) AI setup | Between 02a and 02b, only when no AI is connected | Not designed yet. Ticket only |
| 02b Bring it in | Eyebrow "In your own words", title "Tell Resolve what's going on.", one large text box with mic | "Use a template" inserts 8 headings below what is typed: What I'm working on / What I want it to achieve / Who it's for / Where it's at now (what's done, what exists, what's decided) / What's getting in the way / What I'm working within (time, money, people) / What I'm least sure about / How I'll know it worked. Hint "Leave any heading blank. Resolve asks about the gaps next." "✦ Try an example" fills them with a sample. Link "Coming from another AI chat? Get a prompt to pull it in →". Back, Continue |
| 02c Pull-in panel | Opens in place under the box: "Bring it in from ChatGPT, Claude or another AI", step 1 "Copy this into the chat where you've been working on it", the prompt in a scroll box, Copy prompt (turns to "Copied ✓" for 1.5 s), step 2 "Paste its answer into the box above, then press Continue" | Continue reads "Paste the answer to continue" until text exists. × or a click outside closes the panel |
| 03 Questions | Eyebrow "\<Target> · question N", one question (question type), answer box with mic, "▸ Why this question?" expander. Right: "What Resolve has heard" panel, 380 wide (440 at 1600+) | Row: Back · Skip this question · Next → (Enter = Next, Shift+Enter = new line). Below: "Skip the rest and see my proposal →" with a coverage bar and "Covered 78%. More answers make a sharper proposal." Heard items: label menu (Evidence, Assumption, Question), pencil and × on hover, click the text to edit in place, Enter saves, Esc cancels; first-time hint "Click any line to edit it. Click a label to change it." with Got it |
| 04 Confirm | "← Back to the questions", title "Here's what Resolve understood.", sub "Check it. It shapes everything that follows." Five cards: Project, What you want, Where it is, Biggest uncertainty, The outcome: who is better off, and how you'd know | "This looks right, show me the proposal →" and "Something's wrong, let me change it" |
| 04b Confirm, edit mode | Same cards, each an editable field with amber border; sub "Change anything that's wrong. The outcome is best in your own words." | Footer swaps to Save and continue / Cancel |
| 05 No clear outcome | Eyebrow "Before we build this", display "We're not going to guess." Two short paragraphs, then "ONE QUESTION: If Resolve does its job, what is different in three months that isn't true today?" | Answer box, Continue. Answering re-runs the check; passing shows a ✓ and "That's the piece we needed." then moves on |
| 06 Next move | Eyebrow "Your next move", title "Here's what to do next." Dark "Do this first" card (action + reason), then "What happens after this": Now / Next / Later theme strip | Reassurance line "Nothing here is final. You can change any of it once your project is open." Back, Continue, "Skip to the whole picture →" |
| 07 Themes | Eyebrow "Building your roadmap", title "Does this belong on your roadmap?", "Theme 1 of 6" segment bar, one theme card (phase tag, Evidence/Assumption tag, serif title, reason). Right: "On your roadmap" list, empty text "Nothing yet. Each theme you say yes to lands here, in order." | "Yes, add to my roadmap", "Not quite" (opens a field + Save and continue), ← Back, "Skip to the whole picture →", reassurance line |
| 08 First week | Eyebrow "Your first week", title "Here's where to begin.", sub "Built straight from your roadmap. The first concrete moves, in the order they matter." Five numbered task rows; row 1 tagged "Up next" | Back, "See it as a roadmap →", "Skip to the whole picture →", reassurance line |
| 09 Whole picture | Title "Here's the whole picture.", project name field, three lanes Now / Next / Later with themes, their tasks and tags | Back, "Adopt, create this project →" with "Adopt creates the project with its roadmap, tasks, validation and pages. You can undo it." and "Edit the full proposal →" (opens the Pack C proposal editor). Any click finishes the reveal animation |
| 10 Bridge | Full dark. "You just cleared the fog, Simeon." / "A messy café pricing question, turned into a focused path forward. That's the hard part done." Build checklist bound to real progress: Creating the project, Laying out the roadmap, Adding tasks, Setting up validation, Writing the knowledge pages, Saving the intake transcript | Minimum 7.2 s; ends when the build ends |
| 11 Reveal | "Welcome in, Simeon." on dark, amber bloom, light layer: "Your project is ready", project name, three counts (themes, first-week tasks, first move pinned), the Do this first card | "Open my project →" goes to Overview |
| 12 AI limit | Eyebrow "Your proposal", title "You've reached your AI limit for now.", "Nothing is lost…" paragraph, the budget sentence from `ai.rs` | "Continue without AI →", "Change AI settings", "Leave and come back later". Right: "Where it stopped" checklist, frozen, same strings as the waiting checklist |

**Pull-in prompt** (replaces `IMPORT_PROMPT` in `intake.rs`): "I'm moving this project into a planning tool called Resolve. Using only what's in this conversation, answer the questions below. Write as me, in the first person. Keep each answer to a few sentences. If the conversation doesn't cover a question, write "Not covered" instead of guessing. Put "(assumption)" after anything I believe but haven't checked." Then 11 numbered questions. The canvas shows 1 to 5: What am I working on? / What do I want it to achieve? / Who is it for, and who else is affected? / Why does it matter, and why now? / Where is it now: what's done, what exists, what's been decided? Questions 6 to 11 must cover the remaining `TARGETS` in `journey.rs`, one each: problem, outcome, evidence, assumptions, constraints, uncertainties. Reuse each target's question wording in the first person.

**States**

| State | Rule |
| --- | --- |
| ST1 Waiting | Card in the question column: "Thinking about what to ask next…", seconds counter (tabular), checklist driven by real progress events (Noting that answer / Checking what's still not covered / Choosing the next question), a bar that fills to 15 s. Something visibly changes at least every 5 s |
| ST2 15 s hand-off | At 15 s the next question comes from `TARGETS`. Note above it: "The AI took longer than 15 s, so this question comes from Resolve's own list." "Why this question?" names the target. The late AI answer is dropped, never swapped in under the user's typing |
| ST3 Long wait | Proposal only (about 60 s today). After 15 s a note: "This step takes about a minute. Your answers are saved, so you can leave and come back to this exact point." with "Leave and come back later" |
| ST4 Error | "Resolve couldn't read that just now." + what failed + "Nothing you wrote is lost." Try again (focused), the step's manual path, Change AI settings. Manual paths: 03 next fixed question; 04 fill the cards yourself (blank cards in edit mode); 06 to 09 regenerate the failed part only, or use the version made without AI; 10 back to the whole picture |
| ST5 Resume | On launch mid-intake: "Welcome back, Simeon." + what they were doing + what is saved ("7 answers and the understanding cards"). "Pick up where you left off →" and "Start this intake again (keeps your name)" |
| ST6 Wide | At 1920: column 960 centred, heard panel 440, rail unchanged |

## Pack B: daily screens

All at 1280 × 800 inside the shell. Work area padding 30 top, 40 sides. Reference: Pack B, pages 2 and 3.

| Screen | File | Layout | Actions | Empty / loading / error |
| --- | --- | --- | --- | --- |
| Home | `dashboard.rs` | Greeting "Good afternoon, Simeon" (title) + a subline holding today's counts (due, overdue, done, events). Left: dark hero "Your highest-value move": project name, the move, the reason, "Focus theme · \<theme>". Then Today (agenda rows, "Open calendar →"). Right column: SINCE MONDAY (done count + list), WAITING ON (stalled items with age), PROJECTS (name + progress, "All →"), CAPTURED ("2 captured, not in a project yet", "Sort →"). Content caps at 1240 at 1920 | Start this → (opens Tasks with the editor on that task), ✦ Plan my day (opens the Plan my day sheet), + New project | Empty: "Start here / Bring in your first project." + New project, and the "Continue setting up" card ("You stopped at Confirm understanding. Everything you wrote is saved." Pick up →). Loading: "Loading your day…" + skeleton. Error: "Resolve couldn't plan your day just now." with Try again and "Open today's tasks"; the rest of Home still loads |
| Plan my day | new, sheet over Home | Sheet 760 wide, 88% high, from the bottom, centred on the work area. Header: ✦, "Your plan for \<weekday>" (card-serif 26), sub "From your agenda at 09:12 · 2 events, 5 tasks, 1 overdue · 1 clash found". A white card of rows: time range (92 wide, tabular), a 4 px bar (amber task, terracotta event or overdue), title + one line on why it sits there, a tag (Task / Event / Overdue). First row is bold on #FFFDF7. Then a terracotta-tint box "Won't fit today." + reason + "Move to Monday" | "Use this plan" (amber, full width), "Plan again", a change box with mic ("Change something: I can't start before 10"), note "Use this plan gives each task today a start time. Nothing moves in your calendar." | Empty: "Nothing to plan today." + Show Upcoming / Close. Loading: "Reading today's 2 events and 5 tasks, overdue included…" + four row skeletons. Error: "The AI didn't reply, so there's no suggested order." + today listed by time + Try again |
| Projects | `sections.rs` | Title + "Each project has its own overview, roadmap, tasks, frameworks and knowledge base." + New project. Cards: name, ⋯, outcome, "Next move · \<move>", progress, Now / Next / Later counts, open questions, last updated. Last card dashed: "+ New project / Opens the intake. About ten minutes." Two columns, three when the sidebar is collapsed or at 1920 | Card opens Overview. ⋯: Open overview, Rename, Edit outcome | "No projects yet." + New project → / skeleton cards / "Resolve couldn't read your projects. Your data is still on this PC. This is usually a file another app has open." Try again, Open Help |
| Tasks, one project | `tasks.rs` | Eyebrow project, title Tasks, List / Board toggle, ✦ Organise, + Add task. Progress text + bar. List sections: Now, Waiting on, Upcoming from roadmap; rows with tick, title, theme chip, label chip, due. "View completed ›". Board: Now / Next / Later / Done columns on surface-sunk, cards draggable between columns, "+ Add" per column | Click the tick completes (toast "Completed: \<title>" + Undo). Click the row opens the editor | Empty title + body + Add task / Open the roadmap. Error: "Resolve couldn't load these tasks. Nothing is lost. Your tasks are saved on this PC." |
| Tasks, all projects | `tasks.rs` | Picker set to All projects: tabs Captured, Today, Upcoming, Filters and labels | Same row actions | Same states |
| Task editor | `tasks.rs` | Push panel 440 from the right. Header "\<Project> · \<phase>", → close, ⋯. Fields: Title, Theme ▾, End date, Priority ▾, Labels, Notes (mic) | Save, Mark complete, Delete. Unsaved edits ask before closing | "Saved" toast |
| Add task | dialog | 640 wide. "Type it the way you'd say it", input with recognised words highlighted (date/time and repeat amber, priority terracotta, #project green), placeholder "e.g. Call the garage tomorrow 9am". "Resolve understood" chips: Task, ↻ Every month, 9:00, Priority 1, Project: Home. Help line: tomorrow 9am sets the date and time / every month repeats it / p1 to p4 sets priority / #Project files it / @label tags it. "Click a chip to change it. Backspace on a highlighted word turns it back into plain text." | Goes to \<dest> ▾, Cancel (Esc), Save. No emoji in chips | Error: "Couldn't save the task. Your text is still here. Press Enter to try again." |
| Calendar | `calendar.rs` | Title (month), ‹ Today ›, Month / Week / Agenda toggle, + Event. Left: CALENDARS list with toggles (Work terracotta, Personal green, Tasks with dates amber), + New calendar, note "Tasks show here too: tasks with a date appear as amber rows, on the day they're due." Week view has a now-line | Click an event opens the editor | "Nothing on your calendar yet. Add an event, or give a task a date and it appears here." + Event / "Loading events…" / "Resolve couldn't load your events. Your calendars are saved on this PC. Tasks with dates still show." |
| Event editor | dialog | 520 wide: title, When (date, from, to), All day, Repeats ▾, Calendar ▾, Notes | Delete, Cancel, Save |  |
| Search | palette | 680 wide over the current screen. "Search everything", Esc. Type chips. Grouped results with snippet and meta. Empty: "Type to search everything in this workspace." + NARROW IT DOWN: task: / page: / project: / event: / db: | Footer: ↑ ↓ move, Enter open, Tab next type, Ctrl K from anywhere | None: "Nothing matches "payroll". Try fewer words, or search one type with task: or page:." + "Add "payroll" as a task". Error: "Search couldn't finish. Your data is fine. Press Enter to try again." |

## Pack C: inside a project

Every section reads the project chosen in the sidebar's picker. Header pattern: eyebrow = project name (terracotta 12.5), title (DM Serif 30), page actions top right. Reference: Pack C, pages 2 and 3.

| Screen | File | Layout | Actions | Empty / loading / error |
| --- | --- | --- | --- | --- |
| Overview | `sections.rs` | Header: project name + outcome line, "Weekly review", ⋯. Row 1: dark DO THIS NEXT card (move, the hypothesis it tests, the gate it opens; Start this, See it on the roadmap) beside a progress card (% done, "1 of 5 tasks done", bar, Now / Next / Later counts, "Hypotheses: 0 of 4 resolved · 3 experiments planned · 8 open questions"). Row 2, three cards: What matters now (coloured dot + line), Resolve suggests (✦ rows, each opens an action), Project context (brief excerpt, Project brief and Plan page buttons). Row 3: mini Tasks board (Now / Next / Later) with "Open Tasks →" | Weekly review runs `weekly_review` | "This project has no plan yet." + Continue setting up → / Add a task. Error: "Resolve couldn't work out your next move." (the rest opens) + Try again / Open the roadmap |
| Roadmap | `sections.rs` | Header + By phase / By theme toggle, Whole picture, + Add a theme. By phase: three columns (Now "this month", Next "after the gates", Later "parked") of theme cards: title, Evidence/Assumption tag, gate chip "◇ H1 result", progress bar + "1 of 3", expand ▾. By theme: one white list, row per theme with phase pill, tags, gate and its tasks inline. Right rail 270: Milestones (diamond mark, date), Dependencies and blockers (blocked item in terracotta-tint), Coming up | Expanded card (in place): description, task checkboxes, "Waiting on" line, + Task, Move to Next, Open E0. Whole picture: 900 dialog, grid of hypotheses (rows H1 to H4) × Now / Next / Later, cells name the theme, gated ones show ◇, a cell click opens the theme | "No roadmap yet." + Draft it from my brief / Add a theme. Error: "That move didn't save. \<Theme> is still in Next. Nothing else changed." + Try again |
| Frameworks | `sections.rs` | Header + "+ Hypothesis", "+ Experiment". Registry strip: Hypothesis Zero card (amber border, "4 hypotheses · 3 experiments · 0 resolved") + dashed "More methods later" card + one explanatory line. Tree: H0 as the one dark row, hypotheses indented 24 (id, statement, "NH1: \<negation>", status pill), experiments indented 56 (id, title, threshold line, Planned) | Click a node opens the editor (400 wide, beside the tree): We believe, It's wrong if (NHx), Status segmented (Unresolved / Supported / Weakened / Refuted), the experiment's prediction and threshold, "Opens the gate on \<theme>", Save, + Sub-hypothesis | "No hypotheses yet." + Write H0 / Suggest from my brief. Error: "H2 didn't save. Your edit is still in the panel." + Save again / Discard my edit |
| Proposal editor ("Edit the full proposal") | `intake_review.rs` | Intake frame, no sidebar. Top bar: "← Back to the summary", "Proposal · \<name>", Saved, "Adopt and create this project". Left nav 210: Outcome and H0, Hypotheses 4, Experiments 3, Uncertainties 8, Roadmap 6, First week 5, Pages 3, then the line "Nothing here is final. You can change any of it once your project is open." Main: section cards, every text an editable field | Back keeps every edit. Adopt does the same as Adopt on 09 | "Resolve didn't have enough to propose yet." + Answer more questions / Start it blank. Loading "Writing the full proposal from your answers…". Error: "Your last edit didn't save." + Save again |
| Knowledge base | `knowledge.rs` | Title "Knowledge base", find field, + Add. Tabs: Brief and documents, Research, Cut list, Evidence and Records, Advanced: databases. Evidence and Records adds filter chips with counts (All, Roadmap, Hypotheses, Experiments, Evidence, Assumptions, Open questions). Library: card grid (3 columns, 4 at 1920): kind tag, meta, title, one line; last card dashed "+ New page, or drop files here". Grounding bar: "Resolve reads everything here when it answers about this project. 9 pages and files, 26 records." + Import files | + Add opens the add sheet (620, from the bottom): New page, From a template, Add files, Record evidence, Open question, plus a drop zone ".md, .txt, .docx or .pdf". A file card opens the preview (520 from the right): kind tag, title, "What Resolve took from it", the document, "Linked to H2 and \<theme>", Open, Move to…, Cut | "Nothing filed yet." + Add files / New page. Error per file: "Resolve couldn't read Pricing notes.pdf. The file looks like a scan with no text in it. The other 2 files are filed." + Keep as attachment / Choose another file |
| Page editor | `pages.rs` | Bar: breadcrumb "Knowledge base › \<tab> › \<page>", toolbar H1, H2, B, I, bullet, to-do, table, Saved state, ⋯. Page text 720 wide. Right rail 250: "To-dos on this page" ("3 to-dos, 1 done. They show in Tasks under this project."), Linked chips, "Saved as Markdown on this PC. Type / for headings, lists, to-dos and tables." | New page dialog (640): Blank, Meeting notes, Business plan, Product spec, Project brief, Weekly review, Daily journal (from `templates.rs`), then Your templates ("Any page's ⋯ menu: Save a copy as template") | Empty: "Untitled" + Use a template / Import a file. Error: "Your last changes aren't saved." + Save now / Copy the text |
| Databases | `databases.rs` | Inside Knowledge base, Advanced. Left list of databases + "+ New database" + "Templates: CRM, Projects". Toolbar: + Row, + Column, Import CSV, Group, Sort. Table in a horizontal-scroll box. Note "Saved as CSV in your workspace folder." | CSV import dialog (600): file name, "36 rows, 4 columns", one row per column with sample value and type ▾, destination (new database or add to an existing one), Import N rows | "No databases in this project." + Import a CSV / Start from CRM / Blank table. Error: "Rows 14 to 16 didn't import. Their Hours column has text where a number should be. The other 33 rows are in." |
| AI Team | `team.rs` | Left 280: role cards (General assistant "same chat as Ask AI", Product Director, Validation Lead, Growth Lead, Tech Lead), then earlier chats with the selected role. Right: chat panel, header role + "Reads your brief, roadmap, H0 tree and N records" + New chat. Replies can carry a PROPOSED CHANGE card (amber border): Add it / Not now / "↶ in the sidebar undoes it". Input with mic and send. No floating Ask bar here |  | Empty: "Ask the \<role>" + two suggestions. Loading: "Reading H2, E1 and your two records…". Error: the budget message from `ai.rs` inside the chat + Open Settings / Copy my question |
| Handoffs | `team.rs` | Left 360: New handoff (For: Claude Code / Codex / A person; Who; What they should do, with mic; Create brief), then Sent and Done tabs with Mark done / Reopen. Right: the brief in five parts (THE JOB, WHAT ALREADY EXISTS, WHY IT MATTERS, OFF LIMITS, DONE WHEN), Copy, Mark sent. Footer: "Current project brief (always up to date) · Copy it" |  | Empty: "Write a brief for someone else" + "No saved handoffs yet." Error: "The AI didn't reply. You have a plain brief made from the project instead." + Try again / Use the plain brief |
| Settings | `settings.rs` | Four tabs. **AI and MCP:** Connect your AI (Claude Code, Ollama, Claude API key, Other; each with status and Test or Set up), MCP setup rows (Claude Desktop, Claude Code, Codex, ChatGPT, each Copy setup), "Other AI apps can read this workspace", "Read only"; Spending card ("$3.40 of $10.00 this month" + bar, Monthly spending limit, price per million input and output, "0 means no limit"). **Sync and phone:** phone access toggle, QR code, address, port 8797, token Copy; Sync folder + last synced; Backups (auto, keep 30, Restore one); Advanced: hub address and token. **Workspaces:** list with Rename / Open, New workspace, "Open Resolve 2 with"; Import; Export ("Every page as Markdown, every database as CSV, all events as .ics, tasks as a Markdown checklist, plus the full workspace as JSON."). **About:** version, privacy line, Report a problem, Open the data folder, Appearance Light/Dark, Keyboard shortcuts |  | Empty: no AI connected. Loading: "Testing Claude Code… this takes up to 20 seconds." Error: "Claude Code didn't reply. It's installed but not signed in. Open a terminal, run claude once and sign in, then press Test again." |
| Help | `help.rs` | Left: guide sections from GUIDE (Welcome, Quick start, Requirements, Home, Projects, Overview, Roadmap, Tasks, Frameworks, Knowledge base, AI Team, Handoffs, Calendar, Tips, Troubleshooting). Welcome is a dark ALPHA card with the existing welcome text and Report a problem. Quick start: five numbered steps | Report a problem dialog (560): "What happened?" with mic, "Also included" box (version, OS, AI, last error; "No names, no project text."), Copy the report | Copied: "Copied. Paste it into a message to Simeon. Thank you." Error: "Resolve couldn't reach the clipboard." + the text to select |

What Frameworks does for the user still needs a plain explanation on the screen. Simeon has parked that for a brainstorm; build the screen as drawn.

## Pack D: the phone page

The phone page is the HTML that `server.rs` serves (`PHONE_HTML`) over home Wi-Fi. Design at 390 × 844, check at 360 × 780, touch targets 44 or more. It uses the same tokens as the desktop; bundle Inter and DM Serif Display into the binary so the page works with no internet. Reference: Pack D.

**Navigation (revised 7 November).** As in the original Resolve: a light bar at the bottom (white, border-top #DDD8CE, padding 6 / 10 / 28 for the home indicator) with three tabs, Tasks, Calendar, Search (icon 22 + label 11.5; active icon amber, active label ink 600). Floating above it, 14 px in from each side and 10 above the bar: the dark Ask pill, 52 high, radius 999: ✦ + "Ask Resolve, or capture a thought", a mic (42) and an amber + (42). Tapping the pill text opens Ask; + opens quick add. Bar and pill together are 150 high; screen content needs 160 px of bottom padding.

| Screen | Layout | Actions | States |
| --- | --- | --- | --- |
| Tasks: Today | Top padding 52 (no fake status bar). Date line, title "Today" (DM Serif 32), workspace tile (44). Segmented control Today / Upcoming / Inbox with counts, 40 high. Dark "YOUR NEXT MOVE" card (Start this, Plan my day). Task rows 64 high: 44 tap target around a 24 circle, title 15.5, project dot + project + due. Done rows at .55 opacity, struck. "Continue setting up a project ›" card when an intake is unfinished | Tap the circle completes, then Undo toast. Swipe left past 35% moves to Tomorrow (amber strip behind). Tap the row opens a task sheet (70% high) | Empty: "Nothing due today." + Add a task. Error (PC answered, request failed): "Your PC answered, but the tasks didn't load." + Try again. Not connected: dark banner "Not connected to your PC. Showing what you saw at 14:02." + Retry, last copy below |
| Tasks: Upcoming | Rows grouped by day ("Tomorrow · Sat 7 Nov") | as Today | "Nothing planned this week." |
| Tasks: Inbox | Captured items with "Captured yesterday"; note "Captured here, sorted on your PC. Tap one to give it a project or a date." | as Today | "Inbox is clear." |
| Quick add | Sheet over the current screen: "Add a task", input with highlighted recognised words + mic, "Resolve understood" chips, button "Add to \<project>" (50 high); sits on the keyboard | Enter or the button saves; lands in its list with a plain toast | Empty: placeholder "Call mum tomorrow 6pm p1 #Home" + hint. Saving. Error: "Your PC didn't accept it. The text is still here. Try once more." Not connected: "Your PC isn't reachable. Saved on this phone. It sends when they're back in touch." + "Keep on this phone" |
| Calendar | "Next 7 days", title Calendar, Today link. Day groups; rows 56+ high: 4 px calendar bar, time (50), title, meta | Tap opens a read-only event sheet | "A clear week." / skeleton / "Your calendar didn't load." / banner |
| Search | Title, 50-high field with clear, type chips (All, Tasks, Events, Pages, Projects, Rows; row scrolls sideways), grouped results | Tap opens the item (desktop-only types open read-only with "Open on your PC") | Empty: Recent searches. None: "Nothing called "VAT return"." + "Add "VAT return" as a task". Error: "Search needs your PC." |
| Ask | Sheet 82% high over the current screen, surface-raised. Question bubble (ink), answer card (white), action buttons ("+ Chase the till export today", "Plan my day"), input 48 + mic |  | Empty: three suggestion buttons. Thinking dots "Looking at today…". Error: budget message. Not connected: "Ask needs your PC. Your question is kept and asks again when it's back." |
| Not connected (nothing cached) | Full dark screen: icon, "Can't reach your PC", "Last in touch today at 14:02.", CHECK THESE: PC on and awake / same Wi-Fi / Resolve 2 open / Phone access is on in Settings, Sync and phone | Try again (amber), "Capture a task anyway" |  |
| Intake | Same steps as Pack A, one column, on pane #201F1C: progress dashes, Save and exit, ⋯. Steps drawn: name, bring it in (template / example / pull-in link), a question (with "Resolve has heard 4 things. Tap to check or edit.", Back, Skip this question, Next, Skip the rest + coverage), next move (Do this first card, reassurance line, Continue, Skip to the whole picture) |  | Loading "Reading what you wrote…". Error: "The AI didn't reply. Carry on with Resolve's own questions." Not connected: "What you've typed is kept on this phone." |

Error and not connected are different. Error: the PC answered but a request failed (401, 5xx). Not connected: a network failure. Use the banner when there is a cached copy; use the full screen only when there is none.

Not on the phone: Roadmap, Frameworks, Knowledge base editing, AI Team, Handoffs, Settings.

## Motion

Build one motion helper first (E16): cubic-bezier easing, keyframe tracks, delay and stagger (delay + index × stagger, in reading order), width and height tweens, origin-aware scale, `ctx.request_repaint()` while anything animates, and the reduced-motion flag. egui 0.29 has none of this built in. Each canvas's page 4 has live loops of the key moments.

| Curve | cubic-bezier | Use |
| --- | --- | --- |
| e-spring | .34, 1.56, .64, 1 | Arrivals, expansions, sheets, panels |
| e-out | .22, .61, .36, 1 | Reveals, restores, settles |
| e-std | .4, 0, .2, 1 | Dims, presses, colour changes |
| e-in | .55, 0, .85, .35 | Exits, collapses |
| ease | .25, .1, .25, 1 | Only where noted |

**Pack A, intake** (ms)

| ID | Moment | Values |
| --- | --- | --- |
| M01.1 | Front door extras | 520 ease, at 420 / 560 / 700, translateY 6 → 0 + fade |
| M01.2 | Field focus | 160 e-std, ring 0 → 3px amber .22, border amber |
| M01.3 | Continue enables | 200 e-std, track → amber (dark) or ink (light) |
| M01.4 | Front door to welcome | out 240 e-in (y − 8); rail in 420 e-spring at 100 (x −308 → 0); pane washes nav-bg → surface 600 e-std at 220 |
| M01.5 | Hi, welcome, paths | One 7000 clock, ease: Hi 0–630; Welcome in 420–910, out 1680–2170; question 1960–2660; paths 2800–3360, 3220–3780, 3640–4200 (y 10 to 14 → 0). Holds at 4200. A click skips to the end |
| M02.1/2 | Step change | out 240 e-in, x 0 → −24; in 400 e-spring at 90, x 28 → 0. Back flips the signs |
| M02.3 | Header settles | 340 e-out, y −8 → 0 |
| M02.4/5 | Stepper | colour 300 e-std at 130; tick pop 300 e-spring, scale 1 → 1.25 → 1 |
| M02.6 | Progress bar | 520 e-out |
| M02.7/8 | Rail collapse / expand | 420 e-spring, width 308 ↔ 72; labels fade first 160 e-std (collapse) or last 220 e-out at 200 (expand) |
| M03.1/2 | Mic | recording pulse 1200 loop (ring 0 → 8px terracotta, fading); press 240 e-std scale .975 |
| M03.3/4 | Template, pull-in panel | 300 e-out, y 8 → 0; panel closes 240 e-in; Copied ✓ for 1.5 s |
| M04.1–4 | Waiting card | in 300 e-out; ring 900 linear loop; checklist tick 150 e-std; 15 s bar linear |
| M04.5 | Question arrives | out 240 e-in (y 8, scale .97); in 460 e-spring (y 18, scale .98) |
| M04.6 | Hand-off note | 300 e-out at 240 |
| M04.7/8 | Heard item | lands 300 e-out, stagger 55, amber tint fades 1.2 s; removed 340 e-in, height → 0 |
| M05.1/2 | Confirm cards, action bar | cards 460 e-spring at 120, stagger 70 (y 18, scale .98); bar 480 e-spring at 460 (y 30) |
| M05.3/4/5 | Edit mode | border → amber 220 e-std, stagger 40; footer swap out 200 e-in −18 / in 380 e-spring +24; card grows 240 e-out (no clip) |
| M05.6/7 | No clear outcome | reason highlight 520 e-out at 300; pass: ✓ 600 e-spring at 100, line 500 ease at 500, route on at 1200 |
| M06.1–7 | Next move, themes | card 450 e-spring (scale .94, y 10); lanes 500 ease at 200 / 350 / 500; yes tick 500 e-spring at 120 (scale 1.6 → 1); row lands 450 e-spring at 200 (x −14); segment 520 e-out; Not quite 400 ease |
| M07.1–5 | First week, whole picture | rows 350 ease at 300, stagger 80; Up next highlight 300 e-std at 800; lanes: headers 200 / 350 / 500, Now 1400, Next 2000, Later 2600, Adopt 3300, in-lane 80; Adopt press 240 e-std; hand to bridge 300 e-in at 160 |
| B0–B4 | Bridge, 9200 clock, once | Glow opacity 0–23% .35, 30% .85, 72% .85, 100% .35. B1 fog card 0–2.85 s (scale .62 → .72 → 1.05 → 1.16, fade in then out). B2 first move pins 2.4–4.8 s. B3 workspace forms 4.6–6.9 s. B4 at 6.6 s: glow fades 600, build checklist fades in and holds until the build ends |
| D1–D4 | Reveal, 6500 clock, once | D1 ✓ 600 e-spring at 100, "Welcome in" 500 ease at 500 and 700. D2 2.0–3.8 s: dark layer 1 → 0 (34–44%), amber bloom scale .4 → 6 → 11 (30–58%), light layer 0 → 1 (36–48%). D3 at 3.25 s: 450 e-spring, stagger 70 (scale .94, y 10). D4 brandmark never moves (x 24, y 19) |
| M10.1–4 | Waiting, error, limit, resume | notes 300 e-out; error row after card exits 240 e-in; limit ring breathes 3200 loop .85 ↔ 1, no spin; resume card 450 e-spring at 200, backdrop 220 e-std |

**Pack B, shell and daily screens**

| ID | Moment | Values |
| --- | --- | --- |
| S1/S2 | Section switch | out 240 e-in (x −24), in 400 e-spring at 90 (x 28); moving up the sidebar flips signs. Active item tint 300 e-std at 130 |
| S3/S4 | Sidebar collapse, tooltip | labels fade 120 e-std, then width 248 ↔ 64 in 320 e-out, work area reflows on the same clock; tooltip after 400 hover, 120 e-out (x −4) |
| S5 | Menus | 300 e-spring, scale .9, y −6, origin at the anchor; rows 220 e-out, stagger 40 |
| A1–A4 | Ask bar and sheet | bar rises 480 e-spring (y 30) at 460 on Home, 760 on Tasks; suggestions 300 e-spring from the bottom centre; sheet: dim 240 e-std, sheet 460 e-spring y 100% → 0 at 40, rows 300 e-out stagger 60; close 300 e-in, dim at 80 |
| H1–H4 | Home | header 340 e-out; cards 460 e-spring at 120, stagger 70 (y 18, scale .98); hero glow breathes 3200 loop; Start this press 240 e-std then route 280 e-in at 160; Waiting on expands: others fade to .5, card 420 e-spring |
| T1–T5 | Tasks | entrance title 340 at 80, bar 640 e-out at 220, sections 460 e-spring; check-off tick 300 e-spring (1 → 1.25 → 1), text 220 e-std at 40, row collapses 340 e-in at 440, bar 520 e-out at 200; editor 420 e-spring from the right, fields 260 e-out at 160 stagger 45, dim 220, close 300 e-in; List ⇄ Board thumb 320 e-spring, out 200 e-in −18, in 380 e-spring +24; filter change 220 + 300 e-out, rows stagger 30, no slide |
| Q1–Q4 | Dialogs, Add task, Search | dim 180 e-std, dialog 300 e-spring at 40 (scale .96, y −6), close 200 e-in; recognised word tint 160 e-std, its chip 220 e-out (y −4); saved task lands 380 e-spring with a "Just captured" chip for 2.6 s; search results swap with no motion |
| C1/C2 | Calendar | view toggle as T4; ‹ › out 240 e-in, in 400 e-spring at 90 (±24 / 28) |
| I1–I4 | Toasts | in 360 e-spring (y 20); Undo timer 4000 linear; out 260 e-in (y 16); Undo returns the row 300 e-spring (x −10); plain toast leaves at 2600; swap 160 e-in |
| Plan my day | Sheet | as A3, 760 wide, 88% high; rows 300 e-out stagger 60 |

**Pack C, project sections**

| ID | Moment | Values |
| --- | --- | --- |
| F2 | Phase ⇄ theme | thumb 320 e-spring; out 200 e-in −18; in 380 e-spring +24. Right rail does not move |
| F3/F4 | Theme expands / collapses | press 200 e-std scale .98; height 360 e-spring; children 260 e-out at 120, stagger 50; collapse 240 e-in. One open at a time |
| F5 | Whole picture | dim 220 e-std; panel 380 e-spring at 40 (scale .96); cells 300 e-out at 200, stagger 40 in reading order; close about 240 e-in |
| F6 | Theme lands in a new phase | 520 e-out slide down; the gap closes on the same clock (subject to confirm) |
| H1 | Knowledge base opens | header 340; tabs and chips 440 at 80; cards 420 e-spring at 300, stagger 55 (y 14, scale .96); grounding bar 480 at 760 |
| H2 | Filter chip | non-matching cards out 200 e-std; library .35 → 1 in 300 e-out |
| H3 | Add sheet | 460 e-spring from below; options 260 e-out at 160, stagger 55; new card 400 e-spring at 120 (scale .9, y −6), amber outline 2 s |
| H4 | Document preview | 420 e-spring x 100% → 0; document 320 e-out at 160; close 300 e-in |
| C1 | Result lands | status pill 300 e-std; one ring pulse 600 e-out; roadmap gate chip 300 e-std after; plain toast "H1 supported. \<Theme> can start." |
| C2–C9 | Other | node editor as T3; tab switch underline 300 e-std + content 200 cross-fade; proposed-change card 360 e-spring (y 8); handoff brief sections 260 e-out, stagger 80; template and CSV dialogs as Q1; new database row 240 e-out + amber flash 600; Saved fades in 160; proposal section jump scroll 420 e-out + flash 600 |

**Pack D, phone** (CSS transitions in the phone page)

| ID | Moment | Values |
| --- | --- | --- |
| P1/P2 | Segment and tab change | thumb 300 e-std; list cross-fades 200 e-std (x ±12); tab change fades 200, no slide |
| P3–P5 | Quick add | + press scale .92; dim 180; sheet 420 e-spring from below; input focuses at 120; words tint 160, chips 220 e-out; save: sheet 240 e-in, row lands 300 e-spring, plain toast |
| P6–P8 | Finish, swipe, undo | circle fills 200 e-std; row stays 600 then leaves 300 e-in; Undo toast 360 e-spring with 4 s timer; swipe commits past 35%, slides 260 e-out, rows close 240; Undo 300 e-spring (x −10) |
| P9 | Ask sheet | dim 220; sheet 460 e-spring to 82%; answer fades up once 260 e-out, then text streams without motion |
| P10/P11 | Lost and found | banner drops 300 e-out and pushes content down; back: turns green 200 e-std, "Back in touch. 1 capture sent.", leaves 300 e-in after 2 s |
| P12–P14 | Other | pull to refresh: ring fills with the pull, 300 e-spring back; intake slides use 60% of desktop distances; task sheet 420 e-spring to 70%, close 260 e-in |

**Reduced motion** everywhere: every move becomes a fade of 120 ms or less, loops hold still, sheets and panels appear in place, the bridge skips to B4, the reveal cross-fades in 120.

## Engineering tickets

Every change the designs need, with what today's build does. "Unverified" means the designer could not confirm it in the code; check before building on it. E18 sits with the visual layer (`theme.rs`, `widgets.rs`); the rest touch view structure or logic.

**Pack A, intake**

| # | Ticket | Today | Where |
| --- | --- | --- | --- |
| E1 | Welcome with three paths; new idea and stuck disabled with Coming soon | Hard-codes "messy\_project", skips the choice | 02a |
| E2 | Voice on every multi-line field: mic, recording state, speech to text | No voice | 02b, 03, 04b, 05, 07 |
| E3 | No clear outcome screen with the floor reason and one question; the answer re-runs the check | v1 has a one-time re-ask (`apply_verdict`); v2 journey has no halt screen | 05 |
| E4 | Stepper rail, 4 steps, collapsing to 72 on wide steps | Top progress bar only | All |
| E5 | Waiting card: checklist driven by real progress events, seconds counter, 15 s bar, a change at least every 5 s | One progress line with a counter | ST1 |
| E6 | 15 s hand-off from `TARGETS`, "Why this question", late answer dropped | Note in the waiting text | ST2 |
| E7 | Long-wait note after 15 s with Leave and come back later | Counter runs on, no note | ST3 |
| E8 | Error state per step: Try again (focused) plus the manual path | Try again at the ending only; raw error strings elsewhere | ST4 |
| E9 | Resume card on launch mid-intake | Saves the stage; no welcome back | ST5 |
| E10 | AI limit screen: catch the budget error and provider caps, freeze the checklist, Continue without AI | Budget message exists in `ai.rs` | 12 |
| E11 | Limit checklist strings match the waiting checklist exactly | New strings | 12, ST3 |
| E12 | Themes screen: On your roadmap list, segment bar, card | Text and two buttons | 07 |
| E13 | Now / Next / Later strip on Next move | Do this first and Continue only | 06 |
| E14 | Bridge beats B0 to B4 bound to live build progress, minimum 7.2 s | Static "Building your project…" | 10 |
| E15 | Reveal D1 to D4 | Repeats the whole picture | 11 |
| E16 | Motion helper (see Motion) | None | All |
| E17 | Glow and bloom as textures or gradient mesh | None | 01, 06, 10, 11 |
| E18 | Tokens: pane, terracotta-on-dark, text-on-dark set, shadows; resize HERO, QUESTION, ANSWER; fix amber and green text on light | As in `theme.rs` today | All |
| E19 | A click skips long reveal sequences | None | 02a, 09 |
| E20 | 1920 rules: column 960 centred, heard panel 440 | Column 680 left-aligned | ST6 |
| E21 | Use a template (8 headings) and Try an example | Two scaffolds in SITUATIONS | 02b |
| E22 | New 11-question pull-in prompt; panel with Copy and Copied ✓ | Short summary prompt | 02c |
| E23 | Heard panel: label menu, pencil and × on hover, inline edit (Enter, Esc), first-time hint | Click a label to cycle it | 03 |
| E24 | Back · Skip this question · Next, and Skip the rest with the coverage bar | Back · Next · Skip + "Show me the proposal" | 03 |
| E25 | Save and exit + ⋯ menu with Restart; Home card | Exit and Restart side by side | All |
| E26 | Skip to the whole picture on 06 to 08; Edit the full proposal on 09 only; reassurance line | "See and edit everything" on every ending screen | 06 to 09 |
| E27 | Self-test: the first move equals task 1 of week 1 and sits in the first Now theme | Not tested | 06, 08 |
| E28 | Validate screen 12: map provider rate limits and the Claude Code usage cap, force the budget stop, screenshot | Only the budget stop is known | 12 |
| E29 | AI setup screen between 02a and 02b when no AI is connected (parked, not designed) | Settings only | none |

**Pack B, shell and daily screens**

| # | Ticket | Today | Engine |
| --- | --- | --- | --- |
| B-E1 | Sidebar collapse to 64 with tooltips, remembered per workspace, Ctrl \\ | Fixed width | internal |
| B-E2 | "Continue setting up…" card in the sidebar footer and on Home | Unfinished intakes listed under the groups | internal |
| B-E3 | Floating Ask Resolve bar on every screen, suggestions, sheet (A3); keep the Ask AI button | Ask AI right panel | internal |
| B-E4 | Home: hero from the project's first move; Since Monday; Waiting on; Projects; Captured | Stat tiles and blocks | `workspace_overview`, `list_tasks`, `list_projects`, `get_agenda` |
| B-E5 | Confirm tasks can be marked waiting; if not, add the state | Unverified | `update_task` |
| B-E6 | A failed suggestion shows its error; the rest of Home loads | Unverified | none |
| B-E7 | Project cards: outcome, next move, phase counts, open questions, ⋯ menu | Name, outcome, counts, progress | `list_projects`, `update_project` |
| B-E8 | Tasks list sections Now / Waiting on / Upcoming from roadmap; chips; View completed | One Now section per project | `list_tasks` |
| B-E9 | Task editor as a 440 push panel | Unverified layout | `update_task`, `complete_task`, `delete_item` |
| B-E10 | One toast at a time, fast swap, 4 s Undo timer, 2.6 s plain | Toasts stack | internal |
| B-E11 | Add task: highlight recognised words, chips without emoji, Goes to picker; decide what #Name does when no project has that name | Chips with emoji | `create_task` (`nlp.rs`) |
| B-E12 | Calendar Agenda view; calendars list with toggles and New calendar | Month, Week, Day | `get_agenda`, `list_calendars`, `create_calendar` |
| B-E13 | Confirm all-day and repeats in events | Unverified | `create_event`, `update_event` |
| B-E14 | Search palette over the current screen | Search screen | `search` (prefixes exist) |
| B-E15 | "Add as a task" from a search with no results | None | `create_task` |
| B-E16 | Voice on the Ask bar, Add task, task notes | None | internal |
| B-E17 | Undo and Redo show a toast naming what changed | Toasts for AI changes only | `history.rs` |
| B-E18 | Width tween and origin-aware scale in the motion helper | None | internal |
| B-E19 | 1920 caps per screen; Projects in three columns | Fluid, uncapped | internal |
| B-E20 | Hero glow and shadows as textures | None | internal |
| B-E21 | Plan my day sheet: ask the AI for a structured reply (time, item, reason, won't fit) and draw rows; fallback lists today by time | Saved prompt in `mcp.rs`, reply as chat text | `get_agenda`, `ai.rs` |
| B-E22 | Use this plan writes a start time to each task. Confirm tasks hold a time | Unverified (`nlp.rs` parses times) | `update_task` |
| B-E23 | Change box re-plans with the typed constraint; Move to Monday sets the due date | None | `update_task` |
| B-E24 | Rename the middle sidebar group to YOUR PROJECT and Knowledge to Knowledge base | "Product management", "Knowledge" | `nav.rs` |

**Pack C, project sections**

| # | Ticket | Today | Engine |
| --- | --- | --- | --- |
| C-E1 | Overview layout (hero, progress, what matters, Resolve suggests, context, mini board) | Do this next, progress, what matters, context | `resolve_project`, `list_records`, `list_tasks` |
| C-E2 | Weekly review button in the Overview header | Unverified placement | `weekly_review` |
| C-E3 | A failed suggestion on Overview shows its error; the rest loads | Unverified | none |
| C-E4 | Roadmap theme cards with tag, gate chip, progress; expand in place | Step cards with Next/Later buttons | `list_records`, `update_record`, `add_record` |
| C-E5 | Gates: link a theme to a hypothesis; open on Supported. Confirm records can hold the link | Unverified | `update_record` |
| C-E6 | Whole picture overlay | None | `list_records` |
| C-E7 | Phase move with F6; a failed move rolls back and says so | Unverified | `update_record` |
| C-E8 | Right rail: Milestones, Dependencies and blockers, Coming up. Confirm milestones exist as records | Present, data unverified | `list_records`, `list_tasks` |
| C-E9 | Frameworks: registry strip, tree with negations, experiments with prediction and threshold, node editor | Tree, no editor panel | `list_records`, `update_record`, `add_record` |
| C-E10 | Result → status → gate → toast, in that order | None | `update_record` |
| C-E11 | Proposal editor: section list, inline edits, Back keeps edits, Adopt in the top bar | Editor exists; layout differs | `update_project`, `add_record`, `create_page` |
| C-E12 | Knowledge base: five tabs, filter chips with counts, card library, grounding bar | Tabs and filters exist | `list_pages`, `list_records` |
| C-E13 | Add sheet with five options; new item lands first, outlined 2 s | New page, Add files | `create_page`, `import_pages`, `add_record` |
| C-E14 | Document preview panel | None | `read_page`, `update_page` |
| C-E15 | Per-file import errors while the rest files normally | Unverified | `import_pages` |
| C-E16 | Page editor toolbar, / menu, Saved state, to-dos count, Linked chips | Editor exists | `update_page`, `list_page_todos` |
| C-E17 | New page dialog: built-in templates plus your own | Menu list | `create_page` (`templates.rs`) |
| C-E18 | Databases list, table, group and sort; CSV import dialog; failed rows listed | Tables exist; import UI unverified | `list_databases`, `create_database`, `query_database`, `add_database_column`, `add_database_row`, `update_database_row`, `import_csv` |
| C-E19 | AI Team role cards, earlier chats, proposed-change card, budget error in the chat | Role chips and chat | `resolve_project`, `add_record`, `update_record` |
| C-E20 | Handoffs: For + Who, five-part brief, Sent and Done, Reopen, plain-brief fallback | For, What, Create brief, current brief | `resolve_project`, `list_records`, `add_record`, `update_record` |
| C-E21 | Settings in four tabs; spending card. Confirm month-to-date cost is tracked | Tabs AI, General, Your data, Advanced; price and budget fields exist | AppSettings |
| C-E22 | Phone pairing QR code (address + token) | Port and token fields | WorkspaceSettings |
| C-E23 | Help: guide list, Welcome card, Report dialog with included details, clipboard fallback | Guide and Report exist | `help.rs` |
| C-E24 | Voice on Handoffs, AI Team, Report | None | internal |
| C-E25 | 1920 caps per screen | Fluid | internal |
| C-E26 | Height spring in the motion helper (F3, add sheet) | None | internal |

**Pack D, phone**

| # | Ticket | Today | Engine |
| --- | --- | --- | --- |
| D-E1 | Serve the intake to the phone | Not served | journey calls need a phone route |
| D-E2 | Restyle with Resolve tokens; bundle the fonts | Dark page, red accent, system font | `server.rs` PHONE\_HTML |
| D-E3 | Light bottom bar (Tasks, Calendar, Search) + Ask pill; Today / Upcoming / Inbox segments | Five top tabs, add box at the bottom | `list_tasks` |
| D-E4 | Your next move card on Today | None | `workspace_overview`; Plan my day per B-E21 |
| D-E5 | Quick add with live highlighting; needs a parse-only call | Plain input | `create_task` (`nlp.rs`) + a new parse preview |
| D-E6 | Offline: cache lists with a time, queue captures, send on reconnect, banner states | Error text only | internal |
| D-E7 | Ask on the phone. Confirm a chat call is reachable over `/mcp`; if not, add one with the budget check | Unverified | `workspace_overview`, `get_agenda`, `ai.rs` |
| D-E8 | Swipe left to Tomorrow | None | `update_task` |
| D-E9 | Motion as CSS transitions; honour reduced motion | None | internal |
| D-E10 | Undo after finishing. Confirm a completed task can be reopened | Unverified | `complete_task`, `update_task` |
| D-E11 | Search chips, grouped results, Add as a task | Flat list | `search`, `read_page`, `create_task` |
| D-E12 | Agenda: dated tasks beside events, calendar colours | Events by day | `get_agenda`, `list_events` |
| D-E13 | Tell error from not connected | One message | internal |
| D-E14 | Voice through the browser's speech input; check testers' phones | None | internal |
| D-E15 | Pairing by QR (with C-E22) | Link with token | WorkspaceSettings |
| D-E16 | Nothing scrolls sideways at 360 except the chip row | Unverified | internal |

## Open decisions and unknowns

Simeon reviewed all four packs on 7 November and approved the screens, with the changes already written into this brief. These items are still open. Build with the current pick and keep each one easy to change.

| Item | Current pick | Who settles it |
| --- | --- | --- |
| Name of the middle sidebar group | YOUR PROJECT | Simeon |
| What Frameworks does for the user, in one plain line on the screen | Parked for a brainstorm | Simeon |
| Intake reveal content (Pack A D4) | Project name, three counts, the first move, Open my project | Simeon |
| 60 s proposal (D5) | Long-wait note now; speed fix later | Builder: measure, then propose |
| "Daily limit" copy means the monthly AI budget (D7) | Rewritten around `check_budget` | Builder: confirm with E28 |
| What #Name does in Add task when no project has that name (B-E11) | Undecided | Simeon |
| F6 motion subject | A theme landing in a new phase | Simeon |
| Phone: Ask tab and swipe to Tomorrow (Pack D D3, D5) | Built as drawn; phone is low priority | Simeon |

Unknowns the builder must check in the code before building on them: tasks can be marked waiting (B-E5); events support all-day and repeats (B-E13); tasks hold a start time (B-E22); records can link a theme to a hypothesis (C-E5); milestones exist as records (C-E8); month-to-date AI cost is tracked (C-E21); a completed task can be reopened (D-E10); a chat call is reachable from the phone (D-E7). Report each answer to Simeon before working around it.

## Done when

A phase is done when every item below holds for each screen it ships.

- [ ] `target\debug\resolve2.exe --selftest` ends "N passed, 0 failed", including the new E27 check.
- [ ] A screenshot of each view in full, empty, loading and error (`--view <name> --screenshot <path>`) matches its canvas frame in layout, copy and colour.
- [ ] The same views at 1920 × 1080 keep the content caps in Rules, item 6.
- [ ] No colour or size literal appears outside `theme.rs` and `widgets.rs`.
- [ ] Every motion uses the helper with the values in Motion, and the reduced-motion flag turns each into a fade of 120 ms or less.
- [ ] Every complete, delete, move and AI write can be undone from a toast.
- [ ] Text contrast passes the pairs in Tokens; no amber text on light.
- [ ] Phone: every screen works at 360 × 780 with nothing scrolling sideways except the chip row, and every target is 44 px or more.
- [ ] Simeon has seen the screenshots for the phase.
