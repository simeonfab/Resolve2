# Phase 3 checks, before building (8 Nov 2026)

Branch `intake/p3-redesign`. These are the brief's "check and report first" items for the intake. I checked each one in the code (and, for D5, by running it) before building on it.

## E28 and D7: how the app hears about AI limits

| Limit | Caught before this phase? | What happened | Now |
|---|---|---|---|
| **Monthly AI budget** (`check_budget` in `ai.rs`) | Yes | It runs before every structured AI call (the intake's understanding, proposal, sorting and next question). Its message, "This month's AI budget ($X) has been reached (≈ $Y). Raise it in Settings → AI, or continue manually.", was passed through without a retry and shown as a plain error line. The cost is estimated from the tokens recorded, so it only ever stops **paid API** use: Claude Code and Codex record 0 tokens (they run on your subscription) and local models cost nothing. | Recognised as a limit (`ai::is_budget`); the intake shows screen 12. Plan my day's API path didn't check the budget; it does now. |
| **Claude or OpenAI API rate limits and quota** (HTTP 429 `rate_limit_error`, `insufficient_quota`, "credit balance is too low") | **No** | The error's type was dropped, leaving "The AI service said: <message>". The intake then retried once and replaced it with "The AI's answer didn't come back in the right shape, even after one retry", so the limit was hidden. | The error keeps its type, `ai::is_limit` recognises it, and there is no retry. |
| **Claude Code usage cap** | **No** | The installed Claude Code CLI writes its cap message as its answer. I searched the CLI on this PC for its own wording: its limit messages start "You've hit your…" or "You've reached your…", plus "You're out of usage credits" and "Your org is out of usage". The app read that text as a broken answer ("didn't return JSON"), retried, and showed "didn't come back in the right shape". | Recognised when the first line of the CLI's answer or error matches; shown as "Claude Code: <its message>". Whether it fires on a real cap is **unverified**: the cap was not hit on this PC. |
| **Codex usage cap** | No | Same path as Claude Code. | Only the generic words above are matched; Codex's own wording was not checked (**unverified**). |
| **Overloaded** (529) | n/a | A temporary failure, not a limit. | Still treated as an error (Try again). |
| Local models (Ollama) | n/a | No limits. | |

**D7, the "Daily limit" copy.** There is no daily limit anywhere in the code: the only budget is monthly (`monthly_budget`, counted from the 1st of the month). The canvas's sentence on screen 12, "This month's AI budget has been reached. Carry on without AI from here, or change the AI in Settings and pick up from this line, not from the start.", fits `check_budget`, so I use it for the budget. A provider's cap (Claude Code, an API rate limit) is not your budget, so for those screen 12 says "Your AI's usage limit has been reached." with the same second sentence, followed by the provider's own message. That second wording is mine: **please confirm**.

## D5: how long the proposal takes today

Measured on this PC with the Test Intake's headless run of the "Service business" scenario, three questions, `--testlab --lab-run "Service business" --max-q 3`. One run per AI.

| Step | Claude Code (your subscription, default model) | Ollama llama3.2 (local) |
|---|---|---|
| Each answer: sorting it, then the next question | 14.6 s, 13.3 s, 10.8 s (sorting 5.1 to 7.3 s; next question 3.6 to 6.0 s) | 5.0 s, 5.3 s, 7.6 s |
| Understanding (the confirm cards) | 27.7 s | 4.5 s |
| **Proposal** (plan and validation side by side) | **47.1 s** (plan 35.6 s, validation 47.0 s) | **17.6 s** (plan 5.7 s, failed the quality check, retried 12.0 s; validation 11.2 s) |
| Whole run | 117.2 s | 42.6 s |

- The headless run sorts and asks one after the other; in the app they run side by side, so the wait after each answer is the next-question call alone.
- The app already starts the understanding and the proposal in the background while you answer and read the cards, so the wait you see is usually shorter than these numbers. I haven't measured that.
- llama3.2 is faster but weaker: its project name came back as "Turning what I do into".
- **Not measured:** the Claude API and OpenAI (no keys on this PC), and qwen3 (not downloaded). These are **unverified**.

**Speed fix, proposed only (not built):** the validation half is the slow part on Claude Code (47.0 s against 35.6 s for the plan). Running it on a faster Claude model (the intake already uses haiku for sorting and sonnet for the next question) would likely bring the proposal near the plan's 35.6 s. This needs your go-ahead, since it may change the validation's quality.

## E14: build progress events from Adopt

- Adopt runs as six real steps (`journey::adopt_step`, step names in `ADOPT_STEPS`): Creating the project, Laying out the roadmap, Adding tasks, Setting up validation, Writing the knowledge pages, Saving the intake transcript. These are the same strings as the canvas's bridge checklist.
- The bridge screen calls one step per frame, held 0.3 s each so each is visible; the writes themselves take milliseconds. So the real build finishes in about 2 s.
- The bridge (E14) can bind its checklist to the real step index. Since the build is faster than the 7.2 s minimum, the beats run their full 7.2 s and the checklist ticks as each real step completes, then holds until both are done.

## Also found while checking

- **A crash:** checking whether a heard item uses your own words cut a word in the middle of a character when the word was short and ended in an accented letter ("café"). The llama3.2 timing run stopped on it. Fixed, with a self-check.
- **Understanding failures were hidden:** if the AI failed to write the confirm cards, the app quietly showed cards made from your own words, with no message. It now shows the error state (ST4) with Try again and "Fill the cards yourself".
- **Progress events:** the waiting cards had nothing real to follow, only a counter. The intake engine now notes real events: the answer sorted, the next question asked and chosen, each half of the proposal finished, merged, saved. The waiting checklists tick on those.
