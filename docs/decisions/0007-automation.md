# 0007 — Automation: rules on system events

- Status: accepted
- Date: 2026-11-20
- Scope: the rules "When <event>, <action>", where they are stored, how the old "apply on connect" mark moved into them, how PrismForge receives sleep, wake and session end, every limit on the way, and what the interface promises

## Current state

The rule model, the store and its migration, lights off, the wake sequence, the read of OpenRGB's own event profiles and the Automation screen are built (Task 12). **None of it has run on Windows.** The evidence is Rust tests with a fake event source, the demo backend and the scripted fake server in simulated time, interface tests, and a browser preview with simulated data. The Windows event window was type-checked and linted for `x86_64-pc-windows-msvc` in a scratch crate on macOS; the desktop crate itself cannot be checked for Windows on macOS, because the Tauri build script needs the Windows resource compiler. What the owner must test on the Windows 11 PC is in `docs/hardware-validation.md`.

The code is in `crates/prismforge-core/src/automation` (rules, lights off, the wake sequence, OpenRGB's event profiles, runs), `crates/prismforge-core/src/store.rs` (rules and runs in the store), `crates/prismforge-openrgb/src/protocol/settings.rs` (packets 52, 53 and 250), `apps/desktop/src-tauri/src/system_events` (the seam, one module per platform), `apps/desktop/src-tauri/src/automation` (running the rules), and `apps/desktop/src/features/automation` (the screen).

## The rules

1. **One rule for each event.** The events are: PrismForge starts, PrismForge connects, the system goes to sleep or hibernates (one event, because Windows gives one signal for both), the system wakes, the user signs out or the system shuts down or restarts (session end), and PrismForge quits.
2. **Three actions.** Apply a profile, by its id; switch the lights off; do nothing. Every event offers all three, as the brief lists them; "When the system wakes, switch the lights off" keeps the lights dark after a wake. At start and connect, lights off runs through the same trigger as a profile.
3. **Every rule is off by default.** A rule acts only when the user switched it on, and only for its own event. A rule that is on with "do nothing" does nothing.
4. **A missing profile is reported, never guessed.** Deleting a profile keeps the rules that name it; the screen says the profile was deleted, and the rule applies nothing. A rule that is switched on must name a profile the store has; a rule that is off may keep a deleted one, so the user can always switch it off.
5. **Rules run only while PrismForge runs.** The screen says so above the rules.
6. **Not built, on purpose:** display-off and display-on triggers, schedules, per-device rules, and any promise before sign-in.

## Where the rules live

The rules are stored in each store's `settings.json`, the file that held the mark, as the field `automation`, so the real store and the demo store each have their own rules. The reason: a rule names a profile by id, and profile ids belong to one store. In `app-settings.json`, which is per user and not per store, a rule could name a demo profile while PrismForge talks to OpenRGB. A rule runs with the rules of the store of the session it acts on.

The settings file moved from schema 1.0 to 1.1, a minor version: it only adds a field. It keeps the store rules of `0002`: format marker, schema version, synced temporary file and rename, backup, recovery from the backup with a notice, damaged files renamed and never deleted, a newer major version left untouched and every rule change refused. Every level keeps the fields it does not know: an unknown event, an unknown action (kept verbatim and never run; the screen says a newer PrismForge set it) and unknown fields of a rule.

The last run of each rule is in `rule-runs.json` (`prismforge.rule-runs`, schema 1.0) beside it, with the same file rules. A damaged run file is set aside and the store opens without it. The run is kept in memory first, then written, so a failed write still shows the run until PrismForge ends.

## The migration of the mark

Before Task 12, the store's `settings.json` held `apply_on_connect`: the one profile marked "Apply this profile when PrismForge connects", the single source of `0004` step 4.

1. **Reading.** While `settings.json` has no `automation` field, the rules are derived from the mark: a marked profile becomes the connects rule, switched on, with that profile (the user switched it on before, so it stays the user's explicit choice). No mark gives a connects rule that is off. A mark of a missing profile stays and is reported. Loading writes nothing.
2. **The first rule change** writes `automation`. From then on the rules are the source, so the migration happens once.
3. **The mark stays for older versions.** Every write sets `apply_on_connect` to the profile the connects rule applies, or to null. An older PrismForge that opens the store keeps `automation` as an unknown field and writes it back, and it applies the same profile on connect as this version.
4. **An older version that changes the mark** is followed: when `apply_on_connect` differs from what the rules mean, the connects rule takes the mark over (switched on with that profile, or switched off). The other rules stay.
5. **Behaviour at start stays the same** for a user who had the mark set: the connect at start applies the same profile once, through the same trigger and the same apply lock. A test opens a store written by the earlier version and checks the apply.

The Profiles screen no longer has the switch or the menu item; it says whether the connects rule applies the profile and links to Automation. Settings step 4 names the rule, and the status strip and the connection screen name "the rule “When PrismForge connects”".

## Starts and connects

Either rule can apply a profile or switch the lights off. Settings step 4 and the start path name what the rule does at the connect at start: "applies <profile>" or "switches the lights off" (`StartProfile::LightsOff`).

The connects rule is the earlier apply on connect: it applies once each time the session becomes live after a user connect (`ConnectTrigger`), and not again after a loss that the session restores by itself. The starts rule applies at the connect at start (`Shell::start_at_launch`) **in place of** the connects rule, when it applies a profile; otherwise the connects rule applies there as before. Without "Connect to OpenRGB when PrismForge starts" the starts rule never runs, and the screen says so. Both record a run.

## Lights off

Lights off is an action of PrismForge, planned before the event from the last device list (`plan_lights_off`), so at the event only writes are left:

| Device | Write |
|---|---|
| Has Direct Mode | Direct Mode, then black on every LED. Direct Mode never saves to device memory. |
| No Direct Mode, a mode named "Off" whose persistence is not `AutomaticSave` | That mode, through the same reapply as a hardware mode. |
| No Direct Mode, and no "Off" mode or one that reports `AutomaticSave` | Skipped, with the reason. Frequent writes to device memory wear it (`docs/hardware-validation.md`, finding 2). The owner's keyboard is this case: it is never written. |
| Not resolved, not there, declared as controlled by another program, being identified | Skipped, with the reason. |

An effect on a device that goes dark ends without a restore; its remembered state stays, so a later effect stop still puts back what the device showed before PrismForge. A dark device has no PrismForge owner, so the release at quit or session end puts nothing back on it. Each device gets its own result, as for a batch in Task 9.

At quit and session end, a rule that applies a profile ends the profile's effects where they are (`keep_lights`: the due frames go out once, nothing is restored), so the devices keep the profile's last frame after PrismForge ends.

## The event seam

The crate `crates/prismforge-system-events` has one small trait, `EventSource`, and one module per platform; the rules live in the app and are tested with a fake source.

**Windows** (research `os-events-research.md` section 5, "The seam on Windows"):

1. A dedicated thread with its own hidden top-level window (`WS_POPUP`, `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`, never shown, not message-only) and its own message loop.
2. `RegisterSuspendResumeNotification(hwnd, DEVICE_NOTIFY_WINDOW_HANDLE)`, needed for the suspend message on Modern Standby. `GUID_SESSION_DISPLAY_STATUS` is not registered, because display-off triggers are out of scope.
3. `PBT_APMSUSPEND` runs the sleep rule and blocks the window thread for at most [`SLEEP_LIMIT`]. `PBT_APMRESUMEAUTOMATIC` posts the wake to the app and returns at once. `PBT_APMRESUMESUSPEND` is ignored.
4. **Deviation from the research:** the research proposes a separate, pre-opened blocking connection to OpenRGB for the sleep write. PrismForge writes through its session instead, and blocks the window thread on the async work with a hard limit. A second connection would write behind the session: running effects would overwrite the dark frame on the next tick, and the write would bypass the apply lock and the lock order. The session's runtime is multi-threaded and is not frozen when `PBT_APMSUSPEND` arrives. What is prepared before the event: the rules and the profiles are in memory, and the lights-off plan comes from the last device list; nothing is read from disk or from the devices at the event.
5. Session end: Tauri's final `Exit` event (tao raises it from `WM_ENDSESSION(TRUE)` and then ends the process), and a redundant `WM_ENDSESSION` handler on the event window. Both await one shared run: the first signal starts it, the second waits for the same run and gets its result.
6. `SetProcessShutdownParameters(0x300, 0)` at start, so Windows closes PrismForge before an OpenRGB at the default 0x280. No `SHUTDOWN_NORETRY`, no `ShutdownBlockReasonCreate`, no change to the system's timeouts.
7. Start with the system is unchanged (`0004`).

**The seam reports its start.** `EventSource::start` returns what it achieved: `Available`; `Limited` with the detail when the events arrive but a part failed (the shutdown level was refused, or the suspend registration failed, which loses sleep on Modern Standby PCs); `Failed` with the detail when the thread, the module handle, the window class or the window could not be made, or the window did not answer within 2 s; `NotOnThisSystem` on macOS and Linux. The Automation screen shows each state in words; with `Failed` the sleep, wake and session-end controls are off and no session-end rule runs. The message loop stops on an error (`GetMessageW` returning -1) instead of spinning.

The Win32 calls need `unsafe`. The seam is its own crate so that the rest of the workspace keeps `unsafe_code = "forbid"`. Only this crate sets the lint to `deny` in its `Cargo.toml`, and only its `src/windows.rs` allows it. The guard `crates/prismforge-system-events/tests/unsafe_code.rs` checks three things: the workspace forbids the lint; every other `Cargo.toml` takes the workspace lints unchanged; no other `.rs` file in the workspace names the lint, `build.rs` included.

**macOS and Linux:** a source that reports the events as not available. The screen says that sleep, wake and session-end rules work on Windows only for now and turns their controls off. Starts, connects and quits work everywhere.

## Limits

Every wait is a named constant with a test in simulated time.

| Constant | Value | Where |
|---|---|---|
| `SLEEP_LIMIT` | 1 s | The whole sleep rule: opening the store, the session slot, the apply slot, the write and the record; the window thread waits at most this long. Windows allows about 2 s. |
| `SESSION_END_LIMIT` | 1 s | The session-end rule. Then the stop of `0004` gets the rest of `LAST_STOP_LIMIT` (4 s in all), under the 5 s after which Windows ends a hidden app. |
| `QUIT_RULE_LIMIT` | 2 s | The quit rule, before the quit's own stop (`QUIT_STOP_LIMIT`, 10 s). |
| `RULE_LOCK_WAIT` | 250 ms | How long a rule waits for a profile apply of the user. |
| `SESSION_END_WAIT` | 1.25 s | How long the final `Exit` waits for the session-end run; longer than `SESSION_END_LIMIT`, so the wait never cancels the run. |
| `RECORD_LIMIT` | 200 ms | The end of each rule's limit kept for writing its run, so the work stops 200 ms earlier and the record still fits. |
| `WAKE_STEP_LIMIT` | 4 s | Each wake step that needs the session slot (read the state, connect, read the devices). |
| `WAKE_BACKOFF` | 0.5, 1, 2, 4 s, then 4 s | Between tries to reach OpenRGB after a wake. |
| `WAKE_RECONNECT_LIMIT` | 30 s | One deadline for reaching OpenRGB and for the device list to settle, counted from the wake. |
| `WAKE_SETTLE_INTERVAL` | 1 s | Between two reads of the device list. |
| `WAKE_SETTLE_LIMIT` | 15 s | The device list is used as it is, or earlier at the 30 s deadline. |
| `WAKE_APPLY_LIMIT` | 10 s | The wake rule's own work. |

A rule whose store or rules could not be read in time is not shown as run, because PrismForge does not know whether it is on. A store that cannot open at all is shown as "no rule could run". Every part of a rule runs under its one deadline, the read of the session slot included, so a disconnect or a connect that holds the slot cannot hold a rule past its limit. A rule that does not finish in time is recorded as stopped at its limit. Nothing is written after the event: a sleep rule checks before each write that no wake came and that its time (counted on the wall clock, which also runs while the PC sleeps) has not passed, and the session itself skips a lights-off command that reaches it after that time (`TooLate`).

## The wake sequence

Only with the wake rule switched on; without one, a wake writes nothing.

1. Reach OpenRGB: ask the session to connect, with the backoff above.
2. Read the device list until two reads a second apart agree, up to 15 s. A failed read never counts as agreement; a connection lost again goes back to step 1. Steps 1 and 2 share one deadline, 30 s after the wake, so "within about 30 seconds" holds.
3. Right before the rule, the session forgets the device state: the next tick sets Direct Mode and every colour again (`resync`), because OpenRGB may report Direct Mode for a device that came back in a firmware mode. Done earlier, the rewrite would reach devices that are not back yet.
4. Apply the wake rule once, within `WAKE_APPLY_LIMIT`. The devices that were there before the sleep and are not back are named in the result; the rule applies to the devices that are there. A wake without a sleep before it names no device as missing.

The sequence never holds a session handle while it waits, so a disconnect is never held up by it. A newer sleep, or a disconnect or a new connect by the user, cancels the wake rule. Not built: the short follow-up on `DBT_DEVNODES_CHANGED` after a wake (research 4b step 6); it is a later item.

## Locks and collisions

A rule takes the apply slot like a profile apply, after the session slot is released and before any store lock, so the lock order of `0002` holds (session slot, apply slot, output gate, on-connect report, store). It never waits on a lock without a bound: it waits at most `RULE_LOCK_WAIT` for the slot and the gate, inside the rule's own limit.

- **A rule meets a user's profile apply:** the rule does not run, and its run says "PrismForge was applying a profile for you".
- **A user command meets a running rule:** a rule that applies a profile refuses it with "PrismForge is applying a profile", as during any apply, and the view names the profile; the status strip says "applying … by a rule". A rule that switches the lights off refuses it with its own reason: "An automation rule is switching the lights off" (`RuleRunning`).
- **Two session-end signals:** the run is one shared future. The first signal starts it; the second waits for the same run and gets its result. `end_of_session` waits `SESSION_END_WAIT` (1.25 s), longer than the rule's own 1 s limit, so its wait never cancels the shared run. A signal that comes later gets the same run and starts nothing. So the stop at `Exit` never starts while the rule still writes.

`tests/command_locks.rs` drives `automation`, `set_rule` and `mark_runs_seen` like every command, and the sleep, wake and session-end events and the quit rule beside a disconnect, a switch to OpenRGB, the window close and a profile apply in every poll order. A guard test reads the variants of `SystemEvent` from the source and fails when one has no row.

## Two programs

When the server advertises the settings-manager flag, PrismForge reads OpenRGB's `ProfileManager` settings with packet 250. It learns the flag by sending empty client flags (packet 52, value 0); the server answers with its own flags (packet 53). PrismForge never sends the packets that change settings (251 to 254) and never sets the ProfileManager client flag.

Sources, OpenRGB `release_1.1` at `8e16de27858cf2be6af54a33ce003df74cb9fd3e`, read for this record: `NetworkProtocol.h:95` (`NET_SERVER_FLAG_SUPPORTS_SETTINGSMANAGER = ( 1 << 4 )`), `:116-117` (`NET_PACKET_ID_SET_CLIENT_FLAGS = 52`, `NET_PACKET_ID_SET_SERVER_FLAGS = 53`), `:167` (`NET_PACKET_ID_SETTINGSMANAGER_GET_SETTINGS = 250`); `NetworkServer.cpp:167` (the server sets the flag), `:1341-1344` (packet 52 is answered with `SendReply_ServerFlags`, then the usual acknowledge), `:3719-3745` (the reply is packet 53 with the 4-byte flags in the server's byte order, for clients at protocol 6 or later), `:1456` and `:2475` (the handler of packet 250). The same lines hold on `master` at `5bcfa9ae` for `NetworkProtocol.h`. PrismForge reads the flags as little-endian, which is the byte order of the x86 and ARM machines OpenRGB runs on. Packets 52 and 250 are sent once at each connect, also with no rule on, so the screen can say whether OpenRGB acts too. Whether a real server answers them as the source says is checked on the owner's PC (`docs/hardware-validation.md`).

| PrismForge event | OpenRGB settings |
|---|---|
| Sleep | `suspend_profile` |
| Wake | `resume_profile` |
| Session end and quit | `exit_profile`, `service_shutdown_profile` |
| Start | `open_profile`, `service_startup_profile` |
| Connect | none |

A setting acts when it is on and names a profile. The rule then says that OpenRGB also loads that profile, that two programs will act and that the last one to write wins. PrismForge reads at connect, when the Automation screen opens, and again before acting at wake. At sleep, session end and quit it uses the value it read last, because a read can take as long as the client timeout (2 s) and those events have less time. A server without the flag, and demo mode, show that PrismForge cannot check. A settings reply PrismForge cannot read is reported and keeps the connection.

## What PrismForge promises

Next to each rule, in the words of the screen:

| Event | Promise |
|---|---|
| Starts, connects | **Will**, while PrismForge runs. |
| Quits | **Will**, before it ends, within 2 s. |
| Sleep | **Tries to**, before the PC sleeps; Windows gives about 2 s and may skip the notice on low battery. |
| Wake | Acts once the devices are back, within about 30 s; the rule's own work then takes at most 10 s more. |
| Session end | **Tries to**; Windows closes programs one by one and can end PrismForge or OpenRGB first; devices that keep power may stay lit. |

A failed sleep or session-end run is recorded in `rule-runs.json` and shown at the next start. Failed means: a failure, a frame that was not kept, a lights-off that reached the session too late, or a lights-off where devices were planned and none went dark. A failed run that could not be recorded in time is still shown while PrismForge runs. The notice: the notices area shows each failed run the user has not seen yet, with Open Automation and Dismiss. Dismissing it, or opening Automation, marks it seen (`mark_runs_seen`, kept in the run file). A run the store could not write is still shown until PrismForge ends, and the failed write is a notice on Automation. When the store cannot be opened at all, the event publishes that no rule could run. If the system ends PrismForge before the run is written, nothing is shown; the screen cannot tell a run that never started from one that was cut off. A profile applied at quit or session end whose last frame could not be kept says so in its run.

## Not verified

- Anything on Windows: the event window, the suspend and resume messages on an S3 desktop and on Modern Standby, the session-end order between the event window and tao's window, the shutdown level, and whether the dark frame reaches the devices before the PC sleeps or shuts down.
- Whether OpenRGB applies the dark frame before it is frozen or ended. The colour write has no acknowledge, and PrismForge sends no barrier request after it.
- OpenRGB's answer to packets 52 and 250 on a real server; the format comes from the OpenRGB SDK document and source.
- The wake timing of the owner's USB and SMBus devices.
