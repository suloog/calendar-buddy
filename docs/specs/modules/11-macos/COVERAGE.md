<!-- Copy to docs/specs/modules/NN-<slug>/COVERAGE.md. Content in the project language; headings fixed.
     One file per module, two parts: requirement → user stories (written by /og:stories) and
     acceptance criterion → tests (written by /og:tests). The `og:part` markers stay — og-status reads them. -->

# Coverage — [MOD-11] macOS

> The maps the other way round from a story's **Covers** field and a test case's **Verifies** field.
> They stop a requirement quietly falling out when the module is sliced, and a criterion quietly
> falling out when the tests are written.
>
> Fill it in the same run that writes the story or the test case. A row with an empty "Covered by"
> is a requirement not yet sliced or a criterion with no test. The module is **sliced** when part 1
> has no empty row; it is **covered** when every row of part 2 has a test **or** an explicit
> justification in the "Note" column (e.g. "manual verification — email delivery").
>
> Source: `REQUIREMENTS.md`, `stories/`, `tests/` in this directory · Registers: `docs/specs/BACKLOG.md`,
> `docs/specs/TESTPLAN.md`

<!-- og:part stories -->

## Part 1 — Requirements → user stories

### Functional requirements

| Requirement | Covered by |
| ----------- | ---------- |
| `MOD-11-F01` | `US-11-01` |
| `MOD-11-F02` | `US-11-01`, `US-11-06` |
| `MOD-11-F03` | `US-11-01`, `US-11-06` |
| `MOD-11-F04` | `US-11-01` |
| `MOD-11-F05` | `US-11-01` |
| `MOD-11-F06` | `US-11-01` |
| `MOD-11-F07` | `US-11-01`, `US-11-06` |
| `MOD-11-F08` | `US-11-01` |
| `MOD-11-F09` | `US-11-02` |
| `MOD-11-F10` | `US-11-03` |
| `MOD-11-F11` | `US-11-04` |
| `MOD-11-F12` | `US-11-04` |

### UI/UX requirements

| Requirement | Covered by |
| ----------- | ---------- |
| `MOD-11-U01` | `US-11-01` |
| `MOD-11-U02` | `US-11-02` |
| `MOD-11-U03` | `US-11-05` |

<!-- og:part tests -->

## Part 2 — Acceptance criteria → tests

### User story acceptance criteria

`#n` is the criterion's position in the story file, counted from 1, with a Given/When/Then scenario counting as one.

| Criterion | Text (short) | Covered by | Automation | Note |
| --------- | ------------ | ---------- | ---------- | ---- |
| `US-11-01#1` | After `cargo install` and one start, `~/Applications` holds "Calendar Bud… | `TC-11-02` | manual-only | — |
| `US-11-01#2` | After a new version is installed and started once, the bundle holds the n… | `TC-11-01` | auto | — |
| `US-11-01#3` | A Google account keeps its token in the login Keychain; after a restart i… | `TC-11-04` | manual-only | real Google account and Keychain |
| `US-11-01#4` | Declining to let the app read its Keychain item shows that unlocking was… | `TC-11-04` | manual-only | real Keychain prompt |
| `US-11-01#5` | Turning on "Start at login" adds the launch agent; at the next login the… | `TC-11-03`, `TC-11-05` | auto, manual-only | the login itself is manual |
| `US-11-01#6` | A reminder shows a macOS notification with the event's title, start time… | `TC-11-05` | manual-only | Notification Center |
| `US-11-01#7` | With the theme following the system, switching macOS between light and d… | `TC-11-05` | manual-only | system setting |
| `US-11-01#8` | Opening the app while it runs brings its window forward and starts no sec… | `TC-11-02` | manual-only | Launch Services |
| `US-11-02#1` | Closing the window leaves a monochrome calendar icon in the menu bar and… | `TC-11-07` | manual-only | menu bar, Dock |
| `US-11-02#2` | The icon's menu opens the window, with the app back in the Dock, and quit… | `TC-11-07` | manual-only | menu bar |
| `US-11-02#3` | Started at login, the app shows only its menu bar icon, without opening t… | `TC-11-07` | manual-only | the login itself is manual |
| `US-11-02#4` | Opening the app from Spotlight, Launchpad or Finder while it is in the me… | `TC-11-07` | manual-only | Launch Services |
| `US-11-03#1` | A notification of the installed app is shown in the app's name and icon,… | `TC-11-08` | manual-only | Notification Center |
| `US-11-03#2` | Snooze 5 minutes shows it again 5 minutes later; Dismiss removes it for g… | `TC-11-06`, `TC-11-08` | auto, manual-only | the buttons themselves are manual |
| `US-11-03#3` | Join opens the event's Meet call in the browser. | `TC-11-06`, `TC-11-08` | auto, manual-only | the buttons themselves are manual |
| `US-11-03#4` | Clicking the notification opens its event in the app's window, also while… | `TC-11-06`, `TC-11-08` | auto, manual-only | the click itself is manual |
| `US-11-04#1` | After the Mac wakes from sleep, the calendars are synchronized within sec… | — | — | draft — tests after `/og:clarify` |
| `US-11-04#2` | Turning Wi-Fi off shows that the app is offline; turning it on synchroniz… | — | — | draft — tests after `/og:clarify` |
| `US-11-04#3` | A `webcal:` link clicked in a browser or mail opens Google Calendar's "ad… | — | — | draft — tests after `/og:clarify` |
| `US-11-05#1` | With the window closed, a click on the menu bar icon opens it in front, … | `TC-11-09` | manual-only | menu bar, Dock |
| `US-11-05#2` | With the window shown behind other apps, a click on the icon brings it t… | `TC-11-09` | manual-only | menu bar |
| `US-11-05#3` | A right click, or a click with Control held, shows the menu with "Open C… | `TC-11-09` | manual-only | menu bar |
| `US-11-06#1` | Started from `~/.cargo/bin` or with `cargo run`, the app brings its bundl… | `TC-11-10` | manual-only | terminal, Notification Center |
| `US-11-06#2` | With a "Calendar Buddy" code signing certificate, the next new version's… | `TC-11-10` | manual-only | real Keychain and certificate |
| `US-11-06#3` | Within a run the app asks about each account's Keychain item at most onc… | `TC-11-10` | manual-only | real Keychain prompt; one read per run unit-tested (`read_once`) |

### Module constraints and edge cases

| Constraint | Covered by | Note |
| ---------- | ---------- | ---- |
| The home directory is not known → no bundle and no launch agent are writ… | — | smoke depth — not tested yet (`DEC-65`) |
| The program runs from inside the bundle → it is its own copy; nothing is… | `TC-11-02` | — |
| The bundle's program is replaced while it runs → the running copy keeps… | — | smoke depth — not tested yet (`DEC-65`); the copy is written to a new file renamed into place |
| A new version was installed → signed with the user's certificate, nothin… | `TC-11-10` | — |
| Notifications not allowed for the app in *System Settings → Notifications… | — | the system decides; not tested |
| A program run outside the bundle (`cargo run`, `~/.cargo/bin`) → it runs… | `TC-11-10` | — |
| "Quit" in the app's menu or Cmd+Q while the window is shown → the app en… | — | smoke depth — not tested yet (`DEC-65`) |
| "Start at login" turned on → it starts the bundle's copy, which the app r… | `TC-11-03` | — |
| Opening the app while it runs → macOS brings the running copy forward; n… | `TC-11-02` | — |
| No network → until `MOD-11-F11` the app counts as online, and a failed s… | — | smoke depth — not tested yet (`DEC-65`) |
