---
id: US-10-03 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-10
covers: [MOD-10-F03, MOD-10-F06, MOD-10-F07, MOD-10-F11, MOD-10-F12, MOD-10-F25] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
mode: A # A | B | C — agent autonomy · PE code review · PE pairs with the agent
depends_on: [US-10-02] # story IDs that must be done first
---

# As a User, I can name guests and ask for Meet in the description, so that a meeting with people is set up in one sentence

|  |  |
| --- | --- |
| **Module** | `[MOD-10]` Events from a description — `docs/specs/modules/10-events-from-a-description/REQUIREMENTS.md` |
| **Covers** | `MOD-10-F03`, `F06`, `F07`, `F11`, `F12`, `F25` |
| **Priority** | must |
| **Mode** | A |
| **Status** | ready |
| **Depends on** | blocked by `US-10-02` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

People and Google Meet named in a description: the guests are looked up as when typing them, a name that is not certain is left for the user to pick, and the Meet switch follows what the description says. Because the "New event" window has no guests or Meet, such a description opens the full editor. No invitation goes out before the user saves.

## Acceptance criteria

- [ ] Names are looked up as when typing a guest — the account's contacts, the organization's directory on Workspace accounts, recent addresses.
- [ ] A description with guests or Meet opens the full editor filled in; one with neither opens the "New event" window (`US-10-02`).
- [ ] Meet is on for "z Meetem" / "with Meet", off for "bez Meeta" / "no Meet", and when the description says nothing it behaves as for any new event — turning on by itself once a guest is added.
- [ ] For a local calendar, guests and Meet are left out and the editor opens as for a local event.
- [ ] Offline, or with no Google account, a Google calendar cannot take the event (`US-10-02`), so guests are not looked up.

### Scenario: two guests and Meet

- **Given** "Ania" and "Tomek" each fit exactly one contact
- **When** the user sends "wrzuć mi spotkanie z Anią i Tomkiem jutro o 10 z Meetem"
- **Then** the full editor opens for tomorrow at 10:00 with both as guests and Meet on; nothing is saved and nobody is invited until Save

### Scenario: a name that is not certain

- **Given** "Kasia" fits two contacts and "Zenek" fits nobody
- **When** the user sends "lunch z Kasią i Zenkiem w piątek o 13"
- **Then** the guest field holds "Kasia" with the two matches suggested and "Zenek" with the usual suggestions; neither is a guest until the user picks someone

## Edge cases

- A word for a role or group ("mama", "klient", "zespół") the model passes as a name → it fits no contact and stays unresolved in the guest field, never invited.
- The account cannot add Meet → the switch stays off and unavailable, as in the editor today.

## Out of scope / what not to worry about

- Adding events without people — `US-10-02`.
- Changing the guests of an existing event by description — the general change path, `US-10-04`.
- Suggesting a time when the guests are free.

## Technical notes

- The guest lookup reuses the editor's suggestion source (`MOD-06-F02`…`F04`); exactly one match is added, anything else is left as typed text with the suggestions open.
- Meet follows `MOD-06-F17` when the description is silent (`DEC-194`).
- Manual check (left for the owner, as for the local loop's stories): real Google contacts and the directory of a Workspace account; the editor's look with an unresolved name.
- Decisions: `DEC-178`, `DEC-194`, `DEC-196`, `DEC-205`, `DEC-211`, `DEC-213`.
