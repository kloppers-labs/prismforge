# 0004 — Window, tray, start-up and quit

- Status: accepted
- Date: 2026-11-19
- Scope: what closing the window does, how PrismForge keeps running in the background, how it starts with the system, and how it quits

## Current state

The desktop app has a tray icon, a Settings screen, a first-close question, an in-app Quit, one instance per user, start with the system, start hidden and connect at start (Task 6a). **None of it has run on Windows.** The evidence is Rust tests through the shell with fake window, exit and start-entry seams, interface tests, and a native macOS build. No real OpenRGB server and no real hardware were used.

The code is in `apps/desktop/src-tauri/src/lifecycle.rs` and its folder (the decisions), `settings.rs` (the settings file for async callers), `system_entry.rs` (the start entry), `tray.rs`, `app_menu.rs` and `native.rs` (the Tauri glue), and `crates/prismforge-core/src/store/app_settings.rs` (the file).

## Close is not quit

1. **Quit ends PrismForge.** It hides the window, stops the PrismForge effects, asks the backend to put back the remembered lighting (the existing release path, stopped after 5 seconds), disconnects and ends the process. These all run this one quit: Quit PrismForge in the tray menu, on the Settings screen and in the first-close question, Quit PrismForge in the macOS app menu and its shortcut Command-Q (PrismForge builds its own app menu so that this item does not go straight to the system's terminate), and Ctrl+Q on Windows and Linux. A rule "When PrismForge quits" runs before this stop, for at most 2 seconds (`0007-automation.md`); the devices it changed keep that state.
1a. **One quit at a time.** While a quit runs, further Quit requests from the tray, the app menu, Settings and the keyboard are ignored; the tray item reads "Quitting PrismForge" and is off, and so is Stop all. The release is bounded, and the quit waits at most 10 seconds for the whole stop before the process ends anyway, so this cannot trap the user. The stop at the final `Exit` event (1b) then does not run again, so a user quit never waits longer than those 10 seconds. A request from the operating system still ends PrismForge, as below.
1b. **When the system ends PrismForge.** The ways below do not pass through the quit above. They raise Tauri's final `Exit` event, and PrismForge runs the same stop there, waiting at most 4 seconds (it is skipped when a user quit already ran its stop); then the process ends whether or not the stop finished. The 4 seconds stay under the 5 seconds after which Windows ends an app without a visible window at session end, or lists a visible one as blocking the shutdown (Microsoft, "Shutdown changes for Windows Vista" and `WM_QUERYENDSESSION`). The bound can cut the session's own 5-second release short. PrismForge does not delay or block the shutdown and does not change the system's timeouts. The system may end PrismForge before that, and then the devices keep the last effect frame.
   - macOS: Quit from the Dock, logging out and shutting down. They call `terminate:`; tao 0.37.2 implements only `applicationWillTerminate`, which becomes `Exit`.
   - Windows: logging off and shutting down. tao 0.37.2 does not handle `WM_QUERYENDSESSION`; on `WM_ENDSESSION` with `TRUE` it raises `Exit` (not `ExitRequested`) and then calls `std::process::exit(0)` (`platform_impl/windows/event_loop.rs`, `WM_ENDSESSION` arm). Since Task 12, PrismForge's own event window also handles `WM_ENDSESSION` and the power events; a session-end rule runs first, for at most 1 second, and the stop gets the rest of the 4 seconds (`0007-automation.md`).
   - Ending the process in Task Manager or Activity Monitor, or a crash, runs nothing: the devices keep the last effect frame.
2. **Close hides or quits, as the user chose.** With "Keep PrismForge running in the tray", closing the window hides it; the process, the session and the effects keep running. With "Quit PrismForge", closing the window quits.
3. **The first close asks.** Until the user chooses, closing the window does nothing but open a question in the window. It says that PrismForge stays in the tray, that its effects keep running, and how to quit. The answers are "Keep running in the tray", "Quit PrismForge" and "Cancel". PrismForge stores the answer and performs it. Cancel changes nothing. If the answer cannot be stored, PrismForge still performs it and shows the store error.
4. **No tray, no background.** If the tray cannot be created, closing the window always quits, "Keep running in the tray" and "Start hidden" are turned off, and Settings says why. A process without a window and without a tray never exists.
5. **The window close shortcut closes the window.** Command-W on macOS (Close Window in the app menu), Alt+F4 and Close window in the taskbar menu on Windows (all `WM_CLOSE`) go through the same close decision.
6. **PrismForge never holds back the system.** It prevents only the window's own close event, to decide between hide and quit. It never cancels a quit, a logout or a shutdown, and the task manager ends it like any process. After its own stop, PrismForge's exit request passes the exit guard at once.

## The tray

- Left click shows the window on Windows and Linux. On macOS a click opens the menu, as menu bar items do there; the Dock icon also shows the window.
- The menu: Show or Hide PrismForge window, three lines of status that cannot be clicked, Stop all PrismForge effects, and Quit PrismForge.
- **The status lines come from the published view only.** The connection line names OpenRGB and its address, or demo mode, and the state (connected, connecting, not reachable and retrying, with problems, stopped, not connected). The effects line counts the devices whose owner is a PrismForge effect, running or waiting. The profile line names the profile applied or saved in this session, or the profile being applied.
- **The tray follows changes without polling.** It waits on the view and on the lifecycle state, and updates at most every 250 ms.
- **Stop all PrismForge effects** runs the per-device stop of the effects screen on each device with an effect, one after the other, through the same shell method. It keeps one result per device. The window shows them as a notice the next time it is open, with the same words as a stop on the effects screen. A refusal, such as `Busy` or "applying a profile", is shown and never retried. The item is off when no effect runs.
- macOS gets a monochrome template icon (`icons/tray-template.svg`, drawn from the app icon). Windows and Linux use the app icon.

## Start-up

Four steps, each an explicit switch that is off by default:

1. **Start PrismForge when you sign in.** The operating system's per-user entry, also when PrismForge runs as administrator:
   - Windows: PrismForge writes the value `PrismForge` in `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` itself, through `windows-registry`. The value is `"<path of PrismForge.exe>" --autostart`, with the path in quotes, so install folders and user names with spaces keep the program and the argument apart (a test checks the written string and how it splits). If Task Manager turned the entry off (`StartupApproved\Run`, an odd first byte), it counts as off; turning the switch on marks it on there again. PrismForge never writes `HKLM`.
   - macOS: `~/Library/LaunchAgents/PrismForge.plist`, through `auto-launch` 0.6.0 in LaunchAgent mode; the program and `--autostart` are separate entries of `ProgramArguments`.
   - Linux: `~/.config/autostart/PrismForge.desktop`, through `auto-launch`, with the program path in quotes.
   PrismForge reads the entry back after every change; the switch shows what the system reports. A refused or unkept change is shown as an error and the switch stays as the system reports it. The state is not stored by PrismForge.
   **An entry for every user.** Earlier builds used `tauri-plugin-autostart`, whose `auto-launch` default writes `HKLM` when the process runs as administrator (and writes the path without quotes). PrismForge reads `HKLM\Software\Microsoft\Windows\CurrentVersion\Run` and, when a `PrismForge` value is there, says so on Settings: the system starts PrismForge from it for every user, whatever the switch shows, and PrismForge cannot remove it without administrator rights. Settings gives the removal command.
2. **Start hidden in the tray.** Only for a start by the system entry and only with a tray. A start by the user always opens the window.
3. **Connect to OpenRGB when PrismForge starts.** It connects to the address of the user's last Connect, through the same path as the Connect button: the same retry and backoff, the same apply-on-connect rule. A failed connect shows the normal retrying state when the window opens. It never starts demo mode. It can be turned on only after one Connect.
4. **A rule in Automation that applies a profile or switches the lights off.** At the connect at start, the rule "When PrismForge starts" runs when it is on; otherwise the rule "When PrismForge connects" runs, as for any connect. Both rules are in the real store (`0007-automation.md`). Until Task 12 this step was the per-profile mark "Apply this profile when PrismForge connects"; that mark became the connects rule, switched on with the same profile, and the behaviour at start stayed the same.

With steps 1, 3 and 4 on, the lights change at system start without a click. Settings lists the four steps in this order with their real state, names the profile and the rule that applies it or says that no rule does, links to Automation, and says in one sentence what happens at sign-in now.

**The automatic paths.** Without a click, PrismForge changes lighting only through a rule the user switched on in Automation (`0007-automation.md`): when it starts or connects, when the system sleeps, wakes or ends the session, or when it quits. Connecting alone writes nothing. Settings says exactly this next to the switches. Demo mode never starts by itself.

## One instance

`tauri-plugin-single-instance` runs first. A second start hands its arguments to the first PrismForge and exits before any window, store or session exists. The first PrismForge shows its window, also when it was hidden, and gives it focus, through `Lifecycle::second_instance`. If the hand-over arrives before the first PrismForge has finished its setup, it shows the window directly. On Windows the plugin uses a named mutex, on Linux a D-Bus name. On macOS it uses the socket `/tmp/io_github_kloppers_labs_prismforge_si.sock`, which is the same path for every user of the Mac, not per user; two users signed in at the same time were not tested. The folder lock of `0002` stays as a second guard.

## Settings are a file of their own

`app-settings.json` sits in the per-user folder, beside `store`, `demo-store` and `instance.lock`. It holds the close choice, start hidden, connect at start and the last OpenRGB address. The settings belong to the user, not to a store: closing the window behaves the same in demo mode and with OpenRGB, and connect at start never uses demo mode.

The file follows the store rules: the format marker `prismforge.app-settings`, schema version 1.0, a synced temporary file and a rename, a backup, recovery from the backup with a notice on Settings, damaged files renamed and never deleted, a newer major version left as it is. Nothing is stored in the WebView.

The file opens after the folder lock is held. Because start-up reads it, PrismForge now takes the folder lock at start. The Settings view also reads the mark of the real store, so the real store opens at start too. Neither writes a file until the user changes something or a recovery renames a damaged file.

## Locks

The lifecycle keeps its state behind a short standard mutex that is never held across an await. Its publish lock sits before the store in the lock order of `0002`: it is taken before the settings file and the real store are read, and never while a store lock is held. Tray actions, the app menu, window events and the second start all go through one enum, `tray::Action`, and call the same lifecycle and shell methods as the commands. `tests/command_locks.rs` drives the four new commands, every `Action` (named by its variant), the tray model and start-up with connect at start armed beside a disconnect, a mode switch and the window close, and beside a profile apply in all orders. A guard test reads the variants of `tray::Action` from the source and fails when one has no row, as the command guard does for `generate_handler!`.

## Uninstall

- **Windows, NSIS installer:** its uninstaller deletes the value `PrismForge` (the product name) from `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, except during an update. It leaves the `PrismForge` value in `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run` if Task Manager made one; that value starts nothing.
- **Windows, MSI installer:** it removes no start entry.
- **Windows, an entry for every user** from an earlier build run as administrator: no uninstaller removes it.
- **macOS:** there is no uninstaller; deleting the app leaves the LaunchAgent.

Turn off "Start PrismForge when you sign in" before you uninstall. If an entry is left:

- Windows, for your user: `reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v PrismForge /f`. No administrator rights are needed.
- Windows, for every user, in a Command Prompt opened as administrator: `reg delete "HKLM\Software\Microsoft\Windows\CurrentVersion\Run" /v PrismForge /f`.
- macOS: `rm ~/Library/LaunchAgents/PrismForge.plist`.

## Not verified

- Anything on Windows: the tray, its click and menu, the start entry in the registry, the single-instance hand-over, Alt+F4 and Ctrl+Q, start hidden at sign-in. CI builds and tests on `windows-latest`; nobody has run the app there.
- The start entry on Windows: the registry code was type-checked and linted for `x86_64-pc-windows-msvc` in a scratch crate on macOS; it has not run. The quoted command and the Task Manager flag are tested as strings on every platform.
- The start entry on Linux: not built for Linux on the build machine.
- The keyboard and the app menu in the built app: Command-Q (PrismForge's own app menu) and Command-W were not pressed by hand. Ctrl+Q is covered by interface tests only.
- The stop at `Exit` on a real logout or shutdown, and whether the system waits for it.
- A Linux desktop where the tray icon is created but not shown: closing the window would then hide PrismForge with no way back except starting it again, which shows the window through the single-instance hand-over.
- Logout and shutdown on any system. PrismForge does not block them, but no test shows whether the release of effects runs before the system ends the process.
- Start at sign-in on macOS. No start entry was registered on the build machine; the registration is tested through the seam only.
- Linux, including a desktop without a tray.
- Whether the WebView reports a hidden native window as hidden. The shell tells the window when it hides it, and the preview stops on that report as well, so the preview does not depend on it.
- The tray glue, the app menu, the window-event glue, the stop at `Exit` in `on_run_event` and the plugin calls. Tauri's mock runtime cannot exit and cannot show a tray, so these lines have no automated test; the bounded stop they call (`stop_within`) is tested.
- A real OpenRGB server and real hardware.
