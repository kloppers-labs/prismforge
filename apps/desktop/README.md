# PrismForge desktop app

The desktop app is a Tauri 2 shell with a Svelte 5 and TypeScript interface. The Rust shell in `src-tauri` owns one session from `prismforge-core`. The interface reads the session through typed commands and one event.

## Requirements

- Rust 1.99.0. The repository's `rust-toolchain.toml` selects it.
- Node.js 22.22.2 or later, and pnpm 10.33.0.
- Windows: WebView2, which Windows 10 and 11 include. macOS needs nothing extra.

## Commands

Run these in `apps/desktop`.

| Command | What it does |
|---|---|
| `pnpm install --frozen-lockfile` | Installs the pinned npm packages. |
| `pnpm tauri:dev` | Starts the app with live reload. |
| `pnpm dev` | Starts only the web interface on port 1420. Without the Tauri shell its commands fail. |
| `pnpm typecheck` | Runs `svelte-check` and `tsc`. |
| `pnpm test` | Runs the interface tests in jsdom. |
| `pnpm build` | Builds the web interface into `dist`. |
| `pnpm tauri:build` | Builds the native app and its installers into `target/release/bundle` at the repository root. The build is not signed. |

## Generated files

Two folders are generated from the Rust code. Do not edit them.

- `src/lib/api/bindings`: TypeScript types from the Rust serde types, made with `ts-rs`.
- `src/test/fixtures`: session snapshots and identify outcomes from real sessions against the demo backend and the fake OpenRGB server.

The tests `bindings` and `fixtures` in `src-tauri/tests` fail when a folder differs from what the Rust code produces now. To update both folders, run this at the repository root:

```sh
PRISMFORGE_UPDATE_GENERATED=1 cargo test -p prismforge-desktop --features ts --test bindings --test fixtures
```

## Commands and events

| Command | Effect |
|---|---|
| `view` | Returns the latest view. |
| `connect_openrgb(host, port)` | Stops any session, then starts one against an OpenRGB server and asks it to connect. |
| `start_demo` | Stops any session, then starts one against the simulated demo backend. |
| `disconnect` | Stops the session. A running identify is restored first. |
| `refresh` | Reads the device list again. |
| `identify(device, zone)` | Lights a device or zone white for 3 seconds, then restores it. |

The event `session-view` carries the latest view. It is sent at most once per 50 ms, and a burst of changes arrives as its latest state. Each view has a rising `revision`; the interface ignores a view with a lower number than the one it shows.

Nothing connects and nothing is written to a device until the user selects an action. Closing the window quits the app after the session has stopped.

## Security and capabilities

The window has one capability, `main` in `src-tauri/capabilities/main.json`. It allows only these permissions:

| Permission | Why |
|---|---|
| `core:event:allow-listen`, `core:event:allow-unlisten` | The interface listens for `session-view`. |
| `allow-view`, `allow-connect-openrgb`, `allow-start-demo`, `allow-disconnect`, `allow-refresh`, `allow-identify` | The six app commands above. `build.rs` declares them, so no other command is reachable. |

The app uses no Tauri plugins: no shell, file system, HTTP, dialog, opener or updater. The window loads only the bundled interface. The Content Security Policy allows scripts, styles, images and fonts from the app itself and IPC to the shell, and nothing else. The interface has no remote content, no CDN files, no telemetry and no network access of its own; only the Rust side opens the TCP connection to OpenRGB. The app runs as a normal user and needs no Administrator rights.
