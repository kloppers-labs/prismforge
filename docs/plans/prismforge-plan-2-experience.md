# PrismForge plan 2 — experience

> **For agentic workers:** This plan continues `docs/plans/prismforge-implementation-plan.md`. All rules of that plan still apply: sections 3, 5, 6, 7, 9, 11 and 12. Execute one task at a time with tests first. Each task gets an independent review before it is merged.

**Date:** 2026-11-19.

**Goal:** Make PrismForge easy and pleasant to use each day, and add the features that set it apart: effects on several devices with a live view, a visual view of the setup, and profiles on system events.

**Source of this plan:** The owner used the application for the first time on 2026-11-19 and gave five directions. This plan turns them into tasks.

1. A full UI and UX sweep, based on research. The navigation bar needs work.
2. Effects on several devices at the same time, with a preview of what the effect does.
3. A preview of the setup: a PC view and a desk view.
4. Profiles on system events: shutdown, sleep, hibernate, startup and wake.
5. Open at system startup and start the active profile.

**Research that this plan uses** (in `docs/research/`):

| File | Content |
|---|---|
| `ux-research.md` | Best practices for hardware configuration software and general HCI, with sources |
| `ux-audit.md` | Audit of the current screens, with three navigation designs and a priority list |
| `openrgb-behaviour.md` | How OpenRGB handles events, previews and layouts, from its source code |
| `os-events-research.md` | Power and session events on Windows, macOS and Linux |
| `setup-views-research.md` | Geometry models, sampling of effects in two dimensions, and layout editors |

A task brief names the sections of these files that it must follow. Where a research file and this plan disagree, this plan is correct.

---

## Decisions that all tasks must keep

1. **Navigation:** a rail on the left for the areas, and one status strip at the top of the content. This is option C of `ux-audit.md`, section 2.6.
2. **Areas, in this order:** Devices, Effects, Setup, Profiles, Automation. Diagnostics and Settings are at the foot of the rail. The items never change with the state.
3. **Words:** "Setup" is the area with the PC view and the desk view. "Automation" is the area with the rules for system events.
4. **Draft and live are two states.** A draft preview shows the pattern and touches no hardware. Live output shows the colours that PrismForge sends to the devices.
5. **Each automatic hardware write needs a setting that the user switched on.** The default is off. The interface says what will change without a click.
6. **PrismForge promises only what it can do.** "Will" is for what PrismForge controls. "Tries to" is for what depends on the operating system, on OpenRGB or on the device.
7. **Original identity.** PrismForge does not copy the layout, artwork, icons or text of another product. Component shapes in the Setup area are generic and are made for PrismForge.
8. **Accessibility:** WCAG 2.2 level AA. Each drag action has an alternative without a drag. A preview on the screen does not flash more than three times each second.
9. **The lock rule and the command lock table stay in force.** Each new command enters the table in `apps/desktop/src-tauri/tests/command_locks.rs`.

---

## Task 7 — Application shell: rail, status strip and design tokens

**Files:** `apps/desktop/src/App.svelte`, a new `apps/desktop/src/lib/shell/`, `apps/desktop/src/app.css` and token files, `apps/desktop/src/lib/state/`.

**Consumes:** `ux-audit.md` sections 2, 5 and 6; `ux-research.md` sections 1, 6 and 10; the session view and the store summary that exist.

**Produces:** the new shell that all later screens use.

- [ ] Build the rail and the status strip of decision 1. The rail has an icon and a label for each area. The current area has two cues that do not depend on colour alone.
- [ ] Put the connection state, the active profile with its unsaved changes, a running profile apply, and demo mode in the status strip. Put the actions that belong to a state behind that state.
- [ ] Use one place for notices. Notices no longer stack as separate strips.
- [ ] Keep each area in the rail in every state. An area that needs a connection shows a clear empty state.
- [ ] Add the design tokens of `ux-audit.md` section 5: colour, type, spacing, radius, elevation and motion. Remove colours and sizes that are outside the tokens.
- [ ] Draw the icons for PrismForge. Do not use icons of another product.
- [ ] Add keyboard access: one shortcut for each area, a way to move between the rail, the strip and the page, and visible focus that the drawer and the strip never hide.
- [ ] Test first: the rail items stay the same in each state; the current area is marked after a disconnect; the strip shows each state from fixtures; the strip does not overlap at 920 px and at 200% text size; demo mode is marked on each screen.
- [ ] Keep Diagnostics as a drawer. Make it not cover the focused control.

**Acceptance:** the navigation holds seven areas, never changes with the state, and has no overlap at 920 × 600. A reviewer can read the global state in one place.

## Task 8 — Screen sweep

**Files:** `apps/desktop/src/features/connection/`, `devices/`, `effects/`, `profiles/`, `settings/`, `diagnostics/`.

**Consumes:** the shell of Task 7; `ux-audit.md` sections 3, 6 and 7; `ux-research.md` sections 2, 4, 6, 9, 10 and 11.

**Produces:** screens that pass the priority list of the audit.

- [ ] Fix each P0 and P1 item of `ux-audit.md` section 7. Record each P2 item that stays open.
- [ ] Put the main action of each screen where the user sees it without a scroll at 920 × 600. On Effects, "Apply" stays visible.
- [ ] Move long explanations behind a short line with "Details". Keep each statement true.
- [ ] Give each error three parts: what happened, why, and what to do.
- [ ] Make each control look like what it is. A button that is enabled does not look disabled. A card that looks like a button is a button.
- [ ] Reach WCAG 2.2 AA for contrast, target size, focus and text at 200%.
- [ ] Remember the last OpenRGB address on the connection screen.
- [ ] Test first for each fixed behaviour. Do not write tests that only repeat styling.

**Acceptance:** each P0 and P1 item of the audit is closed with a test or with a recorded reason. A reviewer runs the checklist of `ux-research.md` section 11 on each screen.

## Task 9 — Effects on several targets, and the live output view

**Files:** `crates/prismforge-core/src/session/`, `crates/prismforge-effects/`, `apps/desktop/src-tauri/src/effects.rs`, `apps/desktop/src/features/effects/`.

**Consumes:** the effect engine, output ownership, `ux-audit.md` section 4, `ux-research.md` sections 3 and 4, `openrgb-behaviour.md` section B.

**Produces:** one apply for several targets, and a view of the colours that PrismForge sends.

- [ ] Core: add a batch apply. One command applies one effect to several targets. All targets get the same start time, so they stay in phase. The result lists each target as applied or skipped, with the reason. A partial result is normal.
- [ ] Core: add a batch stop and a "stop all" with the same form of result.
- [ ] Core: give the interface the colours that PrismForge last wrote, for the visible targets only, at a bounded rate. This is the live output. It uses no backend call and no extra hardware write.
- [ ] Interface: select several targets with a list of check boxes, "select all", groups by device, and a mixed state. A target that cannot take the effect shows its reason and stays out of the selection.
- [ ] Interface: show the draft preview and the live output as two states, each with two cues. The live output draws each zone in its own shape: a row for a linear zone and a grid for a matrix zone.
- [ ] Interface: show the batch result for each target, with "retry failed".
- [ ] Limit flashing in the previews on the screen to three changes each second, and respect reduced motion.
- [ ] Record user commands for a batch in the same way as for one target, so profiles stay correct.
- [ ] Test first: `applies_one_effect_to_several_targets_in_phase`, `reports_each_target_of_a_batch`, `skips_external_devices_in_a_batch`, `live_output_equals_written_frames`, `live_output_rate_is_bounded`, and interface tests for selection, mixed state, results and the two preview states.

**Acceptance:** the user selects several devices, applies one effect, and sees for each device what happened. The live view shows the colours that PrismForge sends, and it never changes the hardware.

## Task 10 — Setup model and effects across the setup

**Files:** `crates/prismforge-core/src/setup/` (new), `crates/prismforge-effects/`, `crates/prismforge-core/src/profile/`, `crates/prismforge-core/src/session/`.

**Consumes:** `setup-views-research.md`; the scene and the effect model; profile schema version 1.

**Produces:** a model of where each LED is, and effects that run across devices as one picture.

- [ ] Model a setup as components with a geometry, placed in a scene. A component belongs to one output: a device, a zone, or a part of a zone.
- [ ] Give the geometry classes that the research recommends: strip, ring, matrix, perimeter, point and free line. Each class has an LED count, a first LED and a direction.
- [ ] Model a mirrored output truthfully: several components that always show the same data. Model a daisy chain truthfully: one zone that is split into parts, one part for each component. The user declares these. PrismForge never guesses them from a picture.
- [ ] Refuse an LED count that is outside the bounds that the backend reports.
- [ ] Give each LED a position in scene space. Add a second apply mode: "across the setup". In this mode an effect uses the scene position of each LED. The mode "each device" stays as it is.
- [ ] Extend the six effects to two dimensions with a direction or a centre. Keep each effect a pure function of its settings, the time and the position.
- [ ] Save the setup in the store with a version. Move profiles to schema version 2 with a tested migration. A profile of version 1 loads without loss.
- [ ] Test first: `represents_mirrored_outputs_as_shared_control`, `rejects_led_count_outside_backend_bounds`, geometry transforms, mirror, rotation, LED order for each class, `samples_effects_at_scene_positions`, `migrates_profile_v1_to_v2_without_loss`, and no change of output for the mode "each device".

**Acceptance:** one effect runs across several devices as one picture, in the right order and direction. A splitter never appears as separate control.

## Task 11 — Setup views: PC view and desk view

**Files:** `apps/desktop/src/features/setup/` (new), `apps/desktop/src-tauri/src/setup_commands.rs` (new).

**Consumes:** the model of Task 10; the live output of Task 9; `setup-views-research.md` sections 3 and 4; `ux-research.md` section 5.

**Produces:** the area that sets PrismForge apart.

- [ ] Add the Setup area with two views as tabs: "PC" and "Desk". Each view is a 2D schematic with generic shapes.
- [ ] Place the detected devices into a view from a list. Start from a simple template or from an empty canvas.
- [ ] Edit a component: move, rotate in fixed steps, mirror, set the LED count and the direction, name it, group it. Each action has a keyboard way and number fields. No action needs a drag.
- [ ] Show the first LED and the direction of each component. Offer "identify" for a component.
- [ ] Show the live colours on the components. Show the draft preview of an effect across the setup before the user applies it.
- [ ] Give a list next to the canvas with the same content, for keyboard and screen reader users.
- [ ] Add undo and redo for edits of the setup. Do not suggest that an undo changes the hardware.
- [ ] Show a mirrored output and a daisy chain as what they are.
- [ ] Test first for the editor logic, the list twin, the keyboard actions and one complete journey: from a detected controller to a placed component to an effect across the setup to a saved profile.

**Acceptance:** the user builds a picture of the PC and of the desk, sees the lighting in it, and applies an effect across it. The picture never claims control that the hardware does not have.

## Task 12 — Automation: profiles on system events

**Files:** `crates/prismforge-core/src/automation/` (new), `apps/desktop/src-tauri/src/system_events/` (new, with one module for each platform), `apps/desktop/src/features/automation/` (new).

**Consumes:** `os-events-research.md`; `openrgb-behaviour.md` section A; profiles; the lifecycle of plan 1 Task 6a.

**Produces:** rules of the form "When [event], do [action]".

- [ ] Events: PrismForge starts, PrismForge connects, the system goes to sleep or hibernate, the system wakes, the user signs out or the system shuts down, PrismForge quits.
- [ ] Actions: apply a profile, switch the lights off, do nothing. "Lights off" is an action of PrismForge. It writes black or selects an "off" mode where the device has one.
- [ ] One rule for each event. No rule is on by default.
- [ ] Move "apply this profile when PrismForge connects" into Automation. Keep the stored setting valid with a tested migration.
- [ ] Windows: receive the power and session events through one small seam. Start the write early. Wait for a bounded time. Never block a sleep or a shutdown without a limit.
- [ ] After a wake: wait until the devices are back, with a bounded retry. Then apply the rule one time.
- [ ] Say in the interface what PrismForge can promise for each event. Sleep and wake: PrismForge acts when it runs. Shutdown: PrismForge tries, and the system can end it first.
- [ ] Detect that OpenRGB has its own profile settings for the same events, where the SDK permits it. Tell the user that two programs will act.
- [ ] Rules run only while PrismForge runs. Say this next to the rules.
- [ ] Test first for the rule model, the event seam with a fake source, the bounded waits, the wake sequence, and the migration.
- [ ] Record what must be tested on a real Windows 11 machine.

**Acceptance:** the user sets "lights off at sleep" and "apply profile X at wake" in plain sentences. Each automatic change is a rule that the user switched on.

---

## Order and what comes after

1. Plan 1 Task 6a (lifecycle and tray) completes first. It covers direction 5.
2. Tasks 7 and 8 come next, because each later screen uses the shell.
3. Task 9 comes before Tasks 10 and 11, because the Setup views show the live output.
4. Task 12 can run after Task 8.

Work of plan 1 that stays open and comes after this plan: Task 6b (packaging and validation on the target machine), Task 4c (measurements), more effects, audio-reactive effects, screen-reactive effects, and support for the OpenRGB Effects plugin.

## What needs the owner

- A look at each new screen. No agent can judge the look as the owner can.
- Tests on the Windows PC with OpenRGB 1.1: the tray, start with the system, sleep, wake and shutdown.
- The models and LED counts of the fans, and how the lighting connects. The Setup area needs them to be truthful for the reference PC.
