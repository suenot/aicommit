# Task: Update Readme Flowchart

## User Request
The user pointed out that the flowchart in `readme.md` was not updated after recent changes to the commit generation logic (diff truncation and prompt refinement).

## Reasoning & Plan

1.  **Analysis:** The flowchart under `## Algorithm of Operation` was missing the step where the fetched git diff is truncated if it exceeds `MAX_DIFF_CHARS`.
2.  **Plan:** Modify the mermaid diagram in `readme.md` to insert a new step (`N_Truncate`) between getting the diff (`N`) and generating the commit message (`O`) in the main commit flow. Also update the `dry-run` flow and add a note to the `watch` mode's commit step.

## Checked Paths/Files
*   `readme.md`: Read, analyzed, and modified.

## Status
- [x] Identified missing step in flowchart.
- [x] Modified mermaid diagram in `readme.md`.
- [x] Created this task file. 