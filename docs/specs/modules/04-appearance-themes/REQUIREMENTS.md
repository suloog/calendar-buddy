---
id: MOD-04
title: Appearance and themes
ticket: # optional YouTrack ID, filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | done
depends_on: [MOD-03] # other module IDs
sources: [SRC-01, SRC-02] # SRC-NN documents this module draws on, e.g. [SRC-01, SRC-03]
---

<!-- Copy to docs/specs/modules/NN-<slug>/REQUIREMENTS.md. Content in the project language; headings
     fixed by this template. Delete sections that have no content; never write "none". -->

# [MOD-04] Appearance and themes

## Context

How the app looks: the Morgen and Apple Calendar references, light and dark themes following the system, restrained use of colour, no unnecessary animation, and an English interface refined on the working app without mock-ups. Answers for the scope area "Visual direction and themes"; comes from `DSC-06` (`discovery/06-visual-themes.md`), drawing on `SRC-01` and `SRC-02`.

## Scope coverage

| Source item | Kind | Delivered by |
| ----------- | ---- | ------------ |
| `SRC-01-02` The experience should combine the visual clarity of Apple Calendar with the modern layo… | informal | `MOD-04-U01`, `MOD-04-U02` |
| `SRC-01-49` Inspired by Morgen for layout density, clear scheduling grids, and modern aesthetics. | informal | `MOD-04-U01` |
| `SRC-01-50` Inspired by Apple Calendar for simplicity and intuitive interactions. | informal | `MOD-04-U02` |
| `SRC-01-51` Provide light and dark themes, optionally following the system setting. | informal | `MOD-04-F01`, `MOD-04-F02` |
| `SRC-01-52` Preserve useful calendar color coding while keeping the interface restrained. | informal | `MOD-04-U03`, `MOD-04-U04` |
| `SRC-01-53` Avoid unnecessary animations and unrelated productivity features in the MVP. | informal | `MOD-04-U05`; no unrelated features (Out of scope) |
| `SRC-01-107` Light/dark themes and keyboard shortcuts. | informal | `MOD-04-F01`; keyboard shortcuts in `MOD-03-F23` |
| `SRC-02-41` The interface is in English only in the MVP; texts kept in one place so translations ca… | informal | `MOD-04-F04` |
| `SRC-02-42` No mock-ups before coding; Morgen and Apple Calendar are the references and the look is… | informal | no mock-up deliverable (`DEC-32`); see Technical notes |

## Functional requirements

Every requirement in this module is priority **must** unless marked otherwise.

- [MOD-04-F01] The app has a light theme and a dark theme.
- [MOD-04-F02] By default the app follows the system's light or dark setting and switches at once when the system changes it. — *source: `DEC-57`*
- [MOD-04-F03] In the settings the user chooses System, Light or Dark, and the choice is remembered. — *source: `DEC-57`*
- [MOD-04-F04] The interface is in English. — *source: `DEC-31`*

## UI/UX requirements

- [MOD-04-U01] The day and week grids are compact: at 100% scale, in a window 1080 pixels high, the hours from 8:00 to 18:00 are visible without scrolling. — *source: `DEC-57`, `DEC-58`*
- [MOD-04-U02] An event in the day, 3-day and week views shows only its title and time; everything else appears when it is opened. — *source: `DEC-58`*
- [MOD-04-U03] Apart from calendar colours, the interface uses neutral tones and one fixed accent colour. — *source: `DEC-57`*
- [MOD-04-U04] Events are drawn in their calendar's colour, with text that stays readable on that colour in both themes.
- [MOD-04-U05] Changing views and periods happens without transition animations. — *source: `DEC-58`*

## Non-functional requirements

- [MOD-04-NFR-01] Text in the interface and inside events has a contrast of at least 4.5:1 against its background in both themes (WCAG 2.1 AA). — *source: `DEC-58`*

## Constraints and edge cases

- A calendar colour is very light or very dark → the event text switches between dark and light so it stays readable (`MOD-04-U04`).
- The system changes between light and dark while the app runs → the app follows at once, unless the user chose Light or Dark.
- The desktop does not report a light or dark preference → the app uses the light theme. — *source: `DEC-58`*
- The screen scale is 150% or 200% → the compact grid scales with it and keeps its proportions (`MOD-03-U02`).

## Integrations

- **Operating system appearance setting** (read): the light or dark preference of the desktop, and its changes while the app runs.

## Technical notes

- Light and dark themes are custom egui visuals (neutral palette, one accent); with the Iced fallback (`DEC-33`), the same palette as an Iced theme.
- The light/dark preference is read from the XDG desktop portal (`org.freedesktop.appearance`, `color-scheme`) and its change signal, behind the platform interface.
- Event text colour is picked per event from the relative luminance of its background, to keep 4.5:1.
- Interface texts live in one table, so translations can be added later without touching the views (`DEC-31`).
- There are no mock-ups (`DEC-32`): the look is reviewed by the product owner on the working app, story by story, starting with the week-view feasibility check of `MOD-03`.

## Out of scope / what not to worry about

- Advanced theming and layout customization — `SRC-01-117`, `MOD-08`.
- A density setting and an accent colour taken from the system — `DEC-57`.
- Translations of the interface, including Polish — `DEC-31`.
- Mock-ups or a design system as a deliverable — `DEC-32`.
- Calendar colours and their local overrides — `MOD-02`; event colours — `MOD-05`.
- Features unrelated to the calendar (tasks, notes, AI assistant) — `SRC-01-53`, `SRC-01-116`, `MOD-08`.

## Acceptance criteria

- [ ] On a system set to dark, the app starts dark; switching the system to light switches the app at once.
- [ ] Choosing Light or Dark in the settings overrides the system and is kept after a restart.
- [ ] In a 1080-pixel-high window at 100% scale, the week view shows 8:00–18:00 without scrolling.
- [ ] Text inside events of very light and very dark calendar colours is readable in both themes, with at least 4.5:1 contrast.
- [ ] Apart from calendar colours, only neutral tones and one accent colour appear in the interface.
