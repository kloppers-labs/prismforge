# 0006 — Setup model and effects across the setup

- Status: accepted
- Date: 2026-11-20
- Scope: where each LED is, how the user declares mirrors and daisy chains, how an effect becomes one picture across several devices, and how the setup is stored. Plan 2, Task 10.

## Current state

The Rust core has the setup model, the planner that gives each LED a place, the 2-D effects, the apply mode "across the setup" in the session, the setup store and profile schema 2 (Task 10). There is no interface yet: the PC view and the desk view, the editor and their Tauri commands are Task 11. **Nothing here has run against a real OpenRGB server or real hardware.** The evidence comes from tests with generic fixtures, the demo backend and the scripted fake server in simulated time.

The code is in `crates/prismforge-core/src/setup` (the model, geometry, validation, controls, resolution and the planner), `crates/prismforge-core/src/store/setup.rs` (the file), `crates/prismforge-effects/src/plane.rs` (the 2-D projection) and the session's `ApplyAcross` command. The research behind it is `docs/research/setup-views-research.md`, sections 1, 2 and 5.

## The model

A setup has three lists.

| Part | Holds |
|---|---|
| Component | A lit thing with a declared shape: an id (`c-…`), a name and a geometry. |
| Output | One channel PrismForge writes: an id (`o-…`), a device and a zone (or the whole device) by stored reference, the device and zone names it had, the first LED, and its branches. |
| View | A named picture, such as "PC" or "Desk": an id (`v-…`), a name and placements. |

A **placement** puts one component in one view: its centre (`x`, `y`) in view units, the length of its longer side (`size`), a turn in whole degrees clockwise (`rotation`, 0 to 359) and a left-right flip before the turn (`flipped`). A component appears at most once in a view. It may be in several views, or in none: an unplaced component still belongs to its output.

**Each component belongs to one output.** An output is a whole device, a zone, or the part of a zone from `first_led`. Two outputs on one zone are two parts of it.

### Geometry classes

Each class has an LED count, a first LED and a direction. Local space is unit-less: the longer side runs from -0.5 to 0.5 around 0, and y grows downwards, as on the screen.

| Class | Parameters | LED order |
|---|---|---|
| Strip | LED count, the end LED 1 is at | From that end to the other, each LED in the middle of an equal share of the length |
| Ring | LED count, angle of LED 1 (degrees clockwise from the top), turn (clockwise or counter-clockwise) | Round the circle in the turn |
| Matrix | Columns, rows, the corner of LED 1, rows or columns first, serpentine | Line by line from the corner; with serpentine, every second line turns back |
| Perimeter | LEDs per side (top, right, bottom, left), width and height, the corner of LED 1, turn | Side by side round the rectangle from the corner in the turn |
| Point | none (one LED) | The middle |
| Free line | LED count, two to 256 points, the end LED 1 is at | Evenly along the path, by length |

A dual-ring fan is two ring components chained on one output (outer ring, then inner ring, as the wire runs), with the inner one placed smaller at the same centre. LED counts come from the user, never from a catalogue; the research shows why (a catalogue count was wrong for a common fan).

### Mirrors and daisy chains

PrismForge never guesses them from a picture or a count. The user declares them in the output's branches.

- **A daisy chain is one branch with several components.** The components follow each other on the wire: the first takes LEDs `first_led` to `first_led + count - 1`, the next starts after it. The zone is split into consecutive parts, one part per component. Example: three fans chained on one ARGB header, 12 LEDs each, are LEDs 0–11, 12–23 and 24–35 of that zone.
- **A mirrored output is several branches.** A passive splitter or a hub with fans wired in parallel gives every branch the same data from the output's first LED, so all of them always show the same colours. The output is **one control**: `Setup::control(component)` reports the same output and LEDs for every mirrored component and lists the others in `shared_with`. Example: three fans on a hub behind one header are one output with three branches of one fan each.
- **Both at once.** A splitter whose branches are chains is several branches of several components each. Branches may differ in length; a shorter branch shows only the first LEDs of the stream (the research's inference, to be measured).

Why branches and not a mirror flag: a branch list states the physics (the same stream to different fans) and says how many LEDs each branch uses. A flag cannot.

### Validation

Without devices, `Setup::validate` refuses: ids that are not valid or used twice; names that are not valid; a shape that is not valid (no LEDs, more than 4096, an angle of 360 or more, a free line with fewer than two points); an output or a branch with no component; a component wired twice or not in the setup; **two outputs that share LEDs** (two parts of one zone that overlap, or a whole device beside one of its zones); a component placed twice in a view, with size 0 or a turn of 360 or more; more than 512 components, 256 outputs, 16 views or 16 branches.

With the devices the session reports, `Setup::check_devices` refuses, per output of a device the session knows:

1. **An LED count outside the bounds the backend reports.** A component may have 1 to `leds_max` LEDs of its zone (the sum of every zone's maximum for a whole device). OpenRGB reports resizable ARGB headers with a size of 0 to 1024 (`docs/hardware-validation.md`); a fixed zone's maximum is its count.
2. **Parts beyond the LEDs the zone has now.** `first_led` plus the longest branch must fit the zone's current LED count.

PrismForge does not resize zones in OpenRGB. That is a hardware configuration write, and no task has decided it. A user whose fans need more LEDs than the zone reports resizes the zone in OpenRGB first.

## LEDs that no component covers

**Ruling: in "across the setup" mode, an LED of a zone that no placed component covers receives black, and the model reports it as not placed.** `PlacedTarget::not_placed()` lists those LEDs by index, so Task 11 can show them.

Why black: OpenRGB takes full frames only (`0001-openrgb-integration.md`), so the session must send a colour for every LED of the zone. The other choices each claim something the setup does not support. Stretching the picture over them invents places. Running them "each device" mixes two modes on one zone. Leaving the last colour freezes a frame that the user did not ask for. Black is the honest "this LED has no place in the picture", and the view says so. Reported counts are often not the real LED count (`docs/hardware-validation.md`): an ARGB header reports 36 LEDs whatever is connected, so unplaced LEDs are normal and must be visible, not hidden.

## Each LED's place: the sampling rule

`plan_across(setup, view, flow, devices)` builds an `AcrossPlan` for one view.

1. **Each LED of each placed component gets a view location:** the geometry's local point, then the placement (flip, scale by `size`, turn, move).
2. **The plane is the box round every wired and placed LED of the view**, normalised on its longer side, so the longer side is 1 and (0, 0) is the middle. The plane does not depend on which devices are found today, so a missing device never moves the picture. Normalising on the longer side keeps angles and circles true; a fixed canvas would make an effect's scale depend on an empty area.
3. **Each target (device or zone) gets a place for each LED, by LED index.** A part takes its LEDs from its own component. For a mirrored output, an LED takes its place from the first branch that places it; the other branches show the same colours and add no samples. LEDs of the zone beyond the parts, and LEDs whose component is not in this view, have no place.
4. **The pitch of the plane** is the widest gap between neighbouring LEDs of one component, on the same scale. Effects use it as a minimum feature width.
5. **Which outputs get nothing.** `resolve_outputs` reports each output with the groups' states (resolved, unavailable, missing, ambiguous, changed) and any problem with its parts today. An output whose zone changed (the zone at that index has another name) or whose parts no longer fit is **excluded**: it is listed in `plan.excluded` and gets no write. Outputs of missing, unavailable or ambiguous devices stay in the plan, so the session refuses them with its own reason, as in a batch. A target with no placed LED in the view is not part of the plan.

The research suggested a footprint filter (five samples per LED). Task 10 samples at the LED's point and raises the moving highlight's band to at least one pitch on each side of its centre, which is the same minimum width the strip renderer keeps. The filter is left for measurement on real hardware.

## Effects in two dimensions

An effect stays a pure function of its values, the time and the position (`0003-effects-engine.md`). The position across a setup is a point on the plane. A **flow** says how the effect crosses it:

- `Along { angle }`: degrees clockwise from left to right (90 is top to bottom). The point is projected on that direction; 0.0 is where the plane starts in that direction, 1.0 where it ends.
- `FromCentre { x, y }`: the centre in percent of the plane's width and height; 0.0 at the centre, 1.0 at the farthest corner.

That position from 0.0 to 1.0 feeds the same colour code as a line of LEDs. Gradient, rainbow wave and moving highlight change along the flow, and their own Forward or Backward reverses it. Solid colour, breathing and colour cycle are the same everywhere (`Effect::follows_position` says which). The moving highlight wraps round as it does on a strip, with a band at least one pitch wide on each side. The flow is part of the apply mode, not an effect control, so the effect definitions, their stored values and every frame in the mode "each device" stay exactly as they were; a test pins the strip output of every effect.

## The two apply modes

| Mode | Position of an LED | Needs |
|---|---|---|
| Each device (`ApplyMode::EachDevice`) | Its index in its device or zone, as before | Nothing |
| Across the setup (`ApplyMode::AcrossSetup { view, flow }`) | Its place on the plane of the view | A setup view that places the target |

**"In sync" stays the default meaning of several targets** (the owner's decision in Task 9): `apply_batch` is unchanged. "Across the setup" is a second, explicit command: `SessionHandle::apply_across(plan, effect)`. It goes through the same path as a batch: one validation, the same take-over, ownership, external-device and ambiguity checks, one shared start time with the start-time rule (an effect already running on a target keeps its start, also when its mode changes), at most 256 targets, and the same `BatchReport` per target. A device the user declared external, or an ambiguous one, is skipped with its reason and gets nothing, not even direct mode. Stop uses `stop_batch` with the plan's targets.

The session holds the places of each target with its running effect and renders a full frame from them at write time; an LED beyond the places is black. The scene shows each assignment's mode, and the user-intent record of an applied effect holds the mode, so changing the mode of a target is a change of that slot (`0002-profile-and-identity.md`).

## Store and profiles

- **`setup.json`** (`prismforge.setup`, schema 1.0) sits in each store folder beside `devices.json`, so a demo setup never meets real devices. It follows the durable store pattern: temporary file, sync, backup, replace; recovery from the backup with a notice; a damaged file is renamed, never deleted; a file from a newer major version is left untouched and every save is refused; unknown fields are kept at every level. The setup is not part of a profile, an export or a diagnostics copy.
- **Saving** (`Profiles::save_setup`) trims names, validates the whole setup and checks every output that is new or whose parts changed against the devices the session reports now. A stored output that no longer fits does not block an unrelated edit; it is reported by `resolve_outputs` and excluded from plans.
- **Profiles move to schema 2.** Each stored effect has a `mode`. A version 1 profile or export is migrated on load: every effect gets "each device", which is all version 1 could mean. Applying a profile runs the effects across a view as one batch per view, flow and effect, so they share one start. An effect whose view is gone, or whose target the view no longer places, is skipped with `SetupChanged`. See `0002-profile-and-identity.md`.

## What Task 11 adds

Task 11 adds the PC and desk views, the editor and the Tauri commands on top of this core, without a change to the model:

- Read and save the setup: `Profiles::setup()` and `Profiles::save_setup(setup, handle)`. They work on the store alone, at the store's place in the lock order, and send nothing to a device, as the group commands do.
- Show each output's state and problems: `resolve_outputs(setup, devices)`; show mirrors and chains: `Setup::parts` and `Setup::control`.
- Preview and apply: `plan_across(setup, view, flow, devices)`, then `Effect::render_across` for a draft preview (no hardware) or `SessionHandle::apply_across(plan, effect)` for output; stop with `stop_batch` on the plan's targets. Show `plan.excluded` and `not_placed()` with the batch report.
- **Lock table.** Every new Tauri command enters `apps/desktop/src-tauri/tests/command_locks.rs`. The core entry points behind them are `Profiles::setup`, `Profiles::save_setup` (store commands: session slot only to read the devices, then the store) and `SessionHandle::apply_across` (an output command: its Tauri command takes a permit under the session slot and is refused while a profile applies, like `apply_effect_batch`; the session records user intent for it like `apply_batch`). `stop_batch` is already in the table. The planner and `resolve_outputs` are pure.
- The interface must say which mode is active, with two cues, and must not send a Live edit of a target that runs across the setup as an "each device" apply.

## Not verified

- Any run against a real OpenRGB server or real hardware: whether one picture looks continuous across devices, the first LED and direction of real fans, splitter behaviour with branches of different lengths, and daisy-chain counts against a resized zone.
- The residual skew between controllers. All targets start on one clock in one tick, but transport and per-device latency are not measured.
- Sampling cost on a real machine. A test samples 600 LEDs over 8 outputs at about 0.13 ms per frame in a debug build on the development Mac; the frame budget at 60 frames per second is 16.7 ms. Release builds and the Windows machine are not measured.
- Whether point sampling with a one-pitch minimum band is smooth enough on sparse LEDs, and whether a footprint filter is needed.
