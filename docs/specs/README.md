# Specification — calendar-buddy

Not sure what's next? `/og:guide` in Claude Code. Full description of the work: the analyst handbook in the og plugin
(plugin/docs/handbook.en.md).

## Phases
[ ]  Analysis — sources register        /og:analyze          → SOURCES.md, sources/, QUESTIONS.md
[ ]  Discovery — workshop by topic      /og:discovery        → discovery/, at closing PROJECT.md + draft constitution.md
[ ]  Discovery report for the client    /og:discovery-report → docs/reports/discovery-<date>/*.pdf
[ ]  Modules — split (tech lead)        /og:modules          → modules/NN-<slug>/, NFR.md, constitution.md active
[ ]  Requirements (module by module)    /og:requirements MOD-NN → modules/NN-<slug>/REQUIREMENTS.md
[ ]  User stories (module by module)    /og:stories MOD-NN   → modules/NN-<slug>/stories/, BACKLOG.md
[ ]  Quality gate                       /og:clarify          → status: ready
[ ]  YouTrack / estimate                /og:sync-youtrack, /og:estimate-project
[ ]  Test cases (module by module)      /og:tests MOD-NN     → modules/NN-<slug>/tests/, TESTPLAN.md
[ ]  Test run and report                /og:run-tests        → test-runs/<date>/report.html
(optional) Autonomous delivery          /og:deliver, then /loop /og:deliver-next → docs/delivery/
After every client contact: /og:decide. Tree status: og-status.

## Tree
  PROJECT.md                     scope areas, actors, glossary, sources — language: pl | en
  constitution.md                engineering constitution
  NFR.md                         NFR-nn, project-wide
  SOURCES.md                     SRC-NN — register of client documents (formal | informal)
  sources/NN-<slug>.md           [SRC-NN-nn] items in the client's words, read-only
  discovery/DISCOVERY.md         topic map, sessions, closing
  discovery/AGENDA.md            questions for the client grouped by topic — to send
  discovery/NN-<topic>.md        [DSC-NN-nn] findings: what we understood, contradictions, ambiguities
  discovery/TECH-CHECKLIST.md    TQnn — technical checklist, answer status for each question
  QUESTIONS.md                   Qnn [A]/[W] — open questions for the client
  DECISIONS.md                   DEC-nn — every decision: who, when, on what basis
  BACKLOG.md                     every US, mirror
  TESTPLAN.md                    TC register — no results; results live in the run report
  modules/NN-<slug>/REQUIREMENTS.md   [MOD-NN] with MOD-NN-Fnn / Unn / NFR-nn, SRC item coverage
  modules/NN-<slug>/COVERAGE.md       requirement -> US · acceptance criterion -> TC
  modules/NN-<slug>/stories/US-NN-MM-<slug>.md
  modules/NN-<slug>/tests/TC-NN-MM-<slug>.md
