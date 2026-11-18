# RGB hub feasibility review

Research date: 18 November 2026.

## Recommendation

Build an original Windows-first Svelte/TypeScript and Tauri application with a Rust application core and an OpenRGB backend adapter. Keep the hardware boundary replaceable so a native adapter can be added later for a particular controller when measurements or missing functionality justify it.

The reviewed evidence does not establish a worthwhile performance advantage from replacing OpenRGB's entire hardware layer. There is a clearer opportunity in component setup, configuration, effect authoring, recovery, and background resource use. A polished application using OpenRGB can deliver those improvements without reproducing its device library.

This is a feasibility assessment, not an approved implementation design or evidence that the proposed application has been tested on the user's PC.

## Review scope and method

Reviewed [Lightloom](https://github.com/MrMoraleDev/Lightloom) at commit [`b8b482b6597fc03782ab4f804cede7dcf654c673`](https://github.com/MrMoraleDev/Lightloom/commit/b8b482b6597fc03782ab4f804cede7dcf654c673), checked out on the research date. Repository metadata reported its most recent push as 16 November 2026.

The public source was downloaded into a research checkout and indexed with codebase-memory-mcp. Graph searches and source snippets were used for Rust/plugin discovery. Svelte components were read directly after the graph did not return component functions. Serena was not exposed in this session.

Inspection covered the manifests, current frontend components, plugin catalog/discovery, device session lifecycle, frame publication and sampling, API routes, tray implementation, architecture decisions, hardware notes, effect benchmarks, licences and GitHub release metadata. No project binaries, build scripts, tests or hardware operations were executed. Hardware and performance results discussed below are explicitly the author's reports, not independently reproduced results.

## What Lightloom actually contains

| Area | Observed implementation | Relevance |
|---|---|---|
| Application core | Rust workspace with daemon, engine, API, renderer, plugin host, platform HALs, broker, setup and tray crates | It is a substantial standalone platform, not just a frontend. |
| Dashboard | Svelte 5, TypeScript and Vite; static files served through a local HTTP API | Very close to the proposed frontend technology. |
| Desktop integration | Current tray uses native platform code; on Windows it opens the dashboard URL through Explorer | The architecture document mentions an optional Tauri shell, but the inspected current tray is not Tauri. |
| Hardware plugins | JavaScript modules hosted through QuickJS, translating sampled colours into HID, USB, serial, SMBus or network operations | Extensibility still requires device-specific protocol knowledge and testing. |
| Effects | Separate CEF/Chromium renderer paints HTML/canvas effects; frames pass through shared memory and are sampled for devices | Broad web-effect compatibility has a runtime and packaging cost. |
| Configuration | Profiles, presets, component chains, layouts, per-device settings and persistence | There is real configuration functionality behind the dashboard. |
| OpenRGB compatibility | An OpenRGB SDK **server** | It allows OpenRGB clients to control Lightloom. It is not a general adapter that obtains OpenRGB's hardware coverage. |

Sources: [workspace manifest](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/Cargo.toml), [frontend manifest](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/ui/package.json), [tray implementation](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/crates/lightloom-tray/src/win.rs), [API routes](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/crates/lightloom-api/src/server.rs), [OpenRGB server](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/crates/lightloom-api/src/openrgb/mod.rs).

### Hardware coverage: declarations are not validation

A local count of first-party `plugins/*/manifest.json` files found:

- **63 plugin-family manifests**. These include overlapping protocol variants, generic transports and network families; they are not 63 individually validated devices.
- **730 distinct declared USB VID/PID pairs**, normalised numerically across manifests.
- **7 USB VID/PID pairs marked tested**, across 5 plugin families.
- Additional SMBus hardware evidence exists outside those USB counts: the author reports testing Crucial Ballistix DDR4 modules and one MSI RTX 3060 model.
- **13 bundled starter HTML effects** and **50 component JSON files**.

Those counts describe the checked-out source, not the released prerelease or a guaranteed supported-device matrix. Several hardware documents contain newer results than the top-level README and plugin summary.

For the user's setup:

| Component | What the review establishes |
|---|---|
| Gigabyte B550 AORUS ELITE V2 | No first-party Gigabyte motherboard plugin was found. Imported third-party plugins are a separate path with separate compatibility and licence questions. |
| RTX 3060 Ti | Lightloom's documented tested NVIDIA RGB implementation is for an MSI RTX 3060 Gaming X Trio LHR with specific PCI/subsystem IDs. That does not establish compatibility with the user's 3060 Ti. |
| RGB RAM | There is a Crucial Ballistix DDR4 implementation and reported tests. The user's RAM brand/model remains unknown, so compatibility cannot be inferred. |
| Case fans/cooler lighting | Coverage depends on the motherboard/header or separate controller. A missing motherboard implementation can leave attached lighting unsupported even when generic fan/component definitions exist. |

Sources: [plugin library and validation policy](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/plugins/README.md), [Crucial hardware notes](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/docs/hardware/crucial-ballistix.md), [MSI GPU notes](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/docs/hardware/msi-gpu.md), [plugin tree](https://github.com/MrMoraleDev/Lightloom/tree/b8b482b6597fc03782ab4f804cede7dcf654c673/plugins).

### Maturity and documentation

GitHub exposes a `v0.1.0` prerelease published on 25 October 2026. The inspected README still describes the project as pre-alpha and says there is no release. Its roadmap also marks several areas planned that already have implementations and later hardware notes. The source therefore needs to be assessed independently of either the README's older limitations or broad feature claims.

This is an early project with substantial implementation. A large amount of code, extensive fixtures or a passing mock test does not establish broad physical-device validation.

Source: [prerelease](https://github.com/MrMoraleDev/Lightloom/releases/tag/v0.1.0), [README at reviewed commit](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/README.md).

## Useful architectural lessons

The reviewed device session code applies placement, brightness, component geometry and device controls outside the frontend. It executes plugin lifecycle functions with budgets and performs shutdown handling even after failures. Frame publication wakes device workers, while sampling uses precomputed pixel offsets. The frontend does not need to drive individual USB writes.

These are useful patterns for our application: keep device identity, configuration, rendering and output separate; share an explicit state model; keep the UI off the timing-critical output path; and use latest-frame delivery rather than allowing slow devices to accumulate a backlog.

The inspected Svelte components implement spatial dragging, rotation/mirroring controls, effect parameters and presets, channel/component selection and tested/untested badges. They are functional foundations. My product assessment is that there remains room for a more guided experience: first-run setup, identifying physical components, visually distinguishing shared versus independent channels, undo/redo, and actionable recovery states.

Sources: [device sessions](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/crates/lightloom-plugin-host/src/session.rs), [frame store](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/crates/lightloom-engine/src/frame_store.rs), [sampler](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/crates/lightloom-engine/src/sampler.rs), [layout UI](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/ui/src/components/Layout.svelte), [effects UI](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/ui/src/components/Effects.svelte), [devices UI](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/ui/src/components/Devices.svelte).

## Would our own hardware layer produce real gains?

Potential gains exist, but none is demonstrated for the user's current setup:

| Potential gain | What would need to be established | Needed for v1? |
|---|---|---|
| Smaller installation without OpenRGB | Our required adapters genuinely cover the target devices and replace its runtime/driver setup | No |
| Lower output latency | An end-to-end comparison identifies OpenRGB's transport or device scheduling as a material bottleneck | No |
| More controller functionality | The hardware supports a useful feature that the available OpenRGB interface does not expose | Only for a proven requirement |
| Better reconnect handling | The failure originates in the backend rather than client state, and a targeted fix/adapter resolves it | First improve our adapter and consider an upstream fix |
| New hardware support | We have the device, a lawful protocol basis and repeatable physical validation | Later, per device |

The OpenRGB SDK is a local TCP protocol. For scale, 300 LED colour entries at 4 bytes per entry and 60 frames/s amount to roughly **72 kB/s before packet overhead**. This example does not prove latency is negligible, but it shows why bandwidth alone is a weak justification for a complete hardware rewrite. Bus transactions, firmware limits, scheduling, rendering and capture can dominate.

Lightloom's own plugins frequently adapt OpenRGB controllers. That is a route to moving existing device knowledge into another runtime, not an automatic gain in protocol coverage or validation. Native code must still handle endpoint variations, firmware, privileged buses, device restoration, conflicts and shared-bus arbitration. A Rust rewrite does not by itself change those constraints.

Source: [OpenRGB SDK protocol and colour packet formats](https://github.com/CalcProgrammer1/OpenRGB/blob/master/Documentation/OpenRGBSDK.md), [Lightloom adaptation guidance](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/plugins/README.md).

### The more promising performance opportunity

Lightloom's published October reference benchmark reports a median renderer process-tree working set around **280 MB**, with **411 MB** on one software-composited effect. Its reported 1-4 ms canvas-to-first-write figures measure one part of the pipeline, not total latency to visible light. They are not an OpenRGB comparison and cannot establish that Lightloom is faster.

A smaller native effects engine for ordinary gradients, waves, breathing and colour transitions could avoid a separate Chromium renderer. That is an architectural hypothesis to measure, not a performance guarantee. Svelte/Tauri can provide the UI while Rust computes effects independently of the WebView. Advanced shaders, screen capture, audio and user-supplied effects can be evaluated separately; they should not force a heavyweight compatibility runtime into every basic scene.

Source: [author's effects benchmark and measurement definitions](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/docs/hardware/effects-m3.md), [renderer decision](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/docs/adr/0002-cef-offscreen-renderer.md).

## Building options

1. **Original application with an OpenRGB adapter — recommended.** Concentrates effort on the user's actual pain: configuration and usability. Start with the working OpenRGB 1.1 installation, capability-aware controls and normal-user client communication. Add our own effects and/or the existing Effects plugin integration only after verifying the relevant APIs.
2. **Fork Lightloom and add an OpenRGB backend.** Reuses a substantial engine and UI, but the current architecture is oriented around its own plugin/HAL stack. An OpenRGB server implementation is not the client adapter we need. This option carries GPL obligations and the complexity of inherited compatibility, broker and renderer systems. It is not automatically the shorter route for a deliberately smaller product.
3. **Independent native hardware platform.** A reasonable long-term objective if independence or missing hardware features become primary goals. It is the largest maintenance and validation commitment and does not currently solve an established problem for the user's already-working OpenRGB setup.

Design the backend interface around devices, capabilities, zones, frame output, hardware modes and lifecycle events. Keep backend-specific IDs/configuration out of the frontend's product model. A later native adapter should be opt-in, with clear device ownership and coordination for shared buses.

## Licensing and product boundaries

Lightloom is GPL-3.0-or-later with an add-on exception. Copying its implementation is different from implementing an independent application that uses similar ideas. Its starter effects are labelled CC0-1.0, while adapted controller files retain provenance. Any reused material needs its actual file-level terms recorded. An add-on exception does not remove GPL obligations from copied core code.

Do not adopt the repository's assertions about third-party compatibility or imported content as legal clearance. Technical format compatibility does not grant permission to redistribute effects, assets or proprietary plugins. For this project, use original branding, UI and effect implementations, and explicitly licensed dependencies/content. SignalRGB's terms restrict its software/content and prohibit source extraction/reverse engineering; those need to remain separate from functional inspiration.

Also, copyleft requires compliant source availability; it does not itself prohibit advertising or telemetry in a redistributed open-source fork.

Sources: [Lightloom licence exception](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/LICENSE-EXCEPTION.md), [licensing decision and provenance policy](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/docs/adr/0010-license-gpl-3.md), [SignalRGB terms](https://signalrgb.com/terms/).

## What the next prototype needs to answer

- Can the selected Rust SDK adapter enumerate and control the user's real OpenRGB 1.1 devices and recover after restart/suspend?
- Which controls can be exposed for the current Effects plugin, including creating/editing effects versus only selecting existing configurations?
- Can ordinary original effects run independently of the UI and stay within measured CPU/RAM budgets?
- Does the proposed component model correctly express motherboard headers, splitters, shared channels and independent controllers without promising hardware capabilities that do not exist?
- Can users complete first-time configuration and recovery more easily than in their current tools?

Those are the gates for a useful product. Native device drivers can remain a later decision supported by evidence.
