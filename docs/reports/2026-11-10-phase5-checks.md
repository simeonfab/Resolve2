# Phase 5 checks, before building (10 Nov 2026)

Branch `nav/p5-settings-help`, from main at 0.5.1 (Phase 4 and the Phase 3 fixes merged). These are the Phase 5 prompt's "check and report first" items.

## C-E21: is month-to-date AI cost tracked today?

**Yes, as an estimate.**
- Every API call writes a line to `ai-usage.jsonl` in the data folder: when, provider, model, input and output tokens, and what it was for (`ai::record_usage`).
- `ai::usage_since(first of the month)` adds them up, in total and per model. `ai::check_budget` already turns that into this month's cost with the prices you enter in Settings (one price per million input tokens and one per million output tokens), and stops AI calls at your budget.
- What it can't know: Claude Code and Codex on a subscription log a call with 0 tokens, so they count as no cost. Ollama reports tokens, so it is costed at your prices if you've entered any; with no prices, every cost is 0.

**How I'll build on it:** the spending card shows this month's estimated cost against your budget, the calls and tokens per model, and says plainly that it's an estimate from your prices and that subscription calls aren't counted. No data change is needed.

## C-E22: what the phone pairing needs, and is a QR crate in the build?

- **What the QR code holds:** the phone page's address, `http://<this PC's address on your network>:<port>/?token=<token>`. The address comes from `server::lan_ip`, the port from Settings (8797 by default), and the token is the workspace's private token (at least 16 characters). Opening that link on the phone stores the token, so it only needs scanning once.
- **QR crate: none in the build.** `image` (PNG) is there, but nothing that makes QR codes.

**Your decision:** which crate to add for the QR code.
- My suggestion: **`qrcodegen`** (MIT, no dependencies of its own, about 1,000 lines). I'd draw its squares with egui, so no image work is needed.
- The other common one is `qrcode` (MIT or Apache-2.0); it pulls in more and is aimed at making images.
- Or no crate: show the link and the token as text with Copy buttons, and no QR code.

I'll build the rest of Phase 5 (Settings in four tabs with the spending card, Help with the Report dialog and voice, About with the changelog) and leave the QR code until you choose.

## Your decisions (10 Nov 2026)

- **No QR code, no new crate.** Settings shows the phone link as text.
- **Mobile is out of scope for all of Resolve.** The phone page and pairing stay exactly as they are; no further work on them. Phase 6 (the phone page) is dropped.
- **Spending card:** month-to-date cost as an estimate from the prices you enter, labelled "Estimated".
