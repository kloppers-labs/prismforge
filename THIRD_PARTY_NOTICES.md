# Third-party notices

PrismForge uses the software below. Rust versions come from `Cargo.lock`. npm versions come from `apps/desktop/pnpm-lock.yaml`. Licences come from the manifest of each package.

Update this file when a lockfile changes.

- Rust: `cargo metadata --format-version 1 --locked` prints the licence of each crate.
- npm: run `pnpm licenses list` in `apps/desktop`.

Every licence below is compatible with GPL-3.0-or-later. No dependency is GPL-2.0-only, proprietary or without a licence.

## Rust: direct dependencies

| Crate | Version | Licence | Used by |
|---|---|---|---|
| `tokio` | 1.54.1 | MIT | all crates except `prismforge-effects` |
| `thiserror` | 2.0.21 | MIT OR Apache-2.0 | all crates |
| `serde` | 1.0.229 | MIT OR Apache-2.0 | all crates |
| `tauri` | 2.12.6 | Apache-2.0 OR MIT | desktop app |
| `tauri-build` | 2.7.5 | Apache-2.0 OR MIT | desktop app, build time |
| `ts-rs` | 12.0.1 | MIT | `ts` feature only, generates the TypeScript types |
| `serde_json` | 1.0.154 | MIT OR Apache-2.0 | desktop app tests and effects crate tests; also a Tauri dependency |

## Rust: transitive dependencies

`Cargo.lock` holds 428 third-party crates for all platforms, most of them from Tauri. The count per licence on 2026-11-18:

| Licence (as the manifest writes it) | Crates |
|---|---|
| MIT OR Apache-2.0, Apache-2.0 OR MIT, MIT/Apache-2.0, Apache-2.0/MIT, Apache-2.0 / MIT | 245 |
| MIT | 105 |
| Unicode-3.0 | 18 |
| Zlib OR Apache-2.0 OR MIT, MIT OR Zlib OR Apache-2.0, MIT OR Apache-2.0 OR Zlib | 21 |
| Unlicense OR MIT, Unlicense/MIT | 11 |
| MPL-2.0 (`cssparser`, `cssparser-macros`, `dtoa-short`, `option-ext`, `selectors`) | 7 |
| Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | 3 |
| Apache-2.0 (`tao`, `sync_wrapper`) | 2 |
| BSD-3-Clause, BSD-3-Clause AND MIT, BSD-3-Clause/MIT, BSD-3-Clause OR MIT OR Apache-2.0 | 6 |
| Zlib | 2 |
| MIT OR Apache-2.0 OR LGPL-2.1-or-later | 2 |
| (MIT OR Apache-2.0) AND Unicode-3.0, Apache-2.0 AND MIT, Apache-2.0 WITH LLVM-exception, 0BSD OR MIT OR Apache-2.0, CC0-1.0 OR MIT-0 OR Apache-2.0, ISC | 6 |

## npm: direct dependencies of `apps/desktop`

| Package | Version | Licence | Role |
|---|---|---|---|
| `@tauri-apps/api` | 2.12.5 | Apache-2.0 OR MIT | runtime, bundled into the window |
| `svelte` | 5.58.0 | MIT | compiler; its runtime is bundled into the window |
| `vite` | 8.3.8 | MIT | build tool |
| `@sveltejs/vite-plugin-svelte` | 7.3.1 | MIT | build tool |
| `typescript` | 6.0.3 | Apache-2.0 | type checker |
| `svelte-check` | 4.7.8 | MIT | type checker |
| `@tsconfig/svelte` | 5.0.9 | MIT | type checker settings |
| `vitest` | 5.1.1 | MIT | test runner |
| `@testing-library/svelte` | 5.4.2 | MIT | tests |
| `jsdom` | 31.0.1 | MIT | tests |
| `@tauri-apps/cli` | 2.12.5 | Apache-2.0 OR MIT | native build tool |

## npm: transitive dependencies

On macOS on 2026-11-18, `pnpm licenses list` reports 107 packages: MIT 89, Apache-2.0 6, Apache-2.0 OR MIT 3, BSD-2-Clause 2, BSD-3-Clause 2, ISC 2, MPL-2.0 2 (`lightningcss`, a build tool), BlueOak-1.0.0 1 (`lru-cache`). Platform packages such as the Windows build of `@tauri-apps/cli` replace the macOS ones on Windows. Only `@tauri-apps/api` and the Svelte runtime ship inside the application.

## Fonts and icons

PrismForge ships no font files. The window uses the system font of the operating system. The app icon and the interface icons are original PrismForge artwork in this repository.

## Software that PrismForge talks to but does not include

- **OpenRGB** (GPL version 2). PrismForge connects to a running OpenRGB server through its network SDK. PrismForge does not include or change OpenRGB.
