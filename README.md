# PrismForge

PrismForge is a free, open-source desktop application for RGB lighting control. It uses a running [OpenRGB](https://openrgb.org) server for all hardware access.

PrismForge has no subscription, no account, no advertisements and no telemetry.

## Status

PrismForge is at an early stage. The first desktop app shows the connection, the devices that OpenRGB reports, and an identify action. It has no effects, profiles or layout yet.

| Part | Status |
|---|---|
| OpenRGB SDK client (protocol 6, OpenRGB 1.0 and later) | Built. Tested against a scripted fake server only. |
| Device identity that survives an OpenRGB restart | Built. Tested against a scripted fake server only. |
| Desktop app: connection, device list, identify, diagnostics, demo mode | Built. Run by hand on macOS (Apple Silicon) only. |
| Windows build of the desktop app | Built and tested in CI on `windows-latest`: format, lint, all tests, typecheck and the unsigned installers. |
| Run of the desktop app on Windows | **Not done** |
| Test against a real OpenRGB server | **Not done** |
| Test on real hardware | **Not done** |
| Effects, profiles, layout, tray, autostart | Not started |

## Requirements

- Rust 1.99.0. `rust-toolchain.toml` selects it.
- For the desktop app: Node.js 22.22.2 or later and pnpm 10.33.0. On Windows, WebView2 (part of Windows 10 and 11).
- For a live test: OpenRGB 1.0 or later with the SDK server turned on.

## Build and test

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p prismforge-desktop --features ts --test bindings
```

## Run the desktop app

```sh
cd apps/desktop
pnpm install --frozen-lockfile
pnpm typecheck
pnpm test
pnpm tauri:dev
```

`pnpm tauri:build` builds the native app and its installers into `target/release/bundle`. The build is not signed. `apps/desktop/README.md` lists every command, the generated files and the security settings.

The app starts on the connection screen. Nothing connects until you choose:

- **Connect to OpenRGB**: give the host and port of the OpenRGB SDK server. The default is `127.0.0.1` and `6742`.
- **Explore demo mode**: five simulated devices. A striped banner labels demo mode on every screen. Demo mode sends nothing to hardware.

![The connection screen](docs/screenshots/connection.png)

Only the connection screen has a screenshot so far. Screenshots of demo mode, a device with an identify outcome, diagnostics and a failed connection are still missing.

## Probe a real OpenRGB server

The probe lists devices, zones, modes and capabilities. It writes nothing to hardware unless you give both `--device` and `--color`.

```sh
cargo run -p prismforge-openrgb --example probe -- --host 127.0.0.1 --port 6742
cargo run -p prismforge-openrgb --example probe -- --device <ref from listing> --color ff8000 [--zone 0]
```

## Repository layout

| Path | Content |
|---|---|
| `crates/prismforge-core` | Device types, capabilities, errors, identity matching, backend contract. No input or output. |
| `crates/prismforge-openrgb` | OpenRGB SDK client and the adapter for the backend contract. |
| `crates/prismforge-testkit` | Scripted fake OpenRGB server, device fixtures for tests, and the simulated demo backend. |
| `apps/desktop` | Svelte and TypeScript interface. |
| `apps/desktop/src-tauri` | Tauri shell: commands, the view event, the window capability. |
| `.github/workflows/ci.yml` | Windows CI: format, lint, tests, drift checks and the unsigned native build. |
| `docs/decisions` | Decision records. |
| `docs/handover.md` | Current state, verification commands and next steps. |
| `docs/provenance.md` | Sources and reuse rules. |
| `docs/plans` | The implementation plan and the feasibility review. |
| `docs/screenshots` | Screenshots of the native app on macOS. |

## Licence

PrismForge is licensed under GPL-3.0-or-later. See `LICENSE` and `THIRD_PARTY_NOTICES.md`.

PrismForge is not part of OpenRGB or SignalRGB, and those projects do not endorse it.
