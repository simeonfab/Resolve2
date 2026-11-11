# Changelog

Every build of Resolve 2 has a version number. The minor number goes up for each merged redesign phase or Careers step, the patch number for each fix. You can see the version in the window title. Newest first.

## 0.8.1 (11 November 2026)

Careers Step 2, continued (branch `careers/s2-intake`). Waiting for your review in the preview, not merged yet.

**What changed**
- PDFs read properly: CVs, LinkedIn exports and documents saved from Google Docs come in as clean text. If a PDF still can't be read, Resolve says "Resolve couldn't read this PDF. Paste the text instead." and gives you a box to paste it into.
- Resolve 2 Preview: a separate app on your desktop for trying a branch before it's merged. It has its own copy of your data plus the café sample, never touches your real workspaces, and its sidebar says "Preview" with the branch and version.

## 0.8.0 (10 November 2026)

Careers Step 2: the Careers intake (branch `careers/s2-intake`). Waiting for review, not merged yet.

**What changed**
- "What do you need help with?" has a fourth path, "I'm looking for a job", marked New. Choosing it runs the Careers intake in the same frame, with "Careers" on the top bar.
- Tell Resolve what job you're looking for: eight headings with Use a template, Try an example, and a prompt to bring a job search in from another AI chat. Attach your CVs and LinkedIn.
- Up to 8 questions that build your gates. Anything your notes already answer is skipped, and the screen says how many. A broad answer gets one push: "That covers a lot. Which one first?" What Resolve heard is labelled Hard gate, Preference or Question.
- Here's what Resolve understood: the job you want, your hard gates, your preferences, what a win looks like, and your CV and LinkedIn (strengths, gaps, what to fix). Edit changes any card. Rebuild my CV opens your CV in the full proposal; the rebuilt CV saves into My CV.
- If your target still covers many jobs, one question first: what job do you have in three months?
- Your kit: do this first, your gates one at a time (keep it, not quite, or make it a preference), five tasks for your first week, then the kit itself with its search name. The full proposal edits the target, gates, CV templates, cover letter, evidence bank, LinkedIn fixes and first week.
- Adopt creates a new workspace in Careers mode, named Job search unless you change it, with your gates, CV templates, cover letter template, evidence bank, LinkedIn fixes, weekly target and first tasks. Open my job search lands on Home. Inside a Careers workspace, + New project starts a second search with its own gates.
- With no AI connected, a fixed set of Careers questions runs, and the cards and kit use only your own words.
- Build checks now take their picture of the café sample's Home, in a sandbox, instead of the first-run screen.

**Known differences and open items**
- A PDF exported by most modern tools didn't read cleanly. Fixed in 0.8.1.

## 0.7.1 (10 November 2026)

A fix to the sandbox rule from 0.7.0 (branch `careers/s1-shot-sandbox`).

**What was fixed**
- A screenshot started without its own data folder used to be refused. Craft Anything's build check starts it that way, so the check failed. Now such a screenshot gets a fresh sandbox folder of its own, and still never opens your real data. A screenshot pointed at your real data folder is still refused.

## 0.7.0 (10 November 2026)

Careers Step 1: modes, the foundation for Resolve Careers (branch `careers/s1-modes`). Nothing you can see changes yet.

**What changed**
- Every workspace now has a mode: PM (the app you use today) or Careers. There are two modes only. A workspace's mode is set when it is made and never changes.
- Every workspace you already have opens as PM, exactly as before. Its file isn't changed: a PM workspace doesn't even write its mode.
- The sidebar, the Ask Resolve wording, Frameworks, the Knowledge base tabs, the Help guide, the intake's questions and steps, and the AI Team now come from tables picked by the open workspace's mode. PM's tables are the ones you have today. Careers has its own, taken from the Careers brief.
- My CV is the one home for your master CV, CV templates, the cover letter template and LinkedIn fixes, so Careers Frameworks no longer lists CV templates or the cover letter. The Knowledge base's CVs and cover letters tab holds the copies you sent with each application.
- There are no Careers screens yet, and no way to make a Careers workspace from the app yet. Those come in the next steps.
- Screenshots and the self-test now run only in a sandbox folder. A screenshot pointed at your real data folder is refused, and so is a self-test outside its own folder. Nothing a test runs can open your workspaces.

## 0.6.0 (10 November 2026)

Redesign Phase 5: Settings and Help, rebuilt to the Pack C canvases (branch `nav/p5-settings-help`).

**What changed**
- Settings has four tabs: AI and MCP, Sync and phone, Workspaces, About (C-E21).
- AI and MCP: Connect your AI as four choices (Claude Code, Ollama, Claude API key, Other), each with its status (Connected, Testing…, No reply, Not set up) and Test or Set up. A test that gets no reply says why under the choices. Copy setup for Claude Desktop, Claude Code, Codex and ChatGPT, and the two switches for other AI apps.
- The Spending card is labelled Estimated: this month's cost from the tokens Resolve logged and the prices you enter, against your monthly limit. Only API keys count; Claude Code and Ollama show $0.
- Sync and phone: the phone link as text, with Copy for the token and the whole link (no QR code); the sync folder with when it last synced; backups with Restore one; the hub under Advanced.
- Workspaces: every workspace with how many projects it has, Rename and Open, and Open Resolve 2 with (the last workspace you used, or one you pick); Import and Export.
- About: the version, what stays on this PC, Report a problem, Open the data folder, Light or Dark, keyboard shortcuts, and What's new: this changelog, word for word.
- Help: the guide list on the left, the dark Welcome card with Report a problem, and Quick start in five steps (C-E23).
- Report a problem is a dialog: what happened (with the mic, C-E24), what's also included (version, Windows, the AI, the last error; no names, no project text), and Copy the report. It says Copied when the copy worked, and shows the text to copy yourself when the clipboard can't be reached.
- The line under each theme on Does this belong on your roadmap? says why the theme belongs on your roadmap, in one sentence, instead of describing the work. One real run on Claude Code gave a reason for all six themes.

**What was fixed**
- The monthly budget no longer counts Ollama's tokens as a cost (confirmed: only pay-per-use API keys count).
- The MCP setup names on AI and MCP stay on one line (Codex was cut off at 1280).

**Known differences and open items**
- Mobile is out of scope: the phone page and pairing stay as they are.
- Things the canvas doesn't draw are folded under each tab: More AI options, Test Intake, Recently deleted, Calendar subscriptions, the hub, Where your files are and Opening Resolve 2.
- Unverified: the mic on Report (Windows voice typing can't be shown in a screenshot).

## 0.5.1 (10 November 2026)

Phase 3 fixes that didn't make the 0.4.0 merge (branch `intake/p3-fixes`).

**What was fixed**
- The AI limit screen (12) quotes "It said:" only for your AI provider's own message, never for Resolve's own budget stop, which the sentence above already explains.
- Every theme card on Does this belong on your roadmap? (07) has its line: the AI is asked for what each theme involves and why it matters now, and a first answer with a theme missing it is sent back once. In one real run (Ollama, llama3.2) all three themes had their line.
- The bridge (10) says your project's name, for example "Café flat-fee bookkeeping, turned into a focused path forward", instead of "A messy project".
- The halo round Open my project on the reveal (11) is the canvas's soft 6 px ring, not a bright amber one.

**Known differences and open items**
- If the AI's second answer still leaves a theme without its line, the card shows none; Resolve doesn't make one up.

## 0.5.0 (10 November 2026)

Redesign Phase 4: project sections, rebuilt to the Pack C canvases (branch `nav/p4-project`). Each section below is the redesigned page when a project is picked; with no project picked, the earlier all-projects view stays.

**What changed**
- You can see the version number in the window title, for example "Resolve 2 0.5.0", and this changelog has Phases 0 to 3 filled in from their reports.
- Overview: your next move, what it tests and the gate it opens (only when the data links them), progress, What matters now, Resolve suggests, Project context and a small Tasks board. Weekly review sits in the header. A suggestion that fails shows its error in the next-move card while the rest of the page loads (C-E1, C-E2, C-E3).
- Roadmap: Now, Next and Later lanes of theme cards with their tag, gate chip and progress, opening in place; By theme; the Whole picture of hypotheses by phase; and a rail with milestones, what is held up and what's coming up. Moving a theme, changing its gate, order or status now has Undo, and a move that can't be made changes nothing and says so (C-E4 to C-E8, C-E26).
- Frameworks: Hypothesis Zero in a registry strip, the tree with each negation and what each experiment holds on, and an editor beside it. A result updates the experiment, then the hypothesis, then the roadmap gate, then tells you, as one change you can undo (C-E9, C-E10).
- The proposal editor ("See and edit everything") has its own window: Back to the summary keeps your edits, Adopt sits in the top bar, a list of sections on the left, and text you edit where it stands (C-E11).
- Knowledge base: five tabs, a card library, filter chips with counts, a line saying what Resolve searches, an Add sheet with five ways to add (a new item lands first, outlined, for 2 s), a preview panel, and each file that couldn't be read named with why (C-E12 to C-E15).
- Pages: a breadcrumb, a toolbar (H1, H2, bold, italic, lists, to-do), Saved / Not saved, the To-dos and Linked cards beside the text, and a New page dialog with the built-in templates and your own (C-E16, C-E17).
- Databases: the list of databases beside the table, + Row, + Column, Import CSV, Group and Sort. The CSV import dialog shows each column's type to check, and lists any rows that didn't import, with why, so you can bring them in as text (C-E18).
- AI Team: role cards, earlier chats, suggestions, and a Proposed change card you accept or not; reaching your AI budget shows in the chat with Open Settings and Copy my question (C-E19).
- No AI changes your workspace from a chat any more, whatever the provider. Every role, and Ask AI, proposes; you add each change with a click, and Undo stays. Ask AI's "Let AI make changes" box is gone. Weekly review proposes a page; Continue writing and Improve writing put their text in the reply; Turn into tasks and Organise my to-dos propose tasks.
- What matters now on Overview says "E0 hasn't started yet: ..." where it said "Run: ...".
- Removing an item from the cut list can be undone.
- Handoffs: For and Who, a brief in five parts (The job, What already exists, Why it matters, Off limits, Done when), Sent and Done with Mark done and Reopen, and a plain brief when the AI doesn't reply (C-E20).
- A mic on Handoffs and AI Team uses Windows voice typing (C-E24). Each page keeps its width on a wide screen (C-E25).

**What was fixed**
- Roadmap moves, gates, order and status had no Undo; they do now.
- The AI's `import_pages` tool now tells you when a page's parent couldn't be found, instead of dropping it silently.
- Error cards said "Try again" even where the action was "Save again" or "Choose another file".

**Known differences and open items**
- A CSV row fails when it has more cells than columns, or a date or number that can't be read (accepted as built).
- Organise my to-dos no longer links the new tasks back to the to-dos in your pages, because the AI only proposes now.
- Edits to cells and rows inside a database still have no Undo; that waits for the polish pass.
- The table, filter and search controls inside a database are the earlier ones; only the frame and toolbar are new.
- Frameworks' one-line description is parked for a brainstorm; its slot stays empty.
- The .98 and .97 scale effects (press and lift) are left for the polish pass.
- Unverified: the mic with Windows voice typing on Handoffs and AI Team (it can't be tested in a screenshot); a real AI writing a Proposed change line.

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
