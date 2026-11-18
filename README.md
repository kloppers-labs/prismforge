# PrismForge

PrismForge is a free, open-source desktop application for RGB lighting control. It uses a running [OpenRGB](https://openrgb.org) server for all hardware access.

PrismForge has no subscription, no account, no advertisements and no telemetry.

## Status

PrismForge is at an early stage. There is no desktop application yet.

| Part | Status |
|---|---|
| OpenRGB SDK client (protocol 6, OpenRGB 1.0 and later) | Built. Tested against a scripted fake server only. |
| Device identity that survives an OpenRGB restart | Built. Tested against a scripted fake server only. |
| Test against a real OpenRGB server | **Not done** |
| Test on real hardware | **Not done** |
| Windows build | **Not done.** Only a type check for the Windows target was run. |
| Desktop application, effects, profiles, layout | Not started |

## Requirements

- Rust 1.99.0. `rust-toolchain.toml` selects it.
- For a live test: OpenRGB 1.0 or later with the SDK server turned on.

## Build and test

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

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
| `crates/prismforge-testkit` | Scripted fake OpenRGB server and device fixtures for tests. |
| `docs/decisions` | Decision records. |
| `docs/handover.md` | Current state, verification commands and next steps. |
| `docs/provenance.md` | Sources and reuse rules. |
| `docs/plans` | The implementation plan and the feasibility review. |

## Licence

PrismForge is licensed under GPL-3.0-or-later. See `LICENSE` and `THIRD_PARTY_NOTICES.md`.

PrismForge is not part of OpenRGB or SignalRGB, and those projects do not endorse it.
