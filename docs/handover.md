# Handover

Last update: 2026-11-20.

## Current state

Task 1 and Task 2 are built. Task 2 is not closed: nobody has run the app on Windows, and four native screenshots and one hand check are missing. Task 4a, the effect engine, and Task 4b, the effects screen, are built. Task 3a, the profile core in Rust, and Task 3b, the profiles screen and the shell wiring, are built. Task 6a, the application lifecycle (tray, close or quit, one instance, start-up, Settings), is built; the rest of Task 6 (packaging, the validation matrix, measurements) is not started. Plan 2 (`docs/plans/prismforge-plan-2-experience.md`) follows. Its Task 7, the window shell, is built. Its Task 8, the screen sweep, is built on branch `task-8-screen-sweep` and waits for review.

The repository has a Rust workspace with four crates and a desktop app. `prismforge-effects` holds six original built-in effects as pure functions, their typed control definitions and the validation of stored effect instances. `prismforge-core` holds the device types, the identity matching, the backend contract, the scene and ownership types, and the session. The session owns the backend, reconnects, keeps a readable snapshot, runs the identify action, and runs effects through an output scheduler with explicit output ownership. `prismforge-openrgb` holds our own OpenRGB SDK client for protocol 6 and the adapter. `prismforge-testkit` holds a scripted fake OpenRGB server and a simulated demo backend.

`prismforge-core` also holds the profile core (Task 3a): profile schema version 1, a durable local store with backups and recovery, the known-device registry on disk, capture of the session as a profile, apply through the session commands with a result per setting, and export and import without private machine data. `Profiles` runs every file operation outside the session task. See `docs/decisions/0002-profile-and-identity.md`.

The session has commands for effects: list the effect definitions, apply or replace an effect on a device or zone, stop it, set the frame rate, declare or clear external control, and select a hardware mode. Applying the same effect to the same target again keeps its start time. The snapshot shows the frame rate, the scene, and the owner and output condition of each device.

`apps/desktop` is a Tauri 2 app with a Svelte 5 and TypeScript interface. It has a connection screen, a Devices screen with device details and identify, an Effects screen, a diagnostics drawer and a labelled demo mode. The Effects screen lists every device and zone with its owner and output condition. It renders parameter controls from the effect definitions, shows a preview rendered in Rust, applies and stops effects, selects hardware modes, records that another program controls a device, and sets the frame rate. The shell in `apps/desktop/src-tauri` owns at most one session. The interface types and the test fixtures are generated from the Rust code.

The desktop app saves profiles (Task 3b). The shell opens a real store and a separate demo store below the per-user configuration folder on first use, after it holds a folder lock. It seeds each session with the stored device list, keeps the list on disk beside the session, and applies the profile marked for apply on connect once per user connect. A profile apply holds a lock in the shell that refuses every other output command. The Profiles screen lists the profiles with their contents and how many devices are found today, shows each stored setting with the reason it would be skipped, and saves, updates (change list first), applies (a result per setting), renames, duplicates, deletes (confirmed), exports and imports. Export and import use the native dialog, opened from Rust. Devices and zones can have the user's names. The status strip names the active profile and its unsaved changes.

The desktop app keeps running in the background when the user chooses it (Task 6a). It has a tray icon with the connection, the PrismForge effects and the active profile, Show or Hide, Stop all PrismForge effects and Quit. The first close of the window asks whether to keep running in the tray or to quit, and remembers the answer. Settings holds the close choice and the start-up path in four steps, each off by default: start with the system, start hidden in the tray, connect to OpenRGB at start, and the profile marked to apply on connect, named on the screen. A second start shows the running PrismForge. The app settings are a durable file beside the stores. See `docs/decisions/0004-lifecycle.md`.

The window has a new shell (plan 2, Task 7). A rail on the left holds Devices, Effects and Profiles, with Diagnostics and Settings at its foot, in every connection state. A status strip at the top of the page shows the connection, the active profile with its unsaved changes, a running profile apply, the profile set to apply on connect and a striped demo marker; its chips open panels with the actions of each state. One notice slot under the strip replaces the stacked strips. Effects without a connection shows how to connect. Diagnostics docks beside the page from 1400 px wide and covers it below that. Ctrl+1, Ctrl+2, Ctrl+4, Ctrl+, and F6 work from the keyboard; Ctrl+3 opens Setup since Task 11, and Ctrl+5 is kept for Automation. The design tokens are in `apps/desktop/src/lib/tokens.css`. `pnpm preview:ui` shows the real interface in a browser with simulated data. See `docs/decisions/0005-shell-and-design-tokens.md`.

The screens were swept against the audit (plan 2, Task 8). Effects has a command bar at the top with Apply effect, Stop effect and the edit mode (Live, Draft or Off); the lifecycle text, hardware modes and the other-program form sit behind Details; the frame rate moved to Settings. Devices has Set lighting, says what drives each device, tells devices with the same name apart, and has short zone buttons. The connection form starts from the last address and the waiting page offers a way back and demo mode. Profiles shows the selected profile first and saves the current lighting from a dialog. Settings is grouped with one line per switch. Diagnostics docks at the default window size with the compact rail and a 320 px drawer, and copies itself as text. Lists share one selection cue and one Tab stop each. See decisions 10 and 14 to 18 in `docs/decisions/0005-shell-and-design-tokens.md`.

Rows, effect cards, the status strip chips and the Diagnostics sections have context menus (plan 2, Task 8b, on branch `task-8b-context-menus`, waiting for review). A right-click, Shift+F10 or the Menu key opens an in-app menu with the actions of the buttons on the same screen; each item calls the same method as its button, keeps its reason when it is off and keeps its confirmation dialog. Destructive items sit last in the problem colour. Each Diagnostics section got a Copy section button. A release build no longer shows the WebView's own menu (Reload, Inspect) outside text fields. See decisions 19 and 20 in `docs/decisions/0005-shell-and-design-tokens.md`.

Effects work on several targets at once (plan 2, Task 9, on branch `task-9-multi-target`, waiting for review). The target list is a multi-select list with check marks, a mixed mark for a device with some zones checked, and a keyboard model where focus and selection are apart. One Apply sends one batch: every target runs the whole effect on its own LEDs with one shared start time, so they move in step; each target gets its own result (applied, covered by the whole device, skipped with the reason) and a Retry button for the targets a second try can reach. Stop works on the selection the same way, and Stop all PrismForge effects is one session command, from the tray and from Effects. Saved groups select their members again; each store keeps them in `groups.json`, a member missing today is shown and never guessed, and Everything selects every target that can take an effect. The stage shows the draft preview or the live output, the colours PrismForge last wrote per zone (a row for a linear zone, a grid for a matrix), and both views cap flashing at three a second on the screen only. See "Several targets in one command" and "Live output" in `docs/decisions/0003-effects-engine.md`, "Groups" in `docs/decisions/0002-profile-and-identity.md`, and decisions 21 to 24 in `docs/decisions/0005-shell-and-design-tokens.md`.

The Rust core has the setup model (plan 2, Task 10, on branch `task-10-setup-model`, waiting for review). A setup is components with a declared shape (strip, ring, matrix, perimeter, point, free line), the outputs they are wired to and views that place them. An output with several branches is a mirrored output, such as fans on a hub behind one header: one control, never separate ones. One branch with several components is a daisy chain: one zone split into consecutive parts. PrismForge never guesses either. A component's LED count must lie within the bounds the backend reports, and parts must fit the zone's LEDs today; PrismForge does not resize zones. A planner gives each LED a place on the plane of a view, and the session's second apply mode, "across the setup", runs one effect as one picture across several devices with the batch rules and one shared start; LEDs without a place get black and are reported. The six effects cross the plane along a direction or out from a centre, and the mode "each device" is unchanged. The setup is stored durably in `setup.json` in each store folder, and profiles moved to schema version 2, which stores each effect's mode; version 1 files are migrated on load. Task 11 adds its interface. See `docs/decisions/0006-setup-model.md`.

The Setup area has the PC view and the desk view (plan 2, Task 11, on branch `task-11-setup-views`, waiting for review). Each is a tab with a schematic drawing of generic shapes and, beside it, a list with the same components for the keyboard and screen readers. Detected devices and zones are placed from a list, or the Tower PC and Desk templates propose components that the user wires one by one; a second component on a zone is declared a mirror or the next part of a daisy chain, and nothing is wired without that choice. Every edit (move, turn in 15° steps, flip, size, name, LED count, LED 1, direction, wiring) has buttons and number fields and, from the list, keys; a drag is an extra. Edits change the drawing until Save setup; Undo and Redo change only the drawing. LED 1 and the direction are marked on each component, mirrors are drawn as one control in several places, chains as parts of one zone, and unplaced LEDs are counted. The drawing shows the live output or a draft of an effect across the view, and Apply across the view runs one picture with a result per target, the outputs that got nothing and the LEDs sent black. Effects keeps the mode of targets that run across the setup. Saved groups are made from and select components. Four commands (`setup`, `save_setup`, `apply_across`, `preview_across`) are in the lock table. See "The Setup area" in `docs/decisions/0006-setup-model.md` and decision 25 in `docs/decisions/0005-shell-and-design-tokens.md`.

All tests pass on macOS. The Task 11 commits have no Windows CI result yet. The Task 10 commits have no Windows CI result yet. The Task 9 commits have no Windows CI result yet. The Task 8b commits have no Windows CI result yet. The Task 8 commits have no Windows CI result yet. Windows CI passed on `c8c661a`, the last Task 7 commit (run 44492091594). The Task 6a commits have no Windows CI result yet. The Task 3b commits have no Windows CI result yet. Windows CI passed on the last commit of Task 4a and on the last Task 4b commit before review fix round 1; the fix round 1 commits have no Windows CI result yet. Windows CI also builds the unsigned installers. The app was run by hand on macOS (Apple Silicon) only. Nobody has run it on Windows. No test against a real OpenRGB server or real hardware has been done.

## Decisions

| Decision | Record |
|---|---|
| OpenRGB SDK over TCP is the hardware backend. PrismForge has its own client and does not use the `openrgb2` crate. | `docs/decisions/0001-openrgb-integration.md` |
| SDK protocol 6 is required (OpenRGB 1.0 and later). | same |
| A runtime controller ID is never a device identity. Ambiguous devices get no output. | same |
| Effects are the main feature. Own effect engine first, then Effects plugin support, then plugin editing only if a real-server test passes. | same |
| The licence is GPL-3.0-or-later. | `docs/provenance.md` |
| Identify restores the previous mode with a mode re-apply. PrismForge never saves a mode to device memory. Mode writes with user-edited settings wait for a later task. | this file |
| Profiles, the known-device registry and settings are versioned JSON files in one folder, one file per profile. Writes go through a synced temporary file and a rename, and keep a backup. A damaged file falls back to its backup; damaged files are renamed, never deleted. A newer major version is never written. | `docs/decisions/0002-profile-and-identity.md` |
| A stored device is matched to hardware on its identifying fields with the registry rules. Unresolved devices stay in the profile and are reported. An export has no serial, location, local reference, device name or profile id. | same |
| Applying a profile is an explicit command and reports each setting. Apply on connect is off by default and applies one marked profile once per connect. Loading and import write nothing to hardware. | same |
| Effects are pure functions of values, time, LED position and seed. The six built-in effects are original code. | `docs/decisions/0003-effects-engine.md` |
| The session task also owns output: one batched write per device or zone per tick, 30 FPS by default, clamped to 1 to 60. Late ticks are skipped and counted, never queued. Static effects are written once. | same |
| One owner per device: a PrismForge effect, a hardware mode selected through PrismForge, or another program the user declared. A declared device refuses effects until the user hands it over. Stopping the last effect restores the remembered state. | same |
| One session task owns the backend. The desktop shell reads snapshots and sends commands through a queue with a fixed size. The session publishes the snapshot that shows a command's result before it replies, and sends a snapshot only when it changed. | this file |
| Demo mode uses a simulated backend. Each simulated device carries the origin "simulated". A failed real connection never shows demo devices. | this file |
| The desktop shell owns at most one session. A new session starts only after the previous one has stopped and its identify is restored. Disconnect ends the session. | `apps/desktop/README.md` |
| The window gets the latest snapshot at most once per 50 ms. Each view has a rising revision, and the interface ignores older views. | `apps/desktop/README.md` |
| The TypeScript types come from the Rust serde types through `ts-rs` (feature `ts` of `prismforge-core`). The interface test fixtures come from real sessions. Tests fail when either folder is out of date. | `apps/desktop/README.md` |
| The window has one capability with event listen and the thirty-three app commands. The plugins are `tauri-plugin-dialog` and `tauri-plugin-single-instance`, called from Rust; the window has no plugin permission. The start entry is written per user by PrismForge itself (`windows-registry` on Windows, `auto-launch` on macOS and Linux). | `apps/desktop/README.md` |
| The real store is `store` and demo mode's is `demo-store` below the per-user configuration folder. The shell picks the store from the backend origin. | `docs/decisions/0002-profile-and-identity.md` |
| A store opens on first use, after the shell holds `instance.lock` with the standard file lock. A second PrismForge does not touch the store and says so. | same |
| While a profile applies, the shell refuses every other output command and profile save, update and delete. The lock is released on every exit path. | same |
| The shell takes its locks in one order: session slot, apply slot, output gate, on-connect report, store. A permit is taken while the session slot is held; a holder of a session handle never waits for that slot. A test joins every registered command, through its shell method, with a disconnect, a mode switch and the window close in both poll orders, and with a profile apply and each of those in all six orders. It fails when a registered command is missing from it. It does not drive the Tauri wrappers or their native dialogs. | same |
| Closing the window hides it or quits, as the user chose; the first close asks. Without a tray it always quits. Quit from the tray, Settings, the macOS app menu and the keyboard stops the session first; further quit requests wait. When the system ends PrismForge, the same stop runs for at most 4 seconds. PrismForge never blocks a system quit, logout or shutdown. | `docs/decisions/0004-lifecycle.md` |
| Start with the system, start hidden, connect at start and the apply-on-connect mark are separate switches, all off by default. With start, connect and a mark on, the lights change at system start without a click; Settings says so and names the profile. | same |
| One PrismForge per user: a second start shows the first window and exits. App settings are `app-settings.json` beside the stores, with the store's write and recovery rules. | same |
| The tray shows only the published view and the store: connection kind and state, effect devices running or waiting, the active profile. Stop all uses the per-device stop and the window shows one result per device. | same |
| Every profile result, dialog and draft on screen belongs to one session and one store. It goes when the session ends or the store changes between demo and real. Disconnect clears the apply-on-connect result. | `apps/desktop/README.md` |
| Apply on connect applies once per user connect. A reconnect after a loss does not apply again. The shell publishes, per session, whether it is not applicable, not armed, armed (with the profile), blocked by an unreadable marked profile, or fired (with its result); every interface sentence about apply on connect comes from that state. | same |
| Export and import take no path from the window. A path inside the configuration folder is refused; an import above 1 MiB is refused before it is read. | same |
| Navigation is a left rail with fixed items and one status strip at the top of the page. Global state and its actions sit in the strip; notices share one slot. Colours, type, spacing, radii, shadows, motion and layers come from one token file. The interface preview is for development only and never enters the build. | `docs/decisions/0005-shell-and-design-tokens.md` |
| Applying the same effect id to the same target keeps its start time, so a changed setting does not restart the animation. | `docs/decisions/0003-effects-engine.md` |
| A setup is components with a declared shape, outputs and views. Mirrored outputs are branches of one output and one control; a daisy chain is one branch. The user declares both; PrismForge never guesses them and never resizes a zone. | `docs/decisions/0006-setup-model.md` |
| "In sync" stays the meaning of several targets. "Across the setup" is a second, explicit apply mode with the batch rules and one shared start. LEDs no placed component covers get black and are reported as not placed. | same |
| Profiles and exports are schema version 2: each stored effect has its mode. Version 1 files are migrated on load to "each device". The setup is `setup.json` in each store folder. | `docs/decisions/0002-profile-and-identity.md` |
| The preview is rendered in Rust in bounded batches by the engine's own strip renderer. The interface only plays frames and stops when hidden or closed. | same |
| The effects screen renders controls only from the effect definitions. Rust validation is the authority. The screen states only what the user declared and what PrismForge did. | same |

## Task order

The owner changed the priority on 2026-11-18. The effect engine moves earlier.

1. Task 1: OpenRGB boundary. **Complete.**
2. Task 2: desktop connection and inventory slice. **Built.** Built and tested in Windows CI. A run on Windows, the missing screenshots and two hand checks are still open.
3. Task 4: effect engine, output scheduler and output ownership. **4a built** (engine, scheduler, ownership, session commands). **4b built** (Tauri commands, effects screen, generated parameter controls, preview, ownership hand-over). The Effects plugin provider and the measurements come next.
4. Task 3: profiles. **3a built** (schema, store, registry on disk, capture, apply, export and import, in Rust). **3b built** (shell wiring, Tauri commands, Profiles screen, device and zone names).
5. Task 5: component setup and layout.
6. Task 6: lifecycle, packaging and validation on the target machine. **6a built** (tray, close or quit, one instance, start-up, Settings). Packaging, the validation matrix and measurements come next.
7. After the core engine: more colour, wave and pattern effects; audio-reactive effects; screen-reactive effects; Effects plugin provider and the test of plugin editing.

Task numbers are from `docs/plans/prismforge-implementation-plan.md`.

## Verification

Task 11 run on macOS (Apple Silicon) on 2026-11-20, at the last code commit of Task 11.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass, no warnings |
| `cargo clippy -p prismforge-desktop --all-targets --features ts -- -D warnings` | Pass, no warnings |
| `cargo test --workspace` | Pass: 445 tests at the end of the first round (9 new in `apps/desktop/src-tauri/tests/setup.rs`: read and save, the bounds refusal, apply across with excluded and unplaced LEDs, apply to some targets, plan errors, the draft preview and its limit, the journey to a profile applied again, and the pin for a profile applied after its view changed), with the lock table now driving `setup`, `save_setup`, `apply_across` and `preview_across` in every poll order and the apply-lock refusal test covering `apply_across`, and the fixture drift check with the new setup fixtures |
| `cargo test -p prismforge-desktop --features ts --test bindings` | Pass (new: the setup model types, `SetupStoreView`, `AcrossReport`, `AcrossPreviewRequest`, `AcrossPreview`, `PlanError`; changed: `CommandError`) |
| `pnpm typecheck`, `pnpm test` in `apps/desktop` | Pass: 0 type errors, 455 interface tests in 40 files (75 new in 9 files; the rail tests changed for Setup) |
| `pnpm check:build` in `apps/desktop` | Pass: no module from `src/dev-preview` or `src/test` (325 modules, 3 files checked) |

Task 11 fix round 1 (same machine): fmt and both clippy runs pass; `cargo test --workspace` 451 passed (6 new: frames and outlines change no output, outline and frame validation, an old setup file loads unchanged, the draft preview cap); the bindings and fixture checks pass; `pnpm typecheck` 0 errors; `pnpm test` 479 tests in 42 files (24 new); `pnpm check:build` passes (331 modules). The round adds blocks, free lines with editable points and unlit outlines (case, desk, monitor, PC), truthful texts for targets in different pictures and for a whole device whose zones run across the setup, an exact reverse for a matrix, a 3:1 unlit LED, and the draft preview on a blocking thread with at most 60,000 colours per answer.

`pnpm tauri:build` was not run. Screenshots of ten Setup preview states at 1280 × 800, 1200 × 780, 920 × 600, 640 × 400 and 460 × 300 were taken in the interface preview in Chromium on macOS; they are design evidence only and are not committed.

Task 10 run on macOS (Apple Silicon) on 2026-11-20, at the last code commit of Task 10.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass, no warnings |
| `cargo clippy -p prismforge-desktop --all-targets --features ts -- -D warnings` | Pass, no warnings |
| `cargo test --workspace` | Pass: 424 tests (60 new in Task 10: setup model and planner 27, effects across a plane 8, the strip output guard 1, the session's apply across 8, setup store 9, profiles across the setup 3, schema 2 and migration 4), with the extended profile property test and the fixture drift check |
| `cargo test -p prismforge-desktop --features ts --test bindings` | Pass |
| `pnpm typecheck`, `pnpm test` in `apps/desktop` | Pass: 0 type errors, 379 interface tests in 31 files (1 new: the text of the skip reason "setup changed") |
| `pnpm check:build` in `apps/desktop` | Pass: no module from `src/dev-preview` or `src/test` (277 modules, 3 files checked) |

`pnpm tauri:build` was not run: Task 10 changes no interface code beyond the generated types and fixtures, one test helper, one store error mapping and two skip reason texts.

Task 10 fix round 1 adds 12 Rust tests (436) and 1 interface test (380 in 31 files): the band of the moving highlight follows each component's own LED spacing and no longer lights both ends of the plane at once; the planner refuses a setup that is not valid, an unknown view and a flow out of range; a stored flow out of range makes a profile not valid; a setup file that needs a newer PrismForge gives its own skip reason; apply and details give one reason in one order; frames across the setup render without a copy per frame. fmt, both clippy runs, the bindings test, typecheck and the interface tests pass.

Task 9 run on macOS (Apple Silicon) on 2026-11-20, at `1ad8a85`, the last code commit of Task 9.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass, no warnings |
| `cargo clippy -p prismforge-desktop --all-targets --features ts -- -D warnings` | Pass, no warnings |
| `cargo test --workspace` | Pass: 363 tests (32 new in Task 9: batch 10, live output 6, batch intent records 2, groups store 10, desktop shell 4), with the extended profile property test, the lock table with 15 new cases and the fixture drift check |
| `cargo test -p prismforge-desktop --features ts --test bindings` | Pass |
| `pnpm typecheck`, `pnpm test` in `apps/desktop` | Pass: 0 type errors, 366 interface tests in 30 files (36 new in Task 9) |
| `pnpm check:build` in `apps/desktop` | Pass: no module from `src/dev-preview` or `src/test` (276 modules, 3 files checked) |
| `pnpm tauri:build` in `apps/desktop` | Pass at `1ad8a85`: `PrismForge.app` (16.36 MiB) and `PrismForge_0.1.0_aarch64.dmg` (4.71 MiB), not signed, built with `CARGO_TARGET_DIR` in a scratch folder. The build ran `pnpm check:build` first (276 modules). |

Fix round 2 adds 3 interface tests (378 in 30 files). Fix round 1, at `2a4bbe3`: fmt and both clippy runs clean; `cargo test --workspace` 364 tests (1 new); bindings pass; 0 type errors; 375 interface tests in 30 files (9 new); `pnpm check:build` pass (277 modules). `pnpm tauri:build` was not run again.

The five new preview states (`effects-several-mixed`, `effects-group-missing`, `effects-batch-skips`, `effects-live`, `effects-draft`) were looked at in Orca's browser (Chromium on macOS) at 1280 × 800, 1200 × 780, 920 × 600, 640 × 400 and 460 × 300. Looking at them found two faults that the tests had not: the draft rows did not redraw, and the groups pushed the targets below the fold at 920 × 600; both are fixed. The screenshots are design evidence only.

Task 8b run on macOS (Apple Silicon) on 2026-11-19, at `474115e`, the last code commit of Task 8b fix round 2.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass, no warnings |
| `cargo clippy -p prismforge-desktop --all-targets --features ts -- -D warnings` | Pass, no warnings |
| `cargo test --workspace` | Pass: 331 tests (no Rust change in Task 8b) |
| `cargo test -p prismforge-desktop --features ts --test bindings` | Pass |
| `pnpm typecheck`, `pnpm test` in `apps/desktop` | Pass: 0 type errors, 330 interface tests in 25 files (76 new in Task 8b: 11 in fix round 1, 4 in fix round 2) |
| `pnpm check:build` in `apps/desktop` | Pass: no module from `src/dev-preview` or `src/test` (265 modules, 3 files checked) |
| `pnpm tauri:build` in `apps/desktop` | Pass at `474115e`: `PrismForge.app` (15.57 MiB) and `PrismForge_0.1.0_aarch64.dmg` (4.53 MiB), not signed, built with `CARGO_TARGET_DIR` in a scratch folder. The build ran `pnpm check:build` first (265 modules). |

Every menu was opened in the interface preview in Orca's browser (Chromium on macOS) at 1280 × 800, 1200 × 780, 920 × 600, 640 × 400 and 460 × 300, at the right and bottom edges, from the keyboard on a profile row and a zone button, and on Diagnostics docked (1440 × 900) and covering (1100 × 700). Every menu stayed inside the window with its 8 px margin. Fix round 1 retook the zone row at 640 × 400, a Diagnostics section at 460 × 300, the keyboard menus, the profile menus with the new labels, and a keyboard menu that moves with its row when the target list scrolls. The screenshots are design evidence only.

Task 8 run on macOS (Apple Silicon) on 2026-11-19, at `10a6bd0`, the last code commit of Task 8 fix round 3.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass, no warnings |
| `cargo clippy -p prismforge-desktop --all-targets --features ts -- -D warnings` | Pass, no warnings |
| `cargo test --workspace` | Pass: 331 tests (Task 8 fix round 2 added one: every change to the marked profile publishes the start profile) |
| `cargo test -p prismforge-desktop --features ts --test bindings` | Pass |
| `pnpm typecheck`, `pnpm test` in `apps/desktop` | Pass: 0 type errors, 254 interface tests in 16 files (53 new in Task 8) |
| `pnpm check:build` in `apps/desktop` | Pass: no module from `src/dev-preview` or `src/test` (256 modules, 3 files checked). Tauri now runs it as `beforeBuildCommand`. |
| `pnpm tauri:build` in `apps/desktop` | Pass at `10a6bd0`: `PrismForge.app` (15.57 MiB) and `PrismForge_0.1.0_aarch64.dmg` (4.52 MiB), not signed, built with `CARGO_TARGET_DIR` in a scratch folder. The build ran `pnpm check:build` first (256 modules). |

Screenshots of the interface preview at 1280 × 800, 1200 × 780, 920 × 600, 640 × 400 and 460 × 300, and of Diagnostics at 1200 × 780, 1100 × 700 and 1400 × 900, were looked at. They are design evidence from Chromium on macOS, not a native or Windows test.

Task 7 run on macOS (Apple Silicon) with Rust 1.99.0 and Node.js 22.23.5 on 2026-11-19, at `e52213c`, the last code commit of Task 7 fix round 1.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass, no warnings |
| `cargo clippy -p prismforge-desktop --all-targets --features ts -- -D warnings` | Pass, no warnings |
| `cargo test --workspace` | Pass: 330 tests. 98 in `prismforge-desktop`, with 6 exit tests: a user quit ends after at most 10 seconds when the stop hangs, and the stop at `Exit` does not wait again after it. |
| `cargo test -p prismforge-desktop --features ts --test bindings` | Pass |
| `pnpm typecheck`, `pnpm test` in `apps/desktop` | Pass: 0 type errors, 201 interface tests (53 for the shell in `src/lib/shell/shell.test.ts`, 3 for panel placement) |
| `pnpm check:build` in `apps/desktop` | Pass: "The production build in dist holds no module from src/dev-preview or src/test (247 modules, 3 files checked)". With `import { catalog } from "./test/fixtures/catalog"` added to `src/main.ts` it fails and names `src/test/fixtures/catalog.ts`. |
| `pnpm tauri:build` in `apps/desktop` | Pass at `e52213c`: `PrismForge.app` (15.40 MiB) and `PrismForge_0.1.0_aarch64.dmg` (4.46 MiB), not signed, built with `CARGO_TARGET_DIR` in a scratch folder. |

The shell was looked at in the interface preview in Orca's browser (Chromium on macOS) at 1280 × 800, 1200 × 780, 920 × 600 and the 200 % equivalents 640 × 400 and 460 × 300, also with a long server address and profile name. The first pass missed three faults that the review found: a chip panel that left the window at 200 %, "Diagnostics" cut in the rail at 460 × 300, and a second scroll area on Devices. Fix round 1 removed them, and a measurement in the same preview found every panel inside the window, no cut rail label and one scroll area for the page. A real Tab pass in the preview reached: Skip to content, the five rail items, the two chips, then the page. F6 went to the rail, the strip and the page. These are browser checks, not native-app or Windows checks. The native app was not started for Task 7.

Task 6a run on macOS (Apple Silicon) with Rust 1.99.0 and Node.js 22.23.5 on 2026-11-19, at `518f2db`, the last product code commit of Task 6a fix round 1, including `pnpm tauri:build`.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass, no warnings |
| `cargo clippy -p prismforge-desktop --all-targets --features ts -- -D warnings` | Pass, no warnings |
| `cargo test --workspace` | Pass: 328 tests. 36 in `prismforge-effects`, 138 in `prismforge-core` (60 for profiles, the store and the app settings file), 49 in `prismforge-openrgb`, 9 in `prismforge-testkit`, 96 in `prismforge-desktop` (2 closed-report unit tests, 6 shell, 9 effect commands, 7 profile wiring, 8 lock order, 8 command lock table, 10 apply-on-connect state, 4 view forwarding, 4 exit, 14 lifecycle decisions, 23 lifecycle through the shell, 1 fixture drift check) |
| `cargo test -p prismforge-desktop --test command_locks` | Pass, about 10 seconds. 56 cases cover all 33 registered commands through their shell or lifecycle methods, and 8 more cover every `tray::Action` of the glue, the tray model and start-up with connect at start armed: 1,536 joins. With the `quit` case removed, the command guard fails with "registered but not in the lock table: [\"quit\"]"; with the `SecondInstance` row renamed, the glue guard fails with "glue actions not in the lock table: [\"SecondInstance\"]" |
| `cargo test -p prismforge-desktop --features ts --test bindings` | Pass (binding drift check) |
| `cargo check -p prismforge-core --all-targets --target x86_64-pc-windows-msvc` | Pass (type check only). The desktop crate cannot be checked this way on macOS: its Windows resource step needs `llvm-rc`. Its Windows start-entry module was type-checked and linted for `x86_64-pc-windows-msvc` in a scratch crate that includes the same source file. |
| `pnpm typecheck`, `pnpm test` in `apps/desktop` | Pass: 0 type errors, 141 interface tests (25 for Settings, the first-close question, Quit and the tray stop report; 1 for the preview with a hidden native window) |
| `pnpm tauri:build` in `apps/desktop` | Pass at `518f2db`: `PrismForge.app` (15.40 MiB) and `PrismForge_0.1.0_aarch64.dmg` (4.45 MiB), not signed. Built with `CARGO_TARGET_DIR` set to a scratch folder, so a running copy in `target/release/bundle` was not replaced. |

Task 6a was not checked by hand. The owner may have PrismForge running, so the app was not started, and no start entry was registered or read on the build machine. No PrismForge process was started or stopped for this task.

The scheduler and ownership tests use simulated time and the demo backend or the fake server. They check recorded writes and snapshot counters: late ticks are skipped and counted and every frame is the frame for its own moment; a refused device makes 7 tries in a minute while another device gets about 1,800 frames; a static scene gets one frame and one more only after a scene change, a reconnect or an inventory change; a declared device gets nothing until the hand-over; after an OpenRGB restart with reused IDs every frame still reaches the same fixture device.

Windows CI (`windows-latest`) runs format, both lint runs, `cargo test --workspace`, the binding drift check, `pnpm install --frozen-lockfile`, typecheck, interface tests and `pnpm tauri:build`. The artifact `prismforge-windows-unsigned` (about 5.2 MB) holds the MSI and NSIS installers. Known results, all passed:

| Commit | Run | What it is |
|---|---|---|
| `548677b` | 44320629268 | Task 2b, first Windows CI |
| `2fd05a1` | 44389727399 | Last commit of Task 4a |
| `b436973` | 44396160088 | Last commit of Task 4b before review fix round 1 |
| `c8c661a` | 44492091594 | Last commit of Task 7 |

The review fix round 1 commits after `b436973` have no CI result. The controller reported that Windows CI passed on `ce6b1a8` (Task 3a fix round 1). The Task 3a fix round 2 and 3 commits have no CI result yet. This is build and test evidence only. Nobody has installed or run the app on Windows.

Checked by hand in the native macOS app built for Task 3b, in demo mode, the same way: started with `/usr/bin/open -g`, driven through the macOS accessibility tree, never brought to the front, nothing captured. The configuration folder did not exist before.

- Start: nothing was written to the configuration folder.
- Demo mode, then Profiles: the demo note, the empty state and the save form. The folder then held `instance.lock` and `demo-store/devices.json` only.
- Rainbow wave on the first demo memory module, then "Save as new profile" as "Hand check": the result said it was saved on this computer and nothing on the devices; the header showed "Profile Hand check"; the item said "Ready today."; the apply-on-connect switch was off.
- "Use Breathing" on the same module: the header showed "2 unsaved changes". "Save changes to Hand check" opened the change list (removes Rainbow wave, adds hardware mode Breathing); "Save changes" reported 2 changes.
- "Apply profile": "Hand check is applied. Its setting was accepted." with one line for the module and the frame rate.
- Devices: "Rename device" on the demo graphics card as "Desk card": the list and the heading showed it, with "Reported as Demo Graphics Card." The name was in `demo-store/devices.json`.
- The apply-on-connect switch on, Quit, start again, demo mode: "PrismForge applied Hand check when it connected" with one line, and the card still named "Desk card".
- At the end the folder held `instance.lock` and `demo-store` with its files. No `store` folder was created.

Not checked by hand for Task 3b: the native export and import dialogs, a second PrismForge process, a full keyboard pass, the look of the screen.

Checked by hand in the native macOS app built at commit `e19e8dc`, in demo mode. Later commits changed wording, the visible reason next to Stop, and the rules for automatic re-apply and result texts (review fix round 1). Interface tests cover those changes; they were not checked by hand. The app was started in the background with `open -g` and driven through the macOS accessibility tree only. No screenshot was taken and the window was never brought to the front. The checks read element text and states, not pixels:

- Devices and Effects navigation. The effect catalog loads from Rust, so the capability allows the new commands.
- Moving highlight applied to the demo graphics card: the owner changed to "PrismForge effect, running" and the result named the effect and the target. Diagnostics showed 385 frames written and 0 ticks skipped about 13 seconds later, with the window behind other windows.
- A brightness change through the slider re-applied the running effect: the scene in Diagnostics showed brightness 99.
- Stop: "Effect stopped. The demo backend accepted the requests to put back the lighting from before the effect." The owner went back to none.
- "Another program controls this device" with no name: Apply effect and the hardware modes turned off with the reason. "Hand back to PrismForge" turned them on again.
- Light bar (no Direct Mode): Apply effect off with the reason. "Use Breathing" set the owner to "hardware mode Breathing".
- Tab order: screen buttons, targets, one stop for the effect radio group, the generated controls, preview toggle, Apply effect, hardware modes, program name, frame rate. Disabled buttons are skipped. Some focus readouts repeated, so this is not a full keyboard audit.
- Closing the window with its close button ended the process, once with no effect and once with a rainbow wave running on the whole demo motherboard.

Not checked by hand: the look of the screen and of the preview animation, because nothing was captured; reduced motion; a validation error and a conflict in the native app (interface tests cover both).

Checked by hand in the native macOS app at the end of Task 2:

- Connection screen at start. Nothing connected before a choice.
- Connect to `127.0.0.1:6742` with no server: the retry state with its count and the cause "Nothing answered at the OpenRGB server at 127.0.0.1:6742". No device list and no demo banner. (The retry text has changed since; see Not done.)
- Demo mode: striped banner, five simulated devices, device details, identify of a motherboard zone with the outcome text, diagnostics drawer with origin, counters, device references and events.
- From demo mode, Disconnect, then Connect to OpenRGB: retry state with no demo devices.
- A macOS Quit request ends the process.

Not done:

- Native screenshots of demo mode, a device with an identify outcome, diagnostics and a failed connection. Only `docs/screenshots/connection.png` exists.
- Closing the window during an identify. The window close button was pressed through the accessibility tree in Task 4b, with and without an effect running, and the process ended. The Tauri event handler that calls the stop-then-exit logic still has no automated test: Tauri's mock runtime cannot exit (`request_exit` is unimplemented in tauri 2.12.6).
- Any screenshot of the effects screen. Window capture needs the owner's consent.
- A hand check of the current build. The hand check above ran on the build before the commits that name the demo backend in demo identify text and that changed the retry text. Interface tests cover both texts.
- Any run of the app on Windows.
- Any test against a real OpenRGB server.
- Any hardware write.
- Any effect on real hardware, and any frame timing on a real machine.

## Next steps

1. Plan 2, Task 8: the screen sweep on top of the new shell. Look at the Task 7 shell in the native window on macOS and on Windows.
2. Check Task 6a by hand on macOS and on Windows: the first close, hide and show from the tray, Stop all, Quit from the tray and the keyboard, a second start while hidden, and the start-up path with a test account or a disposable VM (start with the system, start hidden, connect at start, a marked profile). Remove the start entry afterwards and confirm that no PrismForge process is left.
3. With the owner's consent and screen-capture permission, take the four missing screenshots and one of the effects screen, look at the preview animation, and check the current build by hand, including closing the window during an identify.
4. Task 4: the Effects plugin provider (list, start and stop of plugin effects) with explicit ownership, and the static and animated performance measurements.
5. Install the CI installer on the Windows PC. Check the connection screen, demo mode, and the retry state with OpenRGB closed.
6. Run the probe on the Windows PC with OpenRGB 1.1 and record the result in `docs/hardware-validation.md`. The command is in `README.md`. Remove serial numbers from the output before you commit it.
7. Connect the desktop app to that OpenRGB server and identify one device. Record what the device did. After 4b, run one effect on one device and record the frame rate it reaches and whether OpenRGB reports Direct Mode afterwards.
8. With the owner's consent, check the export and import dialogs by hand on macOS and Windows, and start a second PrismForge to see the hand-over to the first window.
9. Plan 2, Task 12: automation. Before that, build the owner's PC in the Setup area against the real OpenRGB server and check the first LED and direction of each fan with Identify.

## Known issues

- **Task 8 leaves these audit items open.** There are no zoom keys or interface scale setting (A7: the layout reflows at 200 % but text size follows the system only); there is no per-device Control block on Devices (E10: hardware modes and another program stay on Effects behind Details); no reset per control, no palette redesign and no preview pause (E12, E14, A10); Task 9 closed the flash cap (A9) and draws matrix zones as a grid in the live view, but the draft preview is still a line per target (E16 in part); recovery notices have no short headline (G4); profile actions have no More menu (P5); skip reasons are not reworded (P6).
- **The Effects command bar is at the top of the page, not the bottom.** The audit drew it at the bottom. At the top it never covers the content, it never grows with results, disclosures or notices, and Apply comes right after the target list in the Tab order. Task 9 kept it there; the stage sits right under the effect cards.
- **No real-server evidence.** The fake server and the client come from the same reading of the OpenRGB document and source. A review checked the wire format against the OpenRGB 1.1 source, but a real server can still differ.
- **A request completes on its acknowledge packet.** If a real server does not send it, each request stops after the timeout (2 seconds by default) and the client closes the connection.
- **One slow controller fails the whole refresh.** A data request that gets no answer in time closes the connection. The session then reconnects.
- **A stopped backend call costs the connection.** The next call closes the connection and the caller must connect again. The session runs every call to its end, so this happens only to code that calls the backend directly.
- **Device strings must be valid UTF-8.** A device with a different text encoding is reported as an issue and cannot be used.
- **Mode re-apply is not confirmed by the server.** OpenRGB answers UpdateMode with status OK also when it rejects the mode settings. Identify reports "restored" when every restore request was accepted. Only a later inventory refresh shows the active mode.
- **Identify restore is best effort.** It uses the mode and colours that the last inventory reported. If another program changed the device after that refresh, identify restores the older state. PrismForge cannot see other writers, such as the Effects plugin. The session refreshes the inventory after each identify. If an earlier identify was not restored and the device still reports Direct Mode, the next identify reports "partly restored, previous state unknown".
- **Reconnect does not stop by itself.** The session retries with a delay that grows to a fixed cap (30 seconds by default) until the user disconnects. A server that reports a protocol below 6 is not retried. A server that does not answer the version request is retried; while it retries, the state shows "unsupported protocol" without a version, because an old server and a hung server look the same.
- **An identify during a device rescan can fail once.** If a device-list change is pending, identify reports "unchanged" with a stale-inventory error and writes nothing. The session refreshes by itself within about 250 ms. A second try then works.
- **An export cannot tell identical devices apart.** It has no serial numbers or locations. On the importing computer, two devices of the same model with the same zones stay unresolved.
- **One rare command sequence removes a saved zone effect that never ran.** The profile holds a zone effect that this build cannot run, a whole-device effect from before the profile apply still runs on the same device, and the user applies an effect to a different zone. The change list of the update shows the removal.
- **Apply on connect does not apply again after a lost connection.** When the session reconnects by itself, the effects in the session resume, and the marked profile is not applied a second time. It applies again on the next connect by the user.
- **Disconnect during a profile apply waits for the apply.** The apply holds a session handle, so the session ends after the apply's last command. Each command is short, so this is at most a moment, but a hung backend call makes it last until the client timeout. It cannot hang: other commands still wait for the session slot that the disconnect holds, but nothing the disconnect waits for (the apply, a save, a command waiting for its answer) waits for that slot. `tests/command_locks.rs` checks this for every registered command in every poll order.
- **An unreadable marked profile holds the apply lock for a moment.** When PrismForge connects with a marked profile that it cannot read, apply on connect takes the apply lock while the read fails. The view names no profile as being applied. An output command sent in that moment is refused with "PrismForge is applying a profile".
- **A second PrismForge exits.** A second start shows the first window and ends before it opens anything. A copy without the single-instance plugin (an older build) still finds the folder lock: it sees "Another PrismForge window is using the profile folder" and cannot change the settings.
- **Names need a resolved device.** A device that looks identical to another one cannot be named.
- **The hand check left demo data on the build machine.** The folder `~/Library/Application Support/io.github.kloppers-labs.prismforge` holds the demo store and the lock file from the Task 3b hand check. It is safe to delete while PrismForge is closed.
- **A profile update adds only what the user did in this session.** The session records each user command for output (apply, stop, hardware mode, declaration, hand-back, frame rate). An update applies the records after the profile's baseline to the stored profile. Everything else stays as loaded. It needs the active profile (the one applied or saved in this session) and refuses any other. The records live in memory for one session. After an app restart, apply or create a profile before the next update.
- **Power loss is not tested.** The store syncs the temporary file before the rename, and the tests fail each write step, but no test cuts power.
- **Native screenshots are incomplete.** `docs/screenshots/connection.png` is a capture of the native macOS window. The demo inventory, identify outcome, diagnostics and connection failure screenshots are missing: Stage Manager kept the background window as a thumbnail, and later the screen was locked. The window must be in front to capture it, which needs the owner's consent.
- **The retry number is the next retry.** The session counts retries from 1 after each loss. The interface says "retry N comes next" and never counts failed attempts. The counters in Diagnostics count all connects of the session.
- **Ambiguous devices cannot be resolved yet.** The app explains them and turns Identify off for them. A way to tell identical devices apart by hand does not exist.
- **Disconnect ends the session.** Its diagnostics and its list of known devices are gone after a disconnect. Host and port reset to the defaults when the app restarts.
- **Identify always lasts 3 seconds.** The shell sets it; there is no setting.
- **Quitting during an identify waits for the restore.** The app holds the exit until the session has stopped. The window hides at once, so it can be gone a moment before the process ends.
- **Lifecycle on Windows is not verified.** The tray, the start entry in `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` (written by PrismForge, quoted, per user also when elevated), the single-instance hand-over, the keyboard and the stop at logoff or shutdown have not run on Windows. The registry code was only type-checked for Windows. The NSIS uninstaller removes the `HKCU` value; the MSI does not. An `HKLM` entry from an earlier elevated build is reported on Settings with its removal command. See `docs/decisions/0004-lifecycle.md`.
- **A tray that exists but is not shown.** On some Linux desktops the tray icon can be created but not shown. Closing the window would then hide PrismForge with no way back except starting it again, which shows the window through the single-instance hand-over. Not tested.
- **Logout and shutdown are not tested.** PrismForge does not block them. It runs the release for at most 4 seconds at the final `Exit` event, under the 5 seconds after which Windows ends a hidden app or lists a visible one as blocking; no test shows whether the system waits that long. Task Manager, Activity Monitor and a crash run nothing.
- **The single-instance socket on macOS is shared.** It is `/tmp/io_github_kloppers_labs_prismforge_si.sock` for every user of the Mac. Two users signed in at once were not tested.
- **The tray and window glue has no automated test.** Tauri's mock runtime cannot exit and cannot show a tray. The decisions behind it are tested through fake window, exit and start-entry seams.
- **Settings refresh when the screen opens.** A mark changed on Profiles shows on Settings when Settings opens again; the lifecycle view is not republished after a mark.
- **The connection screen still starts with the default address.** Connect at start uses the remembered address, but the Host and Port fields show `127.0.0.1` and `6742` after a restart.
- **Several targets run in step; one picture needs the Setup area.** On Effects each selected target runs the whole effect from its own first LED, on one shared clock. One picture across devices is applied from the Setup area; Effects keeps that mode on targets that run it, and refuses a selection that mixes the two modes. A device with one zone lists only "Whole device". Zones checked one by one stay zone targets; only the device row selects the whole device.
- **The draft preview is a line of LEDs per target.** It shows the pattern on up to 60 LEDs in a line for each selected target. The live view draws a matrix zone as a grid of its width and height but does not use OpenRGB's matrix map, so a keyboard with gaps is drawn as a full grid.
- **A batch holds the output of other devices for its own backend calls.** A batch is one session command: while it takes over several devices (one direct-mode call each), the frames of other devices wait, as they wait for a restore. At most 256 targets.
- **Several targets with different values.** When the selected targets run the chosen effect with different settings, those settings say "Mixed" and show the first target's value; Live stays off until an Apply makes the running values equal, also when the user has changed every Mixed setting, and also when values start to differ under the same selection. There is no per-target editing in one selection.
- **The flash cap counts LEDs, not screen area.** It holds a frame when a tenth or more of the LEDs on the stage would flash a fourth time within a second. It does not measure the drawn size of the stage in pixels.
- **The live view is one feed.** The shell forwards one live feed at a time, at most 25 updates a second, and keeps the request for the next session; a frame between two updates is dropped, never queued. Under reduced motion the live view shows a new still frame every 2 seconds.
- **Groups are kept per store and on this computer only.** Real and demo groups are apart. Groups are not exported and not part of a profile. A group recovery notice stays in the groups panel until the app restarts; it has no Dismiss button.
- **Identical devices in a group.** A member whose device cannot be told apart from an identical one today shows "cannot be told apart from an identical device" and is not selected, as on Profiles.
- **The flash cap protects the screen, not the room.** It holds back frames in the two views only. The devices get every frame of an effect that flashes; a strobing effect at a high speed still flashes the physical lights.
- **Effects still end with the session.** Saving a profile keeps them as a file on this computer. A PrismForge effect runs only while PrismForge runs; keep PrismForge running in the tray, apply the profile again, or mark it to apply on connect.
- **A program name is free text.** The label for "another program" is the user's own words, trimmed and cut to 60 characters. Without a name it is "another program".
- **No real-machine evidence for effects.** All scheduler numbers come from simulated time. Real frame timing and CPU use are not measured.
- **Colour frames are not confirmed.** The client does not read the acknowledgement of a colour write. "Frames written" counts frames sent without an error, not frames OpenRGB applied.
- **Direct mode may be enabled more often than needed.** After each inventory refresh, PrismForge enables direct mode again on each effect device that does not report Direct Mode. If OpenRGB does not report it after "set custom mode", this repeats after every refresh. The fake server never reports it. On such a server, a fresh capture of a device in a per-LED mode would also remember PrismForge's own last frame as the colours to restore, and a later stop would report "restored" for that frame.
- **A backend call in progress still delays a command.** While effects run, a command waits at most for the one output call in progress: direct mode and each frame are separate calls. Other work still runs several calls before the next command: the end of an identify (restore steps and a refresh) and a reconnect (connect and a refresh). A hung call waits for the client timeout (2 seconds by default).
- **Shutdown restore stops after 5 seconds.** Devices not restored by then keep the last effect frame. The session keeps their remembered state for the next take-over until the inventory reports another mode than Direct Mode. Until then, identify reports "previous state unknown" for them.
- **External control is only the user's statement.** PrismForge cannot see the Effects plugin or any other writer. While the user declares a device external, PrismForge sends it nothing: effects, identify and hardware-mode selection are refused.
- **The shell is not seen in the native window.** Task 7 was checked in the browser preview only: no native window chrome, no WebView2, no forced colours, no screen reader. Focus rings were not visible in the preview screenshots, because the preview tab does not hold system focus.
- **The strip divider can stand alone.** When the strip wraps at 200 % text, the divider between the chips ends the first row.
- **Screen copy is still long.** The audit's per-screen findings (E1 to E16, P1 to P10, C1 to C9, D2 to D10) are for Task 8. The Effects paragraph now states the stored close choice, but its full rewrite (E2) is Task 8.
- **Context menus are checked in the browser preview only.** WebView2's own secondary-click and Menu-key events, the suppression of its menu in a release build, and a screen reader on the menu are not checked. Selected text outside a field cannot be copied from a right-click menu in a release build; Ctrl+C still copies it, and Diagnostics says so when the clipboard fails. Whether WebView2 sends `contextmenu` for a disabled button is not known; the guard listens in the capture phase, but if no event is sent, WebView2's own menu may appear on a disabled button.
- **The drawing takes no keyboard focus.** The list beside it holds every component and every action, with keys; the drawing is for the pointer and for sight. There are no zoom, pan, snap, align or distribute tools; the view is a fixed 1000 × 700 units and centres stay inside it.
- **Outlines and frames are drawing only.** A frame round a component's LEDs and the unlit outlines of a case, a desk, a monitor or a PC have no LEDs; no effect uses them. Older PrismForge versions keep them in the file as unknown fields.
- **Template LED counts are placeholders.** The Tower PC and Desk templates propose 8-LED fans and so on. They are never a product's count; the user sets each count, and the store checks it against the zone.
- **Apply across waits for Save.** Apply across a view uses the saved setup, so a profile saved afterwards names stored places; with unsaved changes Apply is off and says why. The draft shows the unsaved drawing.
- **A profile follows later edits of its view.** A profile stores the view id and the flow, not the places, so applying it after the view was edited uses the new places without a warning. Whether it should warn is an open question for the owner; a test pins today's behaviour.
- **Identify lights a whole zone.** For a component that is one part of a chain or one of several mirrored fans, Identify lights every LED of its device or zone and says which components light with it.
- **Live colours on the drawing are what PrismForge sends.** They are cut to each component by its part; LEDs past the parts are not drawn. Under reduced motion the drawing shows a new still frame every 2 seconds.
- **Unplaced LEDs are black across the setup.** An ARGB header reports a fixed LED count whatever is connected, so the LEDs past the declared parts get black in the mode "across the setup", and the plan lists them. PrismForge does not resize zones; a user whose fans need more LEDs resizes the zone in OpenRGB first.
- **An effect across the setup in an export.** An export keeps the view id but not the setup. On another computer those effects are skipped as "setup changed".
- **Point sampling.** Effects across the setup are sampled at each LED's point, with the moving highlight's band at least one LED spacing of the LED's own component wide. The research's footprint filter is not built; real hardware decides whether it is needed.
- **Test gaps:** segments and zone modes have no tests. A partly restored identify is tested only for "previous state unknown", not for a failed second restore step.

## What only the owner can check (Task 8)

- The screens in the native window on Windows, in WebView2, at 100 %, 150 % and 200 % display scaling.
- A Windows contrast theme: the current area, selected rows, focus rings, the demo badge and the switches must stay visible.
- A screen reader pass (Narrator or NVDA) over each screen, the notices and the Details disclosures.
- Copy as text in Diagnostics: that the clipboard works in the native window.
- Real hardware: Set lighting, the Live and Draft marks, and Hand back with a real other program.
- Context menus in the native window on Windows: a right-click, Shift+F10 and the Menu key on a device row, a profile row and the connection chip; no WebView menu (Reload, Inspect) outside text fields in the installed build; Cut, Copy and Paste in the Host field; a menu at the right and bottom edges at 200 % scaling; Narrator or NVDA reading a disabled item with its reason, announcing Shift+F10 on rows, chips and the buttons inside zone rows, and reading "is now off" once when an item turns off while focused; a right-click on a disabled button (for example Identify device on the light bar) shows no WebView menu.

## What only the owner can check (Task 9)

- Several targets on real hardware: that they start together and stay in step, the time a batch of several take-overs needs, and Retry after a real refusal.
- The live view against a real OpenRGB server: that it matches what the lights show (PrismForge cannot read the lights back), its rate in WebView2, and that it stops when the window is hidden in the tray.
- The keyboard model with Narrator or NVDA: the list box, "Some zones selected", the count line, and a blocked target's reason.
- A Windows contrast theme: the check marks, the mixed mark, the dashed mark of a blocked target, and the Draft and Live frames.
- A real keyboard with a matrix zone in the live view.
- Groups in the real store: `groups.json` beside `devices.json`, and a group whose device was unplugged.

## What only the owner can check (Task 11)

- The Setup area in the native window on Windows at 100 %, 150 % and 200 % scaling, a drag with a mouse and with a pen or touch screen, and a Windows contrast theme on the drawing (outlines, LED 1, the selection) and the list.
- Narrator or NVDA over the Setup list, the editor fields, the tabs and the Draft and Live badges.
- Building the owner's PC: the hub behind one header as mirrored fans, the cooler's two strips as a chain or a mirror, the first LED and direction of each fan with Identify, and one effect across the case fans, RAM and GPU in the right order and direction.
- Whether a profile applied after its view changed should warn (open question).

## What only the owner can check (Task 10)

- With the Setup area of Task 11: one picture across the case fans, RAM and GPU on real hardware, in the right order and direction; the first LED and direction of each fan; the hub behind `D_LED1` declared as mirrored fans and seen as one control; a daisy chain against the LED count OpenRGB reports for its header; and the skew between devices on different controllers.

## Missing information from the owner

- Exact RAM and GPU models.
- Case fan models, counts and LED counts.
- How the lighting connects: motherboard headers, a passive splitter, a daisy chain, or a USB controller.
- The OpenRGB device and zone list of the PC. The probe prints it.
