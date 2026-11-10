# Phase 4 checks, before building (10 Nov 2026)

Branch `nav/p4-project`, version 0.5.0. These are the Phase 4 prompt's "check and report first" items. I checked each one in the code before building on it.

## C-E5: can a record link a roadmap theme to a hypothesis?

**Yes, already.** A roadmap theme is a record (`Kind::Roadmap`) with a `gate` field that holds a hypothesis record's id.
- You set it by hand in the Roadmap's theme menu ("Validation gate": No gate, or one of the project's hypotheses). The card then shows "Gate: <hypothesis> (<status>)", open when the hypothesis is Supported. Code: `ui/sections.rs`, `RmAct::Gate` and `theme_card`.
- Adopt never sets a gate: the proposal has no gate data, so nothing is guessed.
- A theme's general `links` list holds the heard items it rests on and its tasks, not hypotheses.

**How I'll build on it:** the gate chip from Phase 0 ("◇ H1 result", or "✓ Gate open" in the Supported pair) uses this field, and the Frameworks result (C-E10) updates it. No data change is needed.

## C-E8: do milestones exist as records?

**Yes.** `Kind::Milestone` is a record kind with the statuses Planned, Reached and Missed and its own fields, and the Roadmap already has "+ Milestone". The right rail (C-E8) can list them as they are. No data change is needed.

## C-E7: what happens today when a phase move fails?

**A move can't fail today, and it can't be undone either.**
- Moving a theme to Now, Next or Later writes the `phase` field straight into memory (`RmAct::Move` in `ui/sections.rs`). If the record has gone, the move is skipped silently.
- It does not go through `history.rs`, so there is no Undo toast, against rule 5. The theme's gate, "after" and status changes work the same way.
- If saving the file fails afterwards, `save_now` shows "Couldn't save: …" as a toast and the change stays on screen, unsaved.

**How I'll build on it:**
- Every roadmap change (move, gate, after, status) goes through `undoable`, with an Undo toast.
- A move whose theme no longer exists (deleted elsewhere, or undone) changes nothing and says "Couldn't move "<theme>". Nothing changed." That is the rollback the brief asks for.
- The F6 motion (a theme landing in a new phase) plays only when the move succeeds.

## C-E15: does import_pages report errors per file?

There are two import paths:
- **Knowledge's Add files** (the button, or dragging files in) **already reports per file.** Each file that can't be read keeps its own error ("⚠ <file>: <reason>") in the Knowledge import log; the others are added as one undoable change. Code: `ui/import_ui.rs`.
- **The MCP tool `import_pages`** (Claude or ChatGPT adding pages) reports no errors. It can't fail on a single page, and a `parent_title` it can't find is dropped silently: the page lands at the top level.

**How I'll build on it:**
- The Add sheet's file option (C-E13) uses the first path and lists each failed file under the result.
- For `import_pages` I'll report a missing parent per page ("parent not found, added at the top level") rather than drop it silently.

## C-E18: is there a CSV import dialog today?

**No.** CSV becomes a database only through the MCP tool `import_csv` and inside the Notion export import (Settings, Import). Both use `import::csv_to_database`, and that never rejects a row:
- a date it can't read is stored empty without a word;
- a number it can't read keeps its text;
- cells past the header's last column are dropped.

**What I'll build:** the CSV import dialog (Pack C, Databases_Import), and "failed rows listed" by this rule. A row fails when:
- it has more cells than the header has columns (cells would be lost), or
- a value in a column read as a date or a number can't be read as one.

Failed rows are listed with their row number and the reason; the other rows are imported as one undoable change. **This rule is mine.** The brief doesn't say what counts as a failed row. Tell me if you want a different one; it's one function.

## Open decision: Frameworks' one-line description

This is parked for a brainstorm. Frameworks gets an empty slot for that line, with no placeholder text pretending to be final. The current "about" text in `nav.rs` stays where it is today; it is not shown as the new line.

## Also noted

- The roadmap moves, gates and statuses skipping Undo (C-E7 above) is a rule 5 gap that has existed since before the redesign. Phase 4 closes it.
