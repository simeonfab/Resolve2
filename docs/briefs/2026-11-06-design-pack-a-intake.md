# Design Pack A: the Resolve 2 intake journey

6 November 2026. From Simeon. Pack 1 of 4. Parent brief: `docs/briefs/2026-11-06-design-brief.md`.

## What you are designing

The journey a new user takes from opening Resolve 2 to "Open my project". It turns an idea, a problem or a messy project into a connected project with a clear first move.

The logic is built and works. Do not change what happens at each step, only how it looks, reads and moves. The original Resolve designed this journey for a phone. Your job is to bring it to a desktop window at the same quality, using the original as the source.

## Sources (read these first)

1. Screen designs: github.com/simeonfab/resolve-docs, folder `mockups/`. Files named `ResolvePM_<name>_dc.html`. Open them in a browser.
2. Motion: `mockups/Resolve_Flow_Spec.html` (surfaces A to D cover this pack) and `mockups/MOTION-SYSTEM-HANDOFF.md`. Use the values exactly. Ship the "match-seam" character.
3. Reference pictures 01 to 10: github.com/simeonfab/Resolve2, `docs/reference/original-resolve/`.
4. Today's working screens, for content and step order: Resolve2 `docs/reports/2026-11-06-part4.md` and its screenshots `01` to `10`.
5. The old live web intake, for the bridge animation: github.com/simeonfab/ResolveOS-intake-frontend, `public/index.html` (keyframes `bridge-centerout`, `bridge-glowpulse`, `bbrand`, `bfog`, `bparkin`, `bpin`, `pulse-mic`).
6. Quality bar: the response.games home page. Match its level of polish and motion, not its design.

## Look

Colours:

| Token | Hex |
|---|---|
| `--nav-bg` (dark intake background) | #1E1E1A |
| `--surface` | #F2EDE4 |
| `--card` | #FFFFFF |
| `--ink` | #1A1A18 |
| `--ink-soft` | #4A4A44 |
| `--ink-ghost` | #9A9A92 |
| `--border` | #DDD8CE |
| `--amber` (through-line accent) | #C8A040 |
| `--amber-light` | #FBF5E6 |
| `--terracotta` | #8B4A2A |
| `--green` | #5E7D4F |

Type: DM Serif Display for headlines, Inter 400 / 500 / 600 for everything else.

Easings: `--e-spring` cubic-bezier(.34,1.56,.64,1) arrivals and sheets. `--e-out` cubic-bezier(.22,.61,.36,1) reveals. `--e-std` cubic-bezier(.4,0,.2,1) dims and presses. `--e-in` cubic-bezier(.55,0,.85,.35) exits.

The intake is dark. The app is light. The reveal at the end carries the user from dark to light, with amber as the thread between them (Flow Spec surfaces A to D).

## The screens

| # | Screen | Start from | What it must do |
|---|---|---|---|
| 1 | Name and welcome | `Sign-in`, `Onboarding`, picture 01 | "What's your name?" A new user lands here inside the main window |
| 2 | Bring it in | `Bring_It_In`, picture 02 | Situation, then free text, with voice input (mic pulse) |
| 3 | Questions | `Bring_It_In`, picture 02 | One question at a time, with a "What Resolve has heard" panel beside it |
| 4 | Confirm understanding | `Confirm_Understanding`, picture 03 | Cards of what Resolve understood. Two actions only: "This looks right, show me the proposal" and "Something's wrong, let me change it" (puts the cards into edit mode). Cards grow to fit. Never cut text |
| 5 | No clear outcome yet | `Floor_Refusal`, picture 04 | When Resolve cannot find an outcome. Check the mockup against picture 04 and use whichever fits this case |
| 6 | Your next move | `Next_Move`, picture 05 | One recommended first move, with why |
| 7 | Roadmap themes | `Roadmap_Cards`, picture 07 | One card at a time: "Yes, add to my roadmap" or "Not quite" |
| 8 | Here's where to begin | `Todo_List`, picture 08 | The first week's tasks, in order |
| 9 | The whole picture | `Roadmap_Reveal`, picture 09 | Now / Next / Later, then Adopt. Small link: "See and edit everything →" (opens the existing editor, not designed in this pack) |
| 10 | Building your project | `Intake_Transition`, picture 06, old web bridge | The bridge animation while the project is built |
| 11 | Reveal | `OS_Reveal`, picture 10 | Dark to light reveal, "Open my project" |
| 12 | Daily limit | `Daily_Limit` | Shown when the user hits their AI limit mid-intake |

## States every screen needs

1. Waiting for AI: say what is happening ("Reading what you wrote…") with a seconds counter. Nothing sits more than about 5 s without visible progress. At 15 s, Resolve moves on with a fixed fallback question, so design how that hand-off looks.
2. Error: the AI call failed. Offer retry and a manual path. The journey never dead-ends.
3. Resume: a user who left mid-intake comes back to the same step.
4. Long content: long answers and card text wrap and grow. Nothing overflows or overlaps.

## Copy rules

1. Second person: "you", never "the user".
2. Every sentence starts with a capital letter.
3. Plain, calm, short. Keep the original Resolve wording where it exists.

## Frames to deliver

1. Desktop at 1280 × 800 for every screen and state above. Check each one at 1920 × 1080 for overflow.
2. Every animation named and placed, with its Flow Spec reference or exact values where the spec does not cover it (desktop layouts will need some new transitions).
3. A tokens sheet: colours, type sizes, spacing, radii, shadows.
4. A list of anything you designed that the current journey does not do, so it becomes an engineering ticket instead of slipping in.

Not in this pack: the app shell, Home, phone frames (Pack D) and the "See and edit everything" editor (Pack C).
