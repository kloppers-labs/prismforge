# OS events for PrismForge: sleep, hibernate, wake, shutdown, sign-in

Research for PrismForge (GPL-3.0-or-later, Tauri 2, Windows 11 first). Access date for every source: 2026-11-19. This file does not repeat `openrgb-behaviour.md`.

## Summary

On Windows, sleep and hibernate arrive as one signal (`PBT_APMSUSPEND` in `WM_POWERBROADCAST`). The app cannot tell S3 from S4, cannot veto the transition, and has about two seconds. Microsoft says "may interrupt" after that. Resume arrives as `PBT_APMRESUMEAUTOMATIC`. On Modern Standby (S0 low-power idle) the suspend message goes only to apps that opt in with `RegisterSuspendResumeNotification`, and the display-off notification is the earliest signal. Shutdown, restart and sign-out reach a GUI app as `WM_QUERYENDSESSION` then `WM_ENDSESSION`. The app has about five seconds before Windows shows the blocking-apps screen, and a process without a visible window is terminated if it does not answer in five seconds. Windows closes user processes one by one in descending shutdown-level order (default 0x280), then closes services. PrismForge can therefore raise its own shutdown level so it runs before an OpenRGB process at the default level, but nothing guarantees that OpenRGB still pushes the "off" frame to hardware. Tauri 2.12.7 / tao 0.37.3 gives us no power or session events. On Windows tao turns `WM_ENDSESSION` into `RunEvent::Exit` and then calls `process::exit(0)`; it never emits `ExitRequested` for it, and it ignores `WM_POWERBROADCAST` and `WM_QUERYENDSESSION`. We need a small in-house seam: a hidden top-level window on its own thread, registered for suspend/resume and display state, plus the existing `RunEvent::Exit` hook for session end. Items I could not confirm are marked **UNCONFIRMED** and collected in "Open points".

## How to read this file

- Microsoft Learn pages were fetched as HTML and read in full. "Last updated" dates are given where they matter.
- Source code was read at pinned tags. Pins:

| Project | Tag | Commit | Local read |
|---|---|---|---|
| tao | `tao-v0.37.3` | `cc987583ab3adec3e0691ea4c06bc13d908d78cf` | whole tree |
| tauri / tauri-runtime-wry | `tauri-v2.12.7`, `tauri-runtime-wry-v2.12.5` | `9319e539f34237035cb4693bed9246660bc0a926` | whole tree |
| wry | `wry-v0.57.3` | `3b6eab2de073cd9da7ffeec94016db7441a0caa7` | whole tree |
| tauri-plugin-autostart | `autostart-v2.7.0` | `d4835d0e947179bac24a383212792d74be3ebe4f` (plugins-workspace) | whole tree |
| auto-launch crate | `0.6.0` (crates.io, git `28816b4717227dfd9359830c0bf086cd1d2f638e`) | n/a | crate archive |
| windows-rs | tag `0.62.0` (crates.io latest is 0.62.2; tag 0.62.2 does not exist in the repo) | n/a | `crates/libs/windows/src/Windows/Win32/{System/Power,System/Shutdown,System/Threading,UI/Shell,UI/WindowsAndMessaging,System/SystemServices}/mod.rs` |
| OpenRGB `release_1.1` | | `8e16de27858cf2be6af54a33ce003df74cb9fd3e` | Codeberg raw |
| OpenRGB Scheduler plugin `release_1.0` | | `475d53e3b616aeb5b33faf173d827fbc11d2cfae` | Codeberg archive |
| Qt `qtbase` v6.8.5-lts | | `feb9556bde250306b5a1157aaccb76106844909e` | GitHub raw |
| Chromium `main` at read time | | `7923202d65b70aed36801d407100c086ca279871` | GitHub raw |

- Tauri 2.12.7 is the newest 2.12 release on crates.io (2026-11-18). It depends on `tao ^0.37.0`, `wry ^0.57.0`, `windows ^0.62` (`tauri-runtime-wry` 2.12.5 dependency list, `https://crates.io/api/v1/crates/tauri-runtime-wry/2.12.5/dependencies`).
- The OpenRGB GitLab issues were read through the public GitLab API (`https://gitlab.com/api/v4/projects/CalcProgrammer1%2FOpenRGB/issues/<n>`). Issue comments need a login, so I read only titles and first posts.

---

## 1. Windows

### 1a. Sleep, hibernate, resume, Modern Standby, Fast Startup

**Message semantics** (all from Microsoft Learn):

| Event | What Microsoft says | URL |
|---|---|---|
| `WM_POWERBROADCAST` | Sent to a window's `WindowProc`. "WM_POWERBROADCAST messages do not distinguish between different low-power states. An application can determine only that the system is entering or has resumed from a low-power state; it cannot determine the specific power state." The system logs transition details in the System event log. | https://learn.microsoft.com/en-us/windows/win32/power/wm-powerbroadcast |
| `PBT_APMSUSPEND` (4) | Sent "immediately before suspending operation", "one last chance to prepare". "In many cases, the system broadcasts these messages without requesting permission", for example when an app calls `SetSuspendState`. The system allows "approximately two seconds" per app, and "may interrupt the application" if it is still working. | https://learn.microsoft.com/en-us/windows/win32/power/pbt-apmsuspend and https://learn.microsoft.com/en-us/windows/win32/power/wm-powerbroadcast-messages |
| Time limit, other page | "The system allows for a maximum of two seconds per application when handling this message before timing out." Critical suspension (for example critical battery) sends no notification at all: "applications must be prepared to handle critical suspensions". | https://learn.microsoft.com/en-us/windows/win32/power/system-power-management-events |
| `PBT_APMRESUMEAUTOMATIC` (18) | "Resuming from sleep or hibernation". Delivered every time the system resumes and "does not indicate whether a user is present". Windows 10 1507 and later: if the system resumes from sleep only to enter hibernation straight away, the event is not delivered. | https://learn.microsoft.com/en-us/windows/win32/power/pbt-apmresumeautomatic |
| `PBT_APMRESUMESUSPEND` (7) | Sent after `PBT_APMRESUMEAUTOMATIC` only if the resume was triggered by user input. Not a reliable "resume" signal. | https://learn.microsoft.com/en-us/windows/win32/power/wm-powerbroadcast |
| `PBT_APMPOWERSTATUSCHANGE` (10) | AC/battery change. Not relevant to profiles. | same |
| `PBT_POWERSETTINGCHANGE` (0x8013) | Delivered only to apps that called `RegisterPowerSettingNotification`. `lParam` points to a `POWERBROADCAST_SETTING`. | same, and https://learn.microsoft.com/en-us/windows/win32/power/registering-for-power-events |
| `PBT_APMQUERYSUSPEND` | "Support for this event was removed in Windows Vista. Use SetThreadExecutionState instead." So no veto message exists. | https://learn.microsoft.com/en-us/windows/win32/power/pbt-apmquerysuspend |
| `SetThreadExecutionState` | "cannot be used to prevent the user from putting the computer to sleep". Applications "should respect that the user expects a certain behavior when they close the lid on their laptop or press the power button". | https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-setthreadexecutionstate |

**Registration APIs:**

- `RegisterSuspendResumeNotification(hRecipient, Flags)`: Windows 8 and later, `user32.dll`. Flags are `DEVICE_NOTIFY_WINDOW_HANDLE` (messages to a window) or `DEVICE_NOTIFY_CALLBACK` (a `DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS` structure). The callback gets `Type` = `PBT_APMSUSPEND`, `PBT_APMRESUMESUSPEND` or `PBT_APMRESUMEAUTOMATIC`. https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registersuspendresumenotification
- `PowerRegisterSuspendResumeNotification(Flags, Recipient, &handle)`: Windows 8 and later, `powrprof.dll`. Flags "must be `DEVICE_NOTIFY_CALLBACK`". No window and no message loop needed. https://learn.microsoft.com/en-us/windows/win32/api/powerbase/nf-powerbase-powerregistersuspendresumenotification
- I found no Microsoft statement on which thread runs the callback, or whether the two-second limit applies to it. **UNCONFIRMED**. Treat the limit as applying.
- `RegisterPowerSettingNotification(hRecipient, guid, flags)`: Vista and later. Interactive apps pass a window handle and flags 0. https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerpowersettingnotification
- In `windows` 0.62.0 these live in `Win32::System::Power` (`PowerRegisterSuspendResumeNotification` at `System/Power/mod.rs:296`, `RegisterPowerSettingNotification` at `:474`, `RegisterSuspendResumeNotification` at `:481`; `DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS` at `:881`). Features needed: `Win32_System_Power` and `Win32_UI_WindowsAndMessaging`. https://raw.githubusercontent.com/microsoft/windows-rs/0.62.0/crates/libs/windows/src/Windows/Win32/System/Power/mod.rs
  - `ShutdownBlockReasonCreate/Destroy` are in `Win32::System::Shutdown` (`System/Shutdown/mod.rs:87,95`). `SetProcessShutdownParameters` is in `Win32::System::Threading` (`:1656`). `SetWindowSubclass` is in `Win32::UI::Shell` (`:4318`). `ENDSESSION_*`, `WM_ENDSESSION`, `WM_QUERYENDSESSION`, `WM_POWERBROADCAST`, `PBT_*` are in `Win32::UI::WindowsAndMessaging` (`:3496-3498`, `:7292`, `:7431`, `:7435`, `:5712-5718`). The `GUID_*` constants are in `Win32::System::SystemServices` (`:864`, `:911`, `:1021`).

**Sleep (S3) versus hibernate (S4).** The app cannot tell them apart from `PBT_APMSUSPEND` (quote above). Hybrid sleep is S3 plus a hibernation file, so it looks like sleep (https://learn.microsoft.com/en-us/windows/win32/power/system-power-states). I found no documented way to learn the target state at suspend time. **Not available.** The only after-the-fact trace is the System event log, which Microsoft says records the details.

**Modern Standby (S0 low-power idle).**

- Systems with Modern Standby "don't use S1-S3". https://learn.microsoft.com/en-us/windows/win32/power/system-power-states
- "Modern Standby (S0 low-power idle) is the default sleep model on most Windows 11 devices." Microsoft recommends `PowerSettingRegisterNotification` with `GUID_MONITOR_POWER_ON` to detect transitions (page last updated 2026-07-17). https://learn.microsoft.com/en-us/windows/win32/power/about-power-management
- "A system enters modern standby when the display turns off." The software phases then run in order: no-CS wait, remote desktop check, store apps, maintenance, Desktop Activity Moderator (DAM) phase, low-power phase, and so on. https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/prepare-software-for-modern-standby
- The DAM suspends all threads of desktop processes in interactive sessions. It triggers a `WM_POWERBROADCAST` suspend message only to processes that opted in via `RegisterSuspendResumeNotification` or `PowerRegisterSuspendResumeNotification` (or an AppCompat shim). "After a few seconds delay, DAM suspends the process." The suspend message is broadcast concurrently to all opted-in processes and "the timeout after the broadcast notification is global, not per process". The resume message arrives "relative time of delivery to the system exit from connected standby is not guaranteed". https://learn.microsoft.com/en-us/windows/compatibility/desktop-activity-moderator (page dated 2022-08-23, written for Windows 8 connected standby; it names the resume message `PBT_APMRESUME`, which I read as `PBT_APMRESUMEAUTOMATIC`).
- Consequence for us: on a Modern Standby PC, PrismForge is frozen a few seconds after display-off, and so is an OpenRGB process in the same session. Sending "lights off" must happen at display-off or at the opt-in suspend message, not later.
- Real-world confirmation that opt-in is needed: Chromium's power monitor says "On machines with modern standby calling RegisterSuspendResumeNotification is required in order to get the PBT_APMSUSPEND message", and creates a hidden `WS_POPUP` window (not message-only) (`base/power_monitor/power_monitor_device_source_win.cc:117-120`, chromium `7923202d`). https://raw.githubusercontent.com/chromium/chromium/main/base/power_monitor/power_monitor_device_source_win.cc
- A Microsoft Q&A answer says only that some power events "may not have the same impact" in Modern Standby, and the asker reports `PBT_APMRESUMESUSPEND` never fired on S0 (anecdote, not a spec). https://learn.microsoft.com/en-us/answers/questions/882272/event-that-gets-triggered-on-modern-standby
- Behaviour on a real Windows 11 box is **UNCONFIRMED** (the DAM page is old). Test on both an S3 desktop and an S0ix laptop.

**Display state and away mode** (https://learn.microsoft.com/en-us/windows/win32/power/power-setting-guids):

| GUID | Data | Notes |
|---|---|---|
| `GUID_CONSOLE_DISPLAY_STATE` | `MONITOR_DISPLAY_STATE`: 0 off, 1 on, 2 dim | Windows 8 and later. "When kernel-mode applications register... they should use" this one. |
| `GUID_SESSION_DISPLAY_STATUS` | same enum | "All applications that run in an interactive user-mode session should use this setting." Sent only to user-mode apps, not session 0. |
| `GUID_MONITOR_POWER_ON` | 0 off, 1 on | Older. On Windows 8 and later "new applications should use `GUID_CONSOLE_DISPLAY_STATE` instead". The about-power-management page above still names it. |
| `GUID_SESSION_USER_PRESENCE` | 0 present, 1 not present, 2 inactive | User-mode apps only. |
| `GUID_SYSTEM_AWAYMODE` | 0 exiting, 1 entering away mode | Away mode only if some app set `ES_AWAYMODE_REQUIRED`. |

Register with `RegisterPowerSettingNotification(hwnd, &GUID, 0)`. Notifications then arrive as `WM_POWERBROADCAST` with `PBT_POWERSETTINGCHANGE` (https://learn.microsoft.com/en-us/windows/win32/power/registering-for-power-events). Windows 8 and later only for the session and console GUIDs.

**Fast Startup (hybrid shutdown)** (https://learn.microsoft.com/en-us/windows/win32/power/system-power-states):

- "Fast startup is where the user is logged off before the hibernation file is created." The system "appears to the user as though a full shutdown (S5) has occurred, even though the system has actually gone through S4." "Fast startup logs off user sessions, but the contents of kernel (session 0) are written to hard disk."
- "In Windows, fast startup is the default transition when a system shutdown is requested. A full shutdown (S5) occurs when a system restart is requested or when an application calls a shutdown API."
- So a user app sees a normal session end (`WM_QUERYENDSESSION`/`WM_ENDSESSION`), not a suspend. At the next boot it is a fresh start after a sign-in. Services and drivers are "restored", not restarted, on a resume from hibernation: "drivers and services are notified but aren't restarted" (same page, "Resuming from hibernation"). The page does not say this applies to fast-startup boots, but it says session 0 contents are saved. For us: the OpenRGB service (if used) is not re-run at a fast-startup boot, so its "load profile on service startup" may not fire. **UNCONFIRMED**. Needs a test.
- What `lParam` carries at a fast-startup shutdown (0 or `ENDSESSION_LOGOFF`) is not documented. **UNCONFIRMED**. Log it on Windows 11.

### 1b. Shutdown, restart, sign-out

**`WM_QUERYENDSESSION` / `WM_ENDSESSION`** (https://learn.microsoft.com/en-us/windows/win32/shutdown/wm-queryendsession, https://learn.microsoft.com/en-us/windows/win32/shutdown/wm-endsession):

- `WM_QUERYENDSESSION` is sent when the user chooses to end the session. If any app returns zero, the session does not end, and the system stops sending it at the first zero. `lParam` bits: `ENDSESSION_CLOSEAPP` 0x1 (Restart Manager, servicing, resources exhausted), `ENDSESSION_CRITICAL` 0x40000000 (forced), `ENDSESSION_LOGOFF` 0x80000000. "If this parameter is 0, the system is shutting down or restarting (it is not possible to determine which event is occurring)." Test with a bit mask.
- "Each application should return TRUE or FALSE immediately upon receiving this message, and defer any cleanup operations until it receives the WM_ENDSESSION message."
- If an app returned TRUE it "receives the WM_ENDSESSION message, regardless of how the other applications respond".
- `WM_ENDSESSION` `wParam` is TRUE if the session is ending, "the session can end any time after all applications have returned from processing this message". With FALSE the shutdown was cancelled and the app should keep running. The app returns 0. It "need not call DestroyWindow or PostQuitMessage".
- Logoff: if `ExitWindowsEx` is called without `EWX_FORCE` the system sends `WM_QUERYENDSESSION`; with force it does not. https://learn.microsoft.com/en-us/windows/win32/shutdown/logging-off

**Order and time limits:**

- "After five seconds, the system displays information about the applications that are preventing shutdown and allows the user to terminate them." Windows Vista and later show a full screen listing blockers. https://learn.microsoft.com/en-us/windows/win32/shutdown/wm-queryendsession and https://learn.microsoft.com/en-us/windows/win32/shutdown/shutdown-changes-for-windows-vista
- Same Vista page: "the system does not allow console applications or applications without a visible window to cancel shutdown. These applications are automatically terminated if they do not respond to WM_QUERYENDSESSION or WM_ENDSESSION within 5 seconds or if they return FALSE in response to WM_QUERYENDSESSION." Best practice there: "Applications should not block shutdown... Applications cannot rely on being able to block shutdown."
- Documented timeout values: `SPI_GETHUNGAPPTIMEOUT` 5000 ms and `SPI_GETWAITTOKILLTIMEOUT` 5000 ms in the console-handler timeout table (https://learn.microsoft.com/en-us/windows/console/handlerroutine). The `HungAppTimeout` registry value (`HKCU\Control Panel\Desktop`, REG_SZ, default 5000) is documented only on a Windows Server 2003 page dated 2009 (https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2003/cc784400(v=ws.10)). I found no current Microsoft page for `WaitToKillAppTimeout`; Windows Internals says its default is 20,000 ms for console handlers while the console-handler page says 5000 ms for `SPI_GETWAITTOKILLTIMEOUT`. **UNCONFIRMED**. `SystemParametersInfo` can set `SPI_SETHUNGAPPTIMEOUT` and `SPI_SETWAITTOKILLTIMEOUT` (https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfow). **My recommendation: PrismForge must not change them.** They are per-user system settings. Microsoft documents no case where an app should.
- Per-process order. Windows Internals, Part 2, 6th ed. (Microsoft Press, a Windows 7 era text): Csrss "loops through all the processes in the logon session of the interactive user... in reverse order of their shutdown level". Valid levels are 0 to 1023, default 640 (0x280). Explorer uses 2, Task Manager 1. "For each process that owns a top-level window, Csrss sends the WM_QUERYENDSESSION message to each thread... If the thread returns TRUE... Csrss then sends the WM_ENDSESSION". It waits `HungAppTimeout` (default 5000 ms) for the thread to exit. If the thread does not exit, it shows the hung-program screen, which "has no timeout, which means that a shutdown request could wait forever at this point". When all window-owning threads have exited, "Csrss terminates the process and goes on to the next process". Then Winlogon and Wininit handle COM processes and then session 0 (services). https://www.microsoftpressstore.com/articles/article.aspx?p=2201310&seqNum=3 (Windows 11 may differ. **UNCONFIRMED** on current builds.)
- `SetProcessShutdownParameters(dwLevel, dwFlags)`: "The system shuts down processes from high dwLevel values to low." Ranges: 000-0FF system last, 100-1FF application last, 200-2FF application "in between", 300-3FF application first, 400-4FF system first. "All processes start at shutdown level 0x280." `SHUTDOWN_NORETRY` suppresses the retry dialog. https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setprocessshutdownparameters
- `ShutdownBlockReasonCreate(hWnd, reason)`: "Applications should call this function as they begin an operation that cannot be interrupted, such as burning a CD or DVD." Call it only on the thread that created `hWnd` (else `ERROR_ACCESS_DENIED`). Keep reasons short. Users "may spend only a few seconds looking at the shutdown reasons". Pair with `ShutdownBlockReasonDestroy`. https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-shutdownblockreasoncreate and https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-shutdownblockreasondestroy. The user then sees the full-screen blocker list with the reason (Vista page above). Not appropriate for a sub-second lights-off write.
- Console control handler: `CTRL_SHUTDOWN_EVENT`: "Interactive applications are not present by the time the system sends this signal, therefore it can be received only be services". `CTRL_LOGOFF_EVENT`: "received only by services. Interactive applications are terminated at logoff." So it is useless for a tray app. https://learn.microsoft.com/en-us/windows/console/handlerroutine
- Services, for completeness: `SERVICE_CONTROL_PRESHUTDOWN` is sent before `SERVICE_CONTROL_SHUTDOWN`. Default preshutdown timeout is 10 s since build 15063 (180 s before). `SERVICE_CONTROL_SHUTDOWN` has "about 20 seconds". https://learn.microsoft.com/en-us/windows/win32/api/winsvc/ns-winsvc-service_preshutdown_info and https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nc-winsvc-lphandler_function_ex

**Design question: "off" to OpenRGB while Windows closes OpenRGB.**

What the sources establish:

1. User-session processes are closed one at a time, highest shutdown level first, each given QES, then ES, then a wait of up to `HungAppTimeout`. Only after that does the next process get its turn (Windows Internals, above).
2. OpenRGB's code has no `SetProcessShutdownParameters` call (earlier research, repo-wide search at `8e16de27`). A GUI OpenRGB therefore sits at 0x280.
3. Services are shut down after the user session (Windows Internals). If OpenRGB runs as a Windows service it is still alive when PrismForge shuts down.
4. Qt delivers `aboutToQuit` on `WM_ENDSESSION` (section 4d). OpenRGB's own exit-profile code runs then, if enabled.
5. A client cannot see hardware completion. The server queues `UPDATELEDS` per controller and sends no reply (`NetworkServer.cpp:1090-1135` and `:3094-3150` at `8e16de27`). A `REQUEST_CONTROLLER_DATA` (packet 1, has a reply) goes through the same per-controller queue (`:1103-1124`), so its reply should arrive after the preceding update was processed. That is my inference. Whether `RGBController::UpdateLEDs()` returns only after the hardware write is **UNCONFIRMED**.

What can be guaranteed: nothing. What is best effort: a final write to the OpenRGB server before PrismForge is terminated, and OpenRGB flushing it before it is terminated. A 2022 OpenRGB user complaint shows the failure from the other side: "OpenRGB server is already down by this time" (`https://gitlab.com/api/v4/projects/CalcProgrammer1%2FOpenRGB/issues/1694`).

Ordering tools:

| Tool | Effect | Limit |
|---|---|---|
| Raise PrismForge shutdown level above 0x280 (for example 0x300) | PrismForge gets QES/ES and is closed before an OpenRGB process at 0x280 | Does not make OpenRGB wait for the hardware. A second process at a higher level could still precede us. Sources say nothing on ties. |
| Work in `WM_ENDSESSION(TRUE)` | The shutdown is certain at that point (no cancel). | Starting in `WM_QUERYENDSESSION` risks leaving lights off if another app cancels (`wParam` FALSE). Per the Windows Internals sequence, QES and ES for our process come back to back, so early start gains almost nothing. |
| `ShutdownBlockReasonCreate` | Delays shutdown and shows our reason | Discouraged. Apps "cannot rely on being able to block shutdown". A hidden-window app cannot cancel and is auto-terminated after 5 s. |
| OpenRGB as a service | Outlives user apps | The OpenRGB service has no suspend/resume handling in 1.1 (earlier research). |

### 1c. Tauri 2 / tao / wry on Windows

Version read: `tauri 2.12.7`, `tauri-runtime-wry 2.12.5`, `tao 0.37.3`, `wry 0.57.3` (pins above).

**What Tauri exposes.**

- `RunEvent` variants (desktop): `Exit`, `ExitRequested { code, api }`, `WindowEvent`, `WebviewEvent`, `Ready`, `Resumed`, `MainEventsCleared`, `MenuEvent`, `TrayIconEvent` (`crates/tauri/src/app.rs:220-290`; docs.rs `https://docs.rs/tauri/2.12.7/tauri/enum.RunEvent.html`). No power or session variant.
- Trap: `RunEvent::Resumed` is "Sent if the event loop is being resumed" (`app.rs:250-252`). It is mapped from tao `StartCause::Poll` (`crates/tauri-runtime-wry/src/lib.rs:4126-4128`). It is not a system resume.
- `WindowEvent` has no power events on desktop. `Suspended` and `Resumed` are `#[cfg(mobile)]` only (`crates/tauri-runtime/src/window.rs:73-80`, `app.rs:164-170`).
- `WindowEvent::CloseRequested` comes from tao `CloseRequested` and can be prevented (`tauri-runtime-wry/src/lib.rs:4354-4383`). `ExitRequested { code: None }` is raised when the last window is destroyed (`:4259-4272`) and `ExitRequested { code: Some(_) }` on `AppHandle::exit`/`restart` (`:4303-4313`); both can be prevented (`api.prevent_exit()`, `app.rs:79-95`). This is the tray-app pattern: prevent close and exit-requested to stay resident.
- `RunEvent::Exit` fires from tao `Event::LoopDestroyed` (`tauri-runtime-wry/src/lib.rs:4134-4136`), then the callback runs, then `cleanup_before_exit` (`app.rs:1449-1456`).

**What tao does with the identifiers** (`src/platform_impl/windows/event_loop.rs` at `cc987583`):

- `WM_POWERBROADCAST`, `PBT_*`, `WM_QUERYENDSESSION`: no handler. A grep across the tao, tauri and wry trees found these names only in the comment at `event_loop.rs:2344-2345`: "We don't process `WM_QUERYENDSESSION` yet until we introduce the same mechanism as Tauri's `ExitRequested` event". So `DefWindowProc` answers TRUE and we cannot veto or react to the query.
- `WM_ENDSESSION` (`:2346-2360`, in `thread_event_target_callback`, fn at `:2268`): if `wParam` is TRUE it calls `userdata.event_loop_runner.loop_destroyed()` (`:2350`) and then `std::process::exit(0)` (`:2357`). If `wParam` is FALSE it returns 0 and does nothing. The comment says Windows ends the session itself after the return, and Restart Manager only sends the message, so tao exits directly.
- Effect for a Tauri app: on session end the app gets `RunEvent::Exit` (via `LoopDestroyed`), **not** `ExitRequested`, and the process is killed right after the callback returns. Async tasks do not get to finish. Any final write must be blocking and bounded inside the `RunEvent::Exit` callback.
- History (`CHANGELOG.md`): `LoopDestroyed` on `WM_ENDSESSION` since 0.34.3 (`a1edbeb4`, PR 1126); `process::exit` since 0.37.0 (`36b31b79`, PR 1157, fixes a crash after Restart Manager messages).
- The handler runs in the event target window's procedure. That window is a hidden top-level window (`CreateWindowExW`, no parent, `WS_OVERLAPPED`, `WS_EX_TOOLWINDOW`; `create_event_target_window`, `:628-673`). It is not message-only, so it receives broadcasts.
- `RunEvent::Exit` fires for any exit, not only session end. Tauri gives no flag to tell them apart.
- A hidden-window app counts as "without a visible window" in Microsoft's terms (Vista page), so the five-second auto-terminate rule applies.

**Ways to receive raw window messages.**

1. tao's message hook, `EventLoopBuilderExtWindows::with_msg_hook` (`src/platform/windows.rs:87,119`). It is called in the `GetMessageW` loop before `TranslateMessage`/`DispatchMessageW` (`event_loop.rs:256-264`). Tauri installs its own hook for menu accelerators and does not expose it (`crates/tauri/src/app.rs:2463-2480`, `tauri-runtime/src/lib.rs:413-414`). It sees only queued messages: Microsoft says sent messages go "immediately to the destination window procedure, bypassing the system message queue" (https://learn.microsoft.com/en-us/windows/win32/winmsg/about-messages-and-message-queues), and `GetMessage` "dispatches incoming sent messages" itself (https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessage). Not usable for `WM_QUERYENDSESSION`/`WM_POWERBROADCAST`. Not exposed by Tauri anyway.
2. Window subclassing on the HWND from `WebviewWindow::hwnd()` (`crates/tauri/src/window/mod.rs:1765`, `#[cfg(windows)]`). `SetWindowSubclass` has the rule "You cannot use the subclassing helper functions to subclass a window across threads" (https://learn.microsoft.com/en-us/windows/win32/api/commctrl/nf-commctrl-setwindowsubclass), so call it on the thread that owns the window. tao requires the event loop on the main thread (`event_loop.rs:175-181`, panic otherwise unless `any_thread`). Precedent in the stack: wry (`src/webview2/mod.rs:1331,1416`) and tauri-runtime-wry (`src/undecorated_resizing.rs:189`). The window receives its own broadcast messages. But the process also has tao's hidden window that calls `exit(0)` on `WM_ENDSESSION`. The relative order of those two windows' `WM_ENDSESSION` is not documented. **UNCONFIRMED**.
3. Own hidden top-level window on a dedicated thread with its own `GetMessage` loop. Message-only windows are out: "does not receive broadcast messages" (https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features). Use `WS_POPUP`, `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`, never shown, as Chromium and `tauri-plugin-power-monitor` do. The window must be created and pumped on one thread. Marshal results to the app via `AppHandle` (it is `Send + Clone`). Do the blocking suspend write directly on this thread so it does not depend on the Tauri main thread or the async runtime.
4. No-window option for suspend/resume only: `PowerRegisterSuspendResumeNotification` with a callback. It has no equivalent for shutdown.

**Existing crates and plugins** (crates.io API, 2026-11-19; we prefer a small in-house seam):

| Name | Version, licence | What it covers | State |
|---|---|---|---|
| `tauri-plugin-power-monitor` (repo `github.com/marrionesa/tauri-plugin-power-monitor`) | 0.1.0, MIT OR Apache-2.0, created 2026-09-15, 52 downloads | Battery, power source, suspend/resume. Windows: hidden top-level window, `RegisterSuspendResumeNotification`, `RegisterPowerSettingNotification` (`src/platform/windows.rs:4-21,183-222`). macOS: `IORegisterForSystemPower`. Linux: logind `PrepareForSleep` (`linux.rs:309`). No `WM_ENDSESSION` or shutdown handling (grep). | New, one release, very small user base. Read, not run. |
| `powerwatch` (repo `github.com/bradbury/powermonitor`) | 0.1.0, MIT, 2026-09-04, 26 downloads | GUI-less power and session events. Windows backend is a stub: "not implemented yet" and returns `unsupported` (`src/platform/windows.rs:1-12`). | Not usable on Windows. |
| `tauri-plugin-idlemonitor` | 0.2.4, MIT, updated 2026-10-13, 996 downloads | Screen lock/unlock and idle time. | Active, narrow scope. |
| `tauri-plugin-screen-lock-status` | 0.1.2, Apache-2.0, 2024-03-01 | Session lock status. | Stale. |
| Official `tauri-apps/plugins-workspace` | n/a | No power or session plugin (`plugins/` listing at `d4835d0e`). | n/a |

### 1d. Start at sign-in

| Mechanism | Facts | Source |
|---|---|---|
| `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` | "The data value for a key is a command line no longer than 260 characters". Order of programs "is indeterminate". "The system does not provide guarantees about how promptly the programs in the Run key are run... the system may choose to delay the execution of programs in the Run key and in the Startup group". Ignored in Safe Mode. | https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys (updated 2026-02-21) |
| Startup folder (`shell:startup`, per user `%userprofile%\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup`) | Shortcut to the exe. Used for apps that are not registered "with a startup task". | https://support.microsoft.com/en-us/windows/configure-startup-applications-in-windows-115a420a-0bff-4a6f-90e0-1934c844e473 |
| Task Scheduler, logon trigger | Starts a task when a user logs on. `Delay`: "amount of time between when the user logs on and when the job is started". | https://learn.microsoft.com/en-us/windows/win32/taskschd/logontrigger |
| Packaged app `StartupTask` | `StartupTaskState`: `Disabled` 0, `DisabledByUser` 1 ("can only be re-enabled by the user"), `Enabled` 2. Needs MSIX. | https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.startuptaskstate |

- User control: "Start > Settings > Apps > Startup" toggles, and Task Manager > Startup apps tab (Enable/Disable) show the same list. Both apply to apps "registered in Windows with a startup task" (same support page). Task Manager shows an impact rating.
- `StartupApproved`. Microsoft has no documentation I could find. Third-party analysis (https://windowsir.blogspot.com/2022/07/startupapprovedrun-pt-ii.html): `HKCU\...\Explorer\StartupApproved\Run` values; opening Task Manager or Settings adds missing entries with first byte `02` (enabled); disabling in the UI changes it to `03`; re-enabling restores `02`. Treat as undocumented and re-verify on Windows 11.
- How the Tauri stack reads the effective state: `auto-launch` 0.6.0 `is_enabled` (`src/windows.rs:147-156`) requires the Run value in HKLM or HKCU, and `is_task_manager_enabled` (`:172-185`, helper `:187`) in both hives: a missing `StartupApproved` value counts as enabled, otherwise the last eight bytes of the 12-byte value must be all zero. It writes the 12-byte value `02 00 00 00 ...` when enabling (`:9-11`, `:85-89`), only if the `StartupApproved\Run` key already exists (`:90-92`).
- Start-up delay of Run-key apps: only the Microsoft statement above (may be delayed, no promptness guarantee). I found no documented figure. **UNCONFIRMED**. A separate Explorer startup-delay registry value is widely cited but I found no Microsoft page for it, so I do not rely on it.

**`tauri-plugin-autostart` 2.7.0 on Windows** (`plugins/autostart/src/lib.rs` at `d4835d0e`, `Cargo.toml:6,40` depends on `auto-launch = "0.6"`, resolved `0.6.0`):

- Registry Run value named after the app (`app_name`, default the package name: `lib.rs:225-229`). Value data is `"{app_path} {args joined by space}"` (`auto-launch src/windows.rs:75-78`). The path comes from `current_exe()` (`lib.rs:233-235`). The code does not quote the path (no quoting found in `auto-launch` `lib.rs`/`windows.rs`). How Windows parses an unquoted path with spaces from a Run value is not confirmed from a primary source here. **UNCONFIRMED**; our own seam should quote it.
- Mode: the plugin never calls `set_windows_enable_mode` (grep in the plugin source), so the default `WindowsEnableMode::Dynamic` applies (`auto-launch src/lib.rs:357-361`).
  - `Dynamic` tries `HKLM\...\Run` first ("requires admin privileges and applies the auto launch to any user in the system") and falls back to HKCU on `E_ACCESSDENIED` (`windows.rs:46-57,66-72`; docs `lib.rs:73-75,345-355`).
  - When PrismForge is elevated, `enable()` therefore writes HKLM and starts for all users. When not elevated it writes HKCU. `disable()` removes from both hives and ignores access-denied unless the mode is `System` (`:106-123`).
  - Whether a Run entry written by an elevated process then launches elevated at sign-in: **UNCONFIRMED**.
- macOS: `MacosLauncher` offers only `LaunchAgent` (default) and `AppleScript` (`lib.rs:27-36`), although `auto-launch` 0.6.0 also has `SMAppService` (`auto-launch src/lib.rs:330-337`, `macos.rs:209-221`). Linux: XDG autostart `.desktop` (`auto-launch src/linux.rs:44-58`), systemd user mode also exists in the crate.
- Tauri docs: https://v2.tauri.app/plugin/autostart/ (arguments to pass at start-up, `MacosLauncher::LaunchAgent` in the sample).

### 1e. "Lights at boot before sign-in"

A user app cannot run before sign-in. "A service application ... can be started automatically at system boot... Services can execute even when no user is logged on to the system." (https://learn.microsoft.com/en-us/windows/win32/services/services). Services "cannot directly interact with a user as of Windows Vista"; the documented pattern is a hidden GUI app started with `CreateProcessAsUser` in the user's session, talking to the service over IPC (https://learn.microsoft.com/en-us/windows/win32/services/interactive-services). So "boot lighting" needs a PrismForge service (or OpenRGB run as a service with its "service startup profile", earlier research). **Out of scope** for now. At boot the hardware shows its firmware default until the first writer arrives (see OpenRGB issue #2961, a startup profile applied at logon by Task Scheduler that missed GPU lighting: https://gitlab.com/api/v4/projects/CalcProgrammer1%2FOpenRGB/issues/2961).

---

## 2. macOS

All Apple facts below come from Apple's documentation data (`https://developer.apple.com/tutorials/data/documentation/<path>.json`, the source of the pages at `developer.apple.com/documentation/...`).

| Item | Fact | URL |
|---|---|---|
| `NSWorkspace.willSleepNotification` | Posted before the device sleeps. "An observer of this message can delay sleep for up to 30 seconds while handling this notification." Register on `NSWorkspace.shared.notificationCenter`, not the default one. | https://developer.apple.com/documentation/appkit/nsworkspace/willsleepnotification |
| `didWakeNotification` | Posted when the device wakes. | https://developer.apple.com/documentation/appkit/nsworkspace/didwakenotification |
| `willPowerOffNotification` | "when the user requests a logout or powers off the device". | https://developer.apple.com/documentation/appkit/nsworkspace/willpoweroffnotification |
| `screensDidSleepNotification` | When the screen sleeps. macOS 10.6+. | https://developer.apple.com/documentation/appkit/nsworkspace/screensdidsleepnotification |
| `IORegisterForSystemPower` | Sleep and wake notifications. "Does not provide system shutdown and restart notifications." `kIOMessageSystemWillSleep`: callers MUST acknowledge with `IOAllowPowerChange`; "If a caller does not acknowledge the sleep notification, the sleep will continue anyway after a 30 second timeout". `kIOMessageCanSystemSleep`: acknowledge with `IOAllowPowerChange` or `IOCancelPowerChange` (idle sleep only; the veto can be cancelled and a `WillSleep` follows within 30 s). `kIOMessageSystemHasPoweredOn`: "Expect this event 1-5 or more seconds after initiating system wakeup"; do not acknowledge. | https://developer.apple.com/documentation/iokit/1557114-ioregisterforsystempower |
| QA1340 | Cocoa notifications can only observe. IOKit can delay idle sleep, but "it is not possible to prevent forced sleep, only delay it". | https://developer.apple.com/library/archive/qa/qa1340/_index.html |
| `applicationShouldTerminate(_:)` | Called after Quit or `terminate(_:)`. Return `terminateNow`, `terminateCancel`, or `terminateLater` (run loop in `NSModalPanelRunLoopMode` until `reply(toApplicationShouldTerminate:)`). | https://developer.apple.com/documentation/appkit/nsapplicationdelegate/applicationshouldterminate(_:) and https://developer.apple.com/documentation/appkit/nsapplication/terminatereply |
| `applicationWillTerminate(_:)` | "The app will terminate after this method returns." Not called on sudden termination. | https://developer.apple.com/documentation/appkit/nsapplicationdelegate/applicationwillterminate(_:) |
| `terminate(_:)` | "Don't bother to put final cleanup code in your app's main() function — it will never be executed." | https://developer.apple.com/documentation/appkit/nsapplication/terminate(_:) |
| `SMAppService` (macOS 13+) | `mainApp` registers the main app as a login item. `register()` "begin launching subject to user approval"; errors `kSMErrorAlreadyRegistered`, `kSMErrorLaunchDeniedByUser`. Replaces `SMLoginItemSetEnabled` and plist installs in `~/Library/LaunchAgents`. | https://developer.apple.com/documentation/servicemanagement/smappservice and https://developer.apple.com/documentation/servicemanagement/smappservice/register() |

**How tao/Tauri surface this on macOS** (tao `cc987583`): `src/platform_impl/macos/app_delegate.rs:76-81` implements only `applicationWillTerminate:`, which calls `AppState::exit()`, which sends `Event::LoopDestroyed` (`app_state.rs:273`), which becomes `RunEvent::Exit`. tao does not implement `applicationShouldTerminate`, so I see no `ExitRequested` path for Cmd-Q, logout or shutdown in tao. Whether Tauri's menu Quit item raises `ExitRequested` on macOS goes through the `muda` crate, which I did not read. **UNCONFIRMED**. tao has no `NSWorkspace` observer (grep for `willSleep`, `didWake`, `NSWorkspace`: none). `tauri-plugin-power-monitor` implements `IORegisterForSystemPower` itself, and `powerwatch` notes that it "does not deliver shutdown/restart".

**Login items and Tauri.** `tauri-plugin-autostart` 2.7.0 offers only `LaunchAgent` (writes `~/Library/LaunchAgents/<name>.plist`) and `AppleScript` (System Events login item) (`lib.rs:27-36`). The underlying crate supports `SMAppService` but the plugin does not expose it.

---

## 3. Linux (brief)

- `org.freedesktop.login1.Manager` signals `PrepareForSleep(b start)` and `PrepareForShutdown(b start)` (plus `PrepareForShutdownWithMetadata` with a `type`: "power-off", "reboot", "halt", "kexec", "soft-reboot"). They are "sent right before (with the argument 'true') or after (with the argument 'false') the system goes down for reboot/poweroff and suspend/hibernate". Suspend and hibernate share `PrepareForSleep`. https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.login1.html
- `PrepareForSleep(false)` is for resume. It "will only be sent out for suspend/resume cycles done via logind". https://systemd.io/INHIBITOR_LOCKS/
- Inhibitor locks: `Inhibit()` returns a file descriptor. Mode `delay` "inhibits operations only temporarily, either until the lock is released or up to a certain amount of time", only for sleep and shutdown. Closing the fd (or dying) releases it. "Watching PrepareForShutdown(true)/PrepareForSleep(true) without taking a delay lock is racy and should not be done." Take the lock again at `PrepareForSleep(false)`. https://systemd.io/INHIBITOR_LOCKS/
- `InhibitDelayMaxSec=` "Defaults to 5" seconds, after which the inhibitor is ignored. https://www.freedesktop.org/software/systemd/man/latest/logind.conf.html
- XDG autostart: `.desktop` files in `$XDG_CONFIG_HOME/autostart` (default `~/.config/autostart/`) and `$XDG_CONFIG_DIRS/autostart` (default `/etc/xdg/autostart/`); user file wins; `Hidden=true` disables. https://specifications.freedesktop.org/autostart-spec/latest/
- Sandboxed apps: `org.freedesktop.portal.Background.RequestBackground` with option `autostart` (bool) and `commandline` (as). https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Background.html
- OpenRGB 1.1 takes no delay lock (earlier research), so its suspend load can lose the race on Linux. PrismForge on Linux (later) should take one.

---

## 4. Cross-cutting design facts

### 4a. Tables per event class (Windows 11)

Legend for "can delay": D = documented way to delay, N = no.

**Sleep, hibernate (suspend)**

| Item | Value |
|---|---|
| Reliable signal | `PBT_APMSUSPEND` via `RegisterSuspendResumeNotification` (window) or `PowerRegisterSuspendResumeNotification` (callback). On Modern Standby, register or you get nothing, and add display-off (`GUID_SESSION_DISPLAY_STATUS`). |
| Time available | About 2 s, "may interrupt" after. On S0ix: a few seconds before the DAM suspends the process; the timeout is global across processes. |
| Can the app delay? | N (`PBT_APMQUERYSUSPEND` removed; `SetThreadExecutionState` cannot stop user sleep). |
| Pattern | Keep an open blocking socket to OpenRGB. On the event, write the off frame at once on the notification thread. Wait for the controller-queue barrier (a `REQUEST_CONTROLLER_DATA` reply) for at most ~500 ms. Return. Never block waiting on the Tauri runtime. |
| Failure modes | No notification on critical suspension (battery). Process frozen by the DAM before the write. OpenRGB not scheduled in time to flush to hardware. OpenRGB's own suspend profile writing at the same time (see 4c). S3 versus S4 not distinguishable. Hibernate with cold devices on resume. |

**Resume (sleep and hibernate)**

| Item | Value |
|---|---|
| Reliable signal | `PBT_APMRESUMEAUTOMATIC` (always sent, except resume-to-hibernate on Windows 10 1507+). Do not use `PBT_APMRESUMESUSPEND`. On S0ix also display-on. |
| Time available | None to act. No timeout. Delivery time on S0ix "not guaranteed". |
| Can the app delay? | N/A |
| Pattern | Do not apply at once. Wait for devices (4b): re-detect, retry with backoff, apply once the inventory is stable. |
| Failure modes | OpenRGB or USB/SMBus not ready. Devices back in a firmware default or hardware mode. OpenRGB believes a device is still in Direct mode when the hardware is not (OpenRGB issue #444). Resume event arrives before the user session unlock (no user present). |

**Modern Standby (display off / display on)**

| Item | Value |
|---|---|
| Reliable signal | `GUID_SESSION_DISPLAY_STATUS` (user-mode, Windows 8+), via `RegisterPowerSettingNotification`. |
| Time available | A few seconds before DAM suspension. |
| Can the app delay? | N |
| Pattern | Treat display-off as "going to sleep" on S0ix machines. Make it a user option, because on S3 machines display-off also happens during normal use (idle timeout). |
| Failure modes | Display-off without sleep (screen saver, idle timeout). Resume ordering between display-on and `PBT_APMRESUMEAUTOMATIC` is not documented. |

**Shutdown, restart (including Fast Startup)**

| Item | Value |
|---|---|
| Reliable signal | `WM_ENDSESSION` with `wParam` TRUE and `lParam` without `ENDSESSION_LOGOFF` (`lParam == 0`: shutdown or restart, not distinguishable). In a Tauri app: `RunEvent::Exit` fired by tao. |
| Time available | About 5 s from the moment our turn starts, then the blocking-apps screen (no timeout there). Hidden-window apps are terminated automatically at 5 s. |
| Can the app delay? | D, but discouraged: `ShutdownBlockReasonCreate`. Not for a sub-second write. |
| Pattern | Raise the shutdown level. In `WM_ENDSESSION(TRUE)`, write the off frame, then run the barrier with a 1 s cap. Return. Never start in `WM_QUERYENDSESSION`. |
| Failure modes | OpenRGB at the same or higher level closes first. OpenRGB closes before it flushes. PrismForge killed first by a forced shutdown (`ENDSESSION_CRITICAL`; `ExitWindowsEx` with `EWX_FORCE` sends no query at all). Hardware that stays powered after the OS is off (ErP off, OpenRGB issue #1694). |

**Sign-out**

| Item | Value |
|---|---|
| Reliable signal | `WM_ENDSESSION` with `ENDSESSION_LOGOFF` set. Same path as shutdown. |
| Time, delay, pattern | As shutdown. Fast Startup's shutdown also logs the user off, so a "shutdown" can look like a logoff to apps. **UNCONFIRMED** whether the flag is then set. |
| Failure modes | The OpenRGB server may stay running after logoff if it is a service, so lights may stay on and no one sends "off". |

**Start at sign-in**

| Item | Value |
|---|---|
| Reliable signal | Our own process start after the sign-in via Run key, Startup folder or Task Scheduler logon trigger. |
| Time available | Not applicable. Run-key apps may be delayed, with no promised time. |
| Can the app delay? | N/A |
| Pattern | On start, wait for OpenRGB to accept connections (bounded retry), then apply the start profile. |
| Failure modes | User disabled the entry in Settings or Task Manager (`StartupApproved`). Entry written to HKLM by an elevated process. OpenRGB server not yet up. Fast Startup leaves hardware cold. |

### 4b. Wake ordering and device settling

- Microsoft gives no settle time. `WM_DEVICECHANGE` / `DBT_DEVNODES_CHANGED`: "a device has been added to or removed from the system. Applications that maintain lists of devices in the system should refresh their lists". The event carries no device information (https://learn.microsoft.com/en-us/windows/win32/devio/dbt-devnodes-changed and https://learn.microsoft.com/en-us/windows/win32/devnotes/wm-devicechange, the latter listing the other `DBT_*` events). Per-device arrival needs `RegisterDeviceNotification`. A settle time for USB or SMBus after S3/S4 is **not documented**.
- Resume from hibernation: system POST, memory restore, "device initialization", "drivers restored", then "the machine returns to a running state, ready for login" (system power states page).
- macOS says wake completion arrives "1-5 or more seconds" after wake starts (`kIOMessageSystemHasPoweredOn`, section 2). That is the only published figure I found, and it is for macOS.
- OpenRGB field reports (first posts read via the GitLab API; no fix confirmed):
  - #444 (Windows 10, 2020): "ASUS Aura Motherboard devices get stuck in Rainbow mode when waking up from standby"; "Reloading a profile doesn't work since OpenRGB still sees them in 'Direct' mode". https://gitlab.com/api/v4/projects/CalcProgrammer1%2FOpenRGB/issues/444
  - #2529 (2022): motherboard lighting "reset after wake from sleep"; the profile was not reapplied. https://gitlab.com/api/v4/projects/CalcProgrammer1%2FOpenRGB/issues/2529
  - #3631 (open): Razer keyboard returns to rainbow after hibernation; asks for rescan plus profile load on wake. https://gitlab.com/api/v4/projects/CalcProgrammer1%2FOpenRGB/issues/3631
  - #3948 (open): OpenRGB hangs at "Remapping ENE SMBus RAM modules" after boot from hibernation. #2654: RAM not found after suspend, and a rescan also fails (search snippet only; I did not open this one).
- Merge request !2268 (merged 2024-11-26) added suspend/resume profile loading to OpenRGB: https://gitlab.com/api/v4/projects/CalcProgrammer1%2FOpenRGB/merge_requests/2268
- **Sound pattern** (my recommendation, built from the above):
  1. On resume, mark all devices "unknown state". Do not trust cached modes.
  2. Wait for OpenRGB to answer (bounded connect retry with backoff, for example 0.5, 1, 2, 4 s, cap about 30 s).
  3. Ask OpenRGB for the device list (rescan if the user enabled it). Compare with the pre-sleep inventory.
  4. Wait until the inventory is the same on two consecutive polls (about 1 s apart), or until a cap (about 15 s).
  5. Apply once. Force a full write (set the mode and colours explicitly), because OpenRGB may think a device is already in the target mode.
  6. Re-check on `DBT_DEVNODES_CHANGED` for a short window (about 10 s), and once more after the first user input. Stop afterwards.
  7. Cap the retries and tell the user in the UI if the final inventory is incomplete.

### 4c. Two programs reacting to the same event

What a client can read (pinned OpenRGB `8e16de27`):

- Packet IDs: `NET_PACKET_ID_SETTINGSMANAGER_GET_SETTINGS = 250`, `..._GET_SETTINGS_SCHEMA = 251`, `..._MODIFY_SETTINGS = 252`, `..._SET_SETTINGS = 253`, `..._SAVE_SETTINGS = 254` (`NetworkProtocol.h:167-172`). The server advertises `NET_SERVER_FLAG_SUPPORTS_SETTINGSMANAGER` (`NetworkServer.cpp:167`). Protocol version 6 (`Documentation/OpenRGBSDK.md:82-86`).
- Request for 250: the key as a null-terminated string, packet size `strlen + 1`. The reply is the settings JSON for that key as a null-terminated string (`OpenRGBSDK.md:683-691`). Handler `NetworkServer.cpp:2475-2500`; dispatch `:1455-1457`. There is no local-client or flag check on get. 252, 253, 254 need local client status (`OpenRGBSDK.md:703-730`).
- Key `ProfileManager`. Six sub-keys, each `{"enabled": bool, "name": string}`: `open_profile`, `exit_profile`, `resume_profile`, `service_shutdown_profile`, `service_startup_profile`, `suspend_profile` (schema `ProfileManager.cpp:56-82`, defaults `:91-141`: `enabled=false`, `name=""`). A profile is loaded only if `enabled == true` and the name is non-empty (`LoadAutoProfile`, `ProfileManager.cpp:1274-1304`).
- Detection rule for PrismForge: for the event we handle, if the matching OpenRGB key has `enabled == true` and a non-empty `name`, OpenRGB will also act on it (GUI only, except the two service keys). Read at connect time and again just before acting, because the user can change it any time.
- Mapping: suspend: `suspend_profile`. Resume: `resume_profile`. Shutdown/logoff/app exit: `exit_profile` (GUI) and `service_shutdown_profile`. Start: `open_profile` and `service_startup_profile`. 
- Risk: Windows broadcasts `PBT_APMSUSPEND` to all processes concurrently, so the two writers race and the last one wins. After resume, OpenRGB loads a profile with no delay (earlier research), likely before devices have settled. Our later write fixes that if we wait and force it; their load does not repeat.
- Recommendation: warn and let the user choose one owner. Do not change OpenRGB's settings with 252/253 without the user asking. (Possible but intrusive: the server marks them local-client only.)

### 4d. Closing the open points from the earlier research

**Does Qt deliver `aboutToQuit` on Windows session end? Yes.**

- Qt docs: `aboutToQuit()` "may happen either after a call to quit() from inside the application or when the user shuts down the entire desktop session". And: "on Windows when the user logs off, the system terminates the process after Qt closes all top-level windows. Hence, there is no guarantee that the application will have time to exit its event loop" (https://doc.qt.io/qt-6/qcoreapplication.html). Session management on Windows maps to `QGuiApplication::commitDataRequest()` (https://doc.qt.io/qt-6/session.html).
- Source, qtbase v6.8.5-lts (`qtwindowsglobal.h:129-130,286-289` maps `WM_QUERYENDSESSION`/`WM_ENDSESSION`; `qwindowscontext.cpp:1284-1325`): on query it runs `commitData()` and returns 1 unless cancelled. On `WM_ENDSESSION` with `wParam != 0` it emits `aboutToQuit` once ("we receive the message for each toplevel window included internal hidden ones, but the aboutToQuit signal should be emitted only once"), then calls `quit()` ("since the process will be killed immediately quit() has no real effect"). Guarded by `QT_CONFIG(sessionmanager)`. Whether OpenRGB's Qt build keeps that feature on is **UNCONFIRMED** (it is on by default in desktop Qt, as far as I know, but I did not check OpenRGB's build).
- Consequence: OpenRGB's exit profile (profile load plus 250 ms sleep, earlier research) runs inside the `WM_ENDSESSION` handler on a Windows shutdown, and so can race with our write. It runs only if "Load Profile on Exit" is enabled. A user report says exit profiles did not apply at PC shutdown on Linux unless the window was open (issue #4797, closed 2026-09-03): https://gitlab.com/api/v4/projects/CalcProgrammer1%2FOpenRGB/issues/4797

**Does a profile load at `PBT_APMSUSPEND` complete before sleep? Unknown.**

- No OpenRGB doc or issue I read states it. The code does not wait after the load (earlier research, `OpenRGBDialog.cpp:1504-1510`).
- Related evidence: a feature request for black-out before sleep (#2984, 2022) led to MR !2268. A 2026 report says suspend and service-stop profiles had no effect when OpenRGB ran as a Linux systemd service (#5953, closed 2026-10-08). That matches the earlier finding that headless mode has no suspend handler: https://gitlab.com/api/v4/projects/CalcProgrammer1%2FOpenRGB/issues/5953
- Microsoft's two-second limit applies to the message handler, not to the hardware writes OpenRGB queues afterwards. **UNCONFIRMED**.

**OpenRGB Scheduler plugin** (https://openrgb.org/plugin_scheduler.html):

- Repository `https://codeberg.org/OpenRGB/OpenRGBSchedulerPlugin` (default branch `master`). Release 1.0 (2026-09-11), Plugin API Version 5, tag `release_1.0`, commit `475d53e3b616aeb5b33faf173d827fbc11d2cfae`. Earlier releases 0.8, 0.9, 1.0rc2.
- Source at that commit: triggers are cron lines only (`libcron`; presets "Every day at 10pm" and so on in `CronPresets.h:16-25`). The only action is "Load Profile" (`ScheduledTask.cpp:37`, `TaskFunction` at `:50-67`, calls `api->LoadProfile`). There is no suspend, resume, startup, shutdown or logon trigger: a grep for those terms across the plugin source found nothing. README: "schedule some actions (load profile, turn off)" (`readme.md`). **It cannot do "profiles on events".**

---

## 5. Recommended design for PrismForge

This section is my recommendation, not a sourced fact.

### Event set to support first (Windows 11)

1. Sleep and hibernate: "apply profile (or lights off) at suspend". One event, because the OS gives one signal.
2. Wake: "apply profile at resume", with the settle-and-retry pattern.
3. Shutdown, restart and sign-out: "apply profile at session end".
4. Sign-in: "apply profile at start" (our own start after the Run key fires).
5. Later: display-off/on as an optional trigger for S0ix machines.

### What to promise the user

| Event | Promise | Wording |
|---|---|---|
| Sleep or hibernate | **Tries to** | "PrismForge tries to apply it before the PC sleeps. Windows gives about two seconds and may skip the notice on low battery." No separate sleep and hibernate choice. |
| Wake | **Tries to**, with retries | "PrismForge applies it once your devices are ready, within about 30 seconds." |
| Shutdown, restart, sign-out | **Tries to** | "Best effort. Windows closes apps one by one. OpenRGB may close before it finishes writing." Be explicit that some devices stay lit if the motherboard keeps power. |
| Sign-in | **Will** apply once PrismForge and OpenRGB are running | "Applies after you sign in and OpenRGB is reachable." Not before sign-in. If the user disabled PrismForge in Startup settings, show it. |
| Fast Startup | Same as shutdown | Mention that boot after Fast Startup leaves hardware cold. |

### The seam on Windows

A small module `os_events` (Windows only for now) with a trait that other platforms implement later. No new dependency beyond `windows` 0.62.

1. A dedicated thread with a hidden top-level window (`WS_POPUP`, `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`, never shown) and its own `GetMessage` loop. Not message-only. Window class with a unique name.
2. On that window: `RegisterSuspendResumeNotification(hwnd, DEVICE_NOTIFY_WINDOW_HANDLE)` (Windows 8+), and `RegisterPowerSettingNotification(hwnd, GUID_SESSION_DISPLAY_STATUS, 0)`.
3. Handle `WM_POWERBROADCAST`: `PBT_APMSUSPEND` runs the suspend action inline on this thread. `PBT_APMRESUMEAUTOMATIC` posts "resumed" to the app. Ignore `PBT_APMRESUMESUSPEND`.
4. Suspend action uses a pre-opened blocking `TcpStream` to OpenRGB (a separate connection, so it does not depend on the Tokio runtime or the Tauri main thread), a hard time cap of about 1 s for write plus barrier, and then returns. Pre-render the off/profile frame when the profile is chosen, so nothing is computed at suspend time.
5. Session end: keep Tauri's `RunEvent::Exit` as the only trigger (tao calls it from `WM_ENDSESSION(TRUE)` before `process::exit(0)`), and do the same bounded blocking write there. Add a second, redundant `WM_ENDSESSION` handler on our own window that sets an `AtomicBool session_ending` and runs the same idempotent action. Whichever fires first runs it; the second becomes a no-op. This also lets the user pick "on quit" versus "on session end" separately.
6. At start of the app (after Tauri setup), call `SetProcessShutdownParameters(0x300, 0)` (any value above 0x280 in the 300-3FF "first shutdown" range) so we are notified before OpenRGB at 0x280. Do not use `SHUTDOWN_NORETRY`. Do not call `ShutdownBlockReasonCreate`. Do not touch the timeout settings.
7. Read OpenRGB's `ProfileManager` settings over SDK packet 250 and show a warning when the event's OpenRGB profile is enabled (4c).
8. Wake handling per 4b, in the app's async runtime: rescan, wait for a stable inventory, force-apply once, short follow-up on `DBT_DEVNODES_CHANGED` (a `RegisterDeviceNotification` on the same window can come later).
9. Autostart: use `tauri-plugin-autostart`, but set `WindowsEnableMode::CurrentUser` through our own `auto-launch` use (the plugin API does not let us) or write the HKCU Run key ourselves, quoted path plus `--minimized`. Read the effective state like `auto-launch` does, but only from HKCU, and show "disabled in Windows Settings" when `StartupApproved` says so.

Dispatch: if the dedicated thread's blocking write fails, record the failure and show it on next start ("Last suspend write failed: ...").

### What must be tested on a real Windows 11 machine

- S3 desktop and S0ix laptop: does `PBT_APMSUSPEND` arrive with and without `RegisterSuspendResumeNotification`, how long before the process is frozen, and how long the write plus barrier really takes.
- Same, for hibernate and hybrid sleep, and for sleep via lid close, power button and Start menu.
- `PBT_APMRESUMEAUTOMATIC` and display-on order. Whether `GUID_SESSION_DISPLAY_STATUS` fires before suspend on S0ix.
- `WM_QUERYENDSESSION` and `WM_ENDSESSION` `lParam` values for: Start > Shut down with Fast Startup on and off, Restart, Sign out, `shutdown /s /t 0`, and `shutdown /s /f`.
- Order between our own window and tao's hidden window at `WM_ENDSESSION`; confirm `RunEvent::Exit` runs and the write goes out.
- Real shutdown ordering with OpenRGB (GUI at the default level, OpenRGB as a service) and our level at 0x300: does the "off" frame reach the hardware? Repeat with OpenRGB's exit profile enabled, to see the race.
- Whether our process with a hidden window is auto-terminated at five seconds in practice.
- Whether the barrier (`REQUEST_CONTROLLER_DATA` reply) really arrives after the hardware write for the devices we support. Time it per device type (USB HID, SMBus RAM).
- Wake: how long USB and SMBus devices take to come back on the owner's hardware; how OpenRGB's detection behaves after resume.
- Autostart: Run key delay at sign-in, `StartupApproved` bytes when disabled in Settings and Task Manager, an elevated install writing HKLM, an unquoted path with spaces.
- Fast Startup boot: does a started-at-boot OpenRGB service re-run its startup profile?

---

## 6. Sources (access date 2026-11-19)

**Microsoft Learn and support**
- https://learn.microsoft.com/en-us/windows/win32/power/wm-powerbroadcast
- https://learn.microsoft.com/en-us/windows/win32/power/wm-powerbroadcast-messages
- https://learn.microsoft.com/en-us/windows/win32/power/pbt-apmsuspend
- https://learn.microsoft.com/en-us/windows/win32/power/pbt-apmresumeautomatic
- https://learn.microsoft.com/en-us/windows/win32/power/pbt-apmquerysuspend
- https://learn.microsoft.com/en-us/windows/win32/power/system-power-management-events
- https://learn.microsoft.com/en-us/windows/win32/power/system-power-states
- https://learn.microsoft.com/en-us/windows/win32/power/about-power-management
- https://learn.microsoft.com/en-us/windows/win32/power/registering-for-power-events
- https://learn.microsoft.com/en-us/windows/win32/power/power-setting-guids
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registersuspendresumenotification
- https://learn.microsoft.com/en-us/windows/win32/api/powerbase/nf-powerbase-powerregistersuspendresumenotification
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerpowersettingnotification
- https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-setthreadexecutionstate
- https://learn.microsoft.com/en-us/windows/compatibility/desktop-activity-moderator
- https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/modern-standby
- https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/prepare-software-for-modern-standby
- https://learn.microsoft.com/en-us/answers/questions/882272/event-that-gets-triggered-on-modern-standby
- https://learn.microsoft.com/en-us/windows/win32/shutdown/wm-queryendsession
- https://learn.microsoft.com/en-us/windows/win32/shutdown/wm-endsession
- https://learn.microsoft.com/en-us/windows/win32/shutdown/shutting-down
- https://learn.microsoft.com/en-us/windows/win32/shutdown/logging-off
- https://learn.microsoft.com/en-us/windows/win32/shutdown/shutdown-changes-for-windows-vista
- https://learn.microsoft.com/en-us/windows/win32/rstmgr/guidelines-for-applications
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-shutdownblockreasoncreate
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-shutdownblockreasondestroy
- https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setprocessshutdownparameters
- https://learn.microsoft.com/en-us/windows/console/handlerroutine
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfow
- https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2003/cc784400(v=ws.10)
- https://learn.microsoft.com/en-us/windows/win32/api/winsvc/ns-winsvc-service_preshutdown_info
- https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nc-winsvc-lphandler_function_ex
- https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features
- https://learn.microsoft.com/en-us/windows/win32/winmsg/about-messages-and-message-queues
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessage
- https://learn.microsoft.com/en-us/windows/win32/api/commctrl/nf-commctrl-setwindowsubclass
- https://learn.microsoft.com/en-us/windows/win32/devio/dbt-devnodes-changed
- https://learn.microsoft.com/en-us/windows/win32/devnotes/wm-devicechange
- https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys
- https://learn.microsoft.com/en-us/windows/win32/taskschd/logontrigger
- https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.startuptaskstate
- https://learn.microsoft.com/en-us/windows/win32/services/services
- https://learn.microsoft.com/en-us/windows/win32/services/interactive-services
- https://support.microsoft.com/en-us/windows/configure-startup-applications-in-windows-115a420a-0bff-4a6f-90e0-1934c844e473
- https://www.microsoftpressstore.com/articles/article.aspx?p=2201310&seqNum=3 (Windows Internals, Part 2, 6th ed., "Troubleshooting Windows Startup and Shutdown Problems")

**Apple**
- https://developer.apple.com/documentation/appkit/nsworkspace/willsleepnotification
- https://developer.apple.com/documentation/appkit/nsworkspace/didwakenotification
- https://developer.apple.com/documentation/appkit/nsworkspace/willpoweroffnotification
- https://developer.apple.com/documentation/appkit/nsworkspace/screensdidsleepnotification
- https://developer.apple.com/documentation/iokit/1557114-ioregisterforsystempower
- https://developer.apple.com/library/archive/qa/qa1340/_index.html
- https://developer.apple.com/documentation/appkit/nsapplicationdelegate/applicationshouldterminate(_:)
- https://developer.apple.com/documentation/appkit/nsapplicationdelegate/applicationwillterminate(_:)
- https://developer.apple.com/documentation/appkit/nsapplication/terminatereply
- https://developer.apple.com/documentation/appkit/nsapplication/terminate(_:)
- https://developer.apple.com/documentation/servicemanagement/smappservice
- https://developer.apple.com/documentation/servicemanagement/smappservice/register()

**freedesktop.org, systemd, XDG**
- https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.login1.html
- https://www.freedesktop.org/software/systemd/man/latest/logind.conf.html
- https://systemd.io/INHIBITOR_LOCKS/
- https://specifications.freedesktop.org/autostart-spec/latest/
- https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Background.html

**Tauri, tao, wry, plugins, crates**
- tao `cc987583`: https://github.com/tauri-apps/tao/blob/cc987583ab3adec3e0691ea4c06bc13d908d78cf/src/platform_impl/windows/event_loop.rs (lines 175-181, 256-264, 628-673, 2268, 2344-2360), `src/platform/windows.rs` (87, 119), `src/platform_impl/macos/app_delegate.rs` (76-81), `src/platform_impl/macos/app_state.rs` (273), `CHANGELOG.md`
- tauri / tauri-runtime-wry `9319e539`: https://github.com/tauri-apps/tauri/blob/9319e539f34237035cb4693bed9246660bc0a926/crates/tauri/src/app.rs (79-95, 220-290, 1449-1456, 2463-2480), `crates/tauri/src/window/mod.rs` (1765), `crates/tauri-runtime-wry/src/lib.rs` (4126-4136, 4259-4272, 4303-4313, 4354-4383), `crates/tauri-runtime-wry/src/undecorated_resizing.rs` (189), `crates/tauri-runtime/src/lib.rs` (413-414), `crates/tauri-runtime/src/window.rs` (73-80)
- wry `3b6eab2d`: https://github.com/tauri-apps/wry/blob/3b6eab2de073cd9da7ffeec94016db7441a0caa7/src/webview2/mod.rs (1331, 1416)
- https://docs.rs/tauri/2.12.7/tauri/enum.RunEvent.html
- plugins-workspace `d4835d0e`: https://github.com/tauri-apps/plugins-workspace/blob/d4835d0e947179bac24a383212792d74be3ebe4f/plugins/autostart/src/lib.rs
- https://v2.tauri.app/plugin/autostart/
- auto-launch 0.6.0: https://static.crates.io/crates/auto-launch/auto-launch-0.6.0.crate (`src/windows.rs`, `src/lib.rs`, `src/macos.rs`, `src/linux.rs`)
- windows-rs 0.62.0: https://raw.githubusercontent.com/microsoft/windows-rs/0.62.0/crates/libs/windows/src/Windows/Win32/System/Power/mod.rs (and `System/Shutdown`, `System/Threading`, `UI/Shell`, `UI/WindowsAndMessaging`, `System/SystemServices`)
- crates.io API: https://crates.io/api/v1/crates/tauri, `.../tauri-runtime-wry/2.12.5/dependencies`, `.../tauri-plugin-power-monitor`, `.../powerwatch`, `.../tauri-plugin-idlemonitor`, `.../tauri-plugin-screen-lock-status`
- `tauri-plugin-power-monitor` 0.1.0: https://static.crates.io/crates/tauri-plugin-power-monitor/tauri-plugin-power-monitor-0.1.0.crate
- `powerwatch` 0.1.0: https://static.crates.io/crates/powerwatch/powerwatch-0.1.0.crate
- Chromium: https://raw.githubusercontent.com/chromium/chromium/main/base/power_monitor/power_monitor_device_source_win.cc (commit `7923202d65b70aed36801d407100c086ca279871`)
- windowsir (third party, `StartupApproved`): https://windowsir.blogspot.com/2022/07/startupapprovedrun-pt-ii.html

**Qt**
- https://doc.qt.io/qt-6/qcoreapplication.html (`aboutToQuit`)
- https://doc.qt.io/qt-6/session.html
- https://github.com/qt/qtbase/blob/feb9556bde250306b5a1157aaccb76106844909e/src/plugins/platforms/windows/qwindowscontext.cpp (1284-1325) and `qtwindowsglobal.h` (129-130, 286-289)

**OpenRGB**
- Core `8e16de27`: `https://codeberg.org/OpenRGB/OpenRGB/raw/commit/8e16de27858cf2be6af54a33ce003df74cb9fd3e/NetworkProtocol.h` (167-172), `.../NetworkServer.cpp` (167, 1090-1135, 1455-1457, 2475-2500, 3094-3150), `.../ProfileManager.cpp` (56-141, 1274-1304), `.../Documentation/OpenRGBSDK.md` (82-86, 683-730)
- Scheduler plugin: https://openrgb.org/plugin_scheduler.html and https://codeberg.org/OpenRGB/OpenRGBSchedulerPlugin/archive/475d53e3b616aeb5b33faf173d827fbc11d2cfae.tar.gz
- GitLab issues and MR (public API): issues 444, 1694, 2529, 2961, 2984, 3631, 3948, 4797, 5953; merge request 2268; base `https://gitlab.com/api/v4/projects/CalcProgrammer1%2FOpenRGB/`

---

## 7. Open points

1. Whether `WM_QUERYENDSESSION` and `WM_ENDSESSION` order across a process's several top-level windows is documented. Needed for the subclass option. Not found.
2. What `lParam` carries at a Fast Startup shutdown, and whether services restore (not restart) at that boot. Test.
3. A documented settle time for USB and SMBus devices after resume. None found for Windows. Only the macOS "1-5 or more seconds" figure.
4. Which thread runs the `PowerRegisterSuspendResumeNotification` callback, and whether the two-second limit applies. Not documented.
5. Whether current Windows 11 still sends `PBT_APMSUSPEND` only to opted-in processes on S0ix, and how long the DAM waits. The DAM page is from 2022 and written for Windows 8.
6. Current Microsoft documentation for `WaitToKillAppTimeout` and `HungAppTimeout`. Only the 2009 Server 2003 page, the console-handler page and the 6th-edition Windows Internals text were found.
7. `StartupApproved` format. No Microsoft documentation. Third-party and `auto-launch` code only. Also `StartupApproved\StartupFolder` for Startup-folder shortcuts was not examined.
8. Run-key start-up delay: no documented figure.
9. Whether an unquoted path with spaces in a Run value works, and whether an entry written to HKLM by an elevated process launches elevated.
10. Whether the Windows Internals shutdown sequence (per-process QES/ES, wait, terminate, next) still holds on Windows 11, and what happens on ties at the same level.
11. Whether `RGBController::UpdateLEDs()` returns after the hardware write, and so whether the `REQUEST_CONTROLLER_DATA` barrier proves anything. Needs a read of the controller code or a hardware test.
12. Whether OpenRGB's Windows Qt build has `sessionmanager` enabled, so that `aboutToQuit` really fires at session end.
13. Whether Tauri's menu Quit raises `ExitRequested` on macOS (the `muda` crate was not read).
14. Maintenance quality of `tauri-plugin-power-monitor` (new, one release, 52 downloads). Read only, not run, not security-reviewed.
15. OpenRGB issue comments (login required) were not read; the issue conclusions above rest on first posts and titles.
