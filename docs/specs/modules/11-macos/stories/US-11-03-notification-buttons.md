---
id: US-11-03 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-11
covers: [MOD-11-F10] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-11-01, US-11-02] # story IDs that must be done first
---

# As a User on a Mac, I can snooze, dismiss or join from a notification and open its event by clicking it, so that I act on a reminder without opening the app

|  |  |
| --- | --- |
| **Module** | `[MOD-11]` macOS — `docs/specs/modules/11-macos/REQUIREMENTS.md` |
| **Covers** | `MOD-11-F10` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-11-01`, `US-11-02` |
| **YouTrack** | — |
| **Open questions** | — (the signature: `DEC-218`) |

## Description

As on Linux (`US-07-01`, `US-07-02`): a notification shown in the app's own name, with Join for an event with Google Meet, Snooze for 5 or 10 minutes and Dismiss; clicking it opens its event in the app.

## Acceptance criteria

- [ ] A notification of the installed app is shown in the app's name and icon, not Script Editor's; macOS asks once to allow them.
- [ ] Snooze 5 minutes shows it again 5 minutes later; Dismiss removes it for good.
- [ ] Join opens the event's Meet call in the browser.
- [ ] Clicking the notification opens its event in the app's window, also while the window is hidden in the menu bar.

## Out of scope / what not to worry about

- What macOS shows in "do not disturb" — the system decides (`MOD-07`).
- A program run outside the bundle (`cargo run`) — its notifications stay without buttons (`DEC-218`); since `DEC-224` it runs as the bundle's copy, with them (`US-11-06`).

## Technical notes

- `platform/macos/notifications.rs`: `UNUserNotificationCenter` with one category for each set of buttons (`Action::buttons`); its delegate answers on the main thread, so it depends on the event loop of `US-11-02`.
- The bundle is signed by `SystemLauncher::install`, ad hoc or with the user's certificate (`DEC-224`); unsigned, macOS refuses notifications without asking (`DEC-218`).
- `Notifier::buttons` answers true from the bundle, and `DEC-64`'s fallback is used on macOS only outside it.
- Decisions: `DEC-38`, `DEC-45`, `DEC-64`, `DEC-217`, `DEC-218`.
