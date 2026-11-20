# 0002 — Profiles, device identity on disk and the local store

- Status: accepted
- Date: 2026-11-18
- Scope: how PrismForge saves profiles and known devices, how it links them to hardware again, and how it recovers damaged files

## Current state

The Rust core has a durable local store, profile schema version 1, capture, apply, export and import (Task 3a). Task 10 moved profiles to schema version 2 and added the setup file. The desktop app uses it through fifteen profile commands and a Profiles screen (Task 3b). **Nothing here has run against a real OpenRGB server or real hardware.** The evidence comes from tests on real files in temporary folders, the demo backend, the scripted fake server, and a hand check of the built macOS app in demo mode.

The code is in `crates/prismforge-core/src/profile` (the model, capture, matching and apply) and `crates/prismforge-core/src/store` (files, recovery, registry, export and import, and `Profiles` for async callers).

## Application save is not device save

1. **The store writes files on this computer only.** It never writes to device memory. No type, command or error in the store uses the word "save mode".
2. **Loading writes nothing to hardware.** Opening the store, importing a file and reading a profile send nothing to OpenRGB.
3. **Applying a profile is an explicit command.** It uses the same session commands as the effects screen.
4. **Apply on connect is off by default.** The user marks at most one profile. Only that profile is applied, once each time the session becomes live after a disconnect.
5. **Nothing goes over the network.** The store has no network code.

## Files

The caller gives the store one folder. The desktop app uses two folders below its per-user configuration folder, as the next section says.

| File | Holds |
|---|---|
| `devices.json` | The known-device registry: each `DeviceRef` with its identifying fields, and the user's names for the device and its zones |
| `settings.json` | The profile to apply on connect, or none |
| `groups.json` | The user's groups of targets (Task 9) |
| `setup.json` | The setup: components, outputs and views (Task 10, `0006-setup-model.md`) |
| `profiles/<id>.json` | One profile |

Each file has a backup next to it with `.bak` added to the name. A write uses a temporary file with `.tmp` added to the name.

**One file per profile.** A damaged profile cannot take the others down. Each write touches one small file. Delete is the removal of one file and its backup.

## The desktop app

### Where the store lives

Tauri resolves the per-user configuration folder for the identifier `io.github.kloppers-labs.prismforge`:

| System | Folder |
|---|---|
| Windows | `%APPDATA%\io.github.kloppers-labs.prismforge` |
| macOS | `~/Library/Application Support/io.github.kloppers-labs.prismforge` |
| Linux | `~/.config/io.github.kloppers-labs.prismforge` |

| Below it | Holds |
|---|---|
| `store` | The profiles, devices and settings for real hardware |
| `demo-store` | The same files for demo mode |
| `instance.lock` | The lock that keeps a second PrismForge out |

Diagnostics shows the folder in use. A store opens on first use: when a session starts or when the user opens Profiles. Since Task 6a, start-up also takes the folder lock and reads the app settings and the mark of the real store (see `0004-lifecycle.md`); it writes nothing unless a recovery renames a damaged file.

### Demo profiles are kept apart

A session against the demo backend uses `demo-store`. Any other session, and no session, uses `store`. The shell picks the store from the backend's origin, which the session cannot change.

- **Why a second folder.** A profile captured from simulated devices names models such as "Demo Graphics Card". Kept in the real store, it would be listed next to real profiles with its devices "not found", and the user could mark it to apply on connect for real hardware. A separate folder means it is never listed, matched or applied while PrismForge talks to OpenRGB. The demo device list and its names also stay out of the real registry.
- **What still crosses.** An export from demo mode can be imported while connected to OpenRGB. Its devices are then unresolved, as for any import of unknown models, and the report says so.

### One PrismForge per folder

Before a store opens, the shell takes an exclusive lock on `instance.lock` with the standard library's `File::try_lock`. It holds the lock until the process ends; the operating system then releases it, also after a crash.

- A second PrismForge that finds the lock taken opens neither store. Every profile command answers `StoreUnavailable::InUse`, and the Profiles screen says another PrismForge window uses the folder. Its sessions still run, without the stored device list, and it writes nothing to the folder.
- It tries again on the next profile command, so it works once the first PrismForge has closed.
- Not covered: a copy of PrismForge that predates the lock, and a folder on a network share whose server ignores file locks.

### Each session

The shell seeds the backend with `Profiles::known_devices`, creates the session, starts `keep_registry` with the session's registry, starts the apply-on-connect watcher, then asks the session to connect. Disconnect aborts the watcher first (it holds a session handle), then closes the session's on-connect report, then ends the session, then waits for `keep_registry` to write the last registry.

### Apply on connect

The watcher feeds every snapshot to `ConnectTrigger` and calls `apply_marked` when it is due. The profile applies once each time the session becomes live after the user connects. A connection that the session restores by itself after a loss is the same connect: the profile is not applied again, and the effects in the session resume. The connection screen says that Connect applies a marked profile.

The shell publishes the session's state in the view (`on_connect`), taken from the trigger and the store's current mark, and nothing else:

| State | Meaning |
|---|---|
| `NotApplicable` | No session, or its store is not available. Nothing is applied on connect. |
| `NotArmed` | The trigger has not come due and no profile is marked. A profile marked before the devices are read is applied then. |
| `Armed` | The trigger has not come due; this profile is applied as soon as the devices are read. This holds also while the session retries after a connect whose first device scan failed. |
| `Unreadable` | The trigger has not come due, and the marked profile needs a newer PrismForge or is no longer in the store. When the trigger comes due, the apply fails at reading the profile and sends nothing; the result is a failure. The view never shows this profile as being applied, and the result carries no name. |
| `Fired` | The trigger came due in this session. `ran` is its result: the same report or failure as the apply-on-connect notice, or nothing when no profile was marked then. Nothing more is applied by itself in this session, also not after a lost connection; the user can still apply a profile. `next` is the profile the next user connect applies, or why it cannot be read. |

The trigger reads the mark when it comes due, so marking or unmarking a profile before then changes what is applied; after it, a change of mark affects only the next connect. The shell publishes the state again after every command that can change the mark or the marked profile (mark, rename, delete), also when the command fails, and when a store opens after a recovery. An import stores a new profile under a new id, so it cannot change the mark and does not publish again. Each session has its own report. Ending the session closes the report, and a closed report publishes nothing. So a publish that started before a disconnect or a mode switch cannot overwrite the state that follows. The interface derives the sentence for a fired apply on connect from its result with the same rule as a manual apply, in one function that the notice and the status line share, so a partial, skipped or failed result is never shown as complete. A session that stopped by itself (for example an unsupported protocol) re-arms the trigger, but it does not retry; the next connect is a new session with a fresh trigger. The interface picks every sentence about a pending apply on connect from this state, and drops a command result that arrives after its session or store changed. PrismForge connects by itself only when the user turned on "Connect to OpenRGB when PrismForge starts" (`0004-lifecycle.md`); that connect goes through the same start and the same trigger, so the marked profile applies once as for a Connect by the user. The window shows the result of each automatic apply with a line per setting.

### Lock order

The shell has five locks, taken in this order: the session slot (`active`), the apply slot, the output gate (permits and the apply's write side), the on-connect report, and the store (the `Stores` slots, the folder lock and the mutexes inside `Profiles`). The comment above `Shell::active` names the same five in the same order. The lifecycle of `0004-lifecycle.md` adds its publish lock, taken before the store and never while a store lock is held; its state mutex is never held across an await.

- No task waits for an earlier lock while it holds a later one.
- The shell's live request (Task 9) is a plain mutex, taken only to copy the request and never across an await.
- A task that holds a session handle never waits for the session slot. Disconnect, a mode switch and exit hold that slot while they wait for every handle to go.
- A command takes its permit while it holds the session slot, and releases the slot with no await in between.
- Publishing the on-connect state takes the report lock only after the session slot is released. The one exception is `start`, which publishes the new session's state while it holds the slot.
- Ending a session closes its report under the report lock, while it holds the session slot.

A test checks the rule for every command. `tests/command_locks.rs` drives each command registered in `generate_handler!` through its shell method. It joins each one with a disconnect, a switch from demo to OpenRGB and the window close, in both poll orders. It also joins each one with a profile apply and each of those three, in all six poll orders. The poll order decides which call reaches the session slot first, so both known deadlock shapes fail it: a command that holds a session handle and then waits for the slot, and a command that holds a permit and then waits for the slot. Every join runs under bounded simulated time; a hang fails the test and names the command, its variant and the order. A guard test fails when a command in `generate_handler!` or a permission in `capabilities/main.json` has no entry in the table, so a new command cannot skip the lock test. The table drives the shell methods, not the Tauri command wrappers: export and import open a native dialog, so the table calls the shell methods behind the dialog, and the empty-host check of `connect_openrgb` is not driven. A join that finishes shows that no hang occurred in that order; it does not show by itself that a command took its permit under the slot. `tests/lock_order.rs` keeps the earlier hand-written joins.

### The apply lock

A user command sent while a profile applies would get a sequence number before the apply's baseline and never count as a pending change. So the shell, not only the interface, refuses it:

1. A profile apply, by the user or on connect, takes the lock. A second apply by the user is refused; the watcher waits for a user's apply to end.
2. While it holds the lock, the shell answers `ProfileApplying` to every output command (apply or stop an effect, an effect across the setup, hardware mode, declare, hand back, frame rate, identify) and to saving, updating and deleting a profile.
3. An output command already sent holds a permit until the session answers it. The apply waits for those permits before it sends anything.
4. The lock is a guard value, released when it is dropped: after success, a partial result, an error, a cancelled command and the end of the session. Tests cover each path.
5. The view carries the profile being applied, so the interface turns its output controls off with the reason. A marked profile that cannot be read is not shown as being applied, because nothing from it is applied.
6. Disconnect clears the result of the last apply on connect from the view. The interface also drops every profile result, open dialog and draft when the session ends or the store changes between demo and real.

### Export and import paths

The export and import commands take no path from the window. Rust opens the native save or open dialog of `tauri-plugin-dialog` and uses only the path it returns. The window has no dialog or file permission. A path inside PrismForge's configuration folder is refused, so an export can never replace a store file. An import whose size is above 1 MiB is refused before it is read.

## Schema and versioning

Every file starts with a format marker and a schema version:

```json
{ "format": "prismforge.profile", "schema_version": { "major": 1, "minor": 0 }, "id": "p-…", "name": "Evening", … }
```

The markers are `prismforge.profile`, `prismforge.profile-export`, `prismforge.devices`, `prismforge.settings`, `prismforge.groups` and `prismforge.setup`.

1. **Each format has its own version.** This build writes profiles and exports as version 2.0 (Task 10) and every other file as version 1.0. It reads profiles and exports with major version 1 or 2, and every other file with major version 1.
2. **A minor version only adds fields.** An older build keeps the fields it does not know and writes them back unchanged. It writes its own version number.
3. **Any other change is a new major version.** That includes a new value for a closed field (device kind, zone kind), a renamed field and a removed field.
4. **A file with a newer major version is refused.** The store reports `FileProblem::NewerVersion` and leaves the file exactly as it is. It never writes, renames or "repairs" that file. Operations on it return `StoreError::NewerFile`. Other files still load.
5. **Older major versions.** A version 1 profile or export is migrated forward in memory when it is read: every stored effect gets the mode "each device", which is all version 1 could mean, and nothing else changes, unknown fields included. Loading writes nothing. The first rewrite copies the version 1 file to the backup, as every write does, and the store writes only version 2. A major version below 1 is refused as `UnknownOldVersion`.
6. **Golden samples pin the stored form.** `crates/prismforge-core/tests/profiles/golden/profile-v2.json` and `export-v2.json` fail the tests if a field name or tag changes. `profile-v1.json`, `profile-v1.1-newer-writer.json` and `export-v1.json` are version 1 inputs: the test `migrates_profile_v1_to_v2_without_loss` loads them, compares them with the expected profile and checks the version 2 file written from them.

### Why the mode is a major version

A version 1 build would keep an unknown `mode` field and write it back, as the minor-version rule says, but it would run every effect on each device. A profile saved across the setup would then light differently with no warning. That is a change of meaning, so profiles moved to version 2, and a version 1 build refuses a version 2 profile as needing a newer PrismForge.

### Profile v2

Version 2 is version 1 with one more field per stored effect. Version 1 is the same table without `mode`.

| Field | Content |
|---|---|
| `id` | Stable id, `p-` and 16 hex digits. It is also the file name. Renaming a profile keeps it. |
| `name` | 1 to 60 characters, trimmed, no control characters. Names are unique in the store. |
| `frame_rate` | 1 to 60 |
| `devices` | Every device the profile uses: `reference`, `label`, `kind`, `vendor`, `model`, `serial`, `location`, `zones` (name, kind, LED minimum, LED maximum) |
| `assignments` | `device`, `zone` (index or `null` for the whole device), `effect`, `mode`: `"EachDevice"` or `{ "AcrossSetup": { "view": "v-…", "flow": { "Along": { "angle": 90 } } } }` (or `{ "FromCentre": { "x": 50, "y": 50 } }`); an angle of 360 or more or a centre above 100 makes the profile not valid |
| `modes` | Hardware modes selected through PrismForge: `device`, `index`, `name` |
| `external` | The user's statements that another program controls a device: `device`, `label` |

What is stored by id and what by index:

- **A device is stored by its identifying fields.** The `reference` is only a key inside the profile. Matching never uses it.
- **A zone is stored by index.** This is safe because a device only resolves when its whole zone list (names, kinds, LED bounds and order) is the same. A changed zone list leaves the device unresolved.
- **A hardware mode is stored by index and name.** If the name at that index changed, apply skips it as "mode list changed". It never selects the mode that moved into that index.
- **An effect is stored as in `0003-effects-engine.md`.** Effect ids and control ids are stable strings. Control values keep their serde form (`{ "Number": 80 }`, `{ "Color": { "r": … } }`). The store keeps each value as raw JSON. A value, control or effect this build does not know stays in the profile and is reported as "not available in this version".
- **Device and zone kinds use the Rust variant names** (`"Gpu"`, `{ "Other": 9 }`). The golden sample pins them.
- **One owner per device.** A device has effects, or one mode, or one external statement. Effects on one device do not overlap. A file that breaks this is not valid.

- **An effect across the setup is stored by view id and flow, not by LED places.** Applying the profile plans the view of the setup in the store at that time. An effect whose view is gone, or whose target the view no longer places, is skipped with `SetupChanged`. An export keeps the view id; on another computer the setup is not there, so those effects are skipped with that reason.

## The local store keeps more than an export

| Data | Local store | Export |
|---|---|---|
| Serial number | Kept, to link the device again on this computer | Left out |
| Location (can hold hardware paths and unique ids) | Kept | Left out |
| Local `DeviceRef` | Kept | Replaced by `device-1`, `device-2`, … |
| Device label | The user's name for the device, or the model | The model |
| Profile id | Kept | Left out; import makes a new id |
| Unknown fields from a newer version | Kept | Left out and counted, because they could hold private data |
| Host name, user name, absolute path, runtime controller ID | Never stored | Never stored |
| Profile name, effects, modes, frame rate, external labels | Kept | Kept |

A test checks the exported bytes for the serial, the location, the user's device name, the local reference, the profile id and the store folder.

## Matching on load, apply and import

1. **The registry is the identity source.** At start the backend is seeded with `devices.json`. The session publishes the registry after each refresh that changes it. `keep_registry` writes the current registry as soon as it starts and then every change, so it may start before or after the connect. A device with a serial keeps its `DeviceRef` after a restart even when its location changed.
2. **A profile device is matched with the registry rules.** Its identifying fields go through the same `reconcile` as the registry: the full fields first, then serial without location. PrismForge never matches by runtime ID or list position.
3. **An exported device has no serial and no location.** It is matched on kind, vendor, model and zones only.
4. **Each stored device gets one status.** Resolved, unavailable (known in this session but not present now), missing, or ambiguous. Two current devices that look the same make every candidate ambiguous. Two stored devices that would match one current device are both ambiguous.
5. **Unresolved devices stay in the profile.** They are not dropped, not guessed and not linked to a similar device. `ProfileDetails` lists each device with its status, and every effect this version cannot run.

## Apply

1. **Order:** frame rate, external statements, hardware modes, effects on each device, then effects across the setup. The effects across one view with one flow and one effect run as one batch, so they share one start time. For an effect, apply and the profile details check the reasons to skip in one order: the device, the effect, the setup (`NewerSetup`, then `SetupChanged`), then the session's reasons (declared external, no Direct Mode).
2. **Each setting gets a result:** applied, or skipped with a reason: missing, ambiguous, unavailable, declared external, no Direct Mode, mode list changed, not available in this version, setup changed or setup needs a newer PrismForge (Task 10), or refused by the session.
3. **Partial application is normal.** Nothing is rolled back. Output changes are not transactional.
4. **A device the user declared external gets nothing.** The session refuses with its conflict, and the report says "declared external" with the program name.
5. **Apply changes only what the profile names.** Other devices keep their owner.
6. **Not connected:** apply is refused with `ApplyError::NotConnected`, unless the profile is the one marked to apply on connect. Then the result is "waiting for connection".

## Capture and update

**One rule for updates: an update never removes what the user did not remove.** PrismForge does not guess what the user did from a change of state. The session records it.

### User intent records

1. **The session records each successful user command for output.** One record per slot: an effect target (device and zone), the hardware mode of a device, the external owner of a device, and the frame rate. A new record on a slot replaces the older one, so the store never grows past the slots in use. Each command gets the next sequence number.
2. **Every side effect of a user command is recorded.** The session compares its ownership state (running effects, hardware modes, external owners, frame rate) before and after the command. Each other slot the command ended or changed gets its own record under the same sequence number, before the command's own record. Examples: a zone effect that displaces a whole-device effect, an effect that ends a hardware mode, a mode selection or a declaration that ends effects. A new ownership rule in the session is covered without a new list.

| User command | Its own record |
|---|---|
| Apply an effect | the target: the complete validated instance and its mode (each device, or across a view with a flow) |
| Stop an effect | the target: stopped |
| Select a hardware mode | the mode: index and name |
| Declare another program | the external owner: the label |
| Hand back | the external owner: handed back (only if the device was declared) |
| Set the frame rate | the frame rate in use |

An ended effect is recorded as stopped, an ended mode as ended, an ended declaration as handed back.

A batch (Task 9) records like one command: every target it applied or stopped gets its own record, and every side effect its record, all under one sequence number. A target it covered or skipped gets none. An apply across the setup (Task 10) records the same way, each applied target with the mode "across the setup". The session compares the mode as part of a running effect, so applying the same effect to the same target in the other mode is a change of that slot. Stop all PrismForge effects records a stop for each device. Because the side effects come from the comparison of the ownership state before and after, a target the command did not name as its own would still be recorded; the own records keep a re-apply with unchanged values in the log.

3. **Nothing else creates a record.** Disconnect, reconnect, a device that becomes unavailable, a refused or paused write, identify and its restore, the release at disconnect or shutdown, and the scheduler change no record. A command that fails creates no record.
4. **A profile apply is not a user edit.** Its commands carry the profile origin and create no record. When the apply ends, the session loop returns its current sequence number in command order. That number is the profile's baseline.
5. **Replies come after the snapshot.** The session publishes the snapshot that shows a command's result before it sends the reply, for every command. A caller that sees a reply also sees its result.
6. **Read-only queries wake nobody.** The session sends a snapshot only when it differs from the last one. Reading the intent records, the baseline, a preview, the summary or the details changes nothing and wakes no subscriber.
7. **Commands during an apply.** A user command sent while a profile apply runs gets a sequence number before the apply's baseline, so the next update does not see it. The interface must not send output commands while an apply runs.

### Update

1. **The active profile.** The profile last applied, created or updated in a session is that session's active profile. `StoreSummary::active` names it. `StoreSummary::unsaved_changes` is the number of settings an update would change now (frame rate included), so it is 0 exactly when an update would report no change. A new session has none.
2. **Update needs the active profile.** `update_from_session` refuses any other profile with `StoreError::NotActive`, which names the active one. Saving the session under a new name is `create_from_session`.
3. **An update is the stored profile plus the user's records after the baseline, applied in order.** An applied effect replaces the stored effect on that target, or is added, with the complete instance and its mode. A change of mode alone is a replaced setting. A stop removes the effect on that target; a stop of the whole device removes every effect on the device, as in the session. A selected mode replaces the device's effects and external owner. A declaration replaces the device's effects and mode. A hand-back removes the declaration. A frame-rate record sets the frame rate.
4. **Nothing else changes.** Every other setting, every unresolved or absent device, every skipped setting, and all unknown fields stay as loaded. The device and zone entries of a replaced setting stay as stored. A setting replaced on the same target keeps its own unknown fields; its effect keeps its unknown fields only when the effect id is the same.
5. **The baseline moves only after a successful update.** An update whose report is empty writes no file and keeps the backup as it was. A second update with no new user command changes nothing.
6. **The report comes from the same computation.** `Saved { id, changes, frame_rate }` lists each setting the update adds, replaces (with what it was) or removes. Everything not listed is unchanged. `preview_update` runs the same computation without writing, so the interface can show it before the save.
7. **Records are linked to stored devices with the matching rules.** A record for a device that resolves to a stored device changes that device's settings. A record for a device the profile does not have adds a device entry from the current inventory.
8. **Invariant.** After an update, the stored profile equals the stored profile before it, with the session's own state for every device the user changed since the last save. A seeded random test checks this over many command sequences (apply to a zone or a whole device, stop, hardware mode, declaration, hand-back, frame rate, since Task 9 batch apply, batch stop and stop all, with overlapping and repeated targets, and since Task 10 an apply across a random part of a setup view with a random flow and a stop of such targets, on a setup with a daisy chain, a mirrored pair and a device without Direct Mode) with updates at random points, together with the rule for `unsaved_changes`. It fails if too few runs ever save an effect across the setup.

### Create from the session

- **With an active profile:** the new profile is the active profile plus the pending user records, under the new name and id. Skipped and unresolved settings move into it.
- **Without an active profile:** the new profile is what runs now: the scene, the hardware modes, the external statements and the frame rate. A device label is the user's name from the registry, or the model.
- The new profile becomes the active profile, with the sequence number read before the save as its baseline.

## The setup

The setup (Task 10) is one file per store, `setup.json`, written durably like every store file, with the same recovery, the same refusal of a newer major version and unknown fields kept at every level. `ProfileStore` holds it beside the profiles, because applying a profile needs it, and `Profiles::setup` and `Profiles::save_setup` reach it at the store's place in the lock order. It stays on this computer: it is not part of a profile, an export or a diagnostics copy. Its outputs are matched to devices by stored reference only, with the groups' states. While the file needs a newer PrismForge, `Profiles::setup` refuses with `NewerFile`, and effects across the setup in a profile are skipped with `NewerSetup`, not "setup changed". See `0006-setup-model.md`.

**Commands (Task 11).** All four are in `tests/command_locks.rs`:

| Command | Locks | Apply lock |
|---|---|---|
| `setup` | The session slot only to read the devices and the store kind, then the store (`Profiles::setup`), as the group commands. | Not refused: it sends nothing to a device. |
| `save_setup` | The same as `setup`; then `Profiles::save_setup` checks new or changed outputs against the devices read. | Not refused: it stores a file and sends nothing. |
| `apply_across` | An output permit taken under the session slot (`Shell::permitted`), then the store to read the saved setup, then the session. The permit is held until the session answers, like `create_profile`. | Refused with `ProfileApplying`; `tests/profiles.rs` checks it with the other output commands. |
| `preview_across` | None. It plans the drawing it is given against no device and renders frames in Rust. | Not refused: it uses no session. |

`apply_across` plans the **saved** setup, never a drawing the window holds, so a profile saved afterwards names a view whose places are stored. The interface refuses Apply while the drawing has unsaved changes.

## Groups

A group (Task 9) is a named set of targets that the user selects again. It selects; it owns no lighting, and profiles keep storing their own per-target settings. Later tasks reuse groups (setup views, automation rules) through the same model in `crates/prismforge-core/src/store/groups.rs`.

1. **One file per store.** `groups.json` sits in the real store folder and in the demo store folder, beside the device list whose references it uses, so demo groups never meet real devices. It is written durably like every store file (temporary file, sync, backup, replace; recovery from the backup with a notice; a damaged file is renamed, never deleted). A file with a newer major version is left untouched and every change is refused with `NewerFile`.
2. **Schema.** `{ "format": "prismforge.groups", "schema_version": { "major": 1, "minor": 0 }, "groups": [ { "id": "g-…", "name": "Case fans", "members": [ { "device": "dev-…", "zone": 1, "label": "Demo Motherboard", "zone_name": "Addressable Header 1" } ] } ] }`. Unknown fields at every level are kept and written back.
3. **Validation.** A name follows the profile name rules (trimmed, 1 to 60 characters, no control character) and is unique without regard to case. At most 100 groups and 256 members a group. A new member must be a device or zone the session reports now; its device and zone names are kept with it. A file that breaks a rule is not used.
4. **Never guessed.** A member is matched by its stored reference only, through the device registry, and reported as resolved, unavailable, missing, ambiguous, or changed (the zone at that index no longer has the name it had). Selecting a group selects only resolved members that can take an effect and names the others. A member missing today is kept through every rename, addition and removal.
5. **Everything** is not stored: it is every target that can take an effect today, computed when it is selected.
6. **Stays on this computer.** Groups are not part of a profile, an export or a diagnostics copy.
7. **Commands.** `groups`, `create_group`, `rename_group`, `add_group_members`, `remove_group_member` and `delete_group` take the session slot only to read the devices and the store kind, then work on the store alone, at the store's place in the lock order. They send nothing to a device, so the apply lock does not refuse them.

## Durable writes and recovery

Every write:

1. writes the new content to `<file>.tmp` and syncs it to disk;
2. if the current file is valid, copies it to `<file>.bak` the same way (temporary file, sync, rename);
3. renames `<file>.tmp` over `<file>`. On Unix the folder is synced after the rename. Windows cannot open a folder this way, so that step is skipped there.

**The store writes nothing when nothing changes.** Renaming to the same name, marking the profile that is already marked, saving the same content or the same registry writes no file, so the backup still holds the previous version.

**The store never writes a file its loader would refuse.** It checks the encoded size against the 4 MiB limit before it writes anything and returns `StoreError::TooLarge`; the old file stays. The same check covers profiles, the registry, settings and an export. An import that fits the 1 MiB import limit but would grow past 4 MiB in the stored form is refused the same way, before anything is stored. The registry is checked before it is written.

If any step fails, the temporary file is removed and the old file stays in place. The store keeps the old content in memory and returns `StoreError::Write` with the step. The temporary file has a fixed name, so failed writes never pile up.

At start, for each file:

| Situation | What the store does | Notice |
|---|---|---|
| File valid | Uses it | none |
| File newer major version | Leaves it as it is and never writes it | `LeftUntouched` |
| File missing or damaged, backup valid | Renames the damaged file to `<file>.unreadable-1` (or the next free number), copies the backup back to `<file>`, uses it | `RestoredFromBackup` with the problem and the new name |
| File and backup damaged | Renames both aside, starts that part empty | `StartedEmpty` with both problems and both new names |

The store never deletes a damaged file. A store file larger than 4 MiB counts as damaged. A failed background write of the registry becomes a `WriteFailed` notice.

## File I/O and the session

The session task does no file I/O. `Profiles` runs every store operation with `tokio::task::spawn_blocking`. Capture reads the published snapshot. Apply sends ordinary session commands. A test holds a profile write for about a second while an effect runs: frames keep their cadence and commands are answered.

## Recovery instructions for users

The files are in the folder `io.github.kloppers-labs.prismforge` of your user configuration folder: under `%APPDATA%` on Windows, `~/Library/Application Support` on macOS and `~/.config` on Linux. Your profiles are in its `store` folder. Demo mode keeps its own in `demo-store`. Diagnostics in PrismForge shows the folder in use. PrismForge shows each repair below on the Profiles screen until you dismiss it.

- **`devices.json`** remembers your devices and the names you gave them. **`settings.json`** remembers the profile to apply on connect. **`setup.json`** remembers where your lights are. **`profiles`** holds one file per profile.
- **A `.bak` file** is the version before the last change. PrismForge uses it by itself when the main file is missing or damaged.
- **After a "restored from backup" notice:** your last change to that file may be missing. Check the profile or the device names. The damaged file is still there with `.unreadable-1` (or a higher number) at the end of its name.
- **After a "could not read the file or its backup" notice:** PrismForge started that part empty. Both damaged files are still there with `.unreadable-…` at the end. Close PrismForge before you touch them. If you have a copy of the file from elsewhere, put it back under the original name. You can also send the damaged files to the project in an issue, but first remove serial numbers and locations from them.
- **After a "needs a newer PrismForge" notice:** a newer version of PrismForge wrote that file. This version does not change it. Install the newer version, or keep using this version without that profile.
- **To move a profile to another computer,** use export and import, not a copy of the local file. The export has no serial numbers or locations. On the other computer, two devices of the same model cannot be told apart from an export; they stay unresolved.
- **A `.tmp` file** is the rest of a write that stopped. It is safe to delete when PrismForge is closed.
- **`instance.lock`** keeps a second PrismForge out of the folder while one runs. It holds nothing. Do not delete it while PrismForge runs.

## Not verified

- Any run against a real OpenRGB server or real hardware.
- Behaviour after a real power loss. The tests fail single steps through a file-operations seam; they cannot cut power between the sync and the rename.
- The Windows file system. CI runs the same tests on `windows-latest`. Replace-over-existing uses `std::fs::rename`, which replaces the target on Windows. A file that another program holds open without delete sharing (for example an antivirus scan) makes a write fail with an error; the old file stays.
- Whether a real device name from OpenRGB can contain private data. The export keeps the model name as OpenRGB reports it.
- The desktop app on Windows: the configuration folder, the folder lock and the native dialogs. CI builds and tests on `windows-latest`, but nobody has run the app there.
- The native save and open dialogs. Rust tests call the export and import code with paths; no test and no hand check opened a dialog.
- Two PrismForge processes on one real folder. A test opens two stores on one folder in one process; the lock works per open file on macOS and Windows, but two real processes were not started.
