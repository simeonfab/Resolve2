# Phase 2: first checks, before building (7 Nov 2026)

Branch `nav/p2-daily`. Phase 1 is merged to main (e3bc450). Build passes and the self-test ends 207 passed, 0 failed.

## Done before Phase 2

- **Inter Medium (500)** is now in `assets/fonts/Inter-Medium.ttf`. It is the Latin subset from the same Fontsource release as the fonts already bundled; today's Regular download is byte-identical to ours, so they match. The brief names weight 500 in one place: the sidebar items (Inter 500 13.5). They now use it through `fm()` in theme.rs. The brief's "small: Inter 400 or 500" is left at 400 until the canvas shows which. A self-check confirms the file is a real TrueType font and not a copy of Regular. The licence note in docs/LICENSING.md is updated.
- **Screenshot sample data from the canvases: not started.** `docs/reference/canvases/` is not on main yet. I fetched origin and found no such folder on main or any other branch; `docs/build-brief-packs-a-d` only has `docs/reference/original-resolve/`. I won't type the café project's tasks, dates and names from memory of the brief, because that would be guessing.

## Unknowns (rule 7)

| Ticket | Question | What the code says |
|---|---|---|
| B-E5 | Can a task be marked waiting? | **No.** `Task` (model.rs) has no waiting state. Today the only "Waiting" is a status on Dependency records (records.rs). To add it, I'd add a `waiting_since: Option<i64>` field to `Task` (additive, `serde(default)`, so old data still loads). The age shown on Home's WAITING ON comes from it. It would be set from the task editor and a "Waiting on" list section, and from the MCP `update_task` tool. Marking and unmarking would go through history.rs with Undo. |
| B-E13 | Do events support all-day and repeats? | **Yes.** `Event` has `all_day` and `recurrence`: None, Daily, Weekdays, Weekly, Monthly, Yearly, every N days or every N weeks. `occurrence_on` and `occurs_on` already work out repeats, and the event editor already has "All day" and "Repeat" (calendar.rs). |
| B-E22 | Does a task hold a start time? | **Partly.** A task holds `due` (date), `due_time` and `duration_min`, and the calendar draws a timed task as a block. There is no separate start time. `due_time` means "due at", and reminders fire from it: `reminder_min` before `due_time`, and a parsed time sets a reminder at 0. If "Use this plan" writes its start times into `due_time`, every planned task would also get a reminder at that time and its due time would change. |
| B-E6 | Does a failed AI suggestion on Home stop the rest loading? | **No AI runs on Home today.** The "next move" comes from the deterministic assessment (`assess.rs`), so nothing on Home can fail that way yet. B-E6 applies to the AI parts Phase 2 adds, mainly Plan my day: each will show its own error in its own card while the rest of Home loads. |

## Questions for you (rule 8)

1. **Canvas images:** please push `docs/reference/canvases/` to main, or tell me where they are. The sample data and the per-screen comparisons need them.
2. **B-E11, as the prompt asks:** what should `#Name` in Add task do when no project has that name? Today it quietly creates a new project with that name (mod.rs `add_task_text`).
3. **B-E22:** should "Use this plan" write the start time into `due_time`? That would change the due time and set a reminder. Or should I add a separate `start_time` field to tasks (additive), which the calendar draws and which doesn't move the due date or time?
4. **B-E5:** is the `waiting_since` field described above the way you want waiting added?
