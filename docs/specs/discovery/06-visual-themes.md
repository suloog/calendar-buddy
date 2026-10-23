---
id: DSC-06
title: Visual direction and themes
kind: business # business (a functional area — becomes a module at /og:modules) | cross-cutting (roles and access, technology and integrations, non-functional requirements — feeds PROJECT.md, NFR.md and the constitution, never a module)
status: settled # open (not yet discussed) | discussed (discussed with the analyst, waiting on the client) | settled (no [W] question in this topic is open)
sources: [SRC-01, SRC-02] # SRC-NN documents whose items this topic gathers, e.g. [SRC-01, SRC-03]
module: # MOD-NN — filled by /og:modules; empty for cross-cutting topics
sessions: [2026-10-22] # dates of the discovery sessions in which the topic was discussed, YYYY-MM-DD
---

<!-- Copy to docs/specs/discovery/NN-<topic>.md. Content in the project language; headings fixed.
     This file records what we UNDERSTOOD about this topic after reading every source and talking
     to the analyst — the layer between the sources (what the client wrote) and the requirements
     (what we will build). It is alive: after every client workshop /og:discovery updates the
     findings. A finding is NOT a requirement: no acceptance criteria, no priority, no UI; one
     finding becomes 1–3 requirements in the module's REQUIREMENTS.md later. -->

# [DSC-06] Visual direction and themes

## Topic scope

The look and feel: the Morgen and Apple Calendar references, light and dark themes following the system, restrained use of calendar colours, no unnecessary animation, and keyboard shortcuts as listed with themes in the MVP scope. Advanced theming is out. May merge into DSC-05 at /og:modules.

## Sources in this topic

<Items from ALL sources that speak about this topic, side by side — so a contradiction between the
contract and the kick-off note is visible in one place, not in two files.>

| Item | Kind | Short |
| ---- | ---- | ----- |
| `SRC-01-02` | informal | The experience should combine the visual clarity of Apple Calendar with the modern layout and workflows of… |
| `SRC-01-49` | informal | Inspired by Morgen for layout density, clear scheduling grids, and modern aesthetics. |
| `SRC-01-50` | informal | Inspired by Apple Calendar for simplicity and intuitive interactions. |
| `SRC-01-51` | informal | Provide light and dark themes, optionally following the system setting. |
| `SRC-01-52` | informal | Preserve useful calendar color coding while keeping the interface restrained. |
| `SRC-01-53` | informal | Avoid unnecessary animations and unrelated productivity features in the MVP. |
| `SRC-01-107` | informal | Light/dark themes and keyboard shortcuts. |
| `SRC-01-117` | informal | Advanced theming and layout customization. |
| `SRC-02-41` | informal | English-only interface in the MVP. |
| `SRC-02-42` | informal | No mock-ups; refined iteratively on the working app. |

## Key findings

- [DSC-06-01] The look combines the clarity and simplicity of Apple Calendar with the density and clear grid of Morgen. (`SRC-01-02`, `SRC-01-49`, `SRC-01-50`)
- [DSC-06-02] The app has a light and a dark theme and can follow the system setting. (`SRC-01-51`, `SRC-01-107`)
- [DSC-06-03] Calendar colours help tell calendars apart, while the rest of the interface stays restrained. (`SRC-01-52`)
- [DSC-06-04] There are no unnecessary animations and no features unrelated to the calendar. (`SRC-01-53`)
- [DSC-06-05] The interface is in English only; adding translations later stays possible. (`SRC-02-41`) — *source: `DEC-31`*
- [DSC-06-06] There are no mock-ups; the look is refined on the working app, with the product owner reviewing it as it grows. (`SRC-02-42`) — *source: `DEC-32`*

## Contradictions

None.

## Ambiguities

- Keyboard shortcuts appear with themes in `SRC-01-107`; they are recorded once, in `DSC-05-05`.

## From the calls, not the documents

- `SRC-02-41`, `SRC-02-42` answers to `Q20`, `Q31`

## For the client workshop

None open — `Q20`, `Q31` resolved on 2026-10-22.

## Parked

- Advanced theming and layout customization (`SRC-01-117`) — outside the MVP per the PRD.
- Translations of the interface (including Polish) — after the MVP, `DEC-31`.
- A UI design deliverable (mock-ups, design system) — not planned, `DEC-32`.
