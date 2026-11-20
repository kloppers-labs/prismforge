# Hardware validation

This file records tests of PrismForge against a real OpenRGB server. Each entry removes serial numbers, Windows device instance paths and other values that identify one machine.

## How to add an entry

1. Start OpenRGB 1.0 or later and turn on the SDK server.
2. Run the read-only probe: `cargo run -p prismforge-openrgb --example probe -- --host 127.0.0.1 --port 6742`.
3. Remove serial numbers and `HID:` location paths from the output.
4. Add a new section below with the date, the OpenRGB version, the operating system and the result.

The probe only reads. It does not write colours and it does not change modes.

## 2026-11-19: first probe, OpenRGB 1.1, Windows 11

### Result

- The probe negotiated SDK protocol 6.
- The client read six controllers. It parsed every controller without a per-device issue.
- The final connection state was `Connected`.

This is the first test against a real server. It confirms the handshake, the controller count reply and the controller data parser for protocol 6. It does not test colour writes, notifications or reconnects.

### Devices

| Device | Type | LEDs | Zones | Direct Mode | Location kind |
|---|---|---|---|---|---|
| ENE DRAM (module 1) | Memory | 8 | 1 linear zone, 8 LEDs | Yes, mode 0 | SMBus, address `0x72` |
| ENE DRAM (module 2) | Memory | 8 | 1 linear zone, 8 LEDs | Yes, mode 0 | SMBus, address `0x73` |
| Gigabyte B550 AORUS ELITE V2 | Motherboard | 77 | `D_LED1` and `D_LED2` (linear, 36 LEDs each, size 0 to 1024); `I/O Cover`, `LED_C1`, `LED_CPU`, `PCI-E Accent`, `LED_C2` (1 LED each) | Yes, mode 0, brightness 0 to 255 | HID |
| Microsoft HID VHF Driver | Accessory | 36 | 1 matrix zone, 18 × 12 | Yes, mode 0 | HID |
| Keychron Gaming Keyboard | Keyboard | 84 | 1 matrix zone, 16 × 6 | **No** | HID |
| NVIDIA GeForce RTX 3060 Ti LHR | GPU | 2 | `0 - RGBW`, `1 - SINGLE COLOR` (1 LED each) | Yes, mode 1, brightness 0 to 100 | None reported |

### Findings

1. **Identity of identical modules works.** The two memory modules have the same vendor, name and zone shape, and no serial. Their SMBus addresses are different, so PrismForge gives them different device identities.
2. **The keyboard has no Direct Mode.** Its only per-LED mode is named `Custom`, and every mode on this keyboard reports `AutomaticSave` persistence. This means each change goes to the keyboard's own memory. PrismForge does not send effects to this keyboard, because frequent writes to device memory can wear it out. This agrees with the rule in [decision 0001](decisions/0001-openrgb-integration.md): a colour frame goes only to a mode named `Direct`.
3. **Speed ranges are reversed on two devices.** The memory modules report speed from 4 to 0, and the motherboard reports speed from 9 to 0. A lower number is a faster speed on these devices. PrismForge does not set hardware mode speeds yet. Any later work that does must accept a minimum that is larger than the maximum.
4. **Resizable zones report their maximum.** `D_LED1` and `D_LED2` report 36 LEDs with a permitted size of 0 to 1024. The real number of LEDs on the connected strips or fans is not known yet.
5. **One accessory is not identified.** OpenRGB reports a "Microsoft HID VHF Driver" accessory with 36 LEDs in an 18 × 12 matrix. VHF is the Windows virtual HID framework, so this is probably a virtual device that another program creates. The owner does not know what it is. A colour change on it lights nothing, in OpenRGB and in PrismForge. Treat it as a device without real LEDs.
6. **Some devices have no serial or no location.** The memory modules and the GPU have an empty serial. The GPU also has an empty location. Identity depends on the other fields for these devices.

### Not tested in this run

- Colour writes to any device.
- "Device list updated" notifications and update packets.
- Reconnect after an OpenRGB restart, and whether controller IDs stay the same.
- The PrismForge desktop app. See the next entry.

## 2026-11-19: first desktop app test, OpenRGB 1.1, Windows 11

The owner ran the desktop app from `main` with `pnpm tauri:dev` on the same PC as the first probe. Task 8b (context menus) was the last merged task. Task 9 (several targets, groups) was not merged yet.

### Result

The app connected to OpenRGB and controlled the hardware as expected. The owner found no errors.

| Check | Result |
|---|---|
| Connect to the OpenRGB SDK server | Passed |
| Effect on the memory modules | Passed |
| Effect on the motherboard as a whole device | Passed |
| Effect on single motherboard zones (`D_LED1`, `D_LED2`) | Passed |
| Effect on the GPU | Passed |
| Effects on different devices at the same time, each on its own | Passed |
| Colour changes in effects | Passed |
| Save a profile, change effects, and see the changes in the profile | Passed |
| "Microsoft HID VHF Driver" accessory | No visible change, in PrismForge and in OpenRGB |

### What is on the headers

- `D_LED1` drives the ARGB case fans through a hub.
- `D_LED2` drives the ARGB plate of the CPU cooler.

OpenRGB reports 36 LEDs for each header. The real LED counts are not known yet.

### Not tested in this run

- Reconnect after an OpenRGB restart.
- Start with the system and the start profile.
- A screen reader, high contrast themes and 200 % scaling.

## To test: automation on system events (Task 12)

Nothing in this section has run on Windows yet. Build from the Task 12 branch, connect to OpenRGB 1.1 on the Windows 11 PC, and on **Automation** switch on "When the system goes to sleep, switch the lights off" and "When the system wakes, apply the profile …" with a profile that runs an effect. Switch on the session-end rule only for the sign-out and shutdown checks. After each check, open Automation and read the last run of the rule. Record the result here; remove serial numbers first.

| Check | What to look for |
|---|---|
| Sleep from the Start menu | The lights go dark before the PC sleeps. The last run of the sleep rule says the lights were switched off and names the keyboard as skipped, because its modes save to its memory. The keyboard is not written. |
| Sleep with the power button | The same. Note how long the power button needs before the PC is asleep. |
| Wake | Within about 30 seconds the profile's effect runs again on every device that came back; the last run of the wake rule names any device that did not come back. Note how long the fans, RAM and GPU take to come back. |
| Hibernate, then start | The same as sleep and wake. Windows gives one signal for both. |
| Restart | With the session-end rule on: the lights go dark, or show the rule's profile, before the PC restarts. At the next start the last run of the rule shows the result, or nothing if Windows ended PrismForge first. |
| Shut down with Fast Startup on | The same. Note whether the devices stay lit after the PC is off (motherboard power after shutdown). |
| Shut down with Fast Startup off | The same. |
| Sign out | The same as shutdown. |
| Each check above with OpenRGB's own profile settings on | In OpenRGB, Settings, Profile Manager: turn on "Load Profile on Suspend", "Load Profile on Resume" and "Load Profile on Exit" with a named profile. The rules on Automation say that OpenRGB also loads that profile. Note which program's lighting the devices show after each event; the last writer wins. Turn the OpenRGB settings off again afterwards. |
| Quit from the tray with the quit rule "switch the lights off" | The lights go dark and stay dark after PrismForge has ended. |
| OpenRGB's answer to the flags and settings packets | With OpenRGB running, open Automation. It says either which OpenRGB profiles act on the same events, or that PrismForge cannot check. Diagnostics shows no lost connection at the connect. If the connection drops at every connect, OpenRGB does not answer packet 52 or 250 as its source says; report it. |
| Seam start | Automation shows no "could not start receiving sleep, wake and session end" or "Part of PrismForge's set-up for system events failed" message. If it does, note the detail. |
| A failed rule at the next start | With the sleep rule on, quit OpenRGB, then sleep and wake the PC. At the next start of PrismForge the notices area says the sleep rule did not do what it should, until you dismiss it or open Automation. |
| Settings after the change | Settings, step 4, names the rule "When PrismForge connects" or "When PrismForge starts" and its profile. A profile that was marked "Apply this profile when PrismForge connects" before Task 12 shows as the connects rule, switched on, and still applies at the connect at start. |
