# Handover

Last update: 2026-11-18.

## Current state

Task 1 and Task 2 are built. Task 2 is not closed: nobody has run the app on Windows, and four native screenshots and one hand check are missing. Task 4a, the effect engine, and Task 4b, the effects screen, are built.

The repository has a Rust workspace with four crates and a desktop app. `prismforge-effects` holds six original built-in effects as pure functions, their typed control definitions and the validation of stored effect instances. `prismforge-core` holds the device types, the identity matching, the backend contract, the scene and ownership types, and the session. The session owns the backend, reconnects, keeps a readable snapshot, runs the identify action, and runs effects through an output scheduler with explicit output ownership. `prismforge-openrgb` holds our own OpenRGB SDK client for protocol 6 and the adapter. `prismforge-testkit` holds a scripted fake OpenRGB server and a simulated demo backend.

The session has commands for effects: list the effect definitions, apply or replace an effect on a device or zone, stop it, set the frame rate, declare or clear external control, and select a hardware mode. Applying the same effect to the same target again keeps its start time. The snapshot shows the frame rate, the scene, and the owner and output condition of each device.

`apps/desktop` is a Tauri 2 app with a Svelte 5 and TypeScript interface. It has a connection screen, a Devices screen with device details and identify, an Effects screen, a diagnostics drawer and a labelled demo mode. The Effects screen lists every device and zone with its owner and output condition. It renders parameter controls from the effect definitions, shows a preview rendered in Rust, applies and stops effects, selects hardware modes, records that another program controls a device, and sets the frame rate. The shell in `apps/desktop/src-tauri` owns at most one session. The interface types and the test fixtures are generated from the Rust code.

All tests pass on macOS. Windows CI passed on the last commit of Task 4a and on the last Task 4b commit before review fix round 1; the fix round 1 commits have no Windows CI result yet. Windows CI also builds the unsigned installers. The app was run by hand on macOS (Apple Silicon) only. Nobody has run it on Windows. No test against a real OpenRGB server or real hardware has been done.

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
| The window has one capability with event listen and the fourteen app commands. No Tauri plugins. | `apps/desktop/README.md` |
| Applying the same effect id to the same target keeps its start time, so a changed setting does not restart the animation. | `docs/decisions/0003-effects-engine.md` |
| The preview is rendered in Rust in bounded batches by the engine's own strip renderer. The interface only plays frames and stops when hidden or closed. | same |
| The effects screen renders controls only from the effect definitions. Rust validation is the authority. The screen states only what the user declared and what PrismForge did. | same |

## Task order

The owner changed the priority on 2026-11-18. The effect engine moves earlier.

1. Task 1: OpenRGB boundary. **Complete.**
2. Task 2: desktop connection and inventory slice. **Built.** Built and tested in Windows CI. A run on Windows, the missing screenshots and two hand checks are still open.
3. Task 4: effect engine, output scheduler and output ownership. **4a built** (engine, scheduler, ownership, session commands). **4b built** (Tauri commands, effects screen, generated parameter controls, preview, ownership hand-over). The Effects plugin provider and the measurements come next.
4. Task 3: profiles.
5. Task 5: component setup and layout.
6. Task 6: lifecycle, packaging and validation on the target machine.
7. After the core engine: more colour, wave and pattern effects; audio-reactive effects; screen-reactive effects; Effects plugin provider and the test of plugin editing.

Task numbers are from `docs/plans/prismforge-implementation-plan.md`.

## Verification

Run on macOS (Apple Silicon) with Rust 1.99.0 and Node.js 22.23.5 on 2026-11-18, at the last code commit of Task 4b.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass, no warnings |
| `cargo clippy -p prismforge-desktop --all-targets --features ts -- -D warnings` | Pass, no warnings |
| `cargo test --workspace` | Pass: 193 tests. 36 in `prismforge-effects` (5 for the preview), 76 in `prismforge-core` (35 of them for output and ownership), 49 in `prismforge-openrgb`, 9 in `prismforge-testkit`, 23 in `prismforge-desktop` (6 shell, 9 effect commands, 4 view forwarding, 3 stop-then-exit, 1 fixture drift check) |
| `cargo test -p prismforge-desktop --features ts --test bindings` | Pass (binding drift check) |
| `cargo check -p prismforge-core -p prismforge-effects -p prismforge-openrgb -p prismforge-testkit --all-targets --target x86_64-pc-windows-msvc` | Pass (type check only). The desktop crate cannot be checked this way on macOS: its Windows resource step needs `llvm-rc`. |
| `pnpm typecheck`, `pnpm test` in `apps/desktop` | Pass: 0 type errors, 70 interface tests (22 for the effects screen, 10 for automatic re-apply, 6 for result texts, 6 for the preview) |
| `pnpm tauri:build` in `apps/desktop` | Pass: `PrismForge.app` (11.42 MiB) and `PrismForge_0.1.0_aarch64.dmg` (3.33 MiB), not signed |

The scheduler and ownership tests use simulated time and the demo backend or the fake server. They check recorded writes and snapshot counters: late ticks are skipped and counted and every frame is the frame for its own moment; a refused device makes 7 tries in a minute while another device gets about 1,800 frames; a static scene gets one frame and one more only after a scene change, a reconnect or an inventory change; a declared device gets nothing until the hand-over; after an OpenRGB restart with reused IDs every frame still reaches the same fixture device.

Windows CI (`windows-latest`) runs format, both lint runs, `cargo test --workspace`, the binding drift check, `pnpm install --frozen-lockfile`, typecheck, interface tests and `pnpm tauri:build`. The artifact `prismforge-windows-unsigned` (about 5.2 MB) holds the MSI and NSIS installers. Known results, all passed:

| Commit | Run | What it is |
|---|---|---|
| `548677b` | 44320629268 | Task 2b, first Windows CI |
| `2fd05a1` | 44389727399 | Last commit of Task 4a |
| `b436973` | 44396160088 | Last commit of Task 4b before review fix round 1 |

The review fix round 1 commits after `b436973` have no CI result yet. This is build and test evidence only. Nobody has installed or run the app on Windows.

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

1. With the owner's consent and screen-capture permission, take the four missing screenshots and one of the effects screen, look at the preview animation, and check the current build by hand, including closing the window during an identify.
2. Task 4: the Effects plugin provider (list, start and stop of plugin effects) with explicit ownership, and the static and animated performance measurements.
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
- **One target at a time.** The effects screen applies an effect to one device or zone. Several targets need several applies. A device with one zone lists only "Whole device".
- **The preview is a line of LEDs.** It shows the pattern on up to 60 LEDs in a line, not the real device or its layout.
- **Nothing is saved.** Effects, ownership statements and the frame rate end with the session. Profiles come later.
- **A program name is free text.** The label for "another program" is the user's own words, trimmed and cut to 60 characters. Without a name it is "another program".
- **No real-machine evidence for effects.** All scheduler numbers come from simulated time. Real frame timing and CPU use are not measured.
- **Colour frames are not confirmed.** The client does not read the acknowledgement of a colour write. "Frames written" counts frames sent without an error, not frames OpenRGB applied.
- **Direct mode may be enabled more often than needed.** After each inventory refresh, PrismForge enables direct mode again on each effect device that does not report Direct Mode. If OpenRGB does not report it after "set custom mode", this repeats after every refresh. The fake server never reports it. On such a server, a fresh capture of a device in a per-LED mode would also remember PrismForge's own last frame as the colours to restore, and a later stop would report "restored" for that frame.
- **A backend call in progress still delays a command.** While effects run, a command waits at most for the one output call in progress: direct mode and each frame are separate calls. Other work still runs several calls before the next command: the end of an identify (restore steps and a refresh) and a reconnect (connect and a refresh). A hung call waits for the client timeout (2 seconds by default).
- **Shutdown restore stops after 5 seconds.** Devices not restored by then keep the last effect frame. The session keeps their remembered state for the next take-over until the inventory reports another mode than Direct Mode. Until then, identify reports "previous state unknown" for them.
- **External control is only the user's statement.** PrismForge cannot see the Effects plugin or any other writer. While the user declares a device external, PrismForge sends it nothing: effects, identify and hardware-mode selection are refused.
- **Ownership and the scene are in memory only.** They end with the session. Profiles come later.
- **Test gaps:** segments and zone modes have no tests. A partly restored identify is tested only for "previous state unknown", not for a failed second restore step.

## Missing information from the owner

- Exact RAM and GPU models.
- Case fan models, counts and LED counts.
- How the lighting connects: motherboard headers, a passive splitter, a daisy chain, or a USB controller.
- The OpenRGB device and zone list of the PC. The probe prints it.
