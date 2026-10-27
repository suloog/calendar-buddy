---
id: US-10-01 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-10
covers: [MOD-10-F19, MOD-10-F20, MOD-10-F21, MOD-10-F26, MOD-10-U04, MOD-10-U05, MOD-10-NFR-02] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
mode: A # A | B | C — agent autonomy · PE code review · PE pairs with the agent
depends_on: [] # story IDs that must be done first
---

# As a User, I can turn on events from a description and point the app at my Ollama and model, so that the description box works on my computer

|  |  |
| --- | --- |
| **Module** | `[MOD-10]` Events from a description — `docs/specs/modules/10-events-from-a-description/REQUIREMENTS.md` |
| **Covers** | `MOD-10-F19`, `F20`, `F21`, `F26`, `U04`, `U05`, `NFR-02` |
| **Priority** | must |
| **Mode** | A |
| **Status** | ready |
| **Depends on** | — |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The groundwork for describing events: a section in the settings to turn the function on or off and to say where Ollama runs and which model it uses, the app knowing whether Ollama answers and has the model, and a README tutorial on installing both. The box in the side panel is shown here only as available or unavailable with its reason; typing and sending a description is `US-10-02`.

## Acceptance criteria

- [ ] The settings have an "Events from a description" section with an on/off checkbox (on by default), the Ollama address (`http://localhost:11434` by default) and the model's name (`gemma4:e4b` by default); changes are kept after a restart.
- [ ] With the checkbox unticked, the description box, its `q` key and its entry in the list of shortcuts are gone; ticked again, they are back.
- [ ] An address that is not this computer (not `localhost`, `127.0.0.1` or `::1`) shows a warning in the section that descriptions will leave this computer; a local address shows none.
- [ ] With the default address, the app sends no request to anything but Google and this computer.
- [ ] The README has a section on installing Ollama and pulling `gemma4:e4b`, with links to ollama.com, and says the app works without them.

### Scenario: Ollama is not running

- **Given** the function is on and nothing answers at the Ollama address
- **When** the user looks at the side panel
- **Then** the description box is greyed out and says "Ollama is not available at <address>", with a link to the settings

### Scenario: the model is missing

- **Given** Ollama answers but does not have the model set in the settings
- **When** the user looks at the side panel
- **Then** the box is greyed out and says "The model <name> is not installed", with a link to the settings

### Scenario: Ollama starts later

- **Given** the box is unavailable because Ollama was not running
- **When** Ollama starts and the user goes to the box again
- **Then** the box is available, without restarting the app

## Edge cases

- Ollama stops while the box is open → the box turns unavailable and keeps what was typed.
- An address that is not a valid URL → the settings say so and keep the previous address.

## Out of scope / what not to worry about

- Typing, sending and reading a description — `US-10-02`.
- Installing Ollama or downloading a model from the app — `DEC-193`.
- Listing the models Ollama has to choose from; the name is typed.

## Technical notes

- Availability: Ollama's version and model-list endpoints; checked at start, when the settings change and when the box is focused — no polling loop.
- `NFR-06` gains the exception for the Ollama address (`DEC-207`).
- Settings keep the three values in the app's settings file, as the other settings.
- Manual check (left for the owner, as for the local loop's stories): with a real Ollama, stopping and starting it while the app runs; the warning's wording; the README tutorial read once from start to end.
- Decisions: `DEC-193`, `DEC-197`, `DEC-207`, `DEC-211`.
