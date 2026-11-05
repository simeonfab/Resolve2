# Resolve 2 — a quick tutorial

Resolve 2 keeps your pages, databases, tasks and calendars in one app on your PC. Everything is saved automatically as you type, works offline, and belongs to you. This guide takes about 10 minutes to read.

> 💡 Three keys to remember: **Ctrl+Q** adds a task, **Ctrl+K** searches everything, **Ctrl+N** makes a new page.

## 0. Opening Resolve 2 and where things are

- **The first time**: open it from Craft Anything (Play), or double-click **Open Resolve 2.cmd** in the project's `app` folder (`Documents\Craft Anything\build-me-a-single-productivity\app`).
- **Then make it a normal app**: Settings → General → **Put Resolve 2 on my desktop and Start menu**. After that, double-click the Resolve 2 icon or press Windows and type *Resolve 2*. You can pin it to the taskbar.
- **The program itself**: `...\build-me-a-single-productivity\app\target\debug\resolve2.exe`.
- **Your data**: `%APPDATA%\Resolve2` (paste that into File Explorer's address bar), with one folder per workspace holding `data.json`, settings and `backups`. Settings → General → *Open folder* takes you there.
- **This tutorial** is in the app under **Help & tutorial**, and as `TUTORIAL.md` in the `app` folder. **FOR_AI_ASSISTANTS.md** next to it is a full description of the app to paste into ChatGPT or Claude when you want their advice (for example, which local AI model suits your PC).

## 1. Find your way around

The left sidebar is your map:

- **Workspace name** (top left): switch workspaces or make a new one. Each workspace is completely private: its own files, backups, sync and AI access.
- **Add task**: the red button, the same as Ctrl+Q.
- **Search**: finds pages, tasks, events, projects and database rows.
- **Home**: your dashboard, with today's tasks, today's schedule, the next 7 days, recent pages and project progress.
- **Inbox, Today, Upcoming, Filters & Labels**: your task lists.
- **Calendar**: day, week and month views.
- **My Projects, Favorites, Pages, Databases**: everything you've made. Each **+** button opens a menu, including templates.
- **Settings, sync & AI** (bottom): import, backups, phone, AI connections. The line under it tells you when your work was last saved and synced.
- The **☀ / 🌙** button at the top switches light and dark mode.

You start in **"Sample (demo data)"**, a practice workspace full of examples. Your real workspace is **"Personal"**: switch to it from the workspace menu or the banner on Home. Nothing you do in Sample touches Personal.

## 2. Tasks

### Adding tasks quickly

Press **Ctrl+Q** (or click Add task) and type naturally. Resolve 2 picks out the details and shows them as coloured chips before you save:

- Dates: `today`, `tomorrow`, `fri`, `next week`, `in 3 days`, `12/11`, `5 jan`
- Times: `9am`, `14:30`, `at 6pm`
- Priority: `p1` (urgent, red) to `p4` (none)
- Project: `#Home` (created if it doesn't exist), or a section with `#Work/Planning`
- Labels: `@phone`, `@errands`
- Repeats: `every day`, `every weekday`, `every week`, `every 2 weeks`, `every month`, `every mon`
- Reminders: `remind 30m`, `remind 1h`
- Length on the calendar: `for 90m`

Example: `Pay rent every month 9am p1 #Home remind 1h`

Press **Enter** to add. The box stays open so you can capture several in a row; press **Esc** to close it. You can also type in the **Quick capture** bar on Home, or use **+ Add task** at the bottom of any list.

### Working with tasks

- **Tick the circle** to complete a task. A repeating task jumps to its next date instead of disappearing.
- **Click a task** to open its details panel on the right. There you can change the title, description, project, section, date, time, duration, repeat, priority, reminder and labels, and link it to a page.
- **Sub-tasks**: in the details panel, type in "Add sub-task" and press Enter. A sub-task follows its parent if you move it to another project.
- **Move a task** to another project with the Project menu in its panel.
- **Delete** with the 🗑 in the panel.

### Your task lists

- **Inbox**: tasks without a project.
- **Today**: overdue and today's tasks. "Reschedule" moves all overdue tasks to today.
- **Upcoming**: the next three weeks, day by day, with that day's events shown as chips. "+ Add task" under a day gives the task that date.
- **Filters & Labels**: ready-made filters (Priority 1, Overdue, No date, Repeating, Linked to a page, Completed) plus one for each @label you've used.

### Projects

- **+** next to My Projects makes a project; click its title to rename it.
- **Add section** groups a project's tasks (for example Ideas, Doing, Done).
- **Display → Board** shows sections as columns. Drag cards between them.
- **Display → Show completed** brings finished tasks back into view.
- **📄 Create project doc** makes a page for the project (brief, notes, links) and keeps it attached.
- **⋯** changes the colour, or archives or deletes the project.

## 3. Pages (your documents)

- **+** next to Pages makes a blank page or one from a **template**: Meeting notes, Business plan, Product spec, Project brief, Weekly review or Daily journal.
- Click the big **icon** to change it, and the **title** to rename the page.
- Type **/** at the start of a line for the block menu: text, headings, bulleted and numbered lists, to-dos, quote, code, callout, divider, link to page, sub-page or embedded database. Press Enter to pick the first match.
- Shortcuts while typing: `# ` heading, `## ` smaller heading, `- ` bullet, `1. ` numbered list, `[] ` to-do, `> ` quote, ` ``` ` code, `---` divider.
- Formatting: `**bold**`, `*italic*`, `~~strikethrough~~`, `` `code` ``. The marks hide when you click away.
- **Enter** starts a new block. **Backspace** at the start of an empty block removes it or joins it to the one above. **Shift+Enter** adds a line inside a block.
- Hover over a block to see **+** (add a block below) and **⠿** (turn into another type, move up or down, duplicate, delete, or **make it a task** linked to this page).
- **Linking pages**: type `[[` and choose a page. At the bottom of every page you'll see its links, its **backlinks** (pages that link to it), its sub-pages, and the tasks and events linked to it. A link to a page that doesn't exist yet offers to create it.
- **+ Add task for this page** at the bottom makes a task already linked to the page.
- **☆** adds the page to Favorites. **⋯** copies it as Markdown, adds a sub-page, moves it under another page, saves it as your own template, or deletes it.
- Pages can be nested as deep as you like; use the ⏵ arrows in the sidebar to open and close them.

## 4. Databases (tables, boards, CRM)

- **+** next to Databases makes a blank database, or one from a template:
  - **CRM**: Companies, Contacts and Opportunities, already linked to each other, to projects and to tasks.
  - **Project tracker**, linked to projects and tasks.
  - **Content calendar**.
- **Table / Board** switches the view. Boards group rows by a Select property (like Status); drag cards between columns.
- **Properties (columns)**: click a column name to rename it, change its type, edit its options, set its width, move it or delete it. Click **+** at the end of the header to add one. The types are Text, Number, Select, Checkbox, Date, URL, Email, Phone, Linked page and **Relation**.
- **Relation** columns link records to each other: a contact to its company, a deal to its contacts, a project and next-step tasks. Click the cell, then tick the records to link.
- **Filter**: add rules such as "Stage is Customer" or "Value > 1000" (dates are written YYYY-MM-DD). Filters are saved with the database.
- **Sort** by any column, and use **Search** to narrow the rows.
- **New** adds a row (it matches your filters where it can). **↗** opens a row as its own page, with its properties at the top, room for notes and linked tasks, and a "Linked from" list of records that point to it.
- Embed a database inside any page with **/ → Embedded database**.
- **⋯** copies the database as CSV, saves it as a template or deletes it.

## 5. Calendar

- **Today**, **‹ ›** and the small month on the left move around. Switch between **Day / Week / Month** at the top right.
- **Click an empty slot** to create an event at that time, or use **+ Create**. **Ctrl+E** adds an event in words, like "Lunch with Sam tomorrow 1pm for 90m".
- **Click an event** to edit it: title, date and times, all-day, repeat, calendar, location, reminder, description and a linked page ("+ Meeting notes" creates one). Changes to a repeating event apply to the whole series.
- **Drag an event** to a new time or day. Timed tasks can be dragged the same way, and their due date and time change to match.
- **Tasks with dates appear automatically**: timed tasks as blocks, untimed ones in the all-day row. Click a task's circle to complete it. Untick **Tasks** on the left to hide them.
- **My calendars**: tick or untick to show or hide each one. **+** adds a calendar, and **⋯** renames it, changes its colour or deletes it.
- **Month view**: click a day to open it, or double-click to add an event.

## 6. Search

Press **Ctrl+K**. Type words to search everything. Start with `page:`, `task:`, `event:` or `db:` to narrow the results. Use the arrow keys and Enter to open a result, or pick **+ Add task** / **+ New page** from whatever you typed. With nothing typed, it's a quick menu for jumping to any screen.

## 7. Bring your data in

Settings → **Import & export**. A backup is made before every import, and importing the same file again won't duplicate anything.

- **Notion**: in Notion choose Settings → Export → *Markdown & CSV* (include subpages), then pick the .zip. Pages keep their nesting and links; databases arrive as tables with sensible column types.
- **Todoist**: Settings → Backups → download (a .zip), or a project's ⋯ → Export as CSV. Projects, sections, sub-tasks, priorities, dates, repeats, labels and comments come across.
- **Google Calendar**: Settings → Import & export → Export (a .zip). Each calendar becomes its own Resolve 2 calendar.
- To keep a Google, Outlook or Apple calendar **live** instead, go to Settings → **Calendars**, paste its secret iCal link and press Subscribe. It refreshes every 30 minutes and is read-only in Resolve 2.
- **Export everything** writes your pages as Markdown, databases as CSV, events as .ics and the whole workspace as JSON, so you're never locked in.

## 8. Backups

Settings → **Backups**. Resolve 2 backs up once a day automatically (keeping 30 by default). Press **Back up now** any time. **Restore…** puts the workspace back to an earlier backup, and your current state is backed up first, so a restore can be undone. "Open backups folder" shows the files; copy them to a USB drive or a cloud folder for an off-PC copy.

## 9. Your phone and other devices

Settings → **Sync & phone**.

- **Android phone (home Wi-Fi)**: tick "Serve the phone page", then open the link shown on your phone and choose ⋮ → *Add to Home screen*. It shows Today, Upcoming, Inbox, your calendar agenda and search; you can add and complete tasks there. Changes reach the PC within seconds.
- **Phone anywhere**: install **Tailscale** (free) on the PC and the phone and sign in to both with the same account. Then use the PC's Tailscale name in the link instead of the Wi-Fi address. The connection is encrypted and private to your account.
- **Moving to a cloud server later**: run the same program on the server with the command shown in that card, put its address and token in "Hub address", and press *Sync now*. Everything uploads and stays merged; no rebuild and no data loss.
- **Sync folder** (optional): choose a Syncthing, OneDrive or Google Drive folder to keep other PCs in step. The newest change to each item wins, and deletions are respected.
- Keep the link private: the long token in it is the password. "New secret token" changes it.

## 10. AI assistants (no Notion AI needed)

Settings → **AI & MCP**.

- **Claude Desktop**: press *Add to Claude Desktop for me*, then restart Claude. Ask things like "What's due this week?", "Summarise my Business plan page" or "Add a CRM contact for Ada at Northwind and a follow-up task for Friday".
- **Claude Code** and **Codex**: press *Add to Claude Code for me* / *Add to Codex config for me*, or copy the command shown.
- **Claude doesn't see Resolve 2?**
  - Fully quit Claude (right-click its icon by the clock → **Quit**; closing the window isn't enough) and open it again.
  - Still missing? Quit Claude, press *Add to Claude Desktop for me* again, then reopen Claude.
  - The Microsoft Store version of Claude keeps its settings in `%LOCALAPPDATA%\Claude-Data\…`, and the button now writes there. Settings → AI & MCP shows the exact file it used.
  - In a new Claude chat, open the tools / connectors menu and make sure **resolve2_personal** is switched on. Then ask "what's due today in Resolve 2?"
- **ChatGPT** needs extra setup: turn on the phone page, run a free tunnel (cloudflared or Tailscale Funnel), then add the https link as a connector in ChatGPT's developer mode.
- Assistants can read, search, create and update pages, tasks, projects, events and database rows, but only in the workspace you connect them to. Untick **Allow AI assistants** to block them, or tick **Read-only** so they can look but not change anything.
### Let Claude or ChatGPT bring your stuff in

Once an assistant is connected, it can move information in for you in bulk. Paste or attach what you have and ask, for example:

- "Here's my CRM spreadsheet (pasted CSV). Import it into Resolve 2 as a database called Contacts." (uses *import_csv*)
- "Turn these meeting notes into Resolve 2 pages, one per meeting, all under a page called Meetings." (uses *import_pages*)
- "Add this to-do list to my Home project, with dates where I mention them." (uses *import_tasks*; indented lines become sub-tasks)
- "Import this .ics calendar into a calendar called Work." (uses *import_ics*)
- "Read my Notion export and recreate the pages in Resolve 2." (it reads the files and uses *import_pages*)

Assistants can also tidy things up afterwards: "Link each opportunity to its company", "Give every overdue task a new date this week", "Summarise what changed in my Business plan".

### ✨ Ask AI: the built-in assistant (like Notion AI)

Click **✨ Ask AI** in the sidebar (or the ✨ AI button on a page). A panel opens on the right, wherever you are in the app.

- **No extra cost with your subscriptions.** Pick **Claude — your Claude Pro/Max subscription** or **ChatGPT — your ChatGPT Plus/Pro subscription** in Settings → AI & MCP. Resolve 2 then works through the Claude Code or Codex app on your PC, which signs in with your account, so the assistant uses your plan's normal limits and there's no API bill. One-time setup:
  - **Claude**: install Claude Code (claude.com/claude-code), open a terminal, type `claude` and sign in.
  - **ChatGPT**: run `npm install -g @openai/codex`, then `codex`, and choose *Sign in with ChatGPT*.
  - Then press *Check it's installed*.
  - Chatting in the Claude Desktop or ChatGPT app connected through MCP (below) also uses your subscription.
- **Other choices** (Settings → AI & MCP → *The in-app assistant*):
  - **Claude**: paste an Anthropic API key (console.anthropic.com). Best at carrying out multi-step changes.
  - **ChatGPT**: paste an OpenAI API key (platform.openai.com) and type a model name.
  - **Ollama** or **LM Studio**: free and private, running on your PC. Install it, download a model (`ollama pull llama3.2`, or `ollama pull qwen3` for better actions), then press *Detect models*.
  - API keys are paid per use and separate from ChatGPT Plus or Claude Pro subscriptions. To use a subscription instead, chat in the Claude or ChatGPT app connected through MCP (above); it gets the same powers.
- **One-click actions**: Summarise, Continue writing, Improve writing and Turn into tasks (on a page); Plan my day, What's overdue? and Weekly review (anywhere).
- **Or just ask**, and press Ctrl+Enter to send:
  - "Add a follow-up with Ada on Friday 10am and link it to the Northwind deal"
  - "Create a page with a launch plan for Boredom Lab and add the first three tasks"
  - "Move all my overdue Home tasks to Saturday"
  - "Which opportunities close this month and what's their total value?"
- **Let AI make changes** (on by default): the assistant can create and edit pages, tasks, events and database rows. A list at the end of each answer says exactly what it changed. Untick it to only get answers. Nothing is deleted without asking, and every change is covered by your backups.
- Under the answer: **Insert into page**, **Make tasks from list** and **Copy**.

### Organise the to-dos in your pages

Scattered `[ ]` to-dos in your pages can become proper tasks:

- **✨ Ask AI → 🗂 Organise my to-dos**: the AI reads every unticked to-do (and the pages they're on). It puts each one in the right project, creating projects and sections when several to-dos share a theme, and works out deadlines, priorities, repeats and sub-tasks from what you wrote. Then it lists what it did.
- **From Claude or ChatGPT**: once connected, ask "organise the to-dos in my pages", or pick **Organise my to-dos** from the app's prompt menu (Claude Desktop: the + / prompts menu). The same goes for *Plan my day* and *Weekly review*.
- **Without AI**: on a page, choose **⋯ → Turn this page's to-dos into tasks**. Dates, times, p1–p4, #project and @labels written in each to-do are used. Tasks go to the project whose document this page is, or to the Inbox.
- Each converted to-do stays on its page with a **→ # Project · date** tag. **Ticking either the page to-do or the task ticks both.** Click the tag to open the task.

### Speak instead of typing

Press **🎤** in Ask AI or in Quick add (Ctrl+Q), then talk: Windows' built-in voice typing writes what you say into the box. You can also press **Windows+H** in any text box in the app. The first time, Windows asks to use the microphone. On your phone, use the microphone on the keyboard. The Claude and ChatGPT apps also have their own voice modes.

### Let your AI tools run things

Claude and ChatGPT, connected through MCP, can do nearly everything you can: search and read; create, edit and move pages; create, update, complete and move tasks between projects; rename and recolour projects and add sections; create calendars and events and reschedule them; create databases, add columns (including links between records) and rows; bulk import; and organise page to-dos. Just ask in plain words, for example "Set up a project for the house move with sections Packing, Admin and Utilities, and schedule the first three tasks next week."

## 11. Good to know

- Reminders appear as pop-ups in the bottom corner and flash Resolve 2's taskbar button, so keep Resolve 2 open (it can sit minimised).
- Your files live in `%APPDATA%\Resolve2` (Settings → General → Open folder).
- **Esc** closes dialogs and the task panel; **⏴** at the top goes back.
- When you're ready, delete or ignore the Sample workspace and work in Personal, or make more workspaces for work, side projects and so on.
