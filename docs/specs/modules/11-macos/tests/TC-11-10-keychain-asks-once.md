---
id: TC-11-10 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-11
covers: [US-11-06#1, US-11-06#2, US-11-06#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [TC-11-04] # test case IDs that must pass first
---

# [TC-11-10] On a Mac, the app runs as its bundle's copy and the Keychain asks about each sign-in once, across updates

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-11-06#1`, `US-11-06#2`, `US-11-06#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | `TC-11-04` |

## Preconditions

- A Mac with the app installed and at least one connected Google account; the app quit.
- In Keychain Access, a self-signed code signing certificate named "Calendar Buddy" (*Certificate Assistant → Create a Certificate…*, "Self Signed Root", "Code Signing").

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Run `cargo install --path crates/app --locked`, then `~/.cargo/bin/calendar-buddy`. | macOS asks once whether `codesign` may use the key "Calendar Buddy" (choose **Always Allow**); then once for each account's item whether the app may read it (choose **Always Allow**) — not more often. | screenshot |
| 2 | In another terminal, run `ps -o command= -p $(pgrep -f "Calendar Buddy.app")` and `codesign -dv ~/Applications/Calendar\ Buddy.app 2>&1 \| grep Authority`. | The process is the bundle's copy; the first authority is "Calendar Buddy". | terminal output |
| 3 | Wait for a reminder, or snooze one. | The notification has Join (with Meet), Snooze and Dismiss. | screenshot |
| 4 | Quit the app; change any line of the program, run the same `cargo install` and start `~/.cargo/bin/calendar-buddy` again. | The Keychain asks nothing; the accounts are connected. | screenshot |
| 5 | Quit the app and run `cargo run -p calendar-buddy`. | The development build runs as `~/Applications/Calendar Buddy (dev).app`, in the terminal it was started from. | terminal output |

## Pass criterion

- After the first **Always Allow** for each item, no start — from the terminal, after an update — asks the Keychain again; and within a run no item is asked about twice.

## Cleanup

- Undo the change to the program and install it again.
