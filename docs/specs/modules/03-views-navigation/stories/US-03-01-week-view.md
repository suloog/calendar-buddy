---
id: US-03-01 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-03
covers: [MOD-03-F02, MOD-03-F14, MOD-03-F19, MOD-03-F21, MOD-03-F22, MOD-03-U01, MOD-03-U02, MOD-03-U05, MOD-03-NFR-01] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [] # story IDs that must be done first
---

# As a User, I can see my week as a clear, smoothly scrolling grid, so that I see at a glance what my week looks like

|  |  |
| --- | --- |
| **Module** | `[MOD-03]` Views and navigation — `docs/specs/modules/03-views-navigation/REQUIREMENTS.md` |
| **Covers** | `MOD-03-F02`, `F14`, `F19`, `F21`, `F22`, `U01`, `U02`, `U05`, `NFR-01` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | — |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The week view — the main view of the app — and the feasibility check of the interface technology that comes with it: if the chosen technology cannot give a smooth, readable week grid, the project switches before any other view is built. It can be built on test data, in parallel with `MOD-01` and `MOD-02`; real events arrive through `US-02-01`. The day, 3-day and month views reuse what this story builds (`US-03-02`, `US-03-03`).

## Acceptance criteria

- [ ] A week grid shows seven days with their hours; all-day and multi-day events sit in a strip above the hours; calendars shared as free/busy only appear as "busy" blocks.
- [ ] Today is highlighted, a line marks the current time, and the view opens scrolled to the current time.
- [ ] Overlapping events are placed side by side, each one readable.
- [ ] The layout adapts to the window size and to 100%, 150% and 200% screen scale.
- [ ] On a profile of 5 accounts, 50 calendars and about 20,000 events, scrolling the week and switching weeks meet the `NFR-03` thresholds; the first measurement of this story sets them.
- [ ] The feasibility check of the interface technology — smooth scrolling, overlapping events, resizing at several scales, and dragging an event, on test data — is done and its result recorded; if it fails, the views move to the fallback technology before other views are built.

## Edge cases

- Many events overlap at the same time → each gets a narrower column and keeps at least the start of its title visible.
- An event runs past midnight → it is shown on both days.
- A day with a daylight-saving change → the hour grid stays 0:00–24:00 and events sit on their clock hours; the skipped hour stays empty, the repeated hour is shared. — *source: `DEC-67`*
- The window is made very small → it does not go below a minimum size at which a day stays readable.

## Out of scope / what not to worry about

- Creating, dragging and resizing events for real — `MOD-05` (the drag here is only part of the feasibility check).
- Fetching events — `MOD-02`.
- Theme and density — `MOD-04`.

## Technical notes

- Feasibility check of `DEC-33`: egui first, Iced as the fallback; the result recorded as a decision.
- Views render from view models fed by the local store (constitution); overlap layout by column-packing per day (`MOD-03` *Technical notes*).
- Decisions: `DEC-14`, `DEC-22`, `DEC-33`, `DEC-49`, `DEC-56`.
