---
id: MOD-06
title: Guests, invitations, RSVP and Google Meet
ticket: # optional YouTrack ID, filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | done
depends_on: [MOD-05] # other module IDs
sources: [SRC-01, SRC-03, SRC-05] # SRC-NN documents this module draws on, e.g. [SRC-01, SRC-03]
---

<!-- Copy to docs/specs/modules/NN-<slug>/REQUIREMENTS.md. Content in the project language; headings
     fixed by this template. Delete sections that have no content; never write "none". -->

# [MOD-06] Guests, invitations, RSVP and Google Meet

## Context

Meetings with other people: adding guests with suggestions from contacts, the Workspace directory and recent addresses, responding to invitations, asking before guests are notified of a change, a simple free/busy preview of guests, "Meet with…" in the side panel laying chosen people's calendars over the user's own, and Google Meet with a one-click Join. Answers for the scope area "Guests, invitations, RSVP and Google Meet"; comes from `DSC-08` (`discovery/08-guests-rsvp-meet.md`), drawing on `SRC-01` and `SRC-03`; "Meet with…" comes from `SRC-05` (`DEC-173`).

## Scope coverage

| Source item | Kind | Delivered by |
| ----------- | ---- | ------------ |
| `SRC-01-60` Add a Google Meet link through Google Calendar API where permitted. | informal | `MOD-06-F16` |
| `SRC-01-67` Add attendees using email addresses. | informal | `MOD-06-F01` |
| `SRC-01-68` Suggest people from Google Contacts through Google People API. | informal | `MOD-06-F02` |
| `SRC-01-69` Optionally include Google Workspace directory search when authorized. | informal | `MOD-06-F04` |
| `SRC-01-70` Offer suggestions from recently used attendee email addresses. | informal | `MOD-06-F03` |
| `SRC-01-71` Show attendee names, addresses, and invitation status. | informal | `MOD-06-F05` |
| `SRC-01-72` Support required and optional attendees to the extent exposed by Google Calendar API. | informal | `MOD-06-F06` |
| `SRC-01-73` Add or remove attendees when editing an event. | informal | `MOD-06-F07` |
| `SRC-01-74` Send appropriate event updates through Google Calendar API. | informal | `MOD-06-F08`, `MOD-06-F12` |
| `SRC-01-75` Identify invitations awaiting a response. | informal | `MOD-06-F09`, `MOD-06-F10` |
| `SRC-01-76` Support Accept, Decline, and Maybe responses. | informal | `MOD-06-F11` |
| `SRC-01-77` Display attendee response statuses. | informal | `MOD-06-F05` |
| `SRC-01-78` Synchronize RSVP changes with Google Calendar. | informal | `MOD-06-F11` |
| `SRC-01-79` Respect permissions and organizer/attendee distinctions. | informal | `MOD-06-F15` |
| `SRC-01-102` Add attendees with Google Contacts suggestions. | informal | `MOD-06-F02` |
| `SRC-01-103` Accept, decline, or tentatively accept invitations. | informal | `MOD-06-F11` |
| `SRC-01-104` Google Meet link creation where supported. | informal | `MOD-06-F16` |
| `SRC-01-112` Advanced attendee free/busy scheduling. | informal | `MOD-06-F19`, `MOD-06-F20`; advanced scheduling not built (`DEC-42`, `MOD-08`) |
| `SRC-01-122` Create a meeting and invite attendees using contact suggestions. | informal | `MOD-06-F01`, `MOD-06-F02` |
| `SRC-01-123` Receive an invitation and respond Accept, Decline, or Maybe. | informal | `MOD-06-F11` |
| `SRC-01-126` Create or open a Google Meet link. | informal | `MOD-06-F18` |
| `SRC-03-02` When an event with guests is changed, the app asks every time whether to send an update… | informal | `MOD-06-F12` |
| `SRC-03-03` On Workspace accounts, guest suggestions also include the organization's directory. | informal | `MOD-06-F04` |
| `SRC-03-04` "Open a Meet link" means joining with one click: a clear Join button on the event, also… | informal | `MOD-06-F18`; Join in notifications in `MOD-07` |
| `SRC-03-05` Invitations in a calendar disabled in the settings are not visible; disabling is the us… | informal | `MOD-06-F21` |
| `SRC-03-06` A simple free/busy preview of guests is wanted in the MVP: busy blocks per guest for th… | informal | `MOD-06-F19`, `MOD-06-F20` |
| `SRC-05-01` What the app has today as availability is hard to use in a sensible way. | informal | `MOD-06-F23`…`MOD-06-F27` |
| `SRC-05-02` The side panel gets "Meet with…" as in Google Calendar: the user enters people and t… | informal | `MOD-06-F23`, `MOD-06-F24`, `MOD-06-F26` |
| `SRC-05-03` The availability preview in the event editor stays; "Meet with…" comes in addition t… | informal | `MOD-06-F19`, `MOD-06-F20` (unchanged) |
| `SRC-05-04` A person's calendar shows as in Google: event details where the person shares them,… | informal | `MOD-06-F25` |
| `SRC-05-05` In the day view the user and the chosen people are columns side by side, as in Googl… | informal | `MOD-06-U05` |
| `SRC-05-06` The chosen people are not remembered: after a restart the list is empty, as Google's… | informal | `MOD-06-F27` |

## Functional requirements

Every requirement in this module is priority **must** unless marked otherwise.

### Guests

- [MOD-06-F01] The user can add guests to an event by e-mail address.
- [MOD-06-F02] While the user types, the app suggests people from the account's Google Contacts.
- [MOD-06-F03] The suggestions also include addresses the user has recently invited.
- [MOD-06-F04] On Workspace accounts, the suggestions also include people from the organization's directory. — *source: `DEC-39`*
- [MOD-06-F05] An event's details show each guest's name, address and response.
- [MOD-06-F06] A guest can be marked as required or optional.
- [MOD-06-F07] The user can add and remove guests when editing an event.
- [MOD-06-F08] Saving an event with guests sends them invitations or updates through Google, according to the user's answer in `MOD-06-F12`.

### Invitations and responses

- [MOD-06-F09] Invitations awaiting the user's response are marked as such in every view. — *source: `DEC-61`*
- [MOD-06-F10] The side panel shows how many invitations await a response; it opens a list of them to respond from. — *source: `DEC-61`*
- [MOD-06-F11] The user can respond to an invitation with Accept, Decline or Maybe; the response is sent to Google.
- [MOD-06-F12] When the user saves a change to an event with guests — also after dragging — or deletes one, the app asks once whether to send the guests an update. — *source: `DEC-40`*
- [MOD-06-F13] Responding to an occurrence of a recurring invitation asks whether the answer applies to this event or to all events. — *source: `DEC-61`*
- [MOD-06-F14] Events the user declined stay in the views, struck through and faded. — *source: `DEC-61`*
- [MOD-06-F22] One easy switch in the views shows or hides declined events; they are shown by default. — *source: `DEC-62`*
- [MOD-06-F15] When the user is a guest, they can respond but cannot change the event's time, details or guests unless the organizer allowed guests to.
- [MOD-06-F21] Invitations that arrive in a calendar disabled in the settings are neither shown nor counted. — *source: `DEC-41`*

### Google Meet

- [MOD-06-F16] The user can add a Google Meet link to an event, where the account allows it.
- [MOD-06-F17] The event editor's "Add Google Meet" switch turns on by itself when the first guest is added; without guests it is off by default. — *source: `DEC-61`*
- [MOD-06-F18] An event with a Google Meet link has a Join button, in its details and in the agenda, that opens the meeting in the browser. — *source: `DEC-38`*

### Availability of guests

- [MOD-06-F19] In the event editor, under the chosen time, the app shows each guest's busy blocks for that day and marks a conflict with the chosen time. — *source: `DEC-42`*
- [MOD-06-F20] Guests who do not share their availability are shown as "no data", and without network the preview is not shown. — *source: `DEC-42`*

### Meeting with people

- [MOD-06-F23] The side panel has a "Meet with…" field; typing in it suggests people as adding guests does (`MOD-06-F02`…`F04`), and a typed address can be chosen too. — *source: `DEC-173`*
- [MOD-06-F24] The chosen people are listed under the field, each with a way to remove them, and their calendars are shown in the day, 3-day and week views over the user's own. — *source: `DEC-173`*
- [MOD-06-F25] A person's event shows its title and time where their calendar shares details with the account, and is a "Busy" block where it shares only free/busy; a person whose calendar shares nothing is marked in the list as such. A person's events are read-only; one with details opens them read-only. — *source: `DEC-173`*
- [MOD-06-F26] **Create event** under the list opens the full editor with the chosen people as guests, starting as the `c` key does: on the selected day at the next full hour, with the default length and calendar. — *source: `DEC-173`, `DEC-174`*
- [MOD-06-F27] The chosen people are not remembered across restarts; without network the field is unavailable and says why, and the people's calendars are not shown. — *source: `DEC-173`*

## UI/UX requirements

- [MOD-06-U01] The guest list shows each response as an icon: accepted, declined, maybe, awaiting. — *source: `DEC-63`*
- [MOD-06-U02] The availability preview is one horizontal strip per guest, with the chosen time highlighted across all strips. — *source: `DEC-63`*
- [MOD-06-U03] An invitation's details open with Accept, Maybe and Decline at the top. — *source: `DEC-63`*
- [MOD-06-U04] An invitation awaiting a response has a dashed outline and a light fill in the views. — *source: `DEC-61`, `DEC-63`*
- [MOD-06-U05] With people chosen in "Meet with…", the day view shows the user and each person as columns side by side, headed by name; the 3-day and week views lay a person's events out beside the user's like overlapping events. A person's all-day events are in the all-day strip (in the day view, in their column). — *source: `DEC-173`, `DEC-174`*
- [MOD-06-U06] Each chosen person has a colour of their own, shown next to their name in the list and on their events and busy blocks. — *source: `DEC-173`*

## Non-functional requirements

- ~~[MOD-06-NFR-01] Guest suggestions appear within half a second of typing, online.~~ — withdrawn, `DEC-62`

## Constraints and edge cases

- An address with an invalid format → it is not added, and the field says why.
- The same guest is added twice, or the user adds their own address → added once / ignored.
- An event has many guests → the guest list is collapsed beyond the first 10; the availability preview shows the first 20 guests. — *source: `DEC-63`*
- Google cannot create a Meet link for the account → the event is saved without it, and the app says so.
- Directory permission was not granted at sign-in → directory suggestions are unavailable for that account (`DEC-52`).
- Responding or editing guests without network → unavailable (`MOD-02-F15`).
- The organizer changes or cancels the event → the change arrives through synchronization; a cancelled event disappears.
- The user chooses themselves, or a person already chosen, in "Meet with…" → ignored. — *source: `DEC-173`*
- The user moves to another day or week with people chosen → their calendars are fetched for the new range; until Google answers, only the user's events are shown. — *source: `DEC-173`*
- The network drops with people chosen → their calendars disappear from the views and the list stays; they come back with the network. — *source: `DEC-173`*

## Integrations

- **Google Calendar API — events** (write): guests, the user's own response, sending updates to guests, creating a Google Meet conference.
- **Google Calendar API — free/busy** (read): busy blocks of guests for one day; busy blocks of people chosen in "Meet with…" whose calendars share only free/busy.
- **Google Calendar API — events** (read): the events of people chosen in "Meet with…", as far as their calendars are shared with the account.
- **Google People API** (read): contacts, "other contacts", and the organization directory on Workspace accounts.

## Technical notes

- The answer of `MOD-06-F12` maps to `sendUpdates=all` or `none` on update and delete.
- A response patches the user's own attendee entry (`responseStatus`: `accepted`, `declined`, `tentative`); "this event" patches the instance, "all events" the series master.
- Meet: `conferenceDataVersion=1` with a `createRequest` of type `hangoutsMeet`; the Join button uses the conference entry point.
- Suggestions: People API `searchContacts` and `otherContacts.search` (with the warm-up request the API requires), and `people.searchDirectoryPeople` on Workspace accounts only; recently invited addresses come from the guests of cached events.
- Availability: `freebusy.query` for the chosen day, with the guests' addresses as items.
- "Meet with…": `events.list` on the person's address as the calendar ID for the shown range (single events expanded), through the account the suggestion came from, or the default calendar's account for a typed address; events without a title (free/busy-only access) are "Busy". When Google refuses the calendar, `freeBusy.query` for the range; when that answers `notFound` too, the person is marked as not visible. The people's events are held in memory, not in the cache, and are fetched again when the people or the range change and with every synchronization round.
- "Invitation awaiting response" = the user's attendee entry has `responseStatus: needsAction`.

## Out of scope / what not to worry about

- Responding with a comment, proposing another time — `SRC-03-01`, `MOD-08`.
- Suggesting free times and dragging over other people's time — `DEC-42`, `DEC-173`, `MOD-08`.
- Other people's calendars in the month and agenda views — `DEC-173`.
- Setting guest permissions (guests may modify the event / invite others / see the guest list) — Google's defaults are kept. — *source: `DEC-63`*
- Event fields, recurrence and the editor itself — `MOD-05`.
- The Join button in notifications — `MOD-07`.

## Acceptance criteria

- [ ] Typing part of a colleague's name suggests them from contacts or, on a Workspace account, the directory; the event saved with them and with Meet sends them an invitation with a Meet link.
- [ ] Guests' responses are shown in the event's details and change when a guest answers.
- [ ] An invitation from someone else is marked in the week view and counted in the side panel; accepting it from the list updates it in Google.
- [ ] Declining one occurrence of a recurring invitation leaves the other occurrences unanswered; the declined one is struck through, and the switch hides and shows it again.
- [ ] Moving an event with guests asks whether to notify them; answering "no" sends no e-mail.
- [ ] In the editor, a colleague's meeting at the chosen time shows as a conflict in the availability preview.
- [ ] The Join button opens the meeting in the browser from the event and from the agenda.
- [ ] Choosing two colleagues in "Meet with…" shows their calendars beside the user's in the week view and as columns in the day view; **Create event** opens the editor with them as guests.
