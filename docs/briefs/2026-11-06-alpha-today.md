# Resolve 2: alpha release for testers, today

6 November 2026. From Simeon. Goal: a build I can hand to three technical testers on Windows today. Only the items below. Everything else waits until they have used it. The earlier briefs still stand where this message is silent.

This file is in docs/briefs. Commit it with your first commit. Work on branch alpha/today. Merge to main when build and self-test pass. At the end, give me the exact push command.

## 1. One download

1. Make a release build (not debug) and package it as Resolve2-alpha-<date>.zip: the exe plus anything it needs, and a short README.txt with the three steps to start.
2. It must run on a clean Windows 10 or 11 account with no Rust, Git or developer tools. Prove it, or tell me exactly what you could not prove and why.
3. A fresh copy starts empty: none of my workspaces, data, keys or tokens inside the zip. First launch goes to "Connect your AI" (section 2), then "Start something new with Resolve".
4. The README tells testers that Windows SmartScreen will warn because the app is unsigned: click "More info", then "Run anyway".
5. Show the app version (for example "Alpha 0.1, 6 Nov 2026") in Help and Settings.

## 2. "Connect your AI" as the first screen

One full-screen step before the first intake, also reachable from Settings. Three big choices, in this order:

1. Claude Code (recommended). "Free if you already pay for Claude Pro or Max." Detect whether Claude Code is installed and signed in. If not, show the two setup steps and a "Check again" button.
2. Ollama. "Free and private, runs on your PC, slower and less accurate." Detect it, and offer to pull qwen3. If only llama3.2 exists, show "Fine for trying the flow, weak for real projects".
3. Claude API key. "Fastest and best results, but you pay per use. Set a monthly spending limit in your Anthropic console first." Paste the key, then Test.

Each choice has a Test button that proves a real reply and shows a green tick or a plain error. The user moves on only after a passing test, or by pressing a small "Skip for now". Skip uses the offline question set.

## 3. Question cap

A hard limit of 8 questions in the intake. After 8, go to the proposal with what it has. Never ask a question already asked, or one with the same meaning.

## 4. Handoff bug

Today a handoff produces the same text whatever I type ("security hardening" and "marketing" gave identical briefs). Fix it so each brief is written from what I typed plus the project's real context. Prove it with two different inputs shown side by side in the report.

## 5. Import knowledge, sorted automatically

1. An "Add files" button in Knowledge, and an "Attach a brief or notes" button on the intake's bring-it-in screen.
2. Accept .md, .txt, .docx and .pdf. Drag and drop works too.
3. Resolve sorts each file automatically into the right place in Knowledge (project brief, requirements, research, documentation, reference, or Evidence and Records) and says where it put it. The user changes the place with one click.
4. In the intake, attached files feed the questions and the proposal like typed text. Claims from a file count as evidence only when the file states them as fact. Everything else is an assumption.
5. Without AI, sort by file name and headings, and say it was a guess.

## 6. Help as a guide

Replace the long page with sections listed down the left, each clickable:

1. Welcome: "This is the Resolve alpha. Thank you for being one of the first testers. I made this, and I want to hear what works and what doesn't. Message Simeon on the number he gave you if you get stuck." Then Report a problem (section 9).
2. Quick start, five steps: connect your AI, start something new with Resolve, answer the questions, check the proposal, open your project.
3. Requirements: Windows 10 or 11, and one of Claude Code, Ollama or a Claude API key.
4. One short section per part of the app: Home, Projects, Overview, Roadmap, Tasks, Frameworks, Knowledge, AI Team, Handoffs, Calendar. Include how milestones, dependencies and "Coming up" get set on the Roadmap.
5. Tips: Windows+H for voice typing in any text box.
6. Troubleshooting: AI not connecting, SmartScreen, where the data lives.

Keep the current tutorial content, reorganised into these sections, and keep "Copy as text".

## 7. Settings in four tabs

1. AI: the same Connect your AI choices as section 2, plus usage and cost. The MCP and Claude Desktop setup moves to Advanced.
2. General: appearance, workspace name and colour, Test Intake, keyboard shortcuts.
3. Your data: backups, recently deleted, import and export.
4. Advanced: sync and phone, the hub, sync folder, MCP and Claude Desktop, calendar subscriptions, "Where your files are", "Opening Resolve 2", all workspaces.

Every card: a title, one plain sentence, then the controls. Longer explanations go behind a "Learn more" toggle. No file paths or tokens on the first three tabs.

## 8. AI Team cards in one line

All role cards the same height, aligned on one row. The selected card gets a highlight border only. It never moves.

## 9. Report a problem

A button in Help (and in Settings, AI tab) that copies to the clipboard: app version, Windows version, the AI in use, and the last error message. No personal data or project content. A note says "Paste this to Simeon".

## 10. Report back

What changed in plain English, evidence for each item (screenshots for every screen change, the handoff side by side, the clean-account test), deviations, files changed, the commit hash on main, the push command, and where the zip is.
