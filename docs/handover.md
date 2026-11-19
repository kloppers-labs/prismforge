# Handover

Last update: 2026-11-19.

## Current state

Task 1 and Task 2 are built. Task 2 is not closed: nobody has run the app on Windows, and four native screenshots and one hand check are missing. Task 4a, the effect engine, and Task 4b, the effects screen, are built. Task 3a, the profile core in Rust, and Task 3b, the profiles screen and the shell wiring, are built. Task 6a, the application lifecycle (tray, close or quit, one instance, start-up, Settings), is built; the rest of Task 6 (packaging, the validation matrix, measurements) is not started.

The repository has a Rust workspace with four crates and a desktop app. `prismforge-effects` holds six original built-in effects as pure functions, their typed control definitions and the validation of stored effect instances. `prismforge-core` holds the device types, the identity matching, the backend contract, the scene and ownership types, and the session. The session owns the backend, reconnects, keeps a readable snapshot, runs the identify action, and runs effects through an output scheduler with explicit output ownership. `prismforge-openrgb` holds our own OpenRGB SDK client for protocol 6 and the adapter. `prismforge-testkit` holds a scripted fake OpenRGB server and a simulated demo backend.

`prismforge-core` also holds the profile core (Task 3a): profile schema version 1, a durable local store with backups and recovery, the known-device registry on disk, capture of the session as a profile, apply through the session commands with a result per setting, and export and import without private machine data. `Profiles` runs every file operation outside the session task. See `docs/decisions/0002-profile-and-identity.md`.

The session has commands for effects: list the effect definitions, apply or replace an effect on a device or zone, stop it, set the frame rate, declare or clear external control, and select a hardware mode. Applying the same effect to the same target again keeps its start time. The snapshot shows the frame rate, the scene, and the owner and output condition of each device.

`apps/desktop` is a Tauri 2 app with a Svelte 5 and TypeScript interface. It has a connection screen, a Devices screen with device details and identify, an Effects screen, a diagnostics drawer and a labelled demo mode. The Effects screen lists every device and zone with its owner and output condition. It renders parameter controls from the effect definitions, shows a preview rendered in Rust, applies and stops effects, selects hardware modes, records that another program controls a device, and sets the frame rate. The shell in `apps/desktop/src-tauri` owns at most one session. The interface types and the test fixtures are generated from the Rust code.

The desktop app saves profiles (Task 3b). The shell opens a real store and a separate demo store below the per-user configuration folder on first use, after it holds a folder lock. It seeds each session with the stored device list, keeps the list on disk beside the session, and applies the profile marked for apply on connect once per user connect. A profile apply holds a lock in the shell that refuses every other output command. The Profiles screen lists the profiles with their contents and how many devices are found today, shows each stored setting with the reason it would be skipped, and saves, updates (change list first), applies (a result per setting), renames, duplicates, deletes (confirmed), exports and imports. Export and import use the native dialog, opened from Rust. Devices and zones can have the user's names. The header names the active profile and its unsaved changes.

The desktop app keeps running in the background when the user chooses it (Task 6a). It has a tray icon with the connection, the PrismForge effects and the active profile, Show or Hide, Stop all PrismForge effects and Quit. The first close of the window asks whether to keep running in the tray or to quit, and remembers the answer. Settings holds the close choice and the start-up path in four steps, each off by default: start with the system, start hidden in the tray, connect to OpenRGB at start, and the profile marked to apply on connect, named on the screen. A second start shows the running PrismForge. The app settings are a durable file beside the stores. See `docs/decisions/0004-lifecycle.md`.

All tests pass on macOS. The Task 6a commits have no Windows CI result yet. The Task 3b commits have no Windows CI result yet. Windows CI passed on the last commit of Task 4a and on the last Task 4b commit before review fix round 1; the fix round 1 commits have no Windows CI result yet. Windows CI also builds the unsigned installers. The app was run by hand on macOS (Apple Silicon) only. Nobody has run it on Windows. No test against a real OpenRGB server or real hardware has been done.

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
| Closing the window hides it or quits, as the user chose; the first close asks. Without a tray it always quits. Quit from the tray, Settings, the macOS app menu and the keyboard stops the session first; further quit requests wait. When the system ends PrismForge, the same stop runs for at most 6 seconds. PrismForge never blocks a system quit, logout or shutdown. | `docs/decisions/0004-lifecycle.md` |
| Start with the system, start hidden, connect at start and the apply-on-connect mark are separate switches, all off by default. With start, connect and a mark on, the lights change at system start without a click; Settings says so and names the profile. | same |
| One PrismForge per user: a second start shows the first window and exits. App settings are `app-settings.json` beside the stores, with the store's write and recovery rules. | same |
| The tray shows only the published view and the store: connection kind and state, effect devices running or waiting, the active profile. Stop all uses the per-device stop and the window shows one result per device. | same |
| Every profile result, dialog and draft on screen belongs to one session and one store. It goes when the session ends or the store changes between demo and real. Disconnect clears the apply-on-connect result. | `apps/desktop/README.md` |
| Apply on connect applies once per user connect. A reconnect after a loss does not apply again. The shell publishes, per session, whether it is not applicable, not armed, armed (with the profile), blocked by an unreadable marked profile, or fired (with its result); every interface sentence about apply on connect comes from that state. | same |
| Export and import take no path from the window. A path inside the configuration folder is refused; an import above 1 MiB is refused before it is read. | same |
| Applying the same effect id to the same target keeps its start time, so a changed setting does not restart the animation. | `docs/decisions/0003-effects-engine.md` |
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

Run on macOS (Apple Silicon) with Rust 1.99.0 and Node.js 22.23.5 on 2026-11-19, at `f401812`, the last product code commit of Task 6a fix round 1, including `pnpm tauri:build`.

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
| `pnpm tauri:build` in `apps/desktop` | Pass at `f401812`: `PrismForge.app` (15.40 MiB) and `PrismForge_0.1.0_aarch64.dmg` (4.45 MiB), not signed. Built with `CARGO_TARGET_DIR` set to a scratch folder, so a running copy in `target/release/bundle` was not replaced. |

Task 6a was not checked by hand. The owner may have PrismForge running, so the app was not started, and no start entry was registered or read on the build machine. No PrismForge process was started or stopped for this task.

The scheduler and ownership tests use simulated time and the demo backend or the fake server. They check recorded writes and snapshot counters: late ticks are skipped and counted and every frame is the frame for its own moment; a refused device makes 7 tries in a minute while another device gets about 1,800 frames; a static scene gets one frame and one more only after a scene change, a reconnect or an inventory change; a declared device gets nothing until the hand-over; after an OpenRGB restart with reused IDs every frame still reaches the same fixture device.

Windows CI (`windows-latest`) runs format, both lint runs, `cargo test --workspace`, the binding drift check, `pnpm install --frozen-lockfile`, typecheck, interface tests and `pnpm tauri:build`. The artifact `prismforge-windows-unsigned` (about 5.2 MB) holds the MSI and NSIS installers. Known results, all passed:

| Commit | Run | What it is |
|---|---|---|
| `548677b` | 44320629268 | Task 2b, first Windows CI |
| `2fd05a1` | 44389727399 | Last commit of Task 4a |
| `b436973` | 44396160088 | Last commit of Task 4b before review fix round 1 |

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

1. The UI and UX sweep with the reworked navigation bar, which the owner asked for after Task 6a.
2. Check Task 6a by hand on macOS and on Windows: the first close, hide and show from the tray, Stop all, Quit from the tray and the keyboard, a second start while hidden, and the start-up path with a test account or a disposable VM (start with the system, start hidden, connect at start, a marked profile). Remove the start entry afterwards and confirm that no PrismForge process is left.
3. With the owner's consent and screen-capture permission, take the four missing screenshots and one of the effects screen, look at the preview animation, and check the current build by hand, including closing the window during an identify.
4. Task 4: the Effects plugin provider (list, start and stop of plugin effects) with explicit ownership, and the static and animated performance measurements.
5. Install the CI installer on the Windows PC. Check the connection screen, demo mode, and the retry state with OpenRGB closed.
6. Run the probe on the Windows PC with OpenRGB 1.1 and record the result in `docs/hardware-validation.md`. The command is in `README.md`. Remove serial numbers from the output before you commit it.
7. Connect the desktop app to that OpenRGB server and identify one device. Record what the device did. After 4b, run one effect on one device and record the frame rate it reaches and whether OpenRGB reports Direct Mode afterwards.
8. With the owner's consent, check the export and import dialogs by hand on macOS and Windows, and start a second PrismForge to see the hand-over to the first window.
9. Task 5: component setup and layout. A layout will be a new field in a later profile schema version.

## Known issues

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
- **Logout and shutdown are not tested.** PrismForge does not block them. It runs the release for at most 6 seconds at the final `Exit` event; no test shows whether the system waits that long. Task Manager, Activity Monitor and a crash run nothing.
- **The single-instance socket on macOS is shared.** It is `/tmp/io_github_kloppers_labs_prismforge_si.sock` for every user of the Mac. Two users signed in at once were not tested.
- **The tray and window glue has no automated test.** Tauri's mock runtime cannot exit and cannot show a tray. The decisions behind it are tested through fake window, exit and start-entry seams.
- **Settings refresh when the screen opens.** A mark changed on Profiles shows on Settings when Settings opens again; the lifecycle view is not republished after a mark.
- **The connection screen still starts with the default address.** Connect at start uses the remembered address, but the Host and Port fields show `127.0.0.1` and `6742` after a restart.
- **One target at a time.** The effects screen applies an effect to one device or zone. Several targets need several applies. A device with one zone lists only "Whole device".
- **The preview is a line of LEDs.** It shows the pattern on up to 60 LEDs in a line, not the real device or its layout.
- **Effects still end with the session.** Saving a profile keeps them as a file on this computer. A PrismForge effect runs only while PrismForge runs; keep PrismForge running in the tray, apply the profile again, or mark it to apply on connect.
- **A program name is free text.** The label for "another program" is the user's own words, trimmed and cut to 60 characters. Without a name it is "another program".
- **No real-machine evidence for effects.** All scheduler numbers come from simulated time. Real frame timing and CPU use are not measured.
- **Colour frames are not confirmed.** The client does not read the acknowledgement of a colour write. "Frames written" counts frames sent without an error, not frames OpenRGB applied.
- **Direct mode may be enabled more often than needed.** After each inventory refresh, PrismForge enables direct mode again on each effect device that does not report Direct Mode. If OpenRGB does not report it after "set custom mode", this repeats after every refresh. The fake server never reports it. On such a server, a fresh capture of a device in a per-LED mode would also remember PrismForge's own last frame as the colours to restore, and a later stop would report "restored" for that frame.
- **A backend call in progress still delays a command.** While effects run, a command waits at most for the one output call in progress: direct mode and each frame are separate calls. Other work still runs several calls before the next command: the end of an identify (restore steps and a refresh) and a reconnect (connect and a refresh). A hung call waits for the client timeout (2 seconds by default).
- **Shutdown restore stops after 5 seconds.** Devices not restored by then keep the last effect frame. The session keeps their remembered state for the next take-over until the inventory reports another mode than Direct Mode. Until then, identify reports "previous state unknown" for them.
- **External control is only the user's statement.** PrismForge cannot see the Effects plugin or any other writer. While the user declares a device external, PrismForge sends it nothing: effects, identify and hardware-mode selection are refused.
- **Test gaps:** segments and zone modes have no tests. A partly restored identify is tested only for "previous state unknown", not for a failed second restore step.

## Missing information from the owner

- Exact RAM and GPU models.
- Case fan models, counts and LED counts.
- How the lighting connects: motherboard headers, a passive splitter, a daisy chain, or a USB controller.
- The OpenRGB device and zone list of the PC. The probe prints it.
