# OpenRGB 1.1 behaviour: system events, effect preview, spatial layout

Research for PrismForge (GPL-3.0-or-later). Behaviour and data layouts only, in my own words. No code, art or text is copied. Access date: 2026-11-19.

## 1. Sources

All source files were read from the pinned commits through the Codeberg mirror. GitLab was not used.

| Item | Pin | URL pattern |
|---|---|---|
| OpenRGB, tag `release_1.1` | `8e16de27858cf2be6af54a33ce003df74cb9fd3e` (tag date 2026-10-16, from Codeberg tags API) | `https://codeberg.org/OpenRGB/OpenRGB/raw/commit/8e16de27858cf2be6af54a33ce003df74cb9fd3e/<path>`; tree `https://codeberg.org/api/v1/repos/OpenRGB/OpenRGB/git/trees/8e16de27858cf2be6af54a33ce003df74cb9fd3e?recursive=true&per_page=1000&page=N`; full archive `https://codeberg.org/OpenRGB/OpenRGB/archive/8e16de27858cf2be6af54a33ce003df74cb9fd3e.tar.gz` |
| OpenRGB Effects Plugin, tag `release_1.1` | `f211d08afd3382318e60f829348b25c6e15fb0ae` (tag date 2026-10-16) | `https://codeberg.org/OpenRGB/OpenRGBEffectsPlugin/archive/f211d08afd3382318e60f829348b25c6e15fb0ae.tar.gz`; files below are relative to `openrgbeffectsplugin/` in that archive |
| OpenRGB Visual Map Plugin, tag `release_1.0` | `1dde279ef57df315927e3c32aa7e0ad8bf9e39f8` (tag date 2026-09-11) | `https://codeberg.org/OpenRGB/OpenRGBVisualMapPlugin/archive/1dde279ef57df315927e3c32aa7e0ad8bf9e39f8.tar.gz`; files relative to `openrgbvisualmapplugin/` |

Notes on the pins:
- `https://openrgb.org/plugin_visual_map.html` lists Visual Map "Release 1.0 (September 11, 2026), Plugin API Version 5". It has no 1.1 release. Its Windows download names end in `1dde279`, which matches the pinned tag.
- Core plugin API version is `OPENRGB_PLUGIN_API_VERSION 5` (`OpenRGBPluginInterface.h:33`).
- `https://openrgb.org/releases.html` shows Release 1.1 (October 16, 2026) as "SDK Version 6 | Plugin API Version 5".
- `https://openrgb.org/plugin_effects.html` shows Effects Release 1.1 (October 16, 2026), Plugin API 5.
- `https://openrgb.org/plugins.html` lists official plugins: Effects, Visual Map, Hardware Sync, E1.31 Receiver, HTTP Hook, Scheduler, Skin. The Scheduler plugin ("schedule your RGB lighting", `https://openrgb.org/plugin_scheduler.html`) was not read.
- Paths such as `ProfileManager.cpp:58` below mean the file at the OpenRGB pin. Add `#L58` to the raw or `src/commit` URL.
- Licence of all three repos: SPDX `GPL-2.0-or-later` in file headers. Not relevant unless code is copied.

## 2. Section A: profiles and lighting on system events (OpenRGB core)

### 2.1 Settings that exist

OpenRGB has no "set all devices to colour or off" setting for any event. Every event setting loads a named profile. "Lights Off" exists only as a tray-menu action (`qt/OpenRGBDialog/OpenRGBDialog.cpp:401-403`, `:1532-1535`). It sets every device to black through the quick-colour path and clears the active profile (`:1952-1968`). So "turn lights off on X" means "make a black profile and load it on X".

All six profile settings share one shape: `{"enabled": bool, "name": "<profile>"}`. The UI shows a checkbox plus a profile combo box (`qt/OpenRGBDynamicSettingsWidget/OpenRGBDynamicSettingsWidget.cpp:169-198`). They live under key `ProfileManager` in `OpenRGB.json` (file path `ResourceManager.cpp:230`). Default for all six is `enabled=false, name=""` (`ProfileManager.cpp:91-141`). A profile is loaded only when `enabled == true` and the name is non-empty (`ProfileManager.cpp:1274-1304`).

The Settings tab is `TabSettings` titled "Settings" (`qt/OpenRGBDialog/OpenRGBDialog.ui:63-69`). Each schema is a group. The Profile Manager group has title "Profile Manager" and order 1 (`ProfileManager.cpp:82`).

### 2.2 Event table

| Event | OpenRGB setting (UI title, JSON key, default) | Detection mechanism per platform | Action | Source file:lines |
|---|---|---|---|---|
| Application start (GUI) | "Load Profile on Open", `ProfileManager.open_profile`, disabled | Not a system event. Runs when detection ends. | Loads the profile, then also applies the active profile's plugin data if no open profile loaded. Runs on every detection-complete, not just the first (no first-only guard in that function). | `OpenRGBDialog.cpp:1617-1664` (call at `:1654`); `ProfileManager.cpp:62-64` |
| Application start (headless server, no GUI) | "Load Profile on Service Startup", `ProfileManager.service_startup_profile`, disabled | Runs on first detection complete, only if `start_server && !start_gui`. | Loads the profile. | `ResourceManager.cpp:881-892`; `ProfileManager.cpp:74-76` |
| Application exit (GUI) | "Load Profile on Exit", `ProfileManager.exit_profile`, disabled | See 2.3 below: window close, tray Exit, and `QCoreApplication::aboutToQuit`. No native shutdown or logoff handler in OpenRGB code. | Unloads plugins first, loads the profile, then sleeps 250 ms "to ensure all profiles are loaded", then quits. | `OpenRGBDialog.cpp:593-629` (profile at `:617`, sleep at `:623`); `:521-555`; `:1518-1530`; `ProfileManager.cpp:58-60` |
| Application exit (headless server) | "Load Profile on Service Shutdown", `ProfileManager.service_shutdown_profile`, disabled | Windows service: `SERVICE_CONTROL_STOP` and `SERVICE_CONTROL_PRESHUTDOWN`. Linux/macOS: `SIGINT`/`SIGTERM` handler sets a stop flag. | Waits for detection, loads profile, sleeps 250 ms, then devices are cleaned up. Does nothing in GUI mode ("the GUI close event handles profile loading"). | `main_Windows.cpp:175-178`, `:208-240`, `:626-648`; `main_FreeBSD_Linux_MacOS.cpp:37-41`, `:97-101`, `:114-128` (all under `startup/`); `ResourceManager.cpp:894-933`; `ProfileManager.cpp:70-72` |
| System suspend, sleep, hibernate (GUI only) | "Load Profile on Suspend", `ProfileManager.suspend_profile`, disabled. Description: "Profile to load before system enters sleep mode". | Windows: Qt native event filter on `WM_POWERBROADCAST` with `wParam == PBT_APMSUSPEND`. Linux/FreeBSD: Qt D-Bus, system bus, `org.freedesktop.login1.Manager.PrepareForSleep(true)`. macOS: IOKit `IORegisterForSystemPower`, message `kIOMessageSystemWillSleep`. | Loads the profile. If a profile was loaded, then unloads all plugins. No wait or sleep after the load. | `SuspendResume/SuspendResume_Windows.cpp:25-46` (`:32-36`); `SuspendResume_Linux_FreeBSD.cpp:14-34`; `SuspendResume_MacOS.cpp:14-42`; `OpenRGBDialog.cpp:1504-1510`; `OpenRGBDialog.h:43,160-161`; `ProfileManager.cpp:78-80` |
| System resume or wake (GUI only) | "Load Profile on Resume", `ProfileManager.resume_profile`, disabled. Description: "Profile to load after system resumes from sleep". | Windows: `WM_POWERBROADCAST` with `wParam == PBT_APMRESUMEAUTOMATIC` (`PBT_APMRESUMESUSPEND` is not handled). Linux: `PrepareForSleep(false)`. macOS: `kIOMessageSystemHasPoweredOn`. | Reloads (starts) plugins first, then loads the resume profile. Plugins are reloaded on every resume, even if suspend loaded no profile. | Same files; `OpenRGBDialog.cpp:1512-1516`; `ProfileManager.cpp:66-68` |
| System shutdown | None. No dedicated setting. | OpenRGB code has no `WM_QUERYENDSESSION`, `WM_ENDSESSION`, `SetProcessShutdownParameters`, console control handler, logind `PrepareForShutdown` or inhibitor. A repo-wide search (excluding `dependencies/`) for these names, `commitData`, `SessionManager`, `logoff`, `inhibit` found nothing. | Only the generic exit path runs if the Qt app quits. Whether Qt emits `aboutToQuit` on OS shutdown is unverified. | Search negative; exit path `OpenRGBDialog.cpp:494`, `:521-555` |
| User logoff | None. No dedicated setting. | Same as shutdown. On Linux, `SIGTERM`/`SIGINT` call `qApp->quit()`, which fires `aboutToQuit`, which runs the exit profile. | Exit profile only if the process receives the signal or Qt quits. | `startup/startup.cpp:32-38`, `:134-137` |

### 2.3 Exit path detail

- Window close: `closeEvent` has two branches (`OpenRGBDialog.cpp:593-629`). If "Minimize on Close" (`UserInterface.minimize_on_close`) is on and the window is visible and the event is spontaneous, the window hides and the event is ignored. Otherwise the exit branch runs.
- Tray menu Exit hides the window and tray icon, then calls `close()` (`:1518-1530`).
- `QCoreApplication::aboutToQuit` is connected to `handleAboutToQuit` (`:494`). It saves window geometry, then calls `closeEvent` with a non-spontaneous event (`:551-554`). So any Qt quit reaches the exit-profile code.
- The exit branch also calls `WaitForDetection()` first (`:595`) and unregisters its resource-manager callback so plugin teardown cannot post blocking calls (`:607-613`).

### 2.4 Detection mechanism notes

- Windows suspend/resume uses a Qt native event filter, not `RegisterSuspendResumeNotification`. Events reach the filter only after the main window has been shown once. For `--startminimized` OpenRGB therefore calls `showMinimized()` then `hide()` (`startup/startup.cpp:106-126`). A code TODO says to try `RegisterSuspendResumeNotification` instead, "that should work in headless mode too" (`startup.cpp:116-117`).
- Windows service mode has the power-event control commented out: `SERVICE_ACCEPT_POWEREVENT` (`main_Windows.cpp:176`) and `SERVICE_CONTROL_POWEREVENT` (`:220`) both carry TODOs. So the headless Windows service and headless Linux/macOS server never react to suspend or resume.
- The suspend/resume listener is a base class of the GUI dialog (`OpenRGBDialog.h:43`). Headless mode has none.
- Linux uses one D-Bus signal for both suspend and hibernate, `PrepareForSleep(bool)`. The code does not distinguish them (`SuspendResume_Linux_FreeBSD.cpp:24-34`). It takes no logind delay inhibitor lock: no `Inhibit` call exists anywhere in the repo.
- macOS calls `OnSuspend()` and then `IOAllowPowerChange` in the same callback, so sleep waits only for `OnSuspend()` to return (`SuspendResume_MacOS.cpp:34-37`). The profile load itself hands work to controller threads, so hardware writes may finish after return. Source for that last point: the 250 ms sleep on the exit path implies it, but I did not trace the controller queue for suspend.
- Sleep versus hibernate on Windows and macOS, and shutdown versus exit: no difference in OpenRGB code. Windows `PBT_APMSUSPEND` semantics for hibernate were not read from a primary Microsoft page.

### 2.5 Autostart, minimised start, server options

UI group "Start at Login" (`AutoStart`, order 2, local-only), `OpenRGBDialog.cpp:171-190`:

| UI title | JSON key (`AutoStart.*`) | Type | Meaning |
|---|---|---|---|
| "Enable Start at Login" | `enabled` | bool | "Start OpenRGB on login" |
| "Start Minimized" | `start_minimized` | bool | "Start minimized to the system tray". Adds `--startminimized` to the arguments. |
| "Custom Arguments" | `custom_arguments` | string | "Additional command line arguments to pass to OpenRGB when starting on login" |

Defaults: none declared in the schema. Missing means false/empty. Applied on every settings update: enable writes the autostart entry with path, args, description `"OpenRGB " + VERSION_STRING`, icon "OpenRGB"; disable removes it (`OpenRGBDialog.cpp:1920-1950`). Pre-1.0 keys (`setminimized`, `setprofile`, `profile`, `setserver*`, `setclient`, `setcustom`) are migrated into the new keys and into `open_profile` and `service_startup_profile` (`:1666-1821`).

Mechanism per platform, entry file name is `OpenRGB` plus extension:

| Platform | Mechanism | Source |
|---|---|---|
| Windows | A `.lnk` shell shortcut (`IShellLink`) in the per-user Start Menu Programs folder plus `\Startup\`. Not a registry Run key. Presence of the file means enabled. | `AutoStart/AutoStart-Windows.cpp:62-132`, `:171-208` |
| Linux | A `.desktop` file in `$XDG_CONFIG_HOME/autostart/`, else `~/.config/autostart/`. Fields: Categories, Comment, Icon, Name, Terminal=false, Type=Application, Exec with escaped args. Uses `$APPIMAGE` for the exe path when set. | `AutoStart/AutoStart-Linux.cpp:62-100`, `:120-146`, `:210-246`, `:248-292` |
| macOS | A LaunchAgent `.plist` in `~/Library/LaunchAgents/` with `Label`, `ProgramArguments`, `RunAtLoad`=true. | `AutoStart/AutoStart-MacOS.cpp:177-212`, `:216-246` |
| Linux Flatpak | FreeDesktop Background Portal, `org.freedesktop.portal.Desktop` `/org/freedesktop/portal/desktop` interface `org.freedesktop.portal.Background` method `RequestBackground`, option `autostart`, plus `commandline` args. It cannot query state. | `AutoStart/AutoStart-Flatpak.cpp:24-150` |

Command-line flags from `cli.cpp:411-462`: `--gui`, `--startminimized` (implies GUI, `:1815-1819`), `--client [IP]:[Port]`, `--server`, `--server-host`, `--server-port` (accepts 1024-65535, setting it also starts the server, `:1558-1594`), `-p/--profile [name]` (loads a profile, `:985-992`, `:1186`), `-sp/--save-profile`, `--localconfig`, `--config path`, `--nodetect`, `--noautoconnect`, `--autostart-check`, `--autostart-disable`, `--autostart-enable arguments`. With no arguments the GUI starts (`:1890-1893`).

Server settings (`Server`, local-only, `ResourceManager.cpp:292-315`): `default_host` (default `127.0.0.1`), `default_port` (default 6742), `all_controllers` ("Serve All Controllers", default false), `legacy_workaround` (default false). Defaults are written at `ResourceManager.cpp:146-176`. The SDK server is not started by the GUI unless `--server` or a server-port option is given, or the user presses the start control on the server info page (`qt/OpenRGBServerInfoPage/OpenRGBServerInfoPage.cpp:127`).

### 2.6 What the SDK exposes

Protocol version 6 (`NetworkProtocol.h:28`). From `NetworkProtocol.h:140-156` and `Documentation/OpenRGBSDK.md:68-79`, `:604-636`:
- 150-155: profile list, save, load, delete, upload, download.
- 156 get active profile.
- 157 `ACTIVE_PROFILE_CHANGED`: server to every client with the ProfileManager flag.
- 158 `PROFILE_LOADED`: carries the full profile JSON, sent only to the "local client" (flag set and connection from localhost).
- 159 `PROFILE_ABOUT_TO_LOAD`: sent to all ProfileManager-capable clients. They should ACK. The server waits for all ACKs and empty controller queues, with a 1000 ms timeout (`NetworkServer.cpp:4028-4071`, `:4093-4115`, ack counting `:1426-1428`).
- 160 profile list updated. 161 clear active profile.
- No packet for suspend, resume, shutdown or logoff. I found none in `NetworkProtocol.h`.
- SettingsManager packets 250-254 (get, get schema, modify, set, save). Get and get-schema work for any client (`NetworkServer.cpp:2475-2540`). Modify/Set require local-client status (`:2556-2559`; same pattern at `:2588`). So a local client can read and change `ProfileManager.*_profile`. The `ProfileManager` key is not local-only (`ProfileManager.cpp:82`); `UserInterface`, `AutoStart`, `Server` are.
- `LoadProfile` (152) has no local-client check (`NetworkServer.cpp:2369-2396`). Loading a profile on the server updates controller state server-side (`Documentation/OpenRGBSDK.md:564`). The profile also carries plugin data and `base_color` (`ProfileManager.cpp:1565-1683`).
- Server-side serving of devices: by default only hardware controllers are given to the SDK server. Plugin and client-provided controllers (including Visual Map virtual devices) are served only if `Server.all_controllers` is true (`ResourceManager.cpp:1029-1046`, `:1435-1449`, description `:297-299`).

## 3. Section B: effects and live preview (Effects plugin 1.1)

### 3.1 Assigning one effect to several devices and zones

Data model, `ControllerZone` (`ControllerZone.h:16-74`):
- `controller` (the device), `zone_idx`, `reverse` (bool), `self_brightness` (int, default 100), `has_direct` (bool), `is_segment` (bool), `segment_idx` (int, -1 if not a segment).
- A segment is a sub-range of a zone. One `ControllerZone` is built per zone, or per segment when the zone is type `ZONE_TYPE_SEGMENTED` with segments (`OpenRGBEffectsPlugin.cpp:390-405`).
- Helpers: LED count, matrix map data, width and height, start index, zone type (`ControllerZone.cpp:61-89`, `:173-224`).
- Colour write: `SetAllZoneLEDs` and `SetLED` apply brightness as `(self_brightness/100) * (effect brightness/100)` plus temperature and tint, and `SetLED` ignores index `0xFFFFFFFF` (`ControllerZone.cpp:150-171`).
- Saved form (`to_json`): `zone_idx`, `reverse`, `self_brightness`, controller `name`, `location`, `serial`, `description`, `version`, `vendor`, `is_segment`, `segment_idx` (`:192-209`).
- Matching a saved zone to a live one: HID location only needs to start with `HID: `; for I2C only the address after the last `, ` is compared; other locations exactly (`:91-148`).
- Per-effect persistence: `ControllerZones` array plus unresolved descriptors that rebind when the device returns, `SelectAll`, `AutoStart` (`OpenRGBEffectTab.cpp:263-320`). This data is stored in the OpenRGB profile under `plugins` (`OpenRGBEffectsPlugin.cpp:206-212`; `ProfileManager.cpp:1660-1663`).

Device list UI:
- `DeviceList` groups zones by controller, one `DeviceListItem` per controller (`DeviceList.cpp:61-124`).
- Controller row: enable checkbox, reverse toggle, brightness slider (hidden until toggled), warning icon if no Direct mode, and a name label (`DeviceListItem.cpp:16-48`). Multi-zone controllers also show one `ZoneListItem` per zone or segment, each with its own enable, reverse and brightness (`DeviceListItem.cpp:63-83`; `ZoneListItem.cpp:15-30`, `:83-97`). Single-zone controllers have one row only (`DeviceListItem.cpp:42-47`).
- Toggling the controller checkbox sets all of its zones; the controller checkbox reflects "all zones enabled" (`:107-117`, `:291-320`).
- List header: "select all" (enables only controllers that have Direct mode, `DeviceList.cpp:126-143`), "reverse all" (`:145-153`), and a brightness-slider toggle (`:155-161`).
- Controllers without a Direct mode are listed only if the global option "hide unsupported" is off (default true). Hidden controllers are always skipped (`OpenRGBEffectsPlugin.cpp:356-379`; `OpenRGBEffectSettings.h:27`).
- Selection is read as the list of enabled zones (`DeviceListItem.cpp:208-231`).

### 3.2 Preview

- A "preview" button on each effect page opens a separate non-modal dialog (`OpenRGBEffectPage.cpp:262-299`).
- The dialog contains a virtual controller named "Preview controller", device type virtual, one zone, one mode "Direct" with per-LED colour and brightness 0-100 (`LivePreviewController.cpp:23-52`). It is created through the plugin API `CreateVirtualRGBController` (`:52`).
- Size presets: matrix 8x8, 8x16, 8x32, 8x64, 16x16, 16x32, 16x64, 32x32, 32x64, 64x64, 64x128, 128x128; linear 8, 10, 12, 16, 20, 24, 48, 72, 128 LEDs; default "Matrix 64x64"; custom width and height spin boxes make a matrix zone (`LivePreviewController.h:62-85`, `.cpp:79-82`, `:229-239`).
- Drawing: one QImage, linear zone is a width-N by height-1 image, matrix zone is width by height. Each pixel is the LED colour scaled by the preview brightness. The image is scaled to the widget with `Qt::FastTransformation`, optional "scale" checkbox, double click or F11 for full screen, with a "reverse" checkbox (`LivePreviewController.cpp:149-222`, `:241-254`; `PreviewWidget.cpp:12-42`; `OpenRGBEffectPage.cpp:289-291`).
- The preview is a separate render, not a copy of device frames. The effect thread calls `StepEffect` once with the assigned zones, writes those controllers, then calls `StepEffect` a second time with only the preview zone and updates the preview controller (`EffectManager.cpp:368-401`). The preview therefore runs at the same rate as the effect thread and shows the effect on a synthetic canvas, not the real zone sizes.
- The preview zone is also appended to the zone list given to effects that implement `OnControllerZonesListChanged` (`EffectManager.cpp:471-484`).
- Rate: effect thread targets `1,000,000 / FPS` microseconds per loop, sleeps the remainder, or 1 ms if over budget (`EffectManager.cpp:410-435`). Per-effect FPS slider range 1-60, default 60 (`OpenRGBEffectPage.ui:291-299`; `Effects/RGBEffect.h:137`; saved as `FPS` at `OpenRGBEffectPage.cpp:462`, `:496`).

Core OpenRGB LED view (`qt/DeviceView.cpp`, `.h`):
- It paints each LED rectangle with the controller's current colour (`GetColor`, `:924-962`, colour read at `:942-944`).
- Refresh: the device page registers an update callback with the controller, which queues the UI update; on `RGBCONTROLLER_UPDATE_REASON_UPDATELEDS` it repaints the view (`qt/OpenRGBDevicePage/OpenRGBDevicePage.cpp:32-37`, `:53`, `:1999-2004`). So it shows what is in the controller's colour buffer, not a separate render. Painting is skipped when hidden (`DeviceView.cpp:883-886`).
- Layout (`InitDeviceView`, `:1112-1419`): zones are stacked top to bottom, each with a name label row. Matrix zones use the matrix map width and height. Linear and single zones are laid out in rows wrapped at 20 columns (`MAX_COLS 20`, `:23`; `:1394-1411`). Segments are laid out the same way under their zone (`:1360-1385`). Keyboards get label text and key expansion (`:36+`, `:1274-1307`); settings "Numerical Labels" and "Disable Key Expansion" exist (`OpenRGBDialog.cpp:233-240`).
- "Show LED View by Default": `UserInterface.show_led_view` (`OpenRGBDialog.cpp:229-231`).

### 3.3 One effect on several zones

- Each zone is rendered independently, starting from LED index 0, not on one continuous canvas. Example, RainbowWave: for each assigned zone, a linear zone gets a hue per LED from `Progress + LedID`, reversed if `reverse` is set; a matrix zone gets a hue per column; a single zone gets one hue (`Effects/RainbowWave/RainbowWave.cpp:46-118`).
- Timing is shared per effect instance. `Progress` is one counter advanced once per step by `Speed / FPS` (`RainbowWave.cpp:120-127`). All zones of one effect move in phase; different effect instances are not synchronised with each other.
- One thread per active effect (`EffectManager.cpp:39`, `:339-444`). After stepping, each distinct controller gets one `UpdateLEDs()` call (`:374-384`). Multiple effects can run at once on different zones (`https://openrgb.org/plugin_effects.html`).
- Global settings in the plugin: `fpscapture` 60, `brightness` 100, `fps` 60, `hide_unsupported` true, `startup_timeout` 2000 (`OpenRGBEffectSettings.h:22-30`). I found no code outside the settings widget that reads `fps`, `brightness` or `startup_timeout`, so their effect is unverified.
- Plugin hooks: it reacts to `OnProfileAboutToLoad` and `OnProfileLoad` (`OpenRGBEffectsPlugin.cpp:182-204`). It has no suspend or resume handling (search negative). OpenRGB core unloads and reloads plugins around suspend and resume (see 2.2).

## 4. Section C: spatial layout

### 4.1 Visual Map plugin (`release_1.0`, plugin API 5)

Purpose (README, `README.md:5-13`): organise real devices on a map, create one or more virtual devices, apply gradients, and expose the virtual devices to other plugins such as Effects.

Data model:
- Map file (JSON, one per virtual controller): `ctrl_zones` array and `grid_settings` (`ui/VirtualControllerTab.cpp:610-613`). Saved under `<config dir>/plugins/settings/virtual-controllers/`; gradients under `.../gradients/` (`VisualMapSettingsManager.cpp:107-120`).
- `grid_settings`: `w`, `h` (1-1024 each, `ui/GridOptions.ui:24-32`, `:79-87`), `show_grid`, `show_bounds`, `grid_size`, `snap_to_grid`, `auto_load`, `auto_register`, `hide_members` (`VisualMapJsonDefinitions.h:132-171`; `ui/GridSettings.h:4-15`).
- Each `ctrl_zones` entry: `controller` (name, vendor, description, version, serial, location), `zone_idx`, `custom_zone_name`, `settings` (`VisualMapJsonDefinitions.h:112-130`). I note that the written JSON has no `is_segment` or `segment_idx`; those exist in the in-memory struct (`ControllerZone.h:200-208`) but are not in `to_json`. Whether segments round-trip correctly was not tested.
- `settings`: `shape`, `x`, `y`, `scale`, `led_spacing`, `reverse`, `custom_shape` (null unless shape is Custom) (`VisualMapJsonDefinitions.h:70-88`; `ControllerZone.h:182-197`).
- Shapes: `HORIZONTAL_LINE`, `VERTICAL_LINE`, `CUSTOM` (`ControllerZone.h:175-180`; UI labels Horizontal, Vertical, Custom, `ui/ItemOptions.cpp:11-15`). Spin-box limits: LED spacing 1-16, x and y from -1024 to 1024, scale 0.1-1024 (`ui/ItemOptions.ui:27-104`).
- Custom shape: width `w`, height `h`, and per-LED `led_num`, `x`, `y` grid cells (`ControllerZone.h:18-163`; `VisualMapJsonDefinitions.h:21-68`). Matrix zones and matrix segments are auto-converted to a Custom shape from their matrix map (`ZoneManager.cpp:64-69`, `:85-90`, `:201-216`).
- Rotation and flip exist only in the custom-shape editor: rotate by 90 degrees (selection or whole shape) and horizontal and vertical flip (`ui/widget-editor/WidgetEditor.cpp:350-428`). There is no per-zone free rotation field; `reverse` only flips LED order on line shapes (`LedRouting.cpp:41-82`).

Virtual device:
- `VirtualController` registers a controller named "Visual Map Controller", type virtual, one zone "Virtual Zone" of type matrix, one mode "Direct" with per-LED colour and brightness (`VirtualController.cpp:22-74`).
- Its matrix map is the grid, width times height. Each cell holding at least one real LED gets a virtual LED index; empty cells are `NA = 0xFFFFFFFF` (`VirtualController.h:13`; `VirtualController.cpp:96-190`). Cells hit by several real LEDs are named "Multiple LEDs (n)" (`:175-178`).
- Consumption by Effects: the Effects plugin lists virtual controllers like any controller with a Direct mode and treats the zone as a matrix; its `SetLED` ignores `0xFFFFFFFF` cells (`ControllerZone.cpp:165-171`). The plugin API gives both plugins `CreateVirtualRGBController` and `RegisterVirtualRGBControllerInThread` (`OpenRGBPluginAPI.cpp:41`, `:59`).
- Output path: when the virtual device's LED buffer updates, `DeviceUpdateLEDs` builds a width x height image from it (transparent for `NA` cells). `ApplyToDevice` then writes it onto each real zone and calls `UpdateLEDs()` once per controller (`VirtualController.cpp:192-220`, `:483-541`).
- Each real LED owns a rectangle of canvas cells (size = `scale`, spacing = `led_spacing`). Its colour is the area-weighted average of the canvas pixels it overlaps (`LedRouting.cpp:41-204`).
- Registering forces member controllers into Direct mode (`VirtualController.cpp:335-354`). Option `hide_members` hides the real members (`SetHidden(true)`) while the map is registered (`:241-332`).
- Gradients: a "BackgroundApplier" paints gradients (presets, custom stops, spread, rotate angle) or images onto the grid (`ui/BackgroundApplier.cpp:20-70`, `:96-98`, `:379`).

Limits and cautions:
- Direct-mode devices only (README `:22-24`; `ForceDirectMode`).
- The virtual device is a single matrix zone, grid up to 1024 x 1024 cells.
- Zone identity for maps uses name, vendor, serial, location and `zone_idx`; HID locations are ignored in matching, so two identical HID devices may be ambiguous (`ui/VirtualControllerTab.cpp:380-386`).
- Virtual controllers are not served to SDK clients by default (see 2.6).

### 4.2 Core OpenRGB 1.1 data about layout

- Zone types: `ZONE_TYPE_SINGLE`, `LINEAR`, `MATRIX`, `LINEAR_LOOP`, `MATRIX_LOOP_X`, `MATRIX_LOOP_Y`, `SEGMENTED` (`RGBController/RGBControllerInterface.h:151-160`).
- `zone` fields: name, display_name, type, LEDs, colours, start_idx, leds_count, leds_min, leds_max, `matrix_map` (height, width, map of LED indexes), `segments`, flags, per-zone modes (`:420-463`). `segment` fields: name, type, start_idx, leds_count, `matrix_map`, flags (`:385-415`). `matrix_map_type` is height, width, row-major vector of LED indexes (`:239-290`). `led` has only `name` and `value` (`:354-380`).
- Zone flags cover manual configuration (size, name, type, matrix map, segments, device-specific) and `ZONE_FLAG_ZONE_GEOMETRY_MAY_CHANGE` (`:128-143`). Segment flags mark group start and member (`:167-171`). Controller flags: local, remote, virtual, hidden, and others (`:221-233`).
- Core has no device position, rotation or room layout. A search of `RGBController.h` and the interface for "position" and "rotat" found nothing in the data model. Layout beyond zone type, matrix maps and segments exists only inside `DeviceView` (UI-only, not stored) and in the Visual Map plugin. The protocol-6 doc lists "matrix map segments, per-zone modes" among additions (`Documentation/OpenRGBSDK.md:17`).

## 5. What this means for a separate client app (my inference, not from sources)

1. A client cannot rely on OpenRGB to light on suspend or shutdown. OpenRGB's own handling is GUI-only, depends on a profile, and no SDK packet tells a client the machine is suspending. PrismForge must detect these events itself, per OS.
2. Conflict risk at suspend and resume. If the user enabled OpenRGB's suspend or resume profile, both programs write at the same time and the last writer wins. Mitigation ideas: read `ProfileManager` via SettingsManager packets 250/251 (readable by any client) and warn when an OpenRGB event profile is enabled; or let the user choose one owner.
3. The server also unloads and reloads plugins around suspend and resume, only when a suspend profile loads. A client that relies on the Effects plugin running through resume should expect it to restart.
4. Exit and shutdown: OpenRGB writes after an exit profile load and waits 250 ms. A client sending a final "off" before disconnect should also wait briefly for the server's controller queue. OpenRGB itself does not wait after the suspend load, and does not hold a sleep inhibitor on Linux, so a late write may miss the sleep.
5. If PrismForge sets the SDK ProfileManager client flag, it must ACK packet 159. The server waits up to 1 s for all ACKs on every profile load, so a slow client delays everyone. Do not set the flag unless needed. Packet 158 is only sent to the local client (flag plus localhost).
6. Autostart: do not duplicate. Windows uses a Startup-folder `.lnk`, Linux a `.desktop`, macOS a LaunchAgent plist. These are plain, easy-to-replicate mechanisms. A client that wants to start with OpenRGB's server must still make sure OpenRGB runs with `--server` (the server is off by default in the GUI).
7. Preview design: the Effects plugin's preview is an independent render of the same effect on a synthetic zone (virtual 64x64 matrix by default) at the effect's FPS. This matches a simple client design: render to an off-screen canvas, show it, separately render to devices.
8. Multi-device effects: OpenRGB renders each zone from index 0 with a shared time. Continuous cross-device effects need a spatial map. If PrismForge wants this, it needs its own layout data, because core has no positions and the Visual Map virtual device is not served over the SDK by default.
9. Visual Map's map file format is plugin-private. It is not part of the SDK, so there is no supported way to read the user's map from a client, other than reading the files on disk. I did not verify the file format is stable.
10. License: all three repos are GPL-2.0-or-later. PrismForge is GPL-3.0-or-later. Re-deriving behaviour in our own code avoids the question; copying code would need a compatibility review.

## 6. Open points not verified

- Whether Qt on Windows, Linux and macOS emits `aboutToQuit` on OS shutdown or logoff (so that the exit profile fires). OpenRGB code has no own handler. Qt source was not read.
- Windows `PBT_APMSUSPEND` and `PBT_APMRESUMEAUTOMATIC` semantics for hibernate and time limits for suspend handlers. No Microsoft primary page was read.
- How long the controller queue takes to flush after a profile load on suspend; whether writes finish before the OS sleeps.
- The OpenRGB wiki and the plugin help pages (GitLab-only; the Codeberg mirror has no wiki repo: `https://codeberg.org/OpenRGB/OpenRGB-Wiki` returned 404). The `Documentation/` folder in the repo has no page on settings, suspend or autostart (listing via `https://codeberg.org/api/v1/repos/OpenRGB/OpenRGB/contents/Documentation?ref=8e16de27858cf2be6af54a33ce003df74cb9fd3e`).
- Visual Map segment round-trip (see 4.1) and map-file stability.
- The Effects plugin's global `fps`, `brightness` and `startup_timeout` settings: stored and edited, but I found no reader.
- The Scheduler and HTTP Hook plugin sources were not read. They may offer time or event triggers relevant to "lighting on system events".
- OpenRGB `auto_connection` (a second instance auto-connects to a local server as a client; flag `--noautoconnect`, `cli.cpp:451`; `ResourceManager.cpp:206-207`, `:711-718`, `:859-866`) was only skimmed. Its effect on which instance handles event profiles was not traced.
