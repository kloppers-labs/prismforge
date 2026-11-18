# Handover

Last update: 2026-11-18.

## Current state

Task 1 of the implementation plan is complete: the OpenRGB integration boundary. The repository has a Rust workspace with three crates. `prismforge-core` holds the device types, the identity matching and the backend contract. `prismforge-openrgb` holds our own OpenRGB SDK client for protocol 6 and the adapter. `prismforge-testkit` holds a scripted fake OpenRGB server. All 47 tests pass against the fake server. No test against a real OpenRGB server, real hardware or Windows has been done. No desktop application exists yet.

## Decisions

| Decision | Record |
|---|---|
| OpenRGB SDK over TCP is the hardware backend. PrismForge has its own client and does not use the `openrgb2` crate. | `docs/decisions/0001-openrgb-integration.md` |
| SDK protocol 6 is required (OpenRGB 1.0 and later). | same |
| A runtime controller ID is never a device identity. Ambiguous devices get no output. | same |
| Effects are the main feature. Own effect engine first, then Effects plugin support, then plugin editing only if a real-server test passes. | same |
| The licence is GPL-3.0-or-later. | `docs/provenance.md` |
| Hardware-mode writes are deferred to the effects task. | this file |

## Task order

The owner changed the priority on 2026-11-18. The effect engine moves earlier.

1. Task 1: OpenRGB boundary. **Complete.**
2. Task 2: desktop connection and inventory slice (Tauri and Svelte).
3. Task 4: effect engine, output scheduler and output ownership.
4. Task 3: profiles.
5. Task 5: component setup and layout.
6. Task 6: lifecycle, packaging and validation on the target machine.
7. After the core engine: more colour, wave and pattern effects; audio-reactive effects; screen-reactive effects; Effects plugin provider and the test of plugin editing.

Task numbers are from `docs/plans/prismforge-implementation-plan.md`.

## Verification

Run on macOS (Apple Silicon) with Rust 1.99.0 on 2026-11-18, at the last code commit of Task 2a (backend half of Task 2).

| Command | Result |
|---|---|
| `cargo fmt --all --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass, no warnings |
| `cargo test --workspace` | Pass: 39 tests in `prismforge-core` (28 session tests), 49 tests in `prismforge-openrgb` (2 in the probe example), 9 tests in `prismforge-testkit` |
| `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` | Pass (type check only) |

Not run:

- Any test against a real OpenRGB server.
- Any hardware write.
- A Windows build. The implementer ran only a type check for the Windows target.

## Next steps

1. Run the probe on the Windows PC with OpenRGB 1.1 and record the result in `docs/hardware-validation.md`. The command is in `README.md`. Remove serial numbers from the output before you commit it.
2. Build Task 2: the desktop slice with connection status, inventory, diagnostics and a labelled demo mode.
3. Add a Windows runner in GitHub Actions for format, lint, tests and the native build.
4. Add persistence for the list of known devices. The profiles task needs it.

## Known issues

- **No real-server evidence.** The fake server and the client come from the same reading of the OpenRGB document and source. A review checked the wire format against the OpenRGB 1.1 source, but a real server can still differ.
- **A request completes on its acknowledge packet.** If a real server does not send it, each request stops after the timeout (2 seconds by default) and the client closes the connection.
- **One slow controller fails the whole refresh.** A data request that gets no answer in time closes the connection. The session then reconnects.
- **A stopped backend call costs the connection.** The next call closes the connection and the caller must connect again. The session runs every call to its end, so this happens only to code that calls the backend directly.
- **Device strings must be valid UTF-8.** A device with a different text encoding is reported as an issue and cannot be used.
- **Mode re-apply is not confirmed by the server.** OpenRGB answers UpdateMode with status OK also when it rejects the mode settings. Identify reports "restored" when every restore request was accepted. Only a later inventory refresh shows the active mode.
- **Identify restore is best effort.** It uses the mode and colours that the last inventory reported. If another program changed the device after that refresh, identify restores the older state. PrismForge cannot see other writers, such as the Effects plugin. The session refreshes the inventory after each identify. If an earlier identify was not restored and the device still reports Direct Mode, the next identify reports "partly restored, previous state unknown".
- **Reconnect does not stop by itself.** The session retries with a delay that grows to a fixed cap (30 seconds by default) until the user disconnects. A server that reports a protocol below 6 is not retried. A server that does not answer the version request is retried; while it retries, the state shows "unsupported protocol" without a version, because an old server and a hung server look the same.
- **The list of known devices is in memory only.** After an application restart, a device with a serial number that moved to a new location gets a new reference.
- **Test gaps:** segments and zone modes have no tests. A partly restored identify is tested only for "previous state unknown", not for a failed second restore step.

## Missing information from the owner

- Exact RAM and GPU models.
- Case fan models, counts and LED counts.
- How the lighting connects: motherboard headers, a passive splitter, a daisy chain, or a USB controller.
- The OpenRGB device and zone list of the PC. The probe prints it.
