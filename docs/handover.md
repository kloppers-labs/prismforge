# Handover

Last update: 2026-11-18.

## Current state

Task 1 and Task 2 are built. Task 2 is not closed: nobody has run the app on Windows, and four native screenshots and two hand checks are missing. Task 4a, the effect engine without an interface, is built on the branch `task-4a-effects-engine`.

The repository has a Rust workspace with four crates and a desktop app. `prismforge-effects` holds six original built-in effects as pure functions, their typed control definitions and the validation of stored effect instances. `prismforge-core` holds the device types, the identity matching, the backend contract, the scene and ownership types, and the session. The session owns the backend, reconnects, keeps a readable snapshot, runs the identify action, and runs effects through an output scheduler with explicit output ownership. `prismforge-openrgb` holds our own OpenRGB SDK client for protocol 6 and the adapter. `prismforge-testkit` holds a scripted fake OpenRGB server and a simulated demo backend.

The session has commands for effects: list the effect definitions, apply or replace an effect on a device or zone, stop it, set the frame rate, declare or clear external control, and select a hardware mode. The snapshot shows the frame rate, the scene, and the owner and output condition of each device. The desktop app has no effects screen and no Tauri command for effects yet. Task 4b adds them.

`apps/desktop` is a Tauri 2 app with a Svelte 5 and TypeScript interface. It has a connection screen, a device list with device details, an identify action, a diagnostics drawer and a labelled demo mode. The shell in `apps/desktop/src-tauri` owns at most one session. The interface types and the test fixtures are generated from the Rust code.

All tests pass on macOS and in Windows CI. Windows CI also builds the unsigned installers. The app was run by hand on macOS (Apple Silicon) only. Nobody has run it on Windows. No test against a real OpenRGB server or real hardware has been done.

## Decisions

| Decision | Record |
|---|---|
| OpenRGB SDK over TCP is the hardware backend. PrismForge has its own client and does not use the `openrgb2` crate. | `docs/decisions/0001-openrgb-integration.md` |
| SDK protocol 6 is required (OpenRGB 1.0 and later). | same |
| A runtime controller ID is never a device identity. Ambiguous devices get no output. | same |
| Effects are the main feature. Own effect engine first, then Effects plugin support, then plugin editing only if a real-server test passes. | same |
| The licence is GPL-3.0-or-later. | `docs/provenance.md` |
| Identify restores the previous mode with a mode re-apply. PrismForge never saves a mode to device memory. Mode writes with user-edited settings wait for a later task. | this file |
| Effects are pure functions of values, time, LED position and seed. The six built-in effects are original code. | `docs/decisions/0003-effects-engine.md` |
| The session task also owns output: one batched write per device or zone per tick, 30 FPS by default, clamped to 1 to 60. Late ticks are skipped and counted, never queued. Static effects are written once. | same |
| One owner per device: a PrismForge effect, a hardware mode selected through PrismForge, or another program the user declared. A declared device refuses effects until the user hands it over. Stopping the last effect restores the remembered state. | same |
| One session task owns the backend. The desktop shell reads snapshots and sends commands through a queue with a fixed size. | this file |
| Demo mode uses a simulated backend. Each simulated device carries the origin "simulated". A failed real connection never shows demo devices. | this file |
| The desktop shell owns at most one session. A new session starts only after the previous one has stopped and its identify is restored. Disconnect ends the session. | `apps/desktop/README.md` |
| The window gets the latest snapshot at most once per 50 ms. Each view has a rising revision, and the interface ignores older views. | `apps/desktop/README.md` |
| The TypeScript types come from the Rust serde types through `ts-rs` (feature `ts` of `prismforge-core`). The interface test fixtures come from real sessions. Tests fail when either folder is out of date. | `apps/desktop/README.md` |
| The window has one capability with event listen and the six app commands. No Tauri plugins. | `apps/desktop/README.md` |

## Task order

The owner changed the priority on 2026-11-18. The effect engine moves earlier.

1. Task 1: OpenRGB boundary. **Complete.**
2. Task 2: desktop connection and inventory slice. **Built.** Built and tested in Windows CI. A run on Windows, the missing screenshots and two hand checks are still open.
3. Task 4: effect engine, output scheduler and output ownership. **4a built** (engine, scheduler, ownership, session commands; no interface). 4b adds the effects screen, the parameter controls and the Tauri commands. Previews, the Effects plugin provider and the measurements come after 4b.
4. Task 3: profiles.
5. Task 5: component setup and layout.
6. Task 6: lifecycle, packaging and validation on the target machine.
7. After the core engine: more colour, wave and pattern effects; audio-reactive effects; screen-reactive effects; Effects plugin provider and the test of plugin editing.

Task numbers are from `docs/plans/prismforge-implementation-plan.md`.

## Verification

Run on macOS (Apple Silicon) with Rust 1.99.0 and Node.js 22.23.5 on 2026-11-18, at the last code commit of Task 4a.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass, no warnings |
| `cargo clippy -p prismforge-desktop --all-targets --features ts -- -D warnings` | Pass, no warnings |
| `cargo test --workspace` | Pass: 162 tests. 29 in `prismforge-effects`, 61 in `prismforge-core` (20 of them for output and ownership), 49 in `prismforge-openrgb`, 9 in `prismforge-testkit`, 14 in `prismforge-desktop` (6 shell, 4 view forwarding, 3 stop-then-exit, 1 fixture drift check) |
| `cargo test -p prismforge-desktop --features ts --test bindings` | Pass (binding drift check) |
| `cargo check -p prismforge-core -p prismforge-effects -p prismforge-openrgb -p prismforge-testkit --all-targets --target x86_64-pc-windows-msvc` | Pass (type check only). The desktop crate cannot be checked this way on macOS: its Windows resource step needs `llvm-rc`. |
| `pnpm typecheck`, `pnpm test` in `apps/desktop` | Pass: 0 type errors, 24 interface tests |

`pnpm tauri:build --bundles app` last passed at the end of Task 2b (`PrismForge.app`, 10.7 MiB, not signed). Task 4a did not rebuild the app.

The scheduler and ownership tests use simulated time and the demo backend or the fake server. They check recorded writes and snapshot counters: late ticks are skipped and counted and every frame is the frame for its own moment; a refused device makes 7 tries in a minute while another device gets about 1,800 frames; a static scene gets one frame and one more only after a scene change, a reconnect or an inventory change; a declared device gets nothing until the hand-over; after an OpenRGB restart with reused IDs every frame still reaches the same fixture device.

Windows CI run 44320629268 on commit `548677b` (`windows-latest`) passed every step: format, both lint runs, `cargo test --workspace`, the binding drift check, `pnpm install --frozen-lockfile`, typecheck, interface tests and `pnpm tauri:build`. The artifact `prismforge-windows-unsigned` (about 5.2 MB) holds the MSI and NSIS installers. This is build and test evidence only. Nobody has installed or run the app on Windows. Commits after `548677b` have no CI result yet.

Checked by hand in the native macOS app:

- Connection screen at start. Nothing connected before a choice.
- Connect to `127.0.0.1:6742` with no server: the retry state with its count and the cause "Nothing answered at the OpenRGB server at 127.0.0.1:6742". No device list and no demo banner. (The retry text has changed since; see Not done.)
- Demo mode: striped banner, five simulated devices, device details, identify of a motherboard zone with the outcome text, diagnostics drawer with origin, counters, device references and events.
- From demo mode, Disconnect, then Connect to OpenRGB: retry state with no demo devices.
- A macOS Quit request ends the process.

Not done:

- Native screenshots of demo mode, a device with an identify outcome, diagnostics and a failed connection. Only `docs/screenshots/connection.png` exists.
- Closing the window by hand. The tests in `apps/desktop/src-tauri/tests/exit.rs` cover the stop-then-exit logic without a window. The Tauri event handler that calls it has no test: Tauri's mock runtime cannot exit (`request_exit` is unimplemented in tauri 2.12.6).
- A hand check of the current build. The hand check above ran on the build before the commits that name the demo backend in demo identify text and that changed the retry text. Interface tests cover both texts.
- Any run of the app on Windows.
- Any test against a real OpenRGB server.
- Any hardware write.
- Any effect on real hardware, and any frame timing on a real machine.

## Next steps

1. Task 4b: an effects screen with parameter controls that render from the effect definitions, and Tauri commands for the session's effect and ownership commands.
2. With the owner's consent and screen-capture permission, take the four missing screenshots and check the current build by hand, including closing the window during an identify.
3. Install the CI installer on the Windows PC. Check the connection screen, demo mode, and the retry state with OpenRGB closed.
4. Run the probe on the Windows PC with OpenRGB 1.1 and record the result in `docs/hardware-validation.md`. The command is in `README.md`. Remove serial numbers from the output before you commit it.
5. Connect the desktop app to that OpenRGB server and identify one device. Record what the device did. After 4b, run one effect on one device and record the frame rate it reaches and whether OpenRGB reports Direct Mode afterwards.
6. Add persistence for the list of known devices. The profiles task needs it.

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
- **The list of known devices is in memory only.** After an application restart, a device with a serial number that moved to a new location gets a new reference.
- **Native screenshots are incomplete.** `docs/screenshots/connection.png` is a capture of the native macOS window. The demo inventory, identify outcome, diagnostics and connection failure screenshots are missing: Stage Manager kept the background window as a thumbnail, and later the screen was locked. The window must be in front to capture it, which needs the owner's consent.
- **The retry number is the next retry.** The session counts retries from 1 after each loss. The interface says "retry N comes next" and never counts failed attempts. The counters in Diagnostics count all connects of the session.
- **Ambiguous devices cannot be resolved yet.** The app explains them and turns Identify off for them. A way to tell identical devices apart by hand does not exist.
- **Disconnect ends the session.** Its diagnostics and its list of known devices are gone after a disconnect. Host and port reset to the defaults when the app restarts.
- **Identify always lasts 3 seconds.** The shell sets it; there is no setting.
- **Closing during an identify waits for the restore.** The app holds the exit until the session has stopped, so the window can close a moment before the process ends.
- **No effects screen yet.** The engine, the scheduler and ownership exist only as session commands. Task 4b adds the interface.
- **No real-machine evidence for effects.** All scheduler numbers come from simulated time. Real frame timing and CPU use are not measured.
- **Colour frames are not confirmed.** The client does not read the acknowledgement of a colour write. "Frames written" counts frames sent without an error, not frames OpenRGB applied.
- **Direct mode may be enabled more often than needed.** After each inventory refresh, PrismForge enables direct mode again on each effect device that does not report Direct Mode. If OpenRGB does not report it after "set custom mode", this repeats after every refresh. The fake server never reports it.
- **A backend call in progress still delays a command.** At 30 FPS a command waits at most for one write. A hung call waits for the client timeout (2 seconds by default).
- **Shutdown restore stops after 5 seconds.** Devices not restored by then keep the last effect frame.
- **External control is only the user's statement.** PrismForge cannot see the Effects plugin or any other writer. Identify still works on a device declared external.
- **Ownership and the scene are in memory only.** They end with the session. Profiles come later.
- **Test gaps:** segments and zone modes have no tests. A partly restored identify is tested only for "previous state unknown", not for a failed second restore step.

## Missing information from the owner

- Exact RAM and GPU models.
- Case fan models, counts and LED counts.
- How the lighting connects: motherboard headers, a passive splitter, a daisy chain, or a USB controller.
- The OpenRGB device and zone list of the PC. The probe prints it.
