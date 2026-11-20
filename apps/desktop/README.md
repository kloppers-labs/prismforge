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
| `pnpm preview:ui` | Starts the interface preview on port 1430: the real interface with simulated data. See below. |
| `pnpm typecheck` | Runs `svelte-check` and `tsc`. |
| `pnpm test` | Runs the interface tests in jsdom. |
| `pnpm build` | Builds the web interface into `dist`. |
| `pnpm check:build` | Builds the web interface as `pnpm build` does, into the folder that the Tauri bundle takes (`dist`), and fails if the bundler used any module from `src/dev-preview` or `src/test`, or if the output holds preview text. CI runs it. |
| `pnpm tauri:build` | Builds the native app and its installers into `target/release/bundle` at the repository root. The build is not signed. Tauri builds the interface with `pnpm check:build`, so the bundle holds the checked files. |

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
| `apply_effect_batch(targets, effect)` | Runs one effect on several targets with one shared start time. Returns a result per target: applied, covered by another target of the batch, or skipped with the reason. At most 256 targets. |
| `stop_effect_batch(targets)` | Stops the effect on each target, with the same result form. |
| `stop_all_effects` | Stops every PrismForge effect, one result per device. The tray's Stop all uses the same session command. |
| `watch_live_output(targets)` | Sends the colours PrismForge last wrote to these targets as `live-output` events, at most 25 per second, until the next call, also from a later session; an empty list stops it. Makes no backend call. |
| `groups` | Returns the groups of the store in use, each member matched against the devices of now (resolved, unavailable, missing, ambiguous or changed), the notices and the limits. |
| `create_group(name, targets)`, `rename_group(id, name)` | Make a group of targets that the session reports now, or rename one. |
| `add_group_members(id, targets)`, `remove_group_member(id, target)`, `delete_group(id)` | Edit or delete a group. Members missing today are kept until removed. |
| `setup` | Returns the stored setup of the store in use, each output matched against the devices of now, and the limits. Refused with `Store(NewerFile)` while `setup.json` needs a newer PrismForge. |
| `save_setup(setup)` | Stores a setup after the store's checks: valid ids, names and shapes, no shared LEDs, and new or changed outputs that fit a device or zone reported now. Sends nothing to a device. |
| `apply_across(view, flow, effect, targets)` | Runs one effect across a view of the saved setup, on every placed output or only `targets`, with one shared start. Returns the batch report, the outputs that got nothing (zone changed or parts no longer fit) and the LEDs sent black. Refused while a profile applies; a `Plan` error sends nothing. |
| `preview_across(request)` | Renders draft frames of an effect across a view of the setup it is given, per target, in Rust. Uses no session and no backend and runs on a blocking thread; at most 16,384 LEDs and 60,000 colours (LEDs times frames) per answer. |
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
| `lifecycle` | Returns the lifecycle view and sends it as `lifecycle-view`: the app settings, the tray, the start entry as the system reports it, whether the window is shown, the first-close question, the last tray stop report, the profile that start-up applies and the per-user folder. |
| `change_setting(change)` | Changes one app setting: the close choice, start with the system, start hidden or connect at start. Returns the new view; a refusal is a `Setting` error and the view still shows what holds. |
| `answer_close(choice)` | Answers the first-close question: keep running in the tray, quit, or `null` to cancel. Stores the choice and performs it. |
| `quit` | Hides the window and asks the app to exit; the exit handler stops the session first. Ignored while a quit already runs. |

The event `session-view` carries the latest view. The view also names the profile being applied (`applying`), the result of the last apply on connect (`connect_apply`), and whether this session will still apply a marked profile when it connects (`on_connect`: not applicable, not armed, armed with the profile, or fired). It is sent at most once per 50 ms, and a burst of changes arrives as its latest state. Each view has a rising `revision`; the interface ignores a view with a lower number than the one it shows.

Nothing connects and nothing is written to a device until the user selects an action, with two exceptions the user turns on: "Connect to OpenRGB when PrismForge starts" connects at start, and when a profile is set to "Apply this profile when PrismForge connects", any connect also applies that profile as soon as the connection is made. Closing the window hides it or quits, as the user chose; the first close asks. Quit stops the session before the process ends. The session restores the devices it runs effects on before it stops. `docs/decisions/0004-lifecycle.md` describes the tray, start-up and quit.

The event `live-output` carries the colours PrismForge last wrote, per device and zone, while the live view on Effects or the live colours in Setup ask for them.

The event `lifecycle-view` carries the lifecycle view after each change. Each has a rising `revision` of its own.

Effect commands return `CommandError`. `InvalidEffect` carries the structured validation errors, `NoEffect` names the target, and `Session` carries `Busy`, the ownership conflict and backend errors. Profile commands add `ProfileApplying`, `NotConnected`, `Store` (with `NotActive` and `SessionUnavailable`), `StoreUnavailable` (`InUse` when another PrismForge holds the folder), `Unreadable` and `File` (a path rule). Settings add `Setting`: no tray, no remembered address, or a start entry the system refused or did not keep. The setup commands add `Plan` (an unknown view, a setup that is not valid, a flow out of range) and `PreviewTooLarge`.

## The window

The window has three parts. `src/lib/shell` holds them. `docs/decisions/0005-shell-and-design-tokens.md` explains the choices.

- **The rail** on the left lists the areas: Devices, Effects, Setup and Profiles, then Diagnostics and Settings at its foot. The list is the same in every connection state. The current area has a filled background, an accent bar and a bold label. Below 1100 px, and while Diagnostics docks below 1400 px, the rail is 72 px wide and shows each label under its icon.
- **The status strip** at the top of the page shows the connection, demo mode and the active profile. The connection chip opens a panel with the reason in words and the actions for that state: Rescan devices, or Connect to OpenRGB and Start demo mode; Disconnect or Stop retrying sit apart below a line. A panel whose only action ends the session opens with focus on the panel, not on that action. The profile chip shows unsaved changes, a profile that is being applied, and the profile set to apply on connect; its panel has Save changes and Open Profiles. In demo mode a striped "Demo mode" marker and a striped edge stay on every screen.
- **The notice slot** under the strip holds command problems, a failed close or quit, connection problems, repaired files, the tray stop report and the apply-on-connect result. Problems come first, then cautions, then the rest; within a tone the newest is first. With more than one notice only the first shows, with "Show all N notices", and a notice that arrives behind the count is announced. A connection notice stays until the connection changes.

Without a connection, Devices shows the connection form, and Effects shows how to connect. From 1182 px wide, Diagnostics opens beside the page: 320 px wide with the compact rail below 1400 px, 400 px wide from 1400 px. Below 1182 px it covers the page, which cannot take focus until Diagnostics closes; choosing an area, any action in the status strip and any dialog close it first. The notices stay visible above it. Diagnostics has Copy as text for a bug report, and Copy section in each section; the copy leaves out the folder path.

## The screens

- **Devices.** The list is one Tab stop; the arrow keys move through it. Devices with the same name show their location, serial tail or count. The device header holds Set lighting (opens Effects with the device chosen), Identify device and Rename, and says what drives the device now. Zone rows have short Identify and Rename buttons.
- **Connection.** The form starts from the address of the last Connect. When a profile is marked to apply on connect, a line by Connect says that connecting applies it. Both ways in, OpenRGB and demo mode, stay side by side down to 920 px. While PrismForge waits for OpenRGB, the page offers Stop and change the address and Start demo mode.
- **Effects.** The target list is a multi-select list with check marks: a click lights one target alone, Space or a Ctrl-click adds or removes a target, Shift adds a range, Ctrl+A selects every target that can take an effect; the arrow keys only move focus. A device with some zones checked shows a mixed mark. A target that cannot take an effect says why and stays out of a selection of several. With one target, a command bar holds the target, Apply effect, Stop effect, the edit mode (Live when edits go to the device, Draft when they wait for Apply, Off with the reason) and the output line, as before. With several, the bar says "Apply to 3 targets" and "Stop on 3 targets", Live edits go to all of them, and the result lists every target with a summary and a Retry button for the targets a second try can reach. Settings that differ between the targets say Mixed and keep Live off. Groups below the targets select their members (Everything selects every target that can take an effect); New group…, Add selection to a group…, Rename…, Delete… and Remove edit them. Stop all PrismForge effects sits at the foot of the list. The stage shows the draft preview or the live output. The whole effect card selects the effect. A device left to another program shows a notice with Hand back to PrismForge. Hardware modes and the other-program form sit behind Details.
- **Setup.** Tabs PC and Desk each show a drawing of generic shapes and, beside it, a list with the same components for the keyboard and screen readers. Place a detected device or zone (a strip, ring, fan with two rings, matrix, screen edge, single light, free line or block), add unlit outlines of a case, a desk, a monitor or a PC, or start from the Tower PC or Desk template and choose each component's output; a second component on a zone is declared a mirror or the next part of a daisy chain. Move, turn in 15° steps, flip, size, rename, set the LED count, LED 1 and the direction with buttons and number fields or, in the list, Alt with the arrows, `[` and `]`, Shift+H and Delete; a drag is an extra. Edits change the drawing until Save setup; Undo and Redo change only the drawing. The drawing shows a draft of an effect across the view or the live output, and Apply across the view runs one picture over it with a result per target. See `docs/decisions/0006-setup-model.md`.
- **Profiles.** The selected profile comes first. Save current lighting… in the list header opens a dialog. Results show under the profile's actions.
- **Settings.** Grouped: closing the window, starting with the system, effect output (the frame rate), files and quit. Each switch says its consequence in one line; more sits behind Details.

Rows, effect cards, the two chips and the Diagnostics sections have a context menu with the actions of their buttons; see decisions 19 and 20 in `docs/decisions/0005-shell-and-design-tokens.md`. A release build shows no WebView menu (Reload, Inspect) outside text fields; a development build still shows it where PrismForge has no menu.

Long explanations sit behind a Details disclosure under one short line. Selected rows everywhere use the rail's cue. See decisions 14 to 18 in `docs/decisions/0005-shell-and-design-tokens.md`.

| Keys | What they do |
|---|---|
| Ctrl+1, Ctrl+2, Ctrl+3, Ctrl+4 (Command on macOS) | Open Devices, Effects, Setup or Profiles. Ctrl+5 is kept for Automation. The keys count by position, so they work on layouts where digits need Shift. Not while you type in a field or while a dialog is open. |
| Ctrl+, (Command-, on macOS) | Opens Settings. |
| F6, Shift+F6 | Move between the rail, the status strip, the notices, the page and Diagnostics. |
| Tab, as the first key | Reaches "Skip to content". |
| Arrow keys, Home, End, Escape in a chip panel | Move between its actions; Escape closes it and returns to the chip. A click outside also closes it. |
| Arrow keys, Space, Shift, Ctrl+A, Enter in the Effects target list | Arrows, Home and End move focus only. Space toggles the target, Shift with an arrow adds the next one, Shift+Space adds a range, Ctrl+A (Command-A) selects all that can take an effect, Enter lights only this target. |
| Arrow keys, Space, Shift, Ctrl+A, Enter, Escape in the Setup list | As in the Effects target list; Escape clears the selection. Alt with an arrow moves the selection (with Shift by 50), `[` and `]` turn it, Shift+H flips it, Delete takes it out of the view. Ctrl+Z, Ctrl+Shift+Z or Ctrl+Y, and Ctrl+S undo, redo and save on the Setup page outside text fields. |
| Shift+F10, the Menu key, or a right-click | Opens the context menu of the focused row, card, chip or Diagnostics section, at that item. Arrow keys wrap, Home and End jump, a letter moves to the next item that starts with it, Enter or Space runs it, Escape or Tab closes the menu and returns focus. Every item is also a button on the same screen. In text fields the system's Cut, Copy and Paste menu stays. |
| Ctrl+Q (Windows and Linux) | Quits PrismForge. |

The design tokens are in `src/lib/tokens.css`: colour roles, type, spacing, radii, shadows, motion, layers and sizes. Use a token instead of a literal value.

## Interface preview

`pnpm preview:ui` serves the real `App` at `http://localhost:1430` with a fake Tauri transport and the fixtures in `src/test/fixtures`. It is for design work, screenshots and contributors without hardware. It never starts Rust and never reaches hardware. A badge in the corner says "Interface preview: simulated data" and opens a list of states.

- `?state=<name>` picks a state. The names are in `src/dev-preview/scenarios.ts`, for example `first-run`, `first-run-remembered`, `retrying`, `lost`, `demo-unsaved`, `effects-running`, `profiles-unresolved`, `applying`, `notices`, `settings`, `diagnostics`, `effects-demo-diagnostics`, `long-names`, and for Task 9 `effects-several-mixed`, `effects-group-missing` (a real-store group with a member not found today), `effects-batch-skips`, `effects-live` and `effects-draft`, and for Task 11 `setup-empty`, `setup-template`, `setup-live`, `setup-mirror`, `setup-desk`, `setup-draft`, `setup-excluded`, `setup-list`, `setup-library` and `setup-points`. A state with an OpenRGB session uses a copy of a store fixture marked as the real store.
- `&area=Effects` opens an area. `&open=connection` or `&open=profile` opens a chip panel. `&diagnostics` opens Diagnostics.
- Commands that the preview does not simulate fail with a plain message.

The preview has its own Vite config (`vite.preview.config.ts`) and entry (`src/dev-preview`). `pnpm build`, and so the Tauri bundle, never includes it. `pnpm check:build` proves that from the bundler's own module list on each CI run.

## Profiles in the shell

The store is in the per-user configuration folder that Tauri resolves for `io.github.kloppers-labs.prismforge`: `%APPDATA%` on Windows, `~/Library/Application Support` on macOS, `~/.config` on Linux. The real store is its `store` folder. Demo mode uses its `demo-store` folder, so a profile of simulated devices is never listed or applied while PrismForge is connected to OpenRGB, and the other way round. Diagnostics shows the folder in use.

- **At start.** PrismForge takes the folder lock and reads `app-settings.json` and the mark of the real store, because start-up needs them. It writes nothing until the user changes something or a recovery renames a damaged file. The demo store opens when a demo session starts or when the user opens Profiles in demo mode.
- **One PrismForge per folder.** Before it opens a store, the shell takes an exclusive lock on `instance.lock` in the folder with the standard library's file lock. A second PrismForge cannot take it and does not read or write the store; its Profiles screen says so. Its sessions still run, without the stored device list. The operating system releases the lock when the process ends, also after a crash.
- **Each session.** The shell seeds the backend with the stored device list, starts the session, keeps the device list on disk beside it, and runs the apply-on-connect watcher beside it. Disconnect stops the watcher first, then the session, then waits for the last device-list write.
- **Apply on connect.** Off by default. When the user turns it on for one profile, the watcher applies that profile once each time the session becomes live after the user connects. A connection that PrismForge restores by itself after a loss does not apply it again; the effects in the session resume. PrismForge connects by itself only when "Connect to OpenRGB when PrismForge starts" is on.
- **Lock order.** Every command takes the shell's locks in one order: the session slot, the apply slot, the output gate, then a store. A command that needs a permit takes it while it holds the session slot, and a task that holds a session handle never waits for the session slot. So a disconnect, a mode switch or a window close during an apply, together with another command, always finishes.
- **The apply lock.** While a profile applies, the shell refuses every other output command (apply or stop an effect, hardware mode, declare, hand back, frame rate, identify, another apply) and saving, updating or deleting a profile, with `ProfileApplying`. A command already sent finishes first. The lock is released on every exit path: success, partial result, error, cancel and the end of the session. Disconnect also clears the apply-on-connect result from the view.
- **Export and import.** The commands take no path from the window. Rust opens the native dialog and uses only the path it returns. A path inside PrismForge's own folder is refused. An import larger than 1 MiB is refused before it is read. Neither writes to hardware.

## Effects screen and preview

The effects screen has no effect maths. Rust computes every frame. The stage has two states. The draft preview asks `preview_effect` for at most 60 LEDs and 90 frames, 20 to 1,000 ms apart, once per distinct LED count of the selected targets, and plays the batches on one clock, a strip per target. It asks for the next batch when the batch runs out, and again 200 ms after a setting changes. The live output asks `watch_live_output` for the selected targets and draws what Rust sends: a row for a linear zone, a grid for a matrix zone. Both stop asking when the stage is closed, the other state is shown, or the window is hidden. With reduced motion the draft shows one still frame and the live view a new still frame every 2 seconds. Both views hold back frames that would flash more than three times a second and say so; the devices still get every frame. Neither view calls an output command.

## Security and capabilities

The window has one capability, `main` in `src-tauri/capabilities/main.json`. It allows only these permissions:

| Permission | Why |
|---|---|
| `core:event:allow-listen`, `core:event:allow-unlisten` | The interface listens for `session-view`. |
| `allow-view`, `allow-connect-openrgb`, `allow-start-demo`, `allow-disconnect`, `allow-refresh`, `allow-identify` | The session commands above. |
| `allow-effect-catalog`, `allow-apply-effect`, `allow-stop-effect`, `allow-select-mode`, `allow-declare-external`, `allow-clear-external`, `allow-set-frame-rate`, `allow-preview-effect` | The effect commands above. |
| `allow-profile-store`, `allow-profile-details`, `allow-create-profile`, `allow-preview-profile-update`, `allow-update-profile`, `allow-rename-profile`, `allow-duplicate-profile`, `allow-delete-profile`, `allow-apply-profile`, `allow-set-apply-on-connect`, `allow-export-profile`, `allow-import-profile`, `allow-set-device-name`, `allow-set-zone-name`, `allow-dismiss-store-notice` | The profile commands above. |
| `allow-lifecycle`, `allow-change-setting`, `allow-answer-close`, `allow-quit` | The Settings screen, the first-close question and Quit. `build.rs` declares all thirty-three commands, so no other app command is reachable. |

The app uses two Tauri plugins and Tauri's own tray support. Only Rust calls them; the capability grants no plugin permission.

| Plugin or feature | Version | Why |
|---|---|---|
| `tauri-plugin-dialog` | 2.8.4 | The native save and open dialogs of profile export and import. The window cannot open a dialog or read a chosen path. |
| `tauri-plugin-single-instance` | 2.5.5 | A second start hands over to the running PrismForge and exits. It has no commands. |
| `windows-registry` (Windows only) | 0.6.1 | Not a plugin. The per-user `Run` value for "Start PrismForge when you sign in", written and read back by Rust. It never writes `HKLM`. |
| `auto-launch` (macOS and Linux) | 0.6.0 | Not a plugin. The per-user LaunchAgent on macOS and the XDG autostart file on Linux. |
| `tauri` feature `tray-icon` | 2.12.6 | The tray icon and its menu. No window permission. |

The window gets no window, tray, autostart or process permission: hide, show and quit go through the four app commands above. On macOS PrismForge sets its own app menu: the PrismForge menu (About, Services, Hide, Hide Others, Show All, Quit PrismForge), Edit, View (Enter Full Screen) and Window (Minimize, Zoom, Close Window). Quit PrismForge (Command-Q) runs PrismForge's quit. Unlike Tauri's default menu it has no File and no Help menu, and it adds Show All. The plugin adds a small script to the window that routes `window.alert` and `window.confirm` to its message command; the interface uses neither, and without the permission the command is refused. The app registers no shell, file system, HTTP, opener, process or updater plugin. Settings shows the store folder as text, because opening it would need the opener plugin. The window loads only the bundled interface. The Content Security Policy allows scripts, styles, images and fonts from the app itself and IPC to the shell, and nothing else. The interface has no remote content, no CDN files, no telemetry and no network access of its own; only the Rust side opens the TCP connection to OpenRGB. The app runs as a normal user and needs no Administrator rights.
