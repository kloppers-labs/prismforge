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
| `tauri` | 2.12.6 | Apache-2.0 OR MIT | desktop app, with the `tray-icon` feature for the tray |
| `tauri-build` | 2.7.5 | Apache-2.0 OR MIT | desktop app, build time |
| `tauri-plugin-dialog` | 2.8.4 | Apache-2.0 OR MIT | desktop app: the native save and open dialogs for profile export and import, opened from Rust only |
| `tauri-plugin-single-instance` | 2.5.5 | Apache-2.0 OR MIT | desktop app: a second start hands over to the running PrismForge |
| `auto-launch` | 0.6.0 | MIT | desktop app on macOS and Linux: the per-user start entry |
| `windows-registry` | 0.6.1 | MIT OR Apache-2.0 | desktop app on Windows: the per-user start entry in `HKCU` |
| `ts-rs` | 12.0.1 | MIT | `ts` feature only, generates the TypeScript types |
| `serde_json` | 1.0.154 | MIT OR Apache-2.0 | `prismforge-core` (profile and store files, with the `preserve_order` feature), desktop app tests, effects crate tests; also a Tauri dependency |

## Rust: transitive dependencies

The `preserve_order` feature of `serde_json` adds `indexmap` 2.14.2 (Apache-2.0 OR MIT) and `foldhash` 0.2.0 (Zlib) to its dependencies. Both were already in `Cargo.lock` through Tauri, so the counts below did not change.

`tauri-plugin-dialog` adds four crates: `tauri-plugin-dialog` itself, `tauri-plugin` 2.7.4 (Apache-2.0 OR MIT, a build helper), `tauri-plugin-fs` 2.7.1 (Apache-2.0 OR MIT) and `rfd` 0.17.2 (MIT, the native dialogs). The dialog crate uses a path type from `tauri-plugin-fs`. PrismForge does not register the file system plugin, and the window has no file system or dialog permission.

The lifecycle work adds 47 crates. `auto-launch` 0.6.0 (MIT) brings `smappservice-rs` 0.1.3 (MIT) and `objc2-service-management` on macOS and `dirs` and `os_info` (MIT); on Windows PrismForge uses `windows-registry` instead. `tauri-plugin-single-instance` brings `zbus` 5.19.0 (MIT) and its `async-*`, `zvariant` and `polling` crates for D-Bus on Linux; they are not built on Windows or macOS. The `tray-icon` feature uses `tray-icon` 0.25.1 (MIT OR Apache-2.0). Every new crate is MIT, Apache-2.0 OR MIT, Zlib OR Apache-2.0 OR MIT, or Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT.

`Cargo.lock` holds 479 third-party crates for all platforms, most of them from Tauri. The count per licence on 2026-11-19:

| Licence (as the manifest writes it) | Crates |
|---|---|
| MIT OR Apache-2.0, Apache-2.0 OR MIT, MIT/Apache-2.0, Apache-2.0/MIT, Apache-2.0 / MIT | 276 |
| MIT | 121 |
| Unicode-3.0 | 18 |
| Zlib OR Apache-2.0 OR MIT, MIT OR Zlib OR Apache-2.0, MIT OR Apache-2.0 OR Zlib | 23 |
| Unlicense OR MIT, Unlicense/MIT | 11 |
| MPL-2.0 (`cssparser`, `cssparser-macros`, `dtoa-short`, `option-ext`, `selectors`) | 7 |
| Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | 5 |
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

The interface does not use the npm package `@tauri-apps/plugin-dialog`. Rust opens the dialogs.

## npm: transitive dependencies

On macOS on 2026-11-18, `pnpm licenses list` reports 107 packages: MIT 89, Apache-2.0 6, Apache-2.0 OR MIT 3, BSD-2-Clause 2, BSD-3-Clause 2, ISC 2, MPL-2.0 2 (`lightningcss`, a build tool), BlueOak-1.0.0 1 (`lru-cache`). Platform packages such as the Windows build of `@tauri-apps/cli` replace the macOS ones on Windows. Only `@tauri-apps/api` and the Svelte runtime ship inside the application.

## Fonts and icons

PrismForge ships no font files. The window uses the system font of the operating system. The app icon, the monochrome tray icon (`icons/tray-template.svg`) and the interface icons are original PrismForge artwork in this repository.

## Software that PrismForge talks to but does not include

- **OpenRGB** (GPL version 2). PrismForge connects to a running OpenRGB server through its network SDK. PrismForge does not include or change OpenRGB.
