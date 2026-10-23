<!-- Copy to docs/specs/discovery/DISCOVERY.md. Content in the project language; headings fixed. -->

# Discovery — topic map

> What we understood from the sources, topic by topic, and what we agree on before the first
> requirement is written. `sources/` says what the client wrote; this directory says how we read
> it; `modules/` says what we will build. The topic file `discovery/NN-<topic>.md` is the source of
> truth; this file is its **mirror** and changes in the same run.
>
> **Business topics** (`kind: business`) are the future modules — `/og:modules` turns them into the
> split. **Cross-cutting topics** (`kind: cross-cutting`: roles and access, technology and
> integrations, non-functional requirements) feed PROJECT.md, NFR.md and the constitution; they are
> never a module. By convention they come first in the numbering.
>
> **Status:** `open` — not discussed · `discussed` — discussed with the analyst, waiting on the
> client · `settled` — no `[W]` question in the topic is open.
> The client questions from every topic, in a form ready to send: `AGENDA.md`.

## Sessions

| Date | Who | Topics | Outcome |
| ---- | --- | ------ | ------- |
| 2026-10-22 | analyst + agent | `DSC-01` | 17 findings (1 withdrawn), 9 questions resolved, 10 decisions (`DEC-01`…`DEC-10`, `DEC-04` superseded by `DEC-10`), new source `SRC-02` |
| 2026-10-22 | analyst + agent | `DSC-02` | 13 findings, 4 questions resolved, 8 decisions (`DEC-11`…`DEC-18`), technical checklist 24/24 |
| 2026-10-22 | analyst + agent | `DSC-03` | 8 findings, 1 question resolved (`Q24`), 1 added (`Q36`), 2 decisions (`DEC-19`, `DEC-20`) |
| 2026-10-22 | analyst + agent | `DSC-04` | 14 findings, 5 questions resolved, 6 decisions (`DEC-21`…`DEC-26`) |
| 2026-10-22 | analyst + agent | `DSC-05` | 14 findings, 4 questions resolved, 4 decisions (`DEC-27`…`DEC-30`); note added to `DSC-04` |
| 2026-10-22 | analyst + agent | `DSC-06` | 6 findings, 2 questions resolved, 2 decisions (`DEC-31`, `DEC-32`) |
| 2026-10-22 | analyst + agent | `DSC-01` (revisit) | `DSC-01-11` withdrawn, `DSC-01-18` added; `DEC-33` supersedes `DEC-03` (egui first, Iced fallback) |
| 2026-10-22 | analyst + agent | `DSC-07` | 14 findings, 4 questions resolved, 4 decisions (`DEC-34`…`DEC-37`) |
| 2026-10-23 | analyst + agent | `DSC-08` | 14 findings, 4 questions resolved, 5 decisions (`DEC-38`…`DEC-42`), new source `SRC-03` |
| 2026-10-23 | analyst + agent | `DSC-09` | 14 findings, 3 questions resolved, 4 decisions (`DEC-43`…`DEC-46`) |
| 2026-10-23 | analyst + agent | close | actor, glossary, scope, goals, risks, stack settled; `PROJECT.md` and draft `constitution.md` written; 2 decisions (`DEC-47`, `DEC-48`) |

## Topics

| Topic | Title | Kind | Sources | Findings | Open questions | Status | Module |
| ----- | ----- | ---- | ------- | -------- | -------------- | ------ | ------ |
| `DSC-01` | Product, roles, platform and integrations | cross-cutting | `SRC-01`, `SRC-02` | 18 | 0 [W] / 0 [A] | settled | — |
| `DSC-02` | Non-functional requirements | cross-cutting | `SRC-01`, `SRC-02` | 13 | 0 [W] / 0 [A] | settled | — |
| `DSC-03` | Google accounts and sign-in | business | `SRC-01`, `SRC-02` | 8 | 0 [W] / 0 [A] | settled | — |
| `DSC-04` | Calendars, synchronisation and offline cache | business | `SRC-01`, `SRC-02` | 14 | 0 [W] / 0 [A] | settled | — |
| `DSC-05` | Views and navigation | business | `SRC-01`, `SRC-02` | 14 | 0 [W] / 0 [A] | settled | — |
| `DSC-06` | Visual direction and themes | business | `SRC-01`, `SRC-02` | 6 | 0 [W] / 0 [A] | settled | — |
| `DSC-07` | Creating and editing events | business | `SRC-01`, `SRC-02` | 14 | 0 [W] / 0 [A] | settled | — |
| `DSC-08` | Guests, invitations, RSVP and Google Meet | business | `SRC-01`, `SRC-03` | 14 | 0 [W] / 0 [A] | settled | — |
| `DSC-09` | Reminders and notifications | business | `SRC-01`, `SRC-02`, `SRC-03` | 14 | 0 [W] / 0 [A] | settled | — |

## Source items with no topic

<`SRC-NN-nn` items that belong to no topic. The list must be empty before discovery closes — a
formal item with no topic is scope that will fall out of the requirements. The sources' "Noted but
not scope" sections do not count here.>

None — all 129 items of `SRC-01` are assigned to a topic.

## Closing

<Filled by `/og:discovery` in close mode. Condition: every business topic discussed, every `[W]`
question answered or deferred by an explicit decision, the list of items with no topic empty.>

- Closed: 2026-10-23, by: Maciej (analyst and product owner) · `PROJECT.md` written, `status: draft` (becomes `approved` with `/og:discovery-report accepted`) · `constitution.md` `status: draft` · 9 of 9 topics settled, 0 open questions, technical checklist 24/24, anchors fixed in `DEC-47`, `DEC-48` → next step `/og:discovery-report`
