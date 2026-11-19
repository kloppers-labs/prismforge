# 0001 — OpenRGB integration boundary

- Status: accepted
- Date: 2026-11-18
- Scope: how PrismForge talks to hardware in the first release

## Current state

PrismForge controls hardware only through the OpenRGB SDK server, a local TCP service that OpenRGB provides for other programs. PrismForge has its own small SDK client in `crates/prismforge-openrgb`. The client is tested against a scripted fake server in `crates/prismforge-testkit`. On 2026-11-19 a read-only probe passed against a real OpenRGB 1.1 server (see [hardware validation](../hardware-validation.md)). Other statements below about server behaviour come from reading the OpenRGB documentation and source code at pinned commits.

## Decisions

1. **OpenRGB owns the hardware.** PrismForge sends commands to the OpenRGB SDK server on TCP port 6742. PrismForge does not include, link or change OpenRGB code.
2. **PrismForge has its own SDK client.** We do not use the Rust crate `openrgb2`. See "Why not `openrgb2`".
3. **SDK protocol 6 is required.** OpenRGB 1.0 and 1.1 use protocol 6. An older server gets a structured "unsupported protocol" error that carries the server version.
4. **A runtime controller ID is never a device identity.** PrismForge stores its own `DeviceRef` and matches it again to the current inventory after each refresh. See "Identity".
5. **Effects are the main feature, and PrismForge builds its own effect engine first.** The Effects plugin's network interface gives only list, start and stop, and it has no ownership signal. Support for the plugin comes after the engine. See "Effects plugin".
6. **PrismForge does not load OpenRGB profiles through the SDK.** A profile load removes all effects in the Effects plugin. See "Effects plugin".

## Sources

All sources were read on 2026-11-18.

| Item | Pinned reference |
|---|---|
| OpenRGB 1.1 | tag `release_1.1`, commit `8e16de27858cf2be6af54a33ce003df74cb9fd3e` (2026-10-16) |
| OpenRGB master | commit `5bcfa9ae01536d2dd45b0c6e5b15677513f7a6d5` (2026-11-16) |
| OpenRGB Effects plugin 1.1 | tag `release_1.1`, commit `f211d08afd3382318e60f829348b25c6e15fb0ae` (2026-10-16) |
| OpenRGB Effects plugin master | commit `64928a4bccd1efdd05b366de38601c96b6bd59d7` (2026-10-28) |
| `openrgb2` crate | version 0.4.1 from crates.io |

- Protocol document: [`Documentation/OpenRGBSDK.md` at `release_1.1`](https://gitlab.com/CalcProgrammer1/OpenRGB/-/blob/8e16de27858cf2be6af54a33ce003df74cb9fd3e/Documentation/OpenRGBSDK.md). This file and `NetworkProtocol.h` are the same at `release_1.1` and at the master commit above.
- Version table: <https://openrgb.org/releases.html>. OpenRGB 1.0 and 1.1 have SDK protocol 6 and plugin API 5. OpenRGB 0.9 has SDK protocol 4 and plugin API 3.
- Effects plugin page: <https://openrgb.org/plugin_effects.html>. Effects plugin 1.0 and 1.1 need plugin API 5.

The SDK protocol version, the plugin API version and the application version are three different numbers. PrismForge reads the negotiated protocol version from the server. It does not guess it from an application version.

## Why not `openrgb2`

| Fact | Evidence |
|---|---|
| The licence field is `GPL-2.0`. In SPDX this means version 2 only. | `Cargo.toml` of `openrgb2` 0.4.1 |
| The protocol 6 packets are not connected to its client API. | README of 0.4.1, section "OpenRGB v1.0 Users" |
| Messages that the server sends without a request do not fit its design. | Same README section |

A GPL-2.0-only dependency does not permit a GPL-3.0-or-later application licence. It also does not permit dependencies that are Apache-2.0 only. PrismForge also needs its own limits for packet size, its own timeouts, and server notifications. The protocol is small, so our own client is the lower risk.

The cost is about one thousand lines of protocol code that we maintain. The adapter hides the client, so a different client can replace it later.

## Identity

In protocol 6 the server gives each controller a unique 32-bit ID. The protocol document does not promise that an ID stays the same after an OpenRGB restart. The rules are:

- PrismForge stores a `DeviceRef` and the identifying fields of the device: serial, vendor, name, location, type, and zone shape.
- After each inventory refresh, PrismForge matches stored devices to current devices by these fields. It never matches by runtime ID or by list position.
- If two candidates look the same, the match stays **ambiguous**. PrismForge sends no output to an ambiguous device. The user must identify it.
- After a "device list updated" message or a reconnect, the old mapping is stale. PrismForge refuses output until the next refresh.

## Output rules

- A colour frame goes only to a device that has Direct Mode.
- The colour count must equal the LED count of the device or zone. If not, PrismForge refuses the frame.
- Discovery never writes to hardware. A hardware write needs an explicit action.
- A controller that sends malformed or inconsistent data becomes a per-device issue. The other devices stay usable.

## Effects plugin

The Effects plugin was studied from source only. Nothing was run.

### What the plugin gives a network client

The SDK has a general "plugin specific" packet (ID 201). The Effects plugin handles three sub-commands in versions 1.0, 1.1 and master.

| Operation | Status in the plugin | Status in PrismForge |
|---|---|---|
| List configured effect instances (name, description, enabled flag) | Supported, sub-command 0 | Not implemented |
| Start an effect by name | Supported, sub-command 20 | Not implemented |
| Stop an effect by name | Supported, sub-command 21 | Not implemented |
| Change effect parameters (speed, colours, brightness) | Not exposed | Not possible |
| Create or delete an effect instance | Not exposed | Not possible |
| Change the devices or zones of an effect | Not exposed | Not possible |
| List or load Effects plugin profiles | Not exposed in 1.0 and 1.1. Sub-commands 22 and 23 existed in 1.0rc2 only. | Not possible |
| Notification when effects change | Not exposed | Not possible |
| Signal that shows which writer controls a device | Does not exist | Not possible |
| Persistence of a start or stop sent through the SDK | Unverified | Not applicable |

Evidence, Effects plugin at `64928a4b`: `OpenRGBEffectsPlugin.h` lines 25-32 (sub-command IDs), `OpenRGBEffectsPlugin.cpp` lines 153-180 (handler), `OpenRGBEffectTab.cpp` lines 557-652 (start, stop and list). The handler code is the same at tag `release_1.1`.

### Limits that affect the product

- **Effect names are not safe identifiers.** The name is translated text. Two instances of one effect have the same name. Start and stop act on the first match.
- **Errors are not visible.** Start and stop have no reply. The server acknowledge is always "OK", also when no effect has that name.
- **The list has no targets.** A client cannot learn which devices or zones an effect writes to.
- **The plugin's own SDK document is wrong in one place.** It gives 41 for "stop". The code uses 21.
- **A profile load removes all effects.** When a client loads any OpenRGB profile through the SDK, the plugin stops and clears all effect instances first. If the profile has no plugin data, the plugin stays empty. Evidence: OpenRGB `ProfileManager.cpp` lines 1627-1632 and 1660-1663 at `5bcfa9ae`; plugin `OpenRGBEffectsPlugin.cpp` lines 182-192.
- **No ownership signal exists.** OpenRGB applies each colour write from each client. The plugin writes again on its next frame. A client cannot learn who wrote a frame.

### Result for PrismForge

The owner decided on 2026-11-18 that effects are the main feature, and chose this order:

1. **Own effect engine first.** PrismForge computes effects in Rust and sends colour frames through the SDK. The owner uses four kinds of effects: colour and wave, animated patterns, audio-reactive, and screen-reactive. All four are planned. Audio capture and screen capture each need their own design record and resource measurement.
2. **Plugin provider second.** PrismForge lists, starts and stops the effects that the user configured in the plugin. This is a separate provider with explicit ownership.
3. **Plugin editing only if a test passes.** One possible route exists: download a profile as JSON (packet 155), change the Effects plugin data in it, upload it (packet 154) and load it (packet 152). Upload needs a connection from the same machine that asks for "local client" status. This route is **not verified**. The JSON format has no documentation, and each load restarts all effects. PrismForge shows no editing screen for plugin effects until this route passes a test on a real OpenRGB 1.1 server.

Rules that apply in all cases:

- PrismForge cannot detect that the plugin writes to a device. The user must tell PrismForge who controls a device. PrismForge then explains the handover.
- PrismForge does not stop the plugin without a request, and it does not stop other processes.
- PrismForge and the plugin must not write to the same device at the same time.

## Protocol behaviour

These facts come from the OpenRGB source at `release_1.1`. Line numbers are for `NetworkServer.cpp` unless a different file is named.

| Behaviour | Source lines |
|---|---|
| The server stores `min(client version, 6)` as the version of the client. | 1680-1691 |
| The version reply comes first. Then the server name follows (protocol 6), then the acknowledge. | 1335-1339, 3773-3791 |
| The server sends an acknowledge only to clients at protocol 6. A reply always comes before its acknowledge. | 3562-3584, 1583-1588 |
| A request for an unknown controller ID gets an acknowledge with status OK and no reply. | 1522, 1587 |
| The controller count reply carries the count and then the unique IDs. | 3586-3635 |
| The server closes a client that sends a packet larger than 8 MiB. | 1297-1301, `NetworkProtocol.h` 38 |
| A colour update must have a size field equal to the packet size. | 3123-3133, 3381-3391 |
| "Device list updated" goes to every client. | 909-919, 4017-4026 |
| Update packets (ID 1150) go to every protocol 6 client, with or without client flags. They can arrive at effect frame rate. | 4152-4360 |
| Unique controller IDs start at 0 when the server starts. **A restart uses the same IDs again, possibly for different devices.** | 161, 644-711 |
| "Set custom mode" selects the first mode named `Direct`, then `Custom`, then `Static`. | `RGBController.cpp` 1490-1522 |
| A colour on the wire is the bytes red, green, blue, zero. | `RGBControllerInterface.h` 29 |

PrismForge policy that is stricter than the server:

- PrismForge reports Direct Mode only for a mode named `Direct` with per-LED colours.
- PrismForge sends a full frame only. The server accepts a partial colour list.
- The client stops a request after 2 seconds by default and then closes the connection, because a late reply could go to the wrong request.
- The client does not set the "supports ProfileManager" flag. With that flag the server waits for the client to acknowledge "profile about to load".
- The client reads update packets (ID 1150) and discards them.

## Known limits of the client

- A request completes on its own acknowledge packet. This rule depends on the server behaviour in the table above, which is read from source and not yet seen on a real server.
- If one controller does not answer a data request in time, the client closes the connection. The whole refresh fails, not only that device.
- The client does not read the acknowledge for a colour frame. PrismForge cannot count the frames that the server has not yet applied. The output scheduler must limit its own rate.
- Device strings must be valid UTF-8. A device with a different text encoding becomes a per-device issue and cannot be used.
- The list of known devices is in memory only. Profile storage adds persistence.
- A backend call must run to its end. If the caller stops a call before it completes, the client closes the connection on the next call. This is safe, but it costs a reconnect.

## Licence notes

- OpenRGB and the Effects plugin have GPL version 2 as the root licence text. The source files that we read have the header `SPDX-License-Identifier: GPL-2.0-or-later`.
- PrismForge contains no OpenRGB code and no OpenRGB documentation text. The client is our own code, written from the public description of the wire format.
- We read OpenRGB server source to learn behaviour that the document does not state. `docs/provenance.md` records this.
- This is an engineering record. It is not legal advice.

## Not verified

- Colour writes, notifications and acknowledge order against a real OpenRGB 1.1 server. Negotiation and inventory passed a read-only probe on 2026-11-19. See [hardware validation](../hardware-validation.md).
- The Effects plugin operations in the table above.
- Whether a controller ID stays the same after an OpenRGB restart on the owner's machine.

## How to verify on a real server

1. Start OpenRGB 1.1 and turn on the SDK server.
2. Run the read-only probe: `cargo run -p prismforge-openrgb --example probe -- --host 127.0.0.1 --port 6742`.
3. Record the output in `docs/hardware-validation.md`. Remove serial numbers first.
4. Run a colour write only with the explicit device and colour arguments of the probe.
