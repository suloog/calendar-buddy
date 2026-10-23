---
id: DSC-08
title: Guests, invitations, RSVP and Google Meet
kind: business # business (a functional area — becomes a module at /og:modules) | cross-cutting (roles and access, technology and integrations, non-functional requirements — feeds PROJECT.md, NFR.md and the constitution, never a module)
status: settled # open (not yet discussed) | discussed (discussed with the analyst, waiting on the client) | settled (no [W] question in this topic is open)
sources: [SRC-01, SRC-03] # SRC-NN documents whose items this topic gathers, e.g. [SRC-01, SRC-03]
module: # MOD-NN — filled by /og:modules; empty for cross-cutting topics
sessions: [2026-10-23] # dates of the discovery sessions in which the topic was discussed, YYYY-MM-DD
---

<!-- Copy to docs/specs/discovery/NN-<topic>.md. Content in the project language; headings fixed.
     This file records what we UNDERSTOOD about this topic after reading every source and talking
     to the analyst — the layer between the sources (what the client wrote) and the requirements
     (what we will build). It is alive: after every client workshop /og:discovery updates the
     findings. A finding is NOT a requirement: no acceptance criteria, no priority, no UI; one
     finding becomes 1–3 requirements in the module's REQUIREMENTS.md later. -->

# [DSC-08] Guests, invitations, RSVP and Google Meet

## Topic scope

Inviting attendees (by e-mail, from contacts, directory and recent addresses), showing their status, required/optional guests, sending updates, responding to invitations (Accept/Decline/Maybe), organizer vs. attendee permissions, and adding or opening a Google Meet link.

## Sources in this topic

<Items from ALL sources that speak about this topic, side by side — so a contradiction between the
contract and the kick-off note is visible in one place, not in two files.>

| Item | Kind | Short |
| ---- | ---- | ----- |
| `SRC-01-60` | informal | Add a Google Meet link through Google Calendar API where permitted. |
| `SRC-01-67` | informal | Add attendees using email addresses. |
| `SRC-01-68` | informal | Suggest people from Google Contacts through Google People API. |
| `SRC-01-69` | informal | Optionally include Google Workspace directory search when authorized. |
| `SRC-01-70` | informal | Offer suggestions from recently used attendee email addresses. |
| `SRC-01-71` | informal | Show attendee names, addresses, and invitation status. |
| `SRC-01-72` | informal | Support required and optional attendees to the extent exposed by Google Calendar API. |
| `SRC-01-73` | informal | Add or remove attendees when editing an event. |
| `SRC-01-74` | informal | Send appropriate event updates through Google Calendar API. |
| `SRC-01-75` | informal | Identify invitations awaiting a response. |
| `SRC-01-76` | informal | Support Accept, Decline, and Maybe responses. |
| `SRC-01-77` | informal | Display attendee response statuses. |
| `SRC-01-78` | informal | Synchronize RSVP changes with Google Calendar. |
| `SRC-01-79` | informal | Respect permissions and organizer/attendee distinctions. |
| `SRC-01-102` | informal | Add attendees with Google Contacts suggestions. |
| `SRC-01-103` | informal | Accept, decline, or tentatively accept invitations. |
| `SRC-01-104` | informal | Google Meet link creation where supported. |
| `SRC-01-112` | informal | Advanced attendee free/busy scheduling. |
| `SRC-01-122` | informal | Create a meeting and invite attendees using contact suggestions. |
| `SRC-01-123` | informal | Receive an invitation and respond Accept, Decline, or Maybe. |
| `SRC-01-126` | informal | Create or open a Google Meet link. |
| `SRC-03-01` | informal | No reply with comment, no proposing another time. |
| `SRC-03-02` | informal | Ask every time whether to notify guests. |
| `SRC-03-03` | informal | Workspace directory in guest suggestions. |
| `SRC-03-04` | informal | One-click Join: event, agenda, notification. |
| `SRC-03-05` | informal | Invitations in disabled calendars not shown. |
| `SRC-03-06` | informal | Simple free/busy preview in the editor. |

## Key findings

- [DSC-08-01] Guests are added by e-mail address. (`SRC-01-67`)
- [DSC-08-02] While typing, the app suggests people from Google Contacts and recently used addresses. (`SRC-01-68`, `SRC-01-70`, `SRC-01-102`, `SRC-01-122`)
- [DSC-08-03] An event shows each guest's name, address and response. (`SRC-01-71`, `SRC-01-77`)
- [DSC-08-04] Guests can be required or optional, as far as Google allows. (`SRC-01-72`)
- [DSC-08-05] Guests can be added and removed when editing an event. (`SRC-01-73`)
- [DSC-08-06] Invitations waiting for a response are easy to find. (`SRC-01-75`)
- [DSC-08-07] An invitation is answered with Accept, Decline or Maybe, and the answer goes to Google. (`SRC-01-76`, `SRC-01-78`, `SRC-01-103`, `SRC-01-123`)
- [DSC-08-08] What the user can do with an event depends on whether they are its organizer or a guest, and on the permissions in Google. (`SRC-01-79`)
- [DSC-08-09] A Google Meet link can be added to an event where Google allows it. (`SRC-01-60`, `SRC-01-104`)
- [DSC-08-10] An event with Meet has a clear Join button that opens the meeting in the browser; the same button is in the agenda view and in the notification before the meeting. (`SRC-01-126`, `SRC-03-04`) — *source: `DEC-38`*
- [DSC-08-11] On Workspace accounts, guest suggestions also include the organization's directory. (`SRC-01-16`, `SRC-01-69`, `SRC-03-03`) — *source: `DEC-39`*
- [DSC-08-12] When a change to an event with guests is saved — also after dragging — the app asks once whether to send the guests an update. (`SRC-01-65`, `SRC-01-74`, `SRC-03-02`) — *source: `DEC-40`*
- [DSC-08-13] Invitations in a calendar disabled in the settings are not visible; that is the user's choice, and invitations normally land in the account's primary calendar. (`SRC-03-05`) — *source: `DEC-41`*
- [DSC-08-14] The event editor shows, under the chosen time, each guest's busy blocks for that day, with a conflict with the proposed time marked; guests who do not share availability show "no data"; there is no preview offline. (`SRC-01-112`, `SRC-03-06`) — *source: `DEC-42`*

## Contradictions

- Workspace directory "optionally, when authorized" (`SRC-01-69`) vs. absent from the MVP list (`SRC-01-102`) → settled: included on Workspace accounts, `DEC-39` (was `Q06`)

## Ambiguities

- "Create or open a Google Meet link" (`SRC-01-126`): (a) a link in the details, (b) one-click join → (b), `DEC-38` (was `Q09`)
- "Send appropriate event updates" (`SRC-01-74`): (a) always, (b) ask each time, (c) a setting → (b), `DEC-40` (was `Q30`)
- "Advanced attendee free/busy scheduling" is out (`SRC-01-112`), but a simple preview was not addressed → simple preview in the MVP, `DEC-42` (was `Q18`)

## From the calls, not the documents

- `SRC-03-01`…`SRC-03-06` — no reply with comment or new-time proposal; answers to `Q30`, `Q06`, `Q09`, `Q18`; invitations in disabled calendars

## For the client workshop

None open — `Q06`, `Q09`, `Q18`, `Q30` resolved on 2026-10-23. The Join button in notifications (`DSC-08-10`) is picked up in `DSC-09`.

## Parked

- Responding with a comment and proposing another time — rarely used (`SRC-03-01`).
- Advanced attendee free/busy scheduling — a multi-day grid, dragging on it, suggesting free slots (`SRC-01-112`) — outside the MVP; a simple preview is in, `DEC-42`.
