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
- `src/test/fixtures`: session snapshots, identify outcomes, the effect catalog, stop outcomes, command errors, preview batches, and the profile store views, details, saves, applies, imports and shell views from real sessions and real stores in temporary folders, against the demo backend and the fake OpenRGB server. Profile ids are replaced by `p-0000000000000001` and on, and the store folder by `store-folder`, so the files are the same on every machine.

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
| `profile_store` | Returns the store for the current session (real or demo): the profiles with their counts and found devices, the active profile, the unsaved changes, the notices, the user's device and zone names, the name limit and the folder. |
| `profile_details(id)` | Returns every stored setting of a profile and what apply would do with it today. |
| `create_profile(name)` | Saves the session as a new profile. Refused while a profile applies. |
| `preview_profile_update(id)` | Returns the change list an update would write. Writes nothing. |
| `update_profile(id)` | Writes the user's changes since the apply or the last save into the active profile. Refused while a profile applies. |
| `rename_profile(id, name)`, `duplicate_profile(id)` | Rename or copy a profile. |
| `delete_profile(id)` | Removes the profile file and its backup. Refused while a profile applies. |
| `apply_profile(id)` | Applies a profile and returns a result per setting. Holds the apply lock. |
| `set_apply_on_connect(id)` | Marks one profile to apply when PrismForge connects, or none with `null`. |
| `export_profile(id)` | Opens the native save dialog from Rust and writes the export there. Returns `null` when the user cancels. |
| `import_profile` | Opens the native open dialog from Rust and stores the file as a new profile. Returns `null` when the user cancels. |
| `set_device_name(device, name)`, `set_zone_name(device, zone, name)` | Set or, with `null`, clear the user's name for a device or zone. |
| `dismiss_store_notice(notice)` | Removes one repaired-file notice from the list. |

The event `session-view` carries the latest view. The view also names the profile being applied (`applying`) and the result of the last apply on connect (`connect_apply`). It is sent at most once per 50 ms, and a burst of changes arrives as its latest state. Each view has a rising `revision`; the interface ignores a view with a lower number than the one it shows.

Nothing connects and nothing is written to a device until the user selects an action. One action writes more than its name says: when a profile is set to "Apply this profile when PrismForge connects", Connect also applies that profile as soon as the connection is made. Closing the window quits the app after the session has stopped. The session restores the devices it runs effects on before it stops.

Effect commands return `CommandError`. `InvalidEffect` carries the structured validation errors, `NoEffect` names the target, and `Session` carries `Busy`, the ownership conflict and backend errors. Profile commands add `ProfileApplying`, `NotConnected`, `Store` (with `NotActive` and `SessionUnavailable`), `StoreUnavailable` (`InUse` when another PrismForge holds the folder), `Unreadable` and `File` (a path rule).

## Profiles in the shell

The store is in the per-user configuration folder that Tauri resolves for `io.github.kloppers-labs.prismforge`: `%APPDATA%` on Windows, `~/Library/Application Support` on macOS, `~/.config` on Linux. The real store is its `store` folder. Demo mode uses its `demo-store` folder, so a profile of simulated devices is never listed or applied while PrismForge is connected to OpenRGB, and the other way round. Diagnostics shows the folder in use.

- **First use.** The store opens when a session starts or when the user opens Profiles. Before that, PrismForge reads and writes nothing in the folder.
- **One PrismForge per folder.** Before it opens a store, the shell takes an exclusive lock on `instance.lock` in the folder with the standard library's file lock. A second PrismForge cannot take it and does not read or write the store; its Profiles screen says so. Its sessions still run, without the stored device list. The operating system releases the lock when the process ends, also after a crash.
- **Each session.** The shell seeds the backend with the stored device list, starts the session, keeps the device list on disk beside it, and runs the apply-on-connect watcher beside it. Disconnect stops the watcher first, then the session, then waits for the last device-list write.
- **Apply on connect.** Off by default. When the user turns it on for one profile, the watcher applies that profile once each time the session becomes live after the user connects. A connection that PrismForge restores by itself after a loss does not apply it again; the effects in the session resume. PrismForge never starts or connects by itself.
- **Lock order.** Every command takes the shell's locks in one order: the session slot, the apply slot, the output gate, then a store. A command that needs a permit takes it while it holds the session slot, and a task that holds a session handle never waits for the session slot. So a disconnect, a mode switch or a window close during an apply, together with another command, always finishes.
- **The apply lock.** While a profile applies, the shell refuses every other output command (apply or stop an effect, hardware mode, declare, hand back, frame rate, identify, another apply) and saving, updating or deleting a profile, with `ProfileApplying`. A command already sent finishes first. The lock is released on every exit path: success, partial result, error, cancel and the end of the session. Disconnect also clears the apply-on-connect result from the view.
- **Export and import.** The commands take no path from the window. Rust opens the native dialog and uses only the path it returns. A path inside PrismForge's own folder is refused. An import larger than 1 MiB is refused before it is read. Neither writes to hardware.

## Effects screen and preview

The effects screen has no effect maths. Rust computes every frame. The preview asks `preview_effect` for at most 60 LEDs and 90 frames, 20 to 1,000 ms apart, and plays the batch in a small strip. It asks for the next batch when the batch runs out, and again 200 ms after a setting changes. It stops asking and animating when its panel is closed or the window is hidden. With reduced motion it shows one still frame. The preview never calls an output command.

## Security and capabilities

The window has one capability, `main` in `src-tauri/capabilities/main.json`. It allows only these permissions:

| Permission | Why |
|---|---|
| `core:event:allow-listen`, `core:event:allow-unlisten` | The interface listens for `session-view`. |
| `allow-view`, `allow-connect-openrgb`, `allow-start-demo`, `allow-disconnect`, `allow-refresh`, `allow-identify` | The session commands above. |
| `allow-effect-catalog`, `allow-apply-effect`, `allow-stop-effect`, `allow-select-mode`, `allow-declare-external`, `allow-clear-external`, `allow-set-frame-rate`, `allow-preview-effect` | The effect commands above. |
| `allow-profile-store`, `allow-profile-details`, `allow-create-profile`, `allow-preview-profile-update`, `allow-update-profile`, `allow-rename-profile`, `allow-duplicate-profile`, `allow-delete-profile`, `allow-apply-profile`, `allow-set-apply-on-connect`, `allow-export-profile`, `allow-import-profile`, `allow-set-device-name`, `allow-set-zone-name`, `allow-dismiss-store-notice` | The profile commands above. `build.rs` declares all twenty-nine commands, so no other app command is reachable. |

The app uses one Tauri plugin: `tauri-plugin-dialog`, for the native save and open dialogs of profile export and import. Only Rust calls it. The capability grants no dialog permission, so the window cannot open a dialog or read a chosen path. The plugin adds a small script to the window that routes `window.alert` and `window.confirm` to its message command; the interface uses neither, and without the permission the command is refused. The app registers no shell, file system, HTTP, opener or updater plugin. The window loads only the bundled interface. The Content Security Policy allows scripts, styles, images and fonts from the app itself and IPC to the shell, and nothing else. The interface has no remote content, no CDN files, no telemetry and no network access of its own; only the Rust side opens the TCP connection to OpenRGB. The app runs as a normal user and needs no Administrator rights.
