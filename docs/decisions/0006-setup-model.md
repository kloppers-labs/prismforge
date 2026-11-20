# 0006 — Setup model and effects across the setup

- Status: accepted
- Date: 2026-11-20
- Scope: where each LED is, how the user declares mirrors and daisy chains, how an effect becomes one picture across several devices, and how the setup is stored (Plan 2, Task 10); the PC view, the desk view, the editor and its save model (Task 11).

## Current state

The Rust core has the setup model, the planner that gives each LED a place, the 2-D effects, the apply mode "across the setup" in the session, the setup store and profile schema 2 (Task 10). The Setup area has the PC view and the desk view, the editor, the draft preview and live colours on the drawing, and Apply across a view, through four Tauri commands (Task 11). **Nothing here has run against a real OpenRGB server or real hardware.** The evidence comes from tests with generic fixtures, the demo backend and the scripted fake server in simulated time.

The code is in `crates/prismforge-core/src/setup` (the model, geometry, validation, controls, resolution and the planner), `crates/prismforge-core/src/store/setup.rs` (the file), `crates/prismforge-effects/src/plane.rs` (the 2-D projection) and the session's `ApplyAcross` command. The research behind it is `docs/research/setup-views-research.md`, sections 1, 2 and 5.

## The model

A setup has three lists.

| Part | Holds |
|---|---|
| Component | A lit thing with a declared shape: an id (`c-…`), a name, a geometry and, optionally, a frame (Task 11). |
| Output | One channel PrismForge writes: an id (`o-…`), a device and a zone (or the whole device) by stored reference, the device and zone names it had, the first LED, and its branches. |
| View | A named picture, such as "PC" or "Desk": an id (`v-…`), a name, placements and unlit outlines (Task 11). |

**Drawing only (Task 11).** A component's `frame` is a rectangle drawn round its LEDs, such as a graphics card or a cooler plate: width and height in percent of the LEDs' longer side, 10 to 1000. A view's `outlines` are unlit drawings of a case, a desk, a monitor or a PC: an id (`u-…`), a name, a kind, a centre, a width and height (1 to 4000 view units) and a turn. Neither has LEDs: the planner ignores both, and no output, part or LED count depends on them. Both fields are optional with serde defaults, so a setup file written before Task 11 loads unchanged (a test loads one byte for byte); a component without a frame is written without the field. The format stays at schema 1.0: an older PrismForge keeps both as unknown fields.

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

`plan_across(setup, view, flow, devices)` builds an `AcrossPlan` for one view. It first checks the flow (an angle below 360, a centre of 0 to 100 percent), the whole setup with `Setup::validate` and that the view exists, and returns a `PlanError` otherwise, so a draft of unsaved edits can never allocate without bound or give NaN places. Only `plan_across` makes an `AcrossPlan`; its view, flow and plane are read through methods.

1. **Each LED of each placed component gets a view location:** the geometry's local point, then the placement (flip, scale by `size`, turn, move).
2. **The plane is the box round every wired and placed LED of the view**, normalised on its longer side, so the longer side is 1 and (0, 0) is the middle. The plane does not depend on which devices are found today, so a missing device never moves the picture. Normalising on the longer side keeps angles and circles true; a fixed canvas would make an effect's scale depend on an empty area.
3. **Each target (device or zone) gets a place for each LED, by LED index.** A part takes its LEDs from its own component. For a mirrored output, an LED takes its place from the first branch that places it; the other branches show the same colours and add no samples. LEDs of the zone beyond the parts, and LEDs whose component is not in this view, have no place.
4. **Each place carries the spacing of its own component**, from the shape and not from the wire order: the cell of a strip or free line (its length divided by its LEDs), the chord between neighbours of a ring, the cell of a matrix (so a progressive matrix's jump from the end of one row to the start of the next does not count), and the widest side spacing of a perimeter; 0 for a point. It is scaled by the placement's size and the plane, like the place. Effects use it as a minimum feature width **for that LED only**: the band never falls between two LEDs of the same component, and a sparse component never widens the band on a dense one.
5. **Which outputs get nothing.** `resolve_outputs` reports each output with the groups' states (`crate::resolution`, shared by groups and the setup) (resolved, unavailable, missing, ambiguous, changed) and any problem with its parts today. An output whose zone changed (the zone at that index has another name) or whose parts no longer fit is **excluded**: it is listed in `plan.excluded` and gets no write. Outputs of missing, unavailable or ambiguous devices stay in the plan, so the session refuses them with its own reason, as in a batch. A target with no placed LED in the view is not part of the plan.

The research suggested a footprint filter (five samples per LED). Task 10 samples at the LED's point and raises the moving highlight's band, for each LED, to at least that LED's spacing on each side of its centre, which is the same minimum width the strip renderer keeps. The filter is left for measurement on real hardware.

## Effects in two dimensions

An effect stays a pure function of its values, the time and the position (`0003-effects-engine.md`). The position across a setup is a point on the plane. A **flow** says how the effect crosses it:

- `Along { angle }`: degrees clockwise from left to right (90 is top to bottom). The point is projected on that direction; 0.0 is where the plane starts in that direction, 1.0 where it ends.
- `FromCentre { x, y }`: the centre in percent of the plane's width and height; 0.0 at the centre, 1.0 at the farthest corner.

That position from 0.0 to 1.0 feeds the same colour code as a line of LEDs. Gradient, rainbow wave and moving highlight change along the flow, and their own Forward or Backward reverses it. Solid colour, breathing and colour cycle are the same everywhere (`Effect::follows_position` says which). The moving highlight crosses the plane and leaves it before it enters again, so its two ends never light together (a strip keeps its ends one LED apart for the same reason); for each LED the band is at least that LED's spacing wide on each side. A flow outside its ranges is refused: by `plan_across`, and in a stored profile, which is then not valid, as a setup with a turn of 360 degrees is not valid. The flow is part of the apply mode, not an effect control, so the effect definitions, their stored values and every frame in the mode "each device" stay exactly as they were; a test pins the strip output of every effect.

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
- **Profiles move to schema 2.** Each stored effect has a `mode`. A version 1 profile or export is migrated on load: every effect gets "each device", which is all version 1 could mean. Applying a profile runs the effects across a view as one batch per view, flow and effect, so they share one start. An effect whose view is gone, or whose target the view no longer places, is skipped with `SetupChanged`; while `setup.json` needs a newer PrismForge, `Profiles::setup` refuses with `NewerFile` and those effects are skipped with `NewerSetup`. Apply and the profile details check in one order: the device, the effect, the setup, then the session's reasons (declared external, no Direct Mode). See `0002-profile-and-identity.md`.

## The Setup area (Task 11)

The code is in `apps/desktop/src/features/setup` and `apps/desktop/src-tauri/src/setup.rs` with `setup_commands.rs`. The model is unchanged; the setup types gained TypeScript bindings, with the kept unknown fields left out of the types and carried through untouched by every edit.

### Commands

| Command | Does | Behind it |
|---|---|---|
| `setup` | Reads the stored setup, each output matched against the devices of now (`SetupOutputStatus`), and the limits | `Profiles::setup`, `resolve_outputs` |
| `save_setup` | Stores a drawing; returns the same view | `Profiles::save_setup` |
| `apply_across` | Plans a view of the **saved** setup against the devices of now and runs one effect across it; with a list of targets, only those, on the plane of the whole view | `plan_across`, `AcrossPlan::only`, `SessionHandle::apply_across` |
| `preview_across` | Renders frames of an effect across a view of the drawing it is given, against no device, on a blocking thread; up to 16,384 LEDs, and fewer frames so that LEDs times frames stay at or under 60,000 colours | `plan_across`, `Effect::render_across` |

`apply_across` returns an `AcrossReport`: the `BatchReport`, `excluded` (outputs whose zone changed or whose parts no longer fit, with their state and problem) and `not_placed` (per target, how many of its LEDs no placed component covers; they are written black). A `PlanError` (unknown view, a setup that is not valid, a flow out of range) is a `CommandError` and sends nothing. The lock places are in `0002-profile-and-identity.md`.

### Two views, a drawing and its list

- **Tabs "PC" and "Desk"** are the views `v-pc` and `v-desk`. They are made in the drawing when the stored setup has none. Views with other ids are kept and not shown.
- **The drawing** is SVG with a fixed extent of 1000 × 700 view units, so a number means the same at every window size. Each component is a generic outline made for PrismForge (a circle with a hub for a ring, a rounded bar for a strip, a grid for a matrix, a frame for a screen edge, a dot for a point, a path for a free line, a filled rectangle for a frame) with its LEDs at the places the Rust model computes; a test compares the TypeScript port with a fixture the Rust model writes. LED 1 is larger, ringed and numbered "1", with an arrow along the direction of the LEDs. Mirrored components are joined by a dashed line; the parts of a daisy chain by an arrow from the last LED of one part to the first of the next, in wire order. A component wired to nothing has hollow, dashed LEDs. Unlit outlines are dashed, behind the components, with an italic name; they have no LEDs. Unlit LEDs use `--led-off`, at least 3:1 against the drawing and against a shape (a test reads the tokens).
- **The list twin** beside the drawing has the same components in the same order, then the outlines as named items ("Outline of a case. No LEDs: it is drawn only and nothing is wired to it."), and is the keyboard and screen-reader path. Each row says the shape and LED count, the device and zone it is wired to, whether it is a mirror ("one control with …; all four always show the same colours") or a chain part ("Part 2 of 2 of a daisy chain on …: LEDs 9 to 16"), where LED 1 is and which way the LEDs run as drawn (after the turn and the flip), and its place. Focus and selection are apart, as in the target list of Effects (`0005-shell-and-design-tokens.md`).
- **Outputs** lists each output once: a mirrored output is "4 fans mirrored: one control shown in 4 places", a chain "2 parts in a daisy chain", what it holds, and how many of its LEDs have a place in this view and how many are not placed and stay dark during an effect across the setup. An output with nothing in this view says that an effect across this view leaves it as it is, because the planner leaves it out.

The drawing never claims more control than the hardware has: a mirror is drawn as one control in several places, a chain as parts of one zone, unplaced LEDs are counted, and Identify says that it lights the whole device or zone and names every component on it.

### Placing and templates

- **From the detected devices.** Each device, or each zone of a device with several, can be placed in the view on screen with a name, a shape from the first library of the research (strip or bar, ring fan, fan with two rings, matrix, screen edge, single light, free line, and a block with a strip, a ring or a grid of LEDs inside a frame) and an LED count, which starts at the zone's count. When the zone already shows a component, the user must say whether the new one is a mirror or the next part of the daisy chain; Place stays off until then. A fan with two rings is two ring components chained on one output, the inner one smaller at the same centre.
- **Templates.** "Tower PC" proposes three front fans, a rear fan, two cooler strips, two memory modules, the graphics card and a board edge; "Desk" a screen edge, a desk strip, a keyboard, a light bar and a mouse logo. They place components and wire none. "Not wired yet" lists each with an output choice and the same mirror-or-chain question. LED counts in a template are placeholders, never a product's count.
- A component of the other view can be placed here; its wiring is the same in both views.
- **Outlines.** "Add an outline" adds a case, a desk, a monitor or a PC to the view on screen. The Tower PC template draws a case behind its components; the Desk template a desk, a monitor and a PC. Outlines move, turn and resize with fields, buttons and keys like components, are never flipped, and are deleted with a confirmation; taking one out of the view deletes it.

### Editing without a drag

Every edit has a button or a number field and, from the list, a key; a drag on the drawing is an extra (WCAG 2.5.7).

| Edit | Buttons and fields | Keys in the list |
|---|---|---|
| Move | X, Y; Move left, right, up, down by 10 | Alt with an arrow by 10, with Shift by 50 |
| Turn | Turn (degrees, any whole number); Turn left or right 15° | `[` and `]` |
| Flip | Flip (pressed state) | Shift+H |
| Size | Size, 10 to 1000 | — |
| LED count, first LED, direction | LED count (or columns and rows, or LEDs per side); LED 1 angle or corner; Reverse the direction | — |
| Points of a free line | Point N, x and y (the arrow keys change a number field); Add a point; Remove point N (2 to 256 points) | — |
| Frame | Add a frame; Frame width and Frame height in percent; Remove the frame | — |
| Outline | Name, Kind, X, Y, Width, Height, Turn; Move and Turn buttons; Delete outline | Alt with an arrow, `[` and `]`, Delete |
| Name | Name | — |
| Wiring | Output, mirror or chain, Earlier or Later in the chain, Starts at LED, Unwire | — |
| Take out of the view, delete | Buttons; Delete component asks first | Delete takes out of the view |

Several selected components move by the same amount, so their spacing stays; each turns and flips round its own centre. Moves keep centres inside the view. Flip mirrors the drawing only; Reverse the direction changes the order on the wire: a strip or free line puts LED 1 at the other end, a ring or screen edge keeps LED 1 and runs the other way round, and a matrix puts LED 1 in the corner of its last LED, so the same LEDs run exactly backwards (a test checks every corner, rows and columns, with and without serpentine). A number field or a name that the drawing does not take (clamped, unchanged or empty) shows the value the drawing keeps; an empty name says why. LED counts are checked before Save with the store's rules (1 to the zone's maximum for each component; the parts must fit the LEDs the zone has now), and the reason shows beside the field.

### Save model and undo

**Edits change only the drawing until Save setup.** The bar shows "Saved" or "Unsaved changes" (a mark and a word), with Discard changes. Saving runs the store's checks; a refusal keeps the drawing and names the component or output. The drawing survives leaving the area; an unsaved drawing of the demo store waits while the real store is in use, and the other way round.

Undo and redo work on the drawing: each finished edit (a button, a key, a field change, a drag, a template) is one step with the selection, up to 100 steps; an edit that changes nothing adds none. The bar says "Undo and redo change this drawing only. Your lights stay as they are." Undo never sends anything.

Apply across a view waits for a saved setup, because a profile saved after it names a view whose places must be stored. The draft preview draws the unsaved drawing.

### Colours on the drawing

- **Draft preview:** the chosen effect across this view of the drawing, rendered in Rust (`preview_across`), on the components; the mark ◇, the word "Draft", a dashed frame and "The pattern only. PrismForge sends nothing for it."
- **Live output:** the frames PrismForge sends (the live feed of Task 9) cut to each component by its part, so mirrored components show the same colours; the mark ●, the word "Live", a solid frame in the ok colour and the same sentence as on Effects. It is asked for only while the Setup area, the window and the document are shown.
- Reduced motion keeps the draft at one still frame and the live drawing at a new still frame every 2 seconds. The flash cap of Effects applies to both.

### Applying across the setup

The panel names the mode with its flow ("Across the setup: PC view, flowing left to right (0°)"), takes an effect and its settings, and a flow by angle or by centre. Apply sends the saved view, the flow and the effect; the result lists each target, the excluded outputs with their reason, and the LEDs sent black. Retry resends the same picture, on its own view and flow, only to targets a second try can reach. Stop stops the targets that run across this view.

### Effects keeps the mode

When the selected targets run an effect across the setup, Effects says so with the flow, and a Live edit, an Apply and a Retry go through `apply_across` with that view and flow and those targets; they are never sent as "each device". Targets that cannot keep one mode are not sent: Live is off and Apply says which case it is. Some run across the setup and some on each device; all run across the setup but on different views or flows; or a whole device is selected whose zones run across the setup, where an effect on the whole device would end that picture on them.

### Groups and menus

"Make a group from the selection" stores a saved group of Task 9 from the outputs of the selected components; "Select its components" selects the components wired to a group's members. There is no second grouping. Component rows and the drawing have a context menu; each item mirrors an editor button (`0005-shell-and-design-tokens.md`).

### Profiles

Saving a profile while effects run across the setup stores the mode, the view and the flow (Task 10). A profile stores the view id, not the places, so a profile applied after its view was edited uses the new places without a warning. Whether it should warn is an open question for the owner; a test pins today's behaviour (`a_profile_applied_after_its_view_changed_applies_without_a_warning`).

## Not verified

- The Setup area in the native window, with a screen reader, and pointer drags on Windows; the drawing at 200 % text. Screenshots come from the interface preview in Chromium on macOS.

- Any run against a real OpenRGB server or real hardware: whether one picture looks continuous across devices, the first LED and direction of real fans, splitter behaviour with branches of different lengths, and daisy-chain counts against a resized zone.
- The residual skew between controllers. All targets start on one clock in one tick, but transport and per-device latency are not measured.
- Sampling cost on a real machine. A test samples 600 LEDs over 8 outputs at about 0.13 ms per frame in a debug build on the development Mac; the frame budget at 60 frames per second is 16.7 ms. Release builds and the Windows machine are not measured.
- Whether point sampling with a band of at least one LED spacing is smooth enough on sparse LEDs, and whether a footprint filter is needed.
