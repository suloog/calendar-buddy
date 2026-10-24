---
id: US-04-03 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-04
covers: [MOD-04-F04, MOD-04-U03, MOD-04-U05, MOD-04-U06] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-04-01, US-03-01] # story IDs that must be done first
---

# As a User, I work in a calm, restrained English interface, so that my calendar colours are what stands out

|  |  |
| --- | --- |
| **Module** | `[MOD-04]` Appearance and themes — `docs/specs/modules/04-appearance-themes/REQUIREMENTS.md` |
| **Covers** | `MOD-04-F04`, `U03`, `U05`, `U06` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-04-01`, blocked by `US-03-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The overall restraint of the interface, in the spirit of Morgen and Apple Calendar: neutral tones with one fixed accent colour, no transition animations, all texts in English. The look is checked on `MOD-03`'s views, starting with the week view of `US-03-01`.

## Acceptance criteria

- [ ] Apart from calendar and event colours, the interface uses only neutral tones and one accent colour, in both themes.
- [ ] Changing views and periods happens without transition animations.
- [ ] All interface texts are in English.
- [ ] Notices appear in the bottom corner over the view without moving it; each but the Undo closes with its cross and goes on its own after 10 seconds, a new message in it showing it for another 10.

## Out of scope / what not to worry about

- Translations — `DEC-31`.
- An accent colour taken from the system — `DEC-57`.
- Mock-ups — `DEC-32`: the look is reviewed by the product owner on the working app.

## Technical notes

- Custom egui visuals; interface texts in one table for later translations (`MOD-04` *Technical notes*).
- Decisions: `DEC-31`, `DEC-57`, `DEC-58`.
