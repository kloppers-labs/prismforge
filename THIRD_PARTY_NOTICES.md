# Third-party notices

PrismForge uses the software below. Versions come from `Cargo.lock`. Licences come from the manifest of each crate.

Update this file when `Cargo.lock` changes. The command `cargo metadata --format-version 1` prints the licence of each crate.

## Direct dependencies

| Crate | Version | Licence |
|---|---|---|
| `tokio` | 1.54.1 | MIT |
| `thiserror` | 2.0.21 | MIT OR Apache-2.0 |
| `serde` | 1.0.229 | MIT OR Apache-2.0 |

## Transitive dependencies

| Crate | Version | Licence |
|---|---|---|
| `bytes` | 1.12.2 | MIT |
| `libc` | 0.2.191 | MIT OR Apache-2.0 |
| `mio` | 1.2.5 | MIT |
| `pin-project-lite` | 0.2.17 | Apache-2.0 OR MIT |
| `proc-macro2` | 1.0.107 | MIT OR Apache-2.0 |
| `quote` | 1.0.47 | MIT OR Apache-2.0 |
| `serde_core` | 1.0.229 | MIT OR Apache-2.0 |
| `serde_derive` | 1.0.229 | MIT OR Apache-2.0 |
| `socket2` | 0.6.5 | MIT OR Apache-2.0 |
| `syn` | 3.0.10 | MIT OR Apache-2.0 |
| `thiserror-impl` | 2.0.21 | MIT OR Apache-2.0 |
| `tokio-macros` | 2.7.2 | MIT |
| `unicode-ident` | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| `wasi` | 0.11.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| `windows-link` | 0.2.1 | MIT OR Apache-2.0 |
| `windows-sys` | 0.61.2 | MIT OR Apache-2.0 |

## Software that PrismForge talks to but does not include

- **OpenRGB** (GPL version 2). PrismForge connects to a running OpenRGB server through its network SDK. PrismForge does not include or change OpenRGB.
