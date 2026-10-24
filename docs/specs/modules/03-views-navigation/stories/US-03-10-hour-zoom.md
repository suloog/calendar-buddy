---
id: US-03-10 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-03
covers: [MOD-03-F27, MOD-03-F28, MOD-03-F29, MOD-03-U08] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-03-01, US-03-02] # story IDs that must be done first
---

# As a User, I can stretch or shrink the hours of the day, so that I see short events clearly or the whole day at once

|  |  |
| --- | --- |
| **Module** | `[MOD-03]` Views and navigation — `docs/specs/modules/03-views-navigation/REQUIREMENTS.md` |
| **Covers** | `MOD-03-F27`, `F28`, `F29`, `U08` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-03-01`, `US-03-02` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Zooming the hour grid of the day, 3-day and week views vertically: Ctrl + wheel or a pinch over the hours, Ctrl + plus / minus / 0, and a slider in the settings. One height for the three views, kept across restarts.

## Acceptance criteria

- [ ] With the pointer over 14:00 in the week view, Ctrl + wheel up makes the hours taller and 14:00 stays under the pointer; Ctrl + wheel down makes them shorter; a touchpad pinch does the same.
- [ ] Ctrl + plus and Ctrl + minus change the height in steps of 25 %, keeping the time at the middle of the view in place; Ctrl + 0 returns to 100 %; the rest of the interface keeps its size.
- [ ] The height stops at 50 % and at 300 %; at 50 % the whole day from 0:00 to 24:00 fits in a window 1080 pixels high at 100 % scale.
- [ ] Only the height of the hours changes: the width of the days, the all-day strip, the side panel and the top bar stay the same.
- [ ] Events, the current-time line, clicking to create, dragging and resizing land on the right times at every height.
- [ ] The settings' "Appearance" section shows the height as a slider with the value in percent; moving it changes the views, and zooming in the views moves it.
- [ ] The height chosen in the week view is the same in the day and 3-day views, and is the same after the app is restarted.

## Out of scope / what not to worry about

- Zooming the width of the days, the month or agenda views, or the whole interface — `DEC-166`.

## Technical notes

- One factor on `HOUR_HEIGHT` in the settings; every minute ↔ pixel conversion goes through it; eframe's `zoom_with_keyboard` off; `zoom_delta` covers wheel and pinch. Anchoring arithmetic test-first (`MOD-03` *Technical notes*).
- Decision: `DEC-166`.
