# 0004 — Window, tray, start-up and quit

- Status: accepted
- Date: 2026-11-19
- Scope: what closing the window does, how PrismForge keeps running in the background, how it starts with the system, and how it quits

## Current state

The desktop app has a tray icon, a Settings screen, a first-close question, an in-app Quit, one instance per user, start with the system, start hidden and connect at start (Task 6a). **None of it has run on Windows.** The evidence is Rust tests through the shell with fake window, exit and start-entry seams, interface tests, and a native macOS build. No real OpenRGB server and no real hardware were used.

The code is in `apps/desktop/src-tauri/src/lifecycle.rs` and its folder (the decisions), `settings.rs` (the settings file for async callers), `tray.rs` and `native.rs` (the Tauri glue), and `crates/prismforge-core/src/store/app_settings.rs` (the file).

## Close is not quit

1. **Quit ends PrismForge.** It stops the PrismForge effects, asks the backend to put back the remembered lighting (the existing release path, stopped after 5 seconds), disconnects and ends the process. The tray menu, the Settings screen, the first-close question, the platform's quit command (Command-Q on macOS) and Ctrl+Q on Windows and Linux all quit the same way.
2. **Close hides or quits, as the user chose.** With "Keep PrismForge running in the tray", closing the window hides it; the process, the session and the effects keep running. With "Quit PrismForge", closing the window quits.
3. **The first close asks.** Until the user chooses, closing the window does nothing but open a question in the window. It says that PrismForge stays in the tray, that its effects keep running, and how to quit. The answers are "Keep running in the tray", "Quit PrismForge" and "Cancel". PrismForge stores the answer and performs it. Cancel changes nothing. If the answer cannot be stored, PrismForge still performs it and shows the store error.
4. **No tray, no background.** If the tray cannot be created, closing the window always quits, "Keep running in the tray" and "Start hidden" are turned off, and Settings says why. A process without a window and without a tray never exists.
5. **The window close shortcut closes the window.** Command-W on macOS and Alt+F4 on Windows go through the same close decision.
6. **PrismForge never holds back the system.** It prevents only the window's own close event, to decide between hide and quit. It never cancels a quit, a logout or a shutdown, and the task manager ends it like any process. A second quit request during the release passes straight through.

## The tray

- Left click shows the window on Windows and Linux. On macOS a click opens the menu, as menu bar items do there; the Dock icon also shows the window.
- The menu: Show or Hide PrismForge window, three lines of status that cannot be clicked, Stop all PrismForge effects, and Quit PrismForge.
- **The status lines come from the published view only.** The connection line names OpenRGB and its address, or demo mode, and the state (connected, connecting, not reachable and retrying, with problems, stopped, not connected). The effects line counts the devices whose owner is a PrismForge effect, running or waiting. The profile line names the profile applied or saved in this session, or the profile being applied.
- **The tray follows changes without polling.** It waits on the view and on the lifecycle state, and updates at most every 250 ms.
- **Stop all PrismForge effects** runs the per-device stop of the effects screen on each device with an effect, one after the other, through the same shell method. It keeps one result per device. The window shows them as a notice the next time it is open, with the same words as a stop on the effects screen. A refusal, such as `Busy` or "applying a profile", is shown and never retried. The item is off when no effect runs.
- macOS gets a monochrome template icon (`icons/tray-template.svg`, drawn from the app icon). Windows and Linux use the app icon.

## Start-up

Four steps, each an explicit switch that is off by default:

1. **Start PrismForge when you sign in.** The operating system's per-user entry, through `tauri-plugin-autostart`: a value in `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` on Windows, `~/Library/LaunchAgents/PrismForge.plist` on macOS. The entry passes `--autostart`. PrismForge reads the entry back after every change; the switch shows what the system reports. A refused or unkept change is shown as an error and the switch stays as the system reports it. The state is not stored by PrismForge.
2. **Start hidden in the tray.** Only for a start by the system entry and only with a tray. A start by the user always opens the window.
3. **Connect to OpenRGB when PrismForge starts.** It connects to the address of the user's last Connect, through the same path as the Connect button: the same retry and backoff, the same apply-on-connect rule. A failed connect shows the normal retrying state when the window opens. It never starts demo mode. It can be turned on only after one Connect.
4. **The profile marked "Apply this profile when PrismForge connects".** This is the existing per-profile mark in the real store, the single source. There is no separate start-up profile.

With steps 1, 3 and 4 on, the lights change at system start without a click. Settings lists the four steps in this order with their real state, names the marked profile or says that none is marked, links to Profiles, and says in one sentence what happens at sign-in now.

**The automatic paths.** Without a click in a session, PrismForge changes lighting only when connect at start is on and a profile is marked to apply on connect. Connecting alone writes nothing. Settings says exactly this next to the switches. Demo mode never starts by itself.

## One instance

`tauri-plugin-single-instance` runs first. A second start hands its arguments to the first PrismForge and exits before any window, store or session exists. The first PrismForge shows its window, also when it was hidden, and gives it focus. On Windows the plugin uses a named mutex, on macOS a socket in `/tmp`, on Linux D-Bus. The folder lock of `0002` stays as a second guard.

## Settings are a file of their own

`app-settings.json` sits in the per-user folder, beside `store`, `demo-store` and `instance.lock`. It holds the close choice, start hidden, connect at start and the last OpenRGB address. The settings belong to the user, not to a store: closing the window behaves the same in demo mode and with OpenRGB, and connect at start never uses demo mode.

The file follows the store rules: the format marker `prismforge.app-settings`, schema version 1.0, a synced temporary file and a rename, a backup, recovery from the backup with a notice on Settings, damaged files renamed and never deleted, a newer major version left as it is. Nothing is stored in the WebView.

The file opens after the folder lock is held. Because start-up reads it, PrismForge now takes the folder lock at start. The Settings view also reads the mark of the real store, so the real store opens at start too. Neither writes a file until the user changes something or a recovery renames a damaged file.

## Locks

The lifecycle keeps its state behind a short standard mutex that is never held across an await. Its publish lock sits before the store in the lock order of `0002`: it is taken before the settings file and the real store are read, and never while a store lock is held. Tray actions and window events call the same lifecycle and shell methods as the commands. `tests/command_locks.rs` drives the four new commands and the tray, window, start-up and second-start actions beside a disconnect, a mode switch and the window close, and beside a profile apply in all orders.

## Uninstall

The NSIS uninstaller of Tauri 2.12 removes the `PrismForge` value from the HKCU `Run` key. The MSI does not. Turn off "Start PrismForge when you sign in" before you uninstall. If the entry is left:

- Windows: `reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v PrismForge /f`. No administrator rights are needed.
- macOS: `rm ~/Library/LaunchAgents/PrismForge.plist`.

## Not verified

- Anything on Windows: the tray, its click and menu, the start entry in the registry, the single-instance hand-over, Alt+F4 and Ctrl+Q, start hidden at sign-in. CI builds and tests on `windows-latest`; nobody has run the app there.
- **An elevated PrismForge on Windows.** The autostart plugin 2.7.0 uses the `Dynamic` mode of `auto-launch`: when PrismForge runs as Administrator, it writes the entry to `HKLM` for every user instead of `HKCU`. PrismForge never asks for Administrator rights. The NSIS uninstaller removes only the `HKCU` value.
- The keyboard in the built app: Command-Q and Command-W on macOS go through the default app menu of Tauri; they were not pressed by hand in this build. Ctrl+Q is covered by an interface test only.
- Logout and shutdown on any system. PrismForge does not block them, but no test shows whether the release of effects runs before the system ends the process.
- Start at sign-in on macOS. No start entry was registered on the build machine; the registration is tested through the seam only.
- Linux, including a desktop without a tray.
- Whether the WebView reports a hidden native window as hidden. The shell tells the window when it hides it, and the preview stops on that report as well, so the preview does not depend on it.
- The tray glue, the window-event glue and the plugin calls. Tauri's mock runtime cannot exit and cannot show a tray, so these lines have no automated test.
- A real OpenRGB server and real hardware.
