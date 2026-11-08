# Commercial use and IP review (5 Nov 2026)

Not legal advice. Where rights can't be established from the material, it says so.

## 1. What Resolve 2 is made of

| Part | Origin | Status |
|---|---|---|
| Workspace code (`src/` except `resolve/`, `ui/intake.rs`, `ui/cockpit.rs`, `ui/resolve.rs`) | Generated for you in Craft Anything by Claude Code (the "Control Centre / Weave" build) | Written from scratch for this project; no copied Notion/Todoist/Google code found. **Check Craft Anything's terms** for ownership of generated output — not verifiable from here. |
| Resolve layer (`src/resolve/`, Resolve UI, tests, docs) | Generated in this session by Claude Code from your specifications | As above (Claude Code session — check the terms of the Anthropic plan/account used). |
| Prompts in `src/resolve/intake.rs` and role/governance files in `resolveos/` | Your own repos: `ResolveOS-intake-frontend`, `cap-pm-cockpit-alaria`, `ResolveOS` | Yours. Those repos have **no LICENSE file**; add one (or keep them private) before release. ResolvePM notes say some pipeline docs were "derived from" packs named product-brainstorming / write-spec with unstated origin — **check before commercial use**. |
| Rust crates (379) | crates.io | All permissive (MIT / Apache-2.0 / BSD / Zlib / ISC / BSL-1.0 / Unicode-3.0); list in `THIRD_PARTY_CRATES.md`. |
| Fonts | `epaint_default_fonts` (bundled by egui: Ubuntu-Light, Hack, Noto Emoji, emoji-icon-font) under OFL-1.1 + Ubuntu Font Licence + MIT/Apache | Bundled in the binary → ship their licence texts with any distribution. Segoe UI, Segoe UI Emoji, Consolas and Georgia are **loaded from Windows at runtime, not redistributed** — fine on Windows; other platforms fall back to bundled fonts. |
| Windows `curl.exe`, `rfd` dialogs | OS components / MIT | No redistribution. |
| **Inter** (Regular, Medium, SemiBold, Bold; Medium added 7 Nov 2026 from the same source and release) and **DM Serif Display** — Latin subsets, `assets/fonts/` | Downloaded 5 Nov 2026 from the Fontsource CDN (jsDelivr), which redistributes the Google Fonts releases; licence texts from github.com/google/fonts | **SIL OFL 1.1** (bundled in the binary). Keep `assets/fonts/OFL-*.txt` with any distribution; don't sell the fonts on their own; DM Serif Display's notice includes Adobe's Reserved Font Name "Source" — don't name a modified font "Source". |

## 2. Obligations to keep
- MIT/Apache/BSD/ISC/Zlib: keep copyright + licence notices in distributed binaries' notices (generate an `about`/NOTICES file from `THIRD_PARTY_CRATES.md` at release).
- Apache-2.0: preserve NOTICE files of crates that have them.
- Unicode-3.0 (ICU data crates): include the Unicode licence text.
- OFL-1.1 / UFL (bundled egui fonts): include licence texts; don't sell the fonts on their own; UFL restricts using "Ubuntu" in derived font names (not done).

## 3. Trade dress / branding risks (fix before commercial release)
| Item | Risk | Done / recommendation |
|---|---|---|
| Todoist-like red accent and P1–P4 colours (#DB4C3F family), circle checkboxes, "Add task" red button | Close to Todoist's look | **Replaced** with Resolve amber/terracotta/blue priority palette. |
| Feature names "Inbox / Today / Upcoming / Filters & Labels", Notion-style block menu, Google-Calendar-like week grid | Generic functional names/layouts are generally not protectable, but the combination mirrors Todoist closely | Keep for personal use; **consider renaming Filters & Labels and restyling the task list** before release. |
| Import of "Notion export", "Todoist backup", "Google Calendar" | Naming third-party products to describe compatibility (nominative use) | OK; avoid their logos. None are bundled. |
| Reference screenshots in Craft Anything `reference/` (Todoist, Notion, Google Calendar) | Third-party UI images | **Never ship them**; they're outside `app/` and git-ignored. |
| Emoji used as icons | Rendered from system/bundled fonts | Replace with your own icon set for a commercial product. |
| Logo | Original brand: "Logo TBC" | Resolve 2 uses only the ✦ glyph + text; commission an original mark. |
| `resolve-docs/snapshots/support.js`, `doc-page.js`; Next.js starter SVGs in ResolvePM `public/` | Unclear provenance / Vercel assets | **Not reused** in Resolve 2. |
| Microsoft Clarity tag in the old intake | Third-party tracking | **Not reused**. |

## 4. Before any commercial release
1. Confirm ownership terms for Craft Anything outputs and for the Claude account used.
2. Add licences to (or keep private) your original Resolve repos; resolve the "derived from" packs question.
3. Generate a NOTICES file from Cargo.lock; bundle font licences.
4. Replace emoji icons and finalise an original logo; review the Todoist-like task styling.
5. Replace the provisional `LICENSE` notice with your chosen terms (lawyer review).
