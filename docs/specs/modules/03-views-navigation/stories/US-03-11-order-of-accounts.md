---
id: US-03-11 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-03
covers: [MOD-03-F30, MOD-03-U09] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-03-06, US-03-07, US-09-02] # story IDs that must be done first
---

# As a User, I can put my accounts and the local calendars in the order I want, so that the calendars I use most are at the top

|  |  |
| --- | --- |
| **Module** | `[MOD-03]` Views and navigation — `docs/specs/modules/03-views-navigation/REQUIREMENTS.md` |
| **Covers** | `MOD-03-F30`, `U09` |
| **Priority** | must |
| **Status** | ready |
| **Depends on** | blocked by `US-03-06`, `US-03-07`, `US-09-02` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The settings' **Accounts** section shows the side panel's groups — each account with its calendars, and "Local calendars" — in their order, with Move up and Move down on each; the order holds wherever calendars are grouped. Single calendars are not reordered.

## Acceptance criteria

- [ ] The settings' Accounts section lists the accounts and "Local calendars" in the side panel's order, each with Move up and Move down; the first cannot move up and the last cannot move down.
- [ ] Moving a group changes at once the side panel, the calendar picker of the quick box and the editor, and the settings' Calendars list; the calendars inside each group keep their order.
- [ ] The order is kept after a restart.
- [ ] Until anything is moved, the accounts are in the order they were connected and "Local" is last; an account connected after the order was chosen comes last; removing an account leaves the others in their order.
- [ ] The order does not change where new events go when no default calendar is chosen, nor the account "Meet with…" searches.

## Out of scope / what not to worry about

- Reordering single calendars inside a group, or dragging groups in the side panel — `DEC-189`.

## Technical notes

- An `order = <account ID>` line per group in the settings file, the local calendars under their pseudo-account ID `local`; `side_panel` sorts its groups by it, and the pickers and the settings take the groups from there.
- Decisions: `DEC-189`.
