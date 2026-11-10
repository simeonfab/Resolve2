# Changelog

Every build of Resolve 2 has a version number. The minor number goes up for each merged redesign phase or Careers step, the patch number for each fix. You can see the version in the window title. Newest first.

## 0.5.0 (in progress, not merged yet)

Redesign Phase 4: project sections (branch `nav/p4-project`).

**What changed**
- You can see the version number in the window title, for example "Resolve 2 0.5.0".
- This changelog, with Phases 0 to 3 filled in from their reports.

**Known differences and open items**
- The rest of Phase 4 is still being built; this entry grows as it lands.

## 0.4.0 (10 November 2026)

Redesign Phase 3: the intake, front door to reveal, rebuilt to the Pack A canvas.

**What changed**
- You get a dark frame with Save and exit, a menu with "Start this intake again (keeps your name)", and a rail of four steps that narrows on Questions and the whole picture (E4, E20, E25).
- You choose a path on Welcome. "I have a messy project" is open; the other two say Coming soon (E1, E19).
- On Bring it in you can use a template with eight headings, try an example, or copy an 11-question prompt to pull in another AI chat (E21, E22).
- On Questions you get Back, Skip this question and Next, a coverage bar with Skip the rest, and a heard panel where you change a label from its menu, edit a line in place or remove it (E23, E24).
- While you wait, a checklist ticks from the work actually done, with a seconds counter and a 15 s bar. After 15 s the next question comes from Resolve's own list and says so. The proposal shows a long-wait note after 15 s with Leave and come back later (E5, E6, E7).
- Errors say what failed and that nothing you wrote is lost, with Try again and a way to carry on by hand (E8). Coming back to an unfinished intake shows a welcome back card (E9).
- If your outcome is missing, a screen asks one question before Resolve builds anything (E3).
- If you reach an AI limit (your monthly budget, a provider's rate limit, or the Claude Code usage cap), you see a limit screen with Continue without AI. Its checklist freezes where the work stopped (E10, E11, E28).
- Next move shows Now, Next and Later; themes come one card at a time with an On your roadmap list; then your first week and the whole picture, with Adopt and Edit the full proposal (E12, E13, E26).
- The bridge follows the real build of your project for at least 7.2 s, then the reveal shows your project with three counts and your first move (E14, E15).
- The first move Adopt pins is always task 1 of your first week (E27, self-tested).
- Pages slide when you change step, the confirm cards come in one after another, heard items land with an amber tint, and a new question arrives as the waiting card leaves. With reduced motion each is a short fade (M02.1/2, M05.1/2, M04.7, M04.5).
- The question on Questions is DM Serif 29, as the canvas draws it.
- Settings, AI has "Faster proposal" (off). Side by side runs showed it was not faster, so it stays off.

**What was fixed**
- Plan my day and the intake no longer crash on a short word with an accent, such as "café".
- API rate limit and quota errors, and the Claude Code usage cap, are recognised as limits instead of being reported as a broken answer. The Claude Code cap was seen live and recognised.
- If the AI fails to write your understanding, you now see the error instead of cards quietly made from your own words.
- Em dashes written by the AI no longer reach your proposal.

**Known differences**
- The small scale changes (.98 and .97) in the question arriving and the confirm cards are not drawn; they are left for the polish pass.
- The full proposal editor keeps its old look until Phase 4.
- An API 429 and the Codex cap have not been seen live.

## 0.3.0 (8 November 2026)

Redesign Phase 2: daily screens, to the Pack B canvas.

**What changed**
- Home shows a greeting, your highest-value move, Today, Since Monday, Waiting on, Projects, Captured and Continue setting up; a failed suggestion shows its error in its own card (B-E4, B-E6).
- Projects shows cards with outcome, next move, phase counts and open questions, three across at 1920 (B-E7, B-E19).
- Tasks has Now, Waiting on and Upcoming from your roadmap, a Board, an All projects view and a 440 editor panel (B-E8, B-E9).
- Tasks can be marked waiting, with the days since; Home's Waiting on lists them (B-E5).
- Add task highlights what it recognises, offers "New project: Name" or "Did you mean …?" for a #Name, and never makes a project quietly (B-E11).
- Plan my day asks your chosen AI for a plan, lists what won't fit with Move to Monday, and Use this plan sets only a planned start time (B-E21, B-E22, B-E23).
- Calendar has an Agenda view, calendars with toggles, New calendar and a new event editor (B-E12).
- Search with no results offers to add your words as a task; voice is on the Ask bar, Add task and task notes (B-E15, B-E16).

**What was fixed**
- Plan my day used the local model whatever AI you chose; it now uses your choice.
- Claude Code calls failed when a local model name was left in settings.
- Deleting a task showed a plain toast that replaced its Undo toast.

**Known differences**
- The calendar's month and week grids keep their old look (handed to Claude Code for polish).
- Counts on the sample screens differ from the canvas where the canvas's sample doesn't name every task.

## 0.2.0 (7 November 2026)

Redesign Phase 1: the shell.

**What changed**
- The sidebar group is YOUR PROJECT and Knowledge is Knowledge base (B-E24).
- You can collapse the sidebar to icons with « or Ctrl+\, and it remembers per workspace (B-E1).
- A "Continue setting up…" card shows in the sidebar while an intake is unfinished (B-E2).
- The Ask Resolve bar is on every screen except AI Team, with suggestions per screen, and opens the Ask sheet (B-E3).
- Undo and Redo show a toast naming what changed (B-E17).
- The search palette opens over the screen you're on (B-E14).
- The mic opens Windows voice typing on its own field.

**What was fixed**
- Em dashes were removed from every screen, and a self-check keeps them out.
- Two task moves skipped Undo; they now go through history with an Undo toast.
- Screenshot runs with sample data could have written over your open workspace; they no longer write.

**Known differences**
- The search palette's old "Jump to" list and the sidebar's light and dark toggle are gone; both still exist elsewhere.

## 0.1.0 (7 November 2026)

Redesign Phase 0: foundations.

**What changed**
- One tokens file for colours, type, spacing, radii, shadows and the contrast fixes (E18).
- Shared components: cards, buttons, pills, chips, fields, skeletons, empty and error blocks.
- One motion helper with the brief's curves and an app-wide Reduce motion setting (E16, B-E18).
- Glow and bloom for the hero cards and the reveal (E17, B-E20).
- One toast at a time, with a 4 s Undo timer (B-E10).
- The mic button and its recording state (E2).
- A component gallery: `--view gallery`.

**Known differences**
- Older screens kept their own sizes until each was rebuilt in later phases.
