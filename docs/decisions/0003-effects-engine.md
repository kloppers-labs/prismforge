# 0003 — Effect engine, output scheduler and output ownership

- Status: accepted
- Date: 2026-11-18
- Scope: how PrismForge computes effects, sends frames and decides who controls a device

## Current state

PrismForge has its own effect engine. `crates/prismforge-effects` holds six built-in effects. The session in `crates/prismforge-core` runs them through an output scheduler and keeps a record of who controls each device. The desktop app has an effects screen and Tauri commands for them (Task 4b). **Nothing here has run against a real OpenRGB server or real hardware.** All evidence comes from the simulated demo backend and the scripted fake server, with simulated time, and from a hand check of the built app in demo mode.

## Effect model

1. **An effect is a pure function.** Its colour depends only on the validated values, the effect time, the LED position and a seed. It keeps no state between frames. Any frame can be computed for any time, so frames can be skipped freely.
2. **The effects crate does no input or output.** It has no async runtime and reads no clock. The caller passes the time and the seed.
3. **The six built-in effects are original PrismForge code.** They are solid colour, breathing, gradient, rainbow wave, colour cycle and moving highlight. They were written from general lighting behaviour. No code from another lighting program was read for them. See `docs/provenance.md`.
4. **LED positions are linear for now.** Position 0.0 is the first LED of the device or zone, and 1.0 is the last. A single LED sits at 0.0. Positions outside 0.0 to 1.0 are clamped. A later layout task can pass other positions; `Effect::render` takes them as a list.
5. **The seed is part of the contract.** No built-in effect uses it yet. The session passes 0. Random effects can use it later and stay reproducible.
6. **Brightness works the same way for every effect.** It is a percent from 0 to 100, applied last with integer rounding. 0 gives black. 100 leaves the colours unchanged.
7. **An effect reports when it is static.** Solid colour and gradient are always static. The others are static at speed 0. Every effect is static at brightness 0.
8. **Speed is in cycles per minute.** Each effect says what one cycle is: one breath, one pass of the colour wheel, one round of the palette, or one round of the highlight.
9. **The moving highlight runs round the target as a ring.** Its centre moves over the LEDs of the target and wraps from the last LED to the first without a jump. Its band is at least two LEDs wide, so at least one LED is always at least half lit, also on a target with one, two or three LEDs. The effect gets the LED count from the list of positions that `Effect::render` receives.
10. **One renderer turns an effect into a device or zone frame.** `Effect::render_strip` renders the LEDs of a target in a line with seed 0. The session writes its output. The preview shows its output.

## Stored form of an effect instance

An `EffectInstance` is the effect id and a map from control id to value. In JSON it looks like this:

```json
{ "effect": "gradient", "values": { "brightness": { "Number": 100 }, "direction": { "Direction": "Backward" }, "palette": { "Palette": [{ "r": 255, "g": 0, "b": 0 }, { "r": 0, "g": 0, "b": 255 }] } } }
```

- Effect ids and control ids are stable strings. They never change meaning.
- A value is a colour, a palette, a whole number or a direction. Whole numbers keep the stored form exact.
- Validation reports every problem at once, sorted by control id: unknown effect, unknown control, missing value, wrong type, out of range, between steps, and wrong palette length.
- A control without a default is required. Solid colour's colour is the only required control.
- Validation fills each missing optional value with its default. The snapshot shows the filled form.
- A new control must have a default, so older stored instances stay valid.
- The instance has no version field. Profiles will carry the schema version for everything they store.

`EffectDefinition` describes each effect for an interface that does not know it: an id, a label, a description and typed controls. The control types are colour, palette (with a minimum and maximum count), speed (range, default, unit), direction, brightness and number (range, step, default, unit).

## Scheduler rules

1. **The session task owns output.** Frames go out from the same task that owns the backend. No other task calls the backend.
2. **The frame rate is configurable and clamped.** The default is 30 frames per second. The session clamps any setting to 1 to 60, at start and on each change.
3. **One batched write per target per tick.** A whole-device effect sends one device frame. A zone effect sends one zone frame. PrismForge never sends a command per LED.
4. **Frames are coalesced, never queued.** Each write renders the frame for the moment it starts. If a tick is not finished when the next tick is due, the ticks that pass are skipped and counted. The next tick stays on the original grid. The 1 ms timer resolution does not slow the rate down.
5. **Commands and output take turns.** A command waits at most for the one backend call in progress. A tick is a queue of single backend calls: direct mode where needed, then one frame per target. Between two calls, the loop handles a waiting command, then makes the rest of the calls of that tick. After a command that ran while output was due, output goes next. A stream of commands therefore cannot stop output, and output cannot block commands for longer than one call.
6. **The loop never spins.** A device is due only when it has a backend call to make: direct mode to enable, or an effect that needs a frame. Paused, unavailable and fully written devices are never due, and a refused device is due only at its retry time. So every output wake-up either makes at least one backend call, or ends a saved tick whose devices are no longer due; after that, the next wake-up is in the future or there is none. Stopping an effect clears its device's refusal. The counter `output_wakeups` counts these wake-ups.
7. **Re-applying keeps the animation running.** Applying the same effect id to the same target again keeps the effect's start time. The new values apply from the next frame, so a changed setting does not restart the animation. Another effect, or the same effect on another target, starts at time zero.
8. **Static effects are written once.** They are written again only after a scene change, a reconnect, an inventory refresh or the end of an identify.
9. **A refused write affects only its device.** The device waits 1 second, then 2, 4, 8 and 16 seconds, and then 16 seconds between tries. Other devices keep their cadence. A stale inventory pauses all output until the next refresh. It does not count as a refusal with a wait.
10. **A lost connection pauses output.** After reconnect and refresh, each device gets direct mode again before its first frame. A device that is ambiguous, missing, without direct mode or with a changed zone gets no output. Its condition shows the reason.
11. **The counters are real.** `frames_written` counts frames the backend accepted. `ticks_skipped` counts ticks dropped because the tick before them was not finished, including commands handled between its calls. `output_writes_refused` counts refusals while the connection stayed open. `output_writes_failed` counts writes that ended the connection. `output_wakeups` counts the times the loop woke up for output. The client does not read acknowledgements for colour frames, so "accepted" means sent without an error.

## Several targets in one command

Plan 2, Task 9. The owner chose "in sync": each selected device or zone runs the whole effect on its own LEDs, and all of them share one start time, so they breathe and cycle together. "Flow in selection order" was not chosen. "Flow by real position" waits for the setup views (Tasks 10 and 11).

1. **One command, one shared start.** `SessionHandle::apply_batch(targets, effect)` validates the effect once; an invalid effect refuses the whole batch and sends nothing. Every target then goes through the same take-over and ownership checks as a single apply. The start time is the earliest start among the targets that already run this effect (the start-time rule of scheduler rule 7, applied to the batch); when none does, the clock starts when the first target starts. So applying the same batch again, or a Live edit, keeps the animation running, and a target added to a running selection joins in phase.
2. **A result per target, in the order given.** `BatchReport` holds `Applied`, `Stopped { restore }`, `Covered { by }` or `Skipped { error }` for each target. `Skipped` carries the same `OutputError` a single command would return: unknown, missing or ambiguous device, no Direct Mode, a zone out of range, a declared external owner (the conflict), an identify in progress, no effect to stop, or a backend refusal. A partial result is normal and never rolled back.
3. **Overlapping targets.** A whole device covers its zones wherever it is listed: the zones are reported `Covered { by: whole device }` and nothing is sent for them. A target listed twice is covered by its first copy. Two zones of one device are two targets on one take-over.
4. **External devices.** A batch that names a device the user declared external skips it with `OutputConflict` and sends nothing to it, also no direct mode.
5. **Stop.** `stop_batch(targets)` stops each target with the single-stop rules and the same report form. `stop_all_effects()` stops every device PrismForge runs effects on, one whole-device result per device; the tray's "Stop all PrismForge effects" now uses it, and the Effects screen has the same control.
6. **Bounded.** A batch takes at most 256 targets (`MAX_BATCH_TARGETS`); more is refused with `TooManyTargets` before anything is sent. A batch is one command, so the output of other devices waits for its backend calls, as it waits for a restore; it never waits longer than the calls the batch makes.
7. **User intent.** A batch records one record per target it changed, and every side effect, under one sequence number (`0002-profile-and-identity.md`).

## Live output

1. **What PrismForge last wrote, never a second render.** `SessionHandle::live_output(targets)` opens a `LiveFeed`. After each loop turn the session copies, for the requested targets only, the colours it holds as last written (`EffectOutput::shown`): per zone, with the zone's index, name, kind and matrix size, in LED order. LEDs no effect has written yet hold the colours reported at take-over. A device without a PrismForge effect (a hardware mode, another program, nothing) has no live data.
2. **No backend call, no write.** The copy reads the output state only. Asking for live output, or stopping, wakes the loop once and changes neither the frames, their times nor the counters; a test compares two runs, with and without a feed, frame by frame.
3. **Bounded rate.** A feed delivers at most 25 updates per second (`LIVE_OUTPUT_RATE`), the latest state each time; frames in between are dropped, never queued. One feed at a time: a newer feed replaces the request, and dropping a feed ends the copying.
4. **In the desktop app.** `watch_live_output(targets)` forwards one feed to the window as the `live-output` event until the next call; an empty list stops it, and the end of the session ends it. The interface asks only while the live view is visible.

## Ownership rules

1. **One owner per device.** The owner is a PrismForge effect, a hardware mode selected through PrismForge, or another program that the user declared. A device that is not listed has no owner.
2. **PrismForge cannot see other writers.** An external owner is only the user's statement. PrismForge never claims that the other program stopped. It never stops another program.
3. **Nothing is sent to a device the user declared external.** Apply, replace, identify and hardware-mode selection are refused with the structured conflict `SessionError::OutputConflict`. Output does not resume after a reconnect, and stop and shutdown have nothing to restore there. The user must hand the device back first with a separate command.
4. **Declaring a device external stops PrismForge output on it.** PrismForge does not restore the device, because the restore would fight the other writer. It keeps the remembered state and marks the device as not restored.
5. **Taking a device remembers its state.** PrismForge stores the active mode and colours that the inventory reported, then enables direct mode. If PrismForge changed the device earlier and never restored it, the take-over reuses that earlier remembered state, but only while nothing newer contradicts it. The newest trustworthy evidence wins: when the inventory reports an active mode other than Direct Mode, someone chose that mode, so PrismForge drops the old state and captures the reported one. When the old state no longer fits the device (its modes or LED count changed), PrismForge also captures afresh, and reports "previous state unknown" where the fresh capture cannot know better.
6. **Stopping the last effect on a device restores that state.** The restore uses the mode re-apply of identify and the same outcome words: restored, partly restored, not restored, unchanged. Before the restore, PrismForge checks that the device still has the remembered modes and LED count.
7. **Ownership is per device.** A mode change affects every zone. Zone effects on one device share one take-over and one restore.
8. **Selecting a hardware mode stops the effect on that device.** It re-applies an existing mode with its reported settings. It edits no settings and saves nothing to device memory.
9. **Identify pauses an effect.** Identify lights the target. Then PrismForge writes back, as one device frame, what every LED showed before identify: its last effect colours, or for LEDs no effect has written, the colours reported at take-over. Then the effect resumes. Identify reports "restored" only when that frame was accepted. It does not restore the hardware mode of a device that PrismForge runs an effect on.
10. **A device PrismForge keeps in direct mode on purpose is not "previous state unknown".** That label comes only from a restore that did not finish: a failed or cut-short restore, or an effect dropped by an external declaration.
11. **Disconnect and shutdown restore first.** The session restores every device it runs effects on before it disconnects. It stops waiting after 5 seconds. Devices it did not restore keep their remembered state for the next take-over and report "previous state unknown" until then.
12. **Output commands use their own error type.** `OutputError` holds session errors (including the conflict), invalid effect values and "no effect on this target". Identify uses `SessionError`.

## Effects interface

1. **The interface knows no effect.** It renders every control from the typed control definitions: colour, palette, speed, direction, brightness and number. It has no code for a single effect.
2. **Rust is the authority on values.** The interface checks ranges, steps, palette lengths and colour text before it sends, for quick feedback. The validation errors that Rust returns are shown next to their control.
3. **A running effect updates in place, but the user's last explicit action wins.** A changed setting is applied again 300 ms after the last change only when, at that moment, the newest view shows this effect on this target, no apply, stop, mode or ownership call for the device is in flight, and Apply is not blocked. Stop, a hardware mode, a declaration or hand-back, a change of target, leaving the effects screen and disconnecting cancel a pending change. After them, nothing is sent automatically for that device or target until the user presses Apply. The stop itself blocks re-apply, so a view that arrives late cannot restart the effect. The start-time rule keeps the animation running.
7. **Status lines come from the newest view or are past-tense results.** Owner, output condition and frame rate in use come from the view. A command result says what was accepted ("PrismForge accepted Breathing for Logo"). It is cleared when a later action on the device succeeds or a newer view contradicts it.
4. **Commands use only the session handle.** Effect commands go through the shell to `SessionHandle`. They return `CommandError`, which keeps validation errors, "no effect on this target" and session errors apart.
5. **The interface says only what the user declared and what PrismForge did.** "Another program controls this device" records the user's statement. "Hand back to PrismForge" records that the other program no longer writes. PrismForge never says that it stopped another program.
6. **Software effects and hardware modes are named apart.** A PrismForge effect runs only while PrismForge runs and is connected. A hardware mode runs on the device itself. Nothing is saved yet, and the screen says so where the user applies an effect.

## Preview

1. **Rust renders the preview.** The command `preview_effect` validates the instance, then renders frames with `Effect::render_strip`. It reads no clock and uses no session and no backend.
2. **A preview batch is bounded.** The request names the instance, the LED count, the start time, the frame count and the interval. The LED count is clamped to 1 to 60, the frame count to 1 to 90 and the interval to 20 to 1,000 ms. A static effect gets one frame.
3. **The interface only plays frames.** It asks for 50 frames 40 ms apart and plays them in a strip. It asks for the next batch when the batch runs out, and again 200 ms after a setting changes.
4. **The preview stops when nobody can see it.** It stops asking and animating when its panel is closed or the window is hidden. With reduced motion it asks for one frame and shows it still.
5. **The preview cannot change lighting.** It calls only `preview_effect`. Physical output never depends on the window.
6. **Draft and live are two states (Task 9).** The stage on Effects shows either the draft preview, the pattern for each selected target at its own LED count on one clock (one batch per distinct LED count, played together), or the live output above. Each state has a text badge, a mark and its own frame. The live view draws a linear zone as a row and a matrix zone as a grid of its width.
7. **Flash cap on the screen only.** Both views hold back any frame that would make a fourth flash within one second (WCAG 2.3.1): a flash is a pair of opposing changes of 10 % or more in the mean relative luminance of any row, with the darker side below 0.8. The view then says "Fast flashing is shown slowed on the screen. The devices get every frame." The cap lives in the interface, which owns playback and drawing; it never touches the frames sent to the devices, and it does not make the physical lights safe.

## Not verified

- Any run against a real OpenRGB server.
- Any frame on real hardware.
- Frame timing on a real machine. All timing tests use simulated time.
- How a real controller handles 30 frames per second. The client cannot see whether the server applied a frame.
- Whether OpenRGB reports Direct Mode as the active mode after PrismForge selects it. If it does not, PrismForge enables direct mode again after each refresh. The fake server does not change the active mode.
- What a fresh capture remembers on such a server. If the server never reports Direct Mode but reports the colours PrismForge wrote, a fresh capture of a device in a per-LED mode remembers PrismForge's own last frame as the colours to restore. A later stop then writes that frame back and reports "restored".
- Performance measurements. A later task measures CPU use and frame timing, including the preview's cost in the WebView.
- The effects screen on Windows. It was run by hand on macOS in demo mode only, through the accessibility tree, without screenshots.
- Whether a real device shows what the preview shows.
- The live view against a real OpenRGB server and real devices, the batch timing on Windows, and the live event rate in WebView2. The preview shows the pattern for LEDs in a line; the device layout is unknown.
