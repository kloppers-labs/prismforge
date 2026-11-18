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
- `src/test/fixtures`: session snapshots, identify outcomes, the effect catalog, stop outcomes, command errors and preview batches from real sessions against the demo backend and the fake OpenRGB server.

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
| `effect_catalog` | Returns the effect definitions, the frame rate range and the preview limits. |
| `apply_effect(target, effect)` | Runs an effect instance on a device or zone. The same effect on the same target keeps its start time. |
| `stop_effect(target)` | Stops the effect on a target. Returns the restore outcome once the device has no effect left. |
| `select_mode(device, mode)` | Re-applies a reported hardware mode and stops any PrismForge effect on the device. |
| `declare_external(device, label)` | Records the user's statement that another program controls a device. The label is trimmed and cut to 60 characters. |
| `clear_external(device)` | Records that the user handed the device back. |
| `set_frame_rate(frameRate)` | Sets the output rate. Returns the rate in use after clamping to 1 to 60. |
| `preview_effect(request)` | Renders a bounded batch of preview frames in Rust. It uses no session and no backend. |

The event `session-view` carries the latest view. It is sent at most once per 50 ms, and a burst of changes arrives as its latest state. Each view has a rising `revision`; the interface ignores a view with a lower number than the one it shows.

Nothing connects and nothing is written to a device until the user selects an action. Closing the window quits the app after the session has stopped. The session restores the devices it runs effects on before it stops.

Effect commands return `CommandError`. `InvalidEffect` carries the structured validation errors, `NoEffect` names the target, and `Session` carries `Busy`, the ownership conflict and backend errors.

## Effects screen and preview

The effects screen has no effect maths. Rust computes every frame. The preview asks `preview_effect` for at most 60 LEDs and 90 frames, 20 to 1,000 ms apart, and plays the batch in a small strip. It asks for the next batch when the batch runs out, and again 200 ms after a setting changes. It stops asking and animating when its panel is closed or the window is hidden. With reduced motion it shows one still frame. The preview never calls an output command.

## Security and capabilities

The window has one capability, `main` in `src-tauri/capabilities/main.json`. It allows only these permissions:

| Permission | Why |
|---|---|
| `core:event:allow-listen`, `core:event:allow-unlisten` | The interface listens for `session-view`. |
| `allow-view`, `allow-connect-openrgb`, `allow-start-demo`, `allow-disconnect`, `allow-refresh`, `allow-identify` | The session commands above. |
| `allow-effect-catalog`, `allow-apply-effect`, `allow-stop-effect`, `allow-select-mode`, `allow-declare-external`, `allow-clear-external`, `allow-set-frame-rate`, `allow-preview-effect` | The effect commands above. `build.rs` declares all fourteen commands, so no other command is reachable. |

The app uses no Tauri plugins: no shell, file system, HTTP, dialog, opener or updater. The window loads only the bundled interface. The Content Security Policy allows scripts, styles, images and fonts from the app itself and IPC to the shell, and nothing else. The interface has no remote content, no CDN files, no telemetry and no network access of its own; only the Rust side opens the TCP connection to OpenRGB. The app runs as a normal user and needs no Administrator rights.
