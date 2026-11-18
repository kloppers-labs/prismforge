# PrismForge Implementation Plan

> **For agentic workers:** Use `superpowers:executing-plans` if available to implement this handover task by task. If that skill is unavailable, use your environment's equivalent staged implementation and verification workflow. Follow all higher-priority instructions and applicable repository rules. The intended implementer is Fable; do not assume another agent's tools or local paths exist in your environment.

**Goal:** Build PrismForge, a free, open-source desktop RGB control application with a modern, understandable configuration experience, using OpenRGB as its initial hardware backend.

**Architecture:** Svelte and TypeScript provide the interface inside Tauri. A Rust application core owns configuration, device identity, effects, output scheduling and the OpenRGB connection. Hardware access stays behind a replaceable adapter so native integrations can be considered later when evidence justifies them.

**Tech stack:** Svelte, TypeScript, Vite, Tauri 2, Rust, Tokio where appropriate, OpenRGB SDK. Verify maintained compatible dependency versions when starting, then pin them and commit lockfiles. SvelteKit and a Node server are unnecessary for this desktop application.

**Spec:** The product brief, requirements and acceptance criteria in this document are the starting specification. The companion `rgb-hub-feasibility-review.md`, if attached, contains a deeper source review; this handover is self-contained.

**Intended repository:** `kloppers-labs/prismforge` on GitHub. This is the intended location, not a claim that the repository has already been created. The name has been selected by the owner; trademark/package/domain availability has not been established.

---

## 1. Your assignment

You are taking over implementation of PrismForge for the owner of Kloppers Labs. Build useful working software, with real integration and measured behaviour, rather than stopping at a plan or a polished simulated dashboard.

Begin by inspecting the supplied workspace and repository instructions. If a repository already exists, preserve its conventions and user changes. If none exists, prepare a local `prismforge` project and report the intended GitHub location; do not silently choose remote visibility or publish unrelated material. Use the owner's existing GitHub authorization and confirmed visibility when remote creation is appropriate.

Use this document as the starting direction. Settle routine implementation details using engineering judgment and record consequential decisions. Ask the owner only when missing information materially affects product behaviour, hardware access, licensing or external publication. Continue independent work while waiting for hardware details. Follow any required design/approval workflow imposed by your own environment; this handover does not override it.

Do not try to build the entire long-term vision in one pass. First prove the OpenRGB boundary, then deliver a coherent vertical slice and expand it in reviewable increments.

## 2. Product context

The owner runs a Windows 11 gaming PC with AtlasOS and values a lean system. OpenRGB already controls the installed RGB hardware, and the OpenRGB Effects plugin works well. The frustration is configuration and navigation: the interface feels dated, the relationship between devices/controllers/zones can be difficult to understand, and centralised control should be more visual and approachable.

SignalRGB is a useful functional benchmark for layout, configuration, effects and polish. PrismForge must have its own visual identity, implementation and properly licensed content. The owner does not want a subscription or a paid feature tier.

Known reference PC:

- Ryzen 7 5700X.
- Gigabyte B550 AORUS ELITE V2 motherboard.
- DeepCool AK620 Digital Pro cooler; the top plate has ARGB lighting, while its digital status display is a separate function.
- RTX 3060 Ti; board partner and exact model are unknown.
- 16 GB DDR4-3200 CL16 as 2 × 8 GB; RAM brand/model are unknown.
- 750 W Gold PSU.
- 1440p, 32-inch, 165 Hz display.
- OpenRGB upgraded from 0.9 to 1.1; Effects plugin in use, but its exact installed build has not been confirmed.
- RGB targets are RAM, case fans, GPU, and the cooler's illuminated plate. The cooler fans themselves are not RGB targets.

Games include RDR2, Rainbow Six Siege, PUBG, Battlefield 6 and Rust. Do not require disabling Secure Boot, TPM, anti-cheat protections or Windows security features to make PrismForge work.

### Information still missing

- Exact RAM and GPU models.
- Case fan models/counts and LED counts.
- Whether lighting uses motherboard headers, a passive splitter, an addressable daisy chain or a separate USB controller/hub.
- The OpenRGB device/zone inventory and Direct Mode capabilities.
- Windows access for actual hardware verification.

Do not invent those details. They are needed for physical validation, not for starting mock-backed development and protocol integration.

## 3. Product requirements and boundaries

### Required direction

1. Free and open source, with no subscription, account requirement, ads or telemetry by default.
2. Windows 11 first. Keep core logic portable; do not promise Linux/macOS hardware support until tested.
3. OpenRGB provides initial hardware detection and communication.
4. PrismForge owns its interface, configuration model, scene/effect definitions, validation and user experience.
5. Original branding and UI. Functional inspiration is allowed as a requirement; copied proprietary code, assets or effects are not part of the assignment.
6. Demonstrate actual OpenRGB SDK behaviour. A visible demo mode is useful, but must never masquerade as physical-device success.
7. Keep configuration local and support portable export/import.
8. Measure resource use. Do not claim that Rust/Tauri automatically makes the application lightweight or improves gaming FPS.
9. Preserve a clean path to optional native device adapters, without implementing a speculative driver platform now.

### Outside the initial release

- A complete independent USB/HID/SMBus/GPU-I2C device library.
- Custom kernel drivers, automatic driver replacement, automatic firmware flashing or broad bus probing.
- Feature parity with every SignalRGB feature, effect or integration.
- SignalRGB account, marketplace, downloaded-effect or installed-plugin import.
- Copying Artemis or Lightloom core code without an explicit compatible licensing decision.
- Fan-speed control, cooling automation, OCR, game integrations, cloud sync, an effects marketplace, MCP control, and arbitrary executable user plugins.
- Automatic changes to security/anti-cheat settings.
- The AK620's digital temperature/status screen, unless independently supported and explicitly added later. Its ARGB lighting remains a normal target.

## 4. Research conclusions to preserve

### OpenRGB is the best initial backbone

OpenRGB intentionally exposes a versioned TCP SDK for third-party control, normally on port 6742. It can expose devices, zones, modes and colour output. A third-party client is an intended integration, not a need to fork its Qt UI.

Research baseline: OpenRGB 1.1, SDK protocol 6 and plugin API 5. These are different version dimensions. Verify the running server's negotiated protocol and the actual installed plugin compatibility rather than assuming them from an application version string.

The Rust `openrgb2` client is a candidate, not a mandated complete solution. At the research date its documentation described protocol 6 support alongside high-level gaps for profile/settings functionality and server notifications. Verify the current release, licence and APIs. Wrap it behind our own adapter and extend or replace only the missing pieces we actually need. Do not build application logic around its internal types.

### Effects integration is an early validation gate

Do not assume an OpenRGB connection grants a full remote editor for the Effects plugin. Its original SDK integration supported listing and starting/stopping configured effects. Establish what the installed versions expose for creation, parameters, targeting, presets and persistence before designing screens that depend on those operations.

Existing OpenRGB Effects support and a new PrismForge effect engine are separate features. They can coexist in the product, but must not write to the same device/zone simultaneously.

### Lightloom is prior art, not an automatically suitable foundation

The reviewed Lightloom commit was `b8b482b6597fc03782ab4f804cede7dcf654c673`, inspected on 18 November 2026. It contains a Rust daemon, a Svelte dashboard, a JavaScript device-plugin runtime and a separate Chromium/CEF effect renderer. The current tray implementation uses native platform code and opens a browser dashboard; an architecture document's Tauri reference is not the current tray implementation.

The source had 63 plugin-family manifests and 730 declared USB VID/PID pairs, but only 7 USB pairs marked tested. There were additional reported physical tests for specific SMBus RAM/GPU hardware. Counts of declared IDs are not validated device support. No first-party Gigabyte motherboard plugin was found, and documented NVIDIA tests were specific to an MSI RTX 3060 model, not every RTX GPU.

Its OpenRGB compatibility is an SDK **server**, not a general client adapter that inherits OpenRGB device support. Many of its first-party controllers adapt OpenRGB code, so a native layer still inherits protocol and testing work.

Its published reference benchmark reported roughly 280 MB median renderer process-tree working set, with a 411 MB peak for one effect. Those are the author's measurements, not independently reproduced results or a comparison proving superiority to OpenRGB. There is a plausible opportunity to avoid a separate Chromium effect renderer for ordinary effects; measure it.

The recommendation is therefore an original application with OpenRGB underneath, not a broad Lightloom fork or a hardware-layer rewrite.

## 5. Licensing, provenance and independent implementation

Apply these rules from the beginning:

- Consult current dependency and file-level licences; record versions/commits, source links, notices and reuse decisions.
- OpenRGB is GPL-licensed. Its root licence text and individual file headers must be considered before copying/linking/adapting code or distributing a modified/bundled backend. Do not assume every source file has identical terms.
- Lightloom uses GPL-3.0-or-later with an add-on exception. That exception does not make copied core code permissively licensed. Its starter effects are labelled CC0-1.0; verify each asset before reuse.
- Artemis's current reviewed licence was PolyForm Noncommercial. Treat it as conceptual prior art and avoid copying implementation into this project unless rights and compatibility are explicitly resolved.
- Publicly visible SignalRGB plugin code is not automatically licensed for reuse. Do not copy, translate, decompile or extract proprietary source, effect libraries, illustrations, device artwork, icons, thumbnails, branding or marketplace content.
- An API/file format being technically compatible does not establish permission to use or redistribute someone else's content. Do not make imported SignalRGB content a product dependency.
- Build original effects from general lighting behaviours, use public protocol documentation, and use explicitly compatible licensed material where appropriate.
- Maintain `THIRD_PARTY_NOTICES.md` and `docs/provenance.md`. Do not commit private invoice details, local file paths, account data, identifiers or hardware logs containing personal information.
- Choose the application licence after a documented dependency/reuse audit. GPL-3.0-or-later is a proposed default for the application if the owner wants copyleft and compatible dependencies; it is not a decision that authorises incompatible code reuse.

Functional inspiration from SignalRGB does not provide blanket legal clearance for copied expression, third-party assets, patents or contractual restrictions. Keep the design and provenance independently defensible. Do not rely on another project's clean-room claims as legal advice.

## 6. Recommended architecture

### Application boundaries

**Frontend:** Svelte + TypeScript + Vite. UI state, forms and visual previews live here. No hardware protocol packets, privileged operations, per-LED native calls or effect timing loops in components.

**Tauri shell:** typed commands/events, tray, single-instance behaviour, window lifecycle and optional autostart. Keep capabilities narrow and use bundled local UI. The application must not need to run its WebView as Administrator merely to connect to OpenRGB.

**Rust application core:** stable device references, capability-aware operations, scene state, effect definitions, output scheduling, persistence and diagnostics. Remains active as required when the window is hidden; never depend on browser animation timers for physical lighting.

**OpenRGB adapter:** connection, protocol negotiation, enumeration, identity reconciliation, capability translation, native modes and batched colour output. OpenRGB keeps ownership of physical hardware communication.

**Mock adapter/transport:** deterministic simulations of device lists, capabilities, timeouts, reconnects and output recording. Explicitly labelled as simulated in the UI.

A standalone PrismForge daemon is not automatically required for the MVP. Start with a Tauri Rust core that can remain resident in the tray; introduce a separate process only for a demonstrated lifecycle/isolation need. Similarly, do not add a localhost HTTP API merely because Lightloom has one.

### Backend contract

Define PrismForge-owned types before implementing frontend features:

- `DeviceRef`: persistent application reference plus reconciled current backend identity.
- `DeviceDescriptor`: manufacturer/model/name, identifying fields, available zones and backend origin.
- `DeviceCapabilities`: Direct Mode, zone sizing, hardware modes, colour/brightness options and supported persistence operations.
- `ZoneDescriptor`: addressability, bounds, geometry and verified/configured LED count.
- `ConnectionState`: disconnected, connecting, connected, retrying, degraded and actionable reason.
- `OutputOwner`: none, PrismForge software effect, hardware mode, or external/plugin-managed output.
- `Scene`, `EffectDefinition`, `EffectInstance`, `Layout`, `ComponentInstance` and versioned profile/export types.
- `BackendError`: structured reason and diagnostic context, separated from user-facing copy.

The adapter must support connection lifecycle, inventory refresh, capability queries, colour/frame output and supported hardware-mode changes. Provide profile/plugin operations only after the spike establishes their semantics. Design the async interface ergonomically; do not force an unimplementable trait-object shape just to match a sketch.

### Identity and ownership rules

- Protocol 6 controller IDs are not persistent hardware identities across every backend restart. Never persist only the current numeric SDK ID or list index.
- Reconcile using available serial/vendor/model/location/controller information. Keep ambiguous matches unresolved until identified rather than lighting the wrong device.
- One output owner per device, or per zone only where the backend truly supports independent ownership. Switching a whole-device mode can affect every zone.
- If we cannot reliably stop/query a competing Effects-plugin writer, explain the required handover. Do not silently run two writers or kill another process.
- Distinguish application profiles from writing settings to device memory. No automatic EEPROM/flash saves, blind resizing or guessed LED counts.
- Future native adapters require explicit ownership and shared-bus coordination. They must not race OpenRGB detection/output on the same hardware.

### Effects and output

Start with a small original built-in collection: solid colour, breathing, gradient, rainbow wave, colour cycle and moving highlight. Make palettes, speed, direction, brightness and supported targeting discoverable through typed control definitions. Add capabilities only when they work end to end.

Use one core scheduler with bounded work and latest-frame coalescing. Start with a configurable 30 FPS target; offer higher rates only where appropriate. Avoid one Tauri invocation or TCP command per LED. Sample scene/layout output in Rust and send controller/zone buffers in batches.

Static scenes should not run unnecessary high-rate animation. Pause hidden previews independently of physical effects. Bound retry/backoff, queues and log growth; one unavailable device must not create unbounded output work. Record limitations of synchronous SDK behaviour rather than pretending slow backend operations cannot affect other devices.

Do not require CEF, Electron or a second Chromium distribution for these built-in effects. Audio, screen capture, shaders, WASM and arbitrary user scripts are later design decisions with separate resource and isolation requirements.

## 7. UX that makes PrismForge worth building

The first release must improve understanding, not merely restyle OpenRGB's controls.

### Main workflows

**First run:** choose/connect to the local OpenRGB server; show a clear connection state; explain missing prerequisites without automatically downloading or modifying drivers. Offer a labelled demo mode if hardware is unavailable.

**Identify and organise:** discover devices, give them friendly names, identify the selected physical target with an explicit temporary lighting action, and group them meaningfully. Restore prior output best-effort and report when the backend cannot provide enough state to do so.

**Configure components:** show controller → header/channel → component relationships where known or manually configured. Use fan/strip geometry and LED count definitions. Clearly distinguish mirrored splitter outputs from independent channels. Never infer independent control simply because several fans are visible.

**Create a look:** select an effect, change its palette/speed/target, see a lightweight preview and apply it to verified compatible targets. Make software effects and hardware modes distinguishable.

**Arrange the setup:** a 2D layout with moving, rotating, mirroring, grouping and clear LED direction. Undo/redo for configuration edits where practical; do not imply physical-output changes are globally transactional when they are not.

**Save and recover:** save named scenes/profiles, export/import a versioned file, recover after app/backend restart, and preserve configuration for temporarily missing hardware.

### Presentation requirements

- Original PrismForge identity, polished dark theme, clear hierarchy and restrained motion.
- Useful empty, loading, disconnected, unsupported and partial-failure states.
- Keyboard-accessible controls, visible focus, readable labels and colour controls usable without relying solely on colour perception.
- Avoid implementation jargon in normal workflows. Detailed IDs, protocol versions and logs belong in an advanced diagnostics view.
- No paid/pro labels, login screens, advertisements or placeholder marketplace features.
- No fabricated device capabilities, success notifications or performance counters.

## 8. Proposed repository structure

Adapt to an existing repository if present; for a greenfield project, use this starting shape:

```text
apps/desktop/
  src/
    lib/api/                 typed Tauri client and DTO boundary
    lib/state/               application view state
    features/connection/     first-run and reconnect experience
    features/devices/        inventory, identification, capabilities
    features/components/     headers, channels, component setup
    features/effects/        effect picker and parameter controls
    features/layout/         spatial editor
    features/profiles/       save/export/import
    features/diagnostics/    advanced status and logs
  src-tauri/                 desktop wiring, commands, events, lifecycle
crates/
  prismforge-core/           domain types, scenes, identity, ownership, profiles
  prismforge-openrgb/        SDK adapter and protocol/client boundary
  prismforge-effects/        deterministic built-in effects and sampling
  prismforge-testkit/        mock backend/transport and fixtures
docs/
  product.md
  architecture.md
  decisions/
  provenance.md
  hardware-validation.md
  performance.md
  handover.md
AGENTS.md
CLAUDE.md                    if used by the owner's workflow
.github/copilot-instructions.md
THIRD_PARTY_NOTICES.md
README.md
LICENSE
```

Keep files focused by responsibility. Do not prebuild a plugin marketplace, dependency-injection framework or dozens of empty packages. Shared DTO generation/validation should prevent drift between Rust and TypeScript; select one practical mechanism after evaluating its dependency/licensing cost.

Configuration is small initially: use versioned local files with atomic replacement, validation and a recoverable backup. Add a database only if actual persistence/query requirements justify it. Do not introduce Azure, Neon, Supabase, Resend, Descope or another cloud service merely because the owner uses them elsewhere.

## 9. Review focus

These failure modes must be exercised by meaningful checks in the tasks that own them:

1. OpenRGB restarts or changes device ordering/IDs: PrismForge reconnects without assigning a scene to the wrong hardware.
2. A controller lacks Direct Mode, reports invalid zone sizes or returns incomplete data: unsupported controls stay disabled and invalid output is refused.
3. The Effects plugin and PrismForge compete for output: ownership is explicit and the app does not silently alternate writers.
4. A device disconnects, a command times out, or the backend stops responding: UI remains informative, state is retained and queues/retries remain bounded.
5. A profile is malformed, newer than supported, or partially incompatible with today's inventory: no silent configuration loss or unsafe guessed remapping.

## 10. Staged implementation tasks

These tasks define milestones and contracts. During initial repository inspection, refine their file paths/signatures into a precise local execution plan without losing the acceptance conditions. Add failing behavioural tests before implementing the substantive Rust/backend logic; do not write tests that merely duplicate styling or implementation details.

### Task 1 — Prove the OpenRGB integration boundary

**Files:** create `crates/prismforge-openrgb/`, core device/capability/identity types in `crates/prismforge-core/`, and transport fixtures in `crates/prismforge-testkit/`. Record findings in `docs/decisions/0001-openrgb-integration.md`.

**Consumes:** local connection settings and a transport/client abstraction.

**Produces:** a usable backend contract, capability-aware inventory, documented SDK compatibility and a clear Effects-plugin capability matrix.

- [ ] Inspect repository instructions and available MCP/CLI tools. Use Serena for code traversal if exposed; prefer codebase-memory graph discovery and index a new repository before code discovery. If a tool is unavailable, document the limitation and use an appropriate fallback.
- [ ] Evaluate the maintained Rust client, licence and protocol support against the target OpenRGB version. Use official docs and pinned source, not model memory.
- [ ] Write tests named `negotiates_supported_protocol`, `rejects_malformed_controller_data`, `reconciles_changed_runtime_ids`, and `reports_ambiguous_identity`. Assert supported negotiation, safe parsing and that reordered/reused IDs never cause a frame to reach the wrong fixture device.
- [ ] Run the new tests and verify that they fail for the missing behaviour, then implement the minimal adapter and identity logic and rerun them.
- [ ] Connect to a real OpenRGB server when available: enumerate devices/zones/modes, perform an explicitly selected colour change and record the result. Hardware writes are opt-in, not an automatic discovery side effect.
- [ ] Investigate the installed Effects plugin: distinguish unsupported, unverified and implemented operations for listing, start/stop, parameters, creation, targeting and persistence.
- [ ] Commit the working spike and findings. If physical hardware is unavailable, keep that validation explicitly pending and continue with the mock-backed app.

**Acceptance:** the adapter works against recorded/mock protocol behaviours, and its real-server evidence or remaining limitation is documented. No frontend feature is promised on the basis of an unverified plugin API.

### Task 2 — Ship an honest desktop connection/inventory slice

**Files:** `apps/desktop/`, its Tauri commands/events/capabilities, `features/connection/`, `features/devices/`, and corresponding frontend integration tests.

**Consumes:** Task 1's inventory and connection contract.

**Produces:** an installed/runnable desktop app showing live connection status and genuine device capability data, plus an unmistakable demo mode.

- [ ] Scaffold the minimal Svelte/TypeScript/Vite/Tauri workspace with pinned dependencies and lockfiles; use narrow capabilities.
- [ ] Implement typed commands/events and a coherent connection screen, inventory view and diagnostics drawer.
- [ ] Verify `shows_connection_failure_without_fake_devices`, `labels_demo_mode`, and `disables_unsupported_controls` through frontend integration tests against explicit fixtures.
- [ ] Implement temporary identification only with an explicit user action and documented restore limitations.
- [ ] Verify backend restart and slow-command behaviour without blocking the UI thread.
- [ ] Build on Windows through the available runner. A browser-only frontend check must not be reported as a successful native Windows app test.
- [ ] Commit a working vertical slice with screenshots and actual run instructions.

**Acceptance:** the user can connect, understand which devices were found, identify a selected target and see truthful failure states. Fixture operation and real hardware operation are clearly separated.

### Task 3 — Add durable profiles, targeting and ownership

**Files:** profile/scene/ownership modules in `prismforge-core`, `features/profiles/`, and `docs/decisions/0002-profile-and-identity.md`.

**Consumes:** stable device references and verified capabilities.

**Produces:** schema-versioned profiles and explicit output ownership that survive reconnects.

- [ ] Test `roundtrips_profile_v1`, `rejects_future_profile_version`, `preserves_missing_device_settings`, `keeps_valid_file_after_failed_write`, and `refuses_conflicting_output_owner` before implementing those behaviours.
- [ ] Implement atomic save, validation, backup recovery and export/import with no private machine data by default.
- [ ] Separate application-save operations from physical device-memory saves. Leave physical saves out until their actual support and semantics are verified.
- [ ] Expose external/plugin-owned output honestly; do not auto-stop a plugin if the relevant operation is unavailable or unverified.
- [ ] Commit with documented schema and user recovery instructions.

**Acceptance:** profiles are durable, imported data cannot silently target the wrong hardware, and the app never pretends it owns a controller that another writer still controls.

### Task 4 — Implement original effects and efficient frame output

**Files:** `crates/prismforge-effects/`, its deterministic test fixtures, core scheduler/output modules, and `features/effects/`.

**Consumes:** `EffectDefinition`/`EffectInstance`, layouts/target selections, capability-aware frame output and ownership.

**Produces:** the initial original effect collection, parameter controls, presets and an output loop independent of frontend rendering.

- [ ] Use an injectable clock and seed for effect tests. Test palette endpoints, direction, brightness bounds, empty targets and deterministic frames at fixed timestamps.
- [ ] Test `coalesces_frames_when_output_is_slow`, `bounds_retry_work`, and `does_not_reanimate_static_scene`; verify recorded output count/queue bounds rather than mirroring implementation details.
- [ ] Implement batched controller/zone output with an initial configurable 30 FPS rate and truthful handling of backend/device limits.
- [ ] Add lightweight previews that can stop when hidden without stopping physical lighting.
- [ ] Where Task 1 verified existing Effects-plugin operations, add them as a distinct provider with explicit ownership. Do not present configured plugin instances as a complete editable library unless that capability actually exists.
- [ ] Measure static and animated scenarios and commit baseline results.

**Acceptance:** the built-in effects run on verified compatible targets, produce correct deterministic frames, and remain active with the main window hidden as designed. No second Chromium runtime is required for basic effects.

### Task 5 — Deliver component setup and spatial configuration

**Files:** `features/components/`, `features/layout/`, core layout/component schemas and geometry tests.

**Consumes:** verified zone geometry/capabilities, explicitly configured relationships and the scene/effect model.

**Produces:** a usable configuration flow for the owner's fans, RAM, GPU and cooler lighting without inventing topology.

- [ ] Provide controller/channel/component configuration with editable friendly names and validated LED counts.
- [ ] Test geometry transforms, mirroring and LED ordering in core logic. Test `represents_mirrored_outputs_as_shared_control` and `rejects_led_count_outside_backend_bounds` where bounds are provided.
- [ ] Implement move/rotate/mirror, targeting and understandable direction previews; persist layout state.
- [ ] Add keyboard access and undo/redo for appropriate configuration edits. Keep diagnostic metadata behind advanced views.
- [ ] Verify at least one complete setup journey from detected controller to saved/applicable scene using fixtures and, when available, the owner's hardware.
- [ ] Commit with screenshots and a short explanation of current topology limitations.

**Acceptance:** the user can understand and configure the real control boundaries. A passive splitter never appears as independently controllable physical ports merely because the UI draws multiple fans.

### Task 6 — Harden lifecycle, package and validate the target machine

**Files:** Tauri lifecycle/tray/autostart modules, GitHub Actions, `README.md`, `docs/hardware-validation.md`, `docs/performance.md`, notices/provenance and release packaging.

**Consumes:** the previous working slice, recorded limitations and local configurations.

**Produces:** a reproducible Windows build and an honest validation record.

- [ ] Test shutdown/reconnect/retry state transitions and preserve profiles after abnormal backend termination.
- [ ] Verify window close versus application quit semantics, tray behaviour and optional startup. Do not leave orphan processes or prevent the user from exiting.
- [ ] Run formatting, type checks, behavioural tests and a Windows native build. Add CI that runs these checks from a clean checkout.
- [ ] Perform the manual matrix below on the target machine when access is available; record unrun items as pending.
- [ ] Review licence/provenance, local-only behaviour, capability scope and configuration export before preparing a distributable.
- [ ] Produce a local Windows installer/artifact. Do not buy signing services, publish a release or alter repository visibility without existing authorization for that action. Explain unsigned-build warnings without instructing the user to disable system protections.
- [ ] Commit the release-ready slice and remaining issues. Do not claim full device coverage or a completed hardware test based on CI mocks.

**Acceptance:** another person can follow the README, build PrismForge, connect to OpenRGB and understand which features have actually been verified.

## 11. Validation matrix

Keep these records separate:

| Scenario | Required evidence |
|---|---|
| Core logic | Deterministic scene, identity, geometry, profile and ownership tests |
| Protocol boundary | Negotiation, parser limits, disconnects, reordered devices, timeouts and simulated slow output |
| Frontend | Connection/error/demo states, capability-disabled controls, configuration and profile journeys |
| Windows app | Actual Tauri/WebView build and launch; tray and shutdown behaviour |
| Physical hardware | Selected device/zone changed as intended; no wrong-target writes; restore/exit behaviour recorded |
| Backend restart | Reconnect, identity reconciliation, scene policy and no stale-ID output |
| Sleep/resume | Verified on the gaming PC, or clearly pending |
| Resource use | Static scene, 30 FPS built-in effect, visible preview, hidden UI and current OpenRGB-only baseline |

Measure the full relevant process set and distinguish PrismForge's incremental cost from OpenRGB's baseline. Report CPU, working set/private memory where available, frame cadence, dropped/coalesced output and reconnect observations. Set an explicit budget after the first baseline; do not use invented benchmark numbers or promise an FPS increase.

The owner's configuration may require application-specific exclusions or backend limitations. Diagnose those concretely; do not broadly disable drivers/services, RGB applications or security features as a first step.

## 12. Repository and collaboration practices

- Inspect `AGENTS.md`, `CLAUDE.md`, Codex instructions and Copilot instructions before reasoning about an existing codebase. Keep applicable project guidance current as features progress; avoid contradictory duplicated instructions.
- Use Serena where available. Prefer codebase-memory MCP graph discovery over broad text search, and index a new codebase first. Use direct search for config/string literals/non-code files or when graph results are insufficient. Do not pretend an unavailable MCP server was used.
- Use connected GitHub tools or authenticated CLI as appropriate for `kloppers-labs`. Reuse a suitable checkout; create isolated work only where appropriate. Preserve uncommitted user changes.
- Small coherent commits and PRs with behaviour, rationale, evidence and remaining limits. No force-push, destructive cleanup, merge or public release without authorization for that action.
- Maintain a compact `docs/handover.md` with decisions, implemented capabilities, exact verification commands/results and next work. Keep unfinished hardware validation visible.
- Do not log secrets, whole private configurations or unrelated connected-account data.

## 13. Expected reports from you

First report after inspection/spike:

1. What exists in the repository and what you have established about OpenRGB/Effects APIs.
2. The refined architecture and dependency/licence choices, including any material deviation from this proposal.
3. A short task plan and the first runnable vertical slice you are implementing.
4. Missing information that actually blocks physical validation; continue non-dependent work.

At each meaningful milestone, report what now works, how it was checked, and remaining material limitations. At handover, provide build/run instructions, screenshots, test/build results, performance measurements where actually obtained, and the target-machine validation status.

Success means a pleasant, truthful, working RGB control application. A pretty mock, a large unverified device list, an invented effect library or an undocumented native driver rewrite is not success.

## 14. Primary references

Research snapshot: 18 November 2026. Recheck current versions and terms when implementing.

- [OpenRGB SDK overview](https://openrgb.org/sdk.html)
- [OpenRGB wire protocol](https://github.com/CalcProgrammer1/OpenRGB/blob/master/Documentation/OpenRGBSDK.md)
- [OpenRGB releases/version dimensions](https://openrgb.org/releases.html)
- [OpenRGB Effects plugin](https://openrgb.org/plugin_effects.html)
- [Original Effects SDK integration](https://gitlab.com/OpenRGBDevelopers/OpenRGBEffectsPlugin/-/merge_requests/272)
- [Rust client and documented limitations](https://docs.rs/crate/openrgb2/latest)
- [Tauri frontend guidance](https://v2.tauri.app/start/frontend/)
- [Tauri capabilities](https://v2.tauri.app/security/capabilities/)
- [Lightloom reviewed source](https://github.com/MrMoraleDev/Lightloom/tree/b8b482b6597fc03782ab4f804cede7dcf654c673)
- [Lightloom architecture](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/docs/architecture.md)
- [Lightloom hardware/plugin evidence](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/plugins/README.md)
- [Lightloom renderer measurements](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/docs/hardware/effects-m3.md)
- [Lightloom licensing exception](https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/LICENSE-EXCEPTION.md)
- [Artemis current licence reference](https://github.com/Artemis-RGB/Artemis/blob/master/LICENSE)
- [SignalRGB terms](https://signalrgb.com/terms/)

Proceed with repository inspection and Task 1. Keep the scope focused, document what you learn, and build the first real PrismForge slice.
