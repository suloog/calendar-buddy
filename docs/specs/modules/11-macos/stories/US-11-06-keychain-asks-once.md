---
id: US-11-06 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-11
covers: [MOD-11-F02, MOD-11-F03, MOD-11-F07] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-11-01, US-11-03] # story IDs that must be done first
---

# As a User on a Mac, the Keychain asks me about the app's sign-ins once, not after every update or start from the terminal, so that installing a new version doesn't bury me in password prompts

|  |  |
| --- | --- |
| **Module** | `[MOD-11]` macOS — `docs/specs/modules/11-macos/REQUIREMENTS.md` |
| **Covers** | `MOD-11-F02`, `MOD-11-F03`, `MOD-11-F07` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-11-01`, `US-11-03` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Started from the terminal, the app asked for the Keychain's password several times: each account's item was read by the sign-in checks and by the synchronization side by side, the program in `~/.cargo/bin` and its copy in the bundle were two apps to the Keychain, and a bundle signed ad hoc was a new app after every build. Now the program started outside its bundle runs as the bundle's copy, the bundle is signed with a certificate of the user's own when there is one, and each item is read once in a run (`DEC-224`).

## Acceptance criteria

- [ ] Started from `~/.cargo/bin` or with `cargo run`, the app brings its bundle up to date and runs as the bundle's copy, in the same terminal; its notifications have the buttons of `US-11-03`.
- [ ] With a "Calendar Buddy" code signing certificate made in Keychain Access, the next new version's bundle is signed with it; once "Always Allow" was chosen for each account's item, installing and starting a later version asks the Keychain nothing.
- [ ] Within a run the app asks about each account's Keychain item at most once, however many of its parts read the token.

## Out of scope / what not to worry about

- A certificate from an Apple developer account, notarization, or a packaged app (`DEC-217`).
- Choosing **Deny** — the app says that unlocking was declined (`US-11-01`), and asks again at the next check.
- Linux, where the program runs as it was started.

## Technical notes

- `platform/macos.rs`: `SystemLauncher::run_from_bundle` (called first in `main`) installs the bundle and `exec`s its copy; `install` signs the bundle when `codesign --verify --strict` fails, with the identity `security find-identity -p codesigning` lists as "Calendar Buddy", or ad hoc; `TOKENS` and `read_once` keep each token read in the run.
- Decisions: `DEC-47`, `DEC-217`, `DEC-218`, `DEC-224`.
