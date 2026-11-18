# 0002 — Profiles, device identity on disk and the local store

- Status: accepted
- Date: 2026-11-18
- Scope: how PrismForge saves profiles and known devices, how it links them to hardware again, and how it recovers damaged files

## Current state

The Rust core has a durable local store, profile schema version 1, capture, apply, export and import (Task 3a). The desktop app does not use it yet: it has no profile screen and no profile command (Task 3b). **Nothing here has run against a real OpenRGB server or real hardware.** The evidence comes from tests on real files in temporary folders, the demo backend and the scripted fake server.

The code is in `crates/prismforge-core/src/profile` (the model, capture, matching and apply) and `crates/prismforge-core/src/store` (files, recovery, registry, export and import, and `Profiles` for async callers).

## Application save is not device save

1. **The store writes files on this computer only.** It never writes to device memory. No type, command or error in the store uses the word "save mode".
2. **Loading writes nothing to hardware.** Opening the store, importing a file and reading a profile send nothing to OpenRGB.
3. **Applying a profile is an explicit command.** It uses the same session commands as the effects screen.
4. **Apply on connect is off by default.** The user marks at most one profile. Only that profile is applied, once each time the session becomes live after a disconnect.
5. **Nothing goes over the network.** The store has no network code.

## Files

The caller gives the store one folder. The desktop app will pass its per-user configuration folder.

| File | Holds |
|---|---|
| `devices.json` | The known-device registry: each `DeviceRef` with its identifying fields, and the user's names for the device and its zones |
| `settings.json` | The profile to apply on connect, or none |
| `profiles/<id>.json` | One profile |

Each file has a backup next to it with `.bak` added to the name. A write uses a temporary file with `.tmp` added to the name.

**One file per profile.** A damaged profile cannot take the others down. Each write touches one small file. Delete is the removal of one file and its backup.

## Schema and versioning

Every file starts with a format marker and a schema version:

```json
{ "format": "prismforge.profile", "schema_version": { "major": 1, "minor": 0 }, "id": "p-…", "name": "Evening", … }
```

The markers are `prismforge.profile`, `prismforge.profile-export`, `prismforge.devices` and `prismforge.settings`.

1. **This build writes version 1.0.** It reads every file with major version 1.
2. **A minor version only adds fields.** An older build keeps the fields it does not know and writes them back unchanged. It writes its own version number.
3. **Any other change is a new major version.** That includes a new value for a closed field (device kind, zone kind), a renamed field and a removed field.
4. **A file with a newer major version is refused.** The store reports `FileProblem::NewerVersion` and leaves the file exactly as it is. It never writes, renames or "repairs" that file. Operations on it return `StoreError::NewerFile`. Other files still load.
5. **Older major versions.** Version 1 is the first. When a version 2 exists, a version 1 file is migrated forward in memory, the original is copied to a backup before the first rewrite, and the store writes only version 2. A major version below 1 is refused as `UnknownOldVersion`.
6. **Golden samples pin the stored form.** `crates/prismforge-core/tests/profiles/golden/profile-v1.json` and `export-v1.json` fail the tests if a field name or tag changes.

### Profile v1

| Field | Content |
|---|---|
| `id` | Stable id, `p-` and 16 hex digits. It is also the file name. Renaming a profile keeps it. |
| `name` | 1 to 60 characters, trimmed, no control characters. Names are unique in the store. |
| `frame_rate` | 1 to 60 |
| `devices` | Every device the profile uses: `reference`, `label`, `kind`, `vendor`, `model`, `serial`, `location`, `zones` (name, kind, LED minimum, LED maximum) |
| `assignments` | `device`, `zone` (index or `null` for the whole device), `effect` |
| `modes` | Hardware modes selected through PrismForge: `device`, `index`, `name` |
| `external` | The user's statements that another program controls a device: `device`, `label` |

What is stored by id and what by index:

- **A device is stored by its identifying fields.** The `reference` is only a key inside the profile. Matching never uses it.
- **A zone is stored by index.** This is safe because a device only resolves when its whole zone list (names, kinds, LED bounds and order) is the same. A changed zone list leaves the device unresolved.
- **A hardware mode is stored by index and name.** If the name at that index changed, apply skips it as "mode list changed". It never selects the mode that moved into that index.
- **An effect is stored as in `0003-effects-engine.md`.** Effect ids and control ids are stable strings. Control values keep their serde form (`{ "Number": 80 }`, `{ "Color": { "r": … } }`). The store keeps each value as raw JSON. A value, control or effect this build does not know stays in the profile and is reported as "not available in this version".
- **Device and zone kinds use the Rust variant names** (`"Gpu"`, `{ "Other": 9 }`). The golden sample pins them.
- **One owner per device.** A device has effects, or one mode, or one external statement. Effects on one device do not overlap. A file that breaks this is not valid.

The layout and component model of a later task will be new fields in a later version.

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

1. **Order:** frame rate, external statements, hardware modes, effects.
2. **Each setting gets a result:** applied, or skipped with a reason: missing, ambiguous, unavailable, declared external, no Direct Mode, mode list changed, not available in this version, or refused by the session.
3. **Partial application is normal.** Nothing is rolled back. Output changes are not transactional.
4. **A device the user declared external gets nothing.** The session refuses with its conflict, and the report says "declared external" with the program name.
5. **Apply changes only what the profile names.** Other devices keep their owner.
6. **Not connected:** apply is refused with `ApplyError::NotConnected`, unless the profile is the one marked to apply on connect. Then the result is "waiting for connection".

## Capture and update

Capture reads the snapshot: the scene, hardware modes, external statements and the frame rate. A device label is the user's name from the registry, or the stored label, or the model.

**One rule for updates: an update never removes what the user did not remove.** An update may replace or remove a stored setting only where the session shows that the user changed its target in this session: the user applied another effect, stopped the effect, chose a hardware mode, or declared or handed back another program.

1. **The active profile.** The profile last applied, created or updated in a session is the active profile of that session. `StoreSummary::active` shows it. A new session has none.
2. **Update needs the active profile.** `update_from_session` refuses any other profile with `StoreError::NotActive`, which names the active one. A profile that was never applied in this session would otherwise take the whole session state, so the user would lose settings without seeing why. To save the session state under a new name, the user creates a profile instead.
3. **The session history.** After an apply, PrismForge remembers which settings ran, which were skipped and why, and the owners and effects of every device right after the apply. After each create or update it remembers the session state again.
4. **Per stored setting:**

| Situation | Result |
|---|---|
| The device is missing or ambiguous | Kept unchanged (`Kept`, reason missing or ambiguous) |
| The session runs exactly this setting | Kept as stored, with its unknown fields (`Unchanged`) |
| Nothing on the device changed since the apply or the last save | Kept unchanged (`Kept`, reason unavailable, or not running with the skip reason) |
| The user changed the device and the session has another state on this target | Removed from the profile (`Replaced`); the new state is `Added` |
| The user changed the device, the session has nothing on this target, and the setting ran from this profile | Removed (`Removed`) |
| The user changed the device, the session has nothing on this target, and the setting never ran | Kept (`Kept`, not running) |

5. **Every skip reason keeps the setting.** Missing, ambiguous, unavailable, declared external, no Direct Mode, mode list changed, not available in this version, and refused by the session. A state that already blocked a setting when it was applied (for example a program the user declared before the apply) does not replace it, and is not added on top of it.
6. **The result lists every setting.** `Saved { id, changes }` says for each setting: unchanged, added, replaced, removed, or kept with its reason. The interface can show it before or after the save.
7. **Create from the session** uses the active profile as its base in the same way, so its unresolved and skipped settings move into the new profile.
8. **Unknown fields stay.** A kept or unchanged setting keeps all its unknown fields. A linked device keeps its unknown fields and the unknown fields of each zone with the same name and position; it gets today's identifying fields.

## Durable writes and recovery

Every write:

1. writes the new content to `<file>.tmp` and syncs it to disk;
2. if the current file is valid, copies it to `<file>.bak` the same way (temporary file, sync, rename);
3. renames `<file>.tmp` over `<file>`. On Unix the folder is synced after the rename. Windows cannot open a folder this way, so that step is skipped there.

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

The files are in the PrismForge folder of your user configuration folder. On Windows this is under `%APPDATA%`. On macOS it is under `~/Library/Application Support`. On Linux it is under `~/.config`. The exact folder name comes with Task 3b.

- **`devices.json`** remembers your devices and the names you gave them. **`settings.json`** remembers the profile to apply on connect. **`profiles`** holds one file per profile.
- **A `.bak` file** is the version before the last change. PrismForge uses it by itself when the main file is missing or damaged.
- **After a "restored from backup" notice:** your last change to that file may be missing. Check the profile or the device names. The damaged file is still there with `.unreadable-1` (or a higher number) at the end of its name.
- **After a "could not read the file or its backup" notice:** PrismForge started that part empty. Both damaged files are still there with `.unreadable-…` at the end. Close PrismForge before you touch them. If you have a copy of the file from elsewhere, put it back under the original name. You can also send the damaged files to the project in an issue, but first remove serial numbers and locations from them.
- **After a "needs a newer PrismForge" notice:** a newer version of PrismForge wrote that file. This version does not change it. Install the newer version, or keep using this version without that profile.
- **To move a profile to another computer,** use export and import, not a copy of the local file. The export has no serial numbers or locations. On the other computer, two devices of the same model cannot be told apart from an export; they stay unresolved.
- **A `.tmp` file** is the rest of a write that stopped. It is safe to delete when PrismForge is closed.

## Not verified

- Any run against a real OpenRGB server or real hardware.
- Behaviour after a real power loss. The tests fail single steps through a file-operations seam; they cannot cut power between the sync and the rename.
- The Windows file system. CI runs the same tests on `windows-latest`. Replace-over-existing uses `std::fs::rename`, which replaces the target on Windows. A file that another program holds open without delete sharing (for example an antivirus scan) makes a write fail with an error; the old file stays.
- Whether a real device name from OpenRGB can contain private data. The export keeps the model name as OpenRGB reports it.
- The desktop flow: the profile screen, the config folder, the apply-on-connect watcher in the shell (Task 3b).
