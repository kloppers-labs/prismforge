import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AcrossPreview } from "./bindings/AcrossPreview";
import type { AcrossPreviewRequest } from "./bindings/AcrossPreviewRequest";
import type { AcrossReport } from "./bindings/AcrossReport";
import type { ApplyOutcome } from "./bindings/ApplyOutcome";
import type { BatchReport } from "./bindings/BatchReport";
import type { GroupId } from "./bindings/GroupId";
import type { GroupsView } from "./bindings/GroupsView";
import type { LiveOutput } from "./bindings/LiveOutput";
import type { CloseAction } from "./bindings/CloseAction";
import type { CommandError } from "./bindings/CommandError";
import type { DeviceRef } from "./bindings/DeviceRef";
import type { EffectCatalog } from "./bindings/EffectCatalog";
import type { EffectInstance } from "./bindings/EffectInstance";
import type { ExportReport } from "./bindings/ExportReport";
import type { Flow } from "./bindings/Flow";
import type { IdentifyOutcome } from "./bindings/IdentifyOutcome";
import type { ImportReport } from "./bindings/ImportReport";
import type { LifecycleView } from "./bindings/LifecycleView";
import type { OutputTarget } from "./bindings/OutputTarget";
import type { PreviewBatch } from "./bindings/PreviewBatch";
import type { PreviewRequest } from "./bindings/PreviewRequest";
import type { ProfileDetails } from "./bindings/ProfileDetails";
import type { ProfileId } from "./bindings/ProfileId";
import type { RestoreOutcome } from "./bindings/RestoreOutcome";
import type { Saved } from "./bindings/Saved";
import type { Setup } from "./bindings/Setup";
import type { SetupStoreView } from "./bindings/SetupStoreView";
import type { AppSettingChange } from "./bindings/AppSettingChange";
import type { AutomationView } from "./bindings/AutomationView";
import type { RuleAction } from "./bindings/RuleAction";
import type { Trigger } from "./bindings/Trigger";
import type { ShellView } from "./bindings/ShellView";
import type { StoreNotice } from "./bindings/StoreNotice";
import type { StoreView } from "./bindings/StoreView";
import type { ViewId } from "./bindings/ViewId";

export const VIEW_EVENT = "session-view";
export const LIFECYCLE_EVENT = "lifecycle-view";
export const LIVE_EVENT = "live-output";
export const DEFAULT_HOST = "127.0.0.1";
export const DEFAULT_PORT = 6742;

export type Failure = CommandError | { Unexpected: string };
export type Answer<T> = { ok: true; value: T } | { ok: false; error: Failure };

async function call<T>(command: string, args?: Record<string, unknown>): Promise<Answer<T>> {
  try {
    return { ok: true, value: await invoke<T>(command, args) };
  } catch (error) {
    return { ok: false, error: asFailure(error) };
  }
}

const PLAIN_ERRORS = ["NoSession", "EmptyHost", "ProfileApplying", "NotConnected"];
const SHAPED_ERRORS = ["Session", "InvalidEffect", "NoEffect", "TooManyTargets", "Store", "StoreUnavailable", "Unreadable", "File", "Setting", "Plan", "PreviewTooLarge"];

function asFailure(error: unknown): Failure {
  if (typeof error === "string" && PLAIN_ERRORS.includes(error)) {
    return error as CommandError;
  }
  if (typeof error === "object" && error !== null && SHAPED_ERRORS.some((key) => key in error)) {
    return error as CommandError;
  }
  return { Unexpected: String(error) };
}

export const api = {
  view: () => call<ShellView>("view"),
  connectOpenRgb: (host: string, port: number) => call<ShellView>("connect_openrgb", { host, port }),
  startDemo: () => call<ShellView>("start_demo"),
  disconnect: () => call<ShellView>("disconnect"),
  refresh: () => call<null>("refresh"),
  identify: (device: DeviceRef, zone: number | null) =>
    call<IdentifyOutcome>("identify", { device, zone }),
  effectCatalog: () => call<EffectCatalog>("effect_catalog"),
  applyEffect: (target: OutputTarget, effect: EffectInstance) => call<null>("apply_effect", { target, effect }),
  stopEffect: (target: OutputTarget) => call<RestoreOutcome | null>("stop_effect", { target }),
  applyEffectBatch: (targets: OutputTarget[], effect: EffectInstance) => call<BatchReport>("apply_effect_batch", { targets, effect }),
  stopEffectBatch: (targets: OutputTarget[]) => call<BatchReport>("stop_effect_batch", { targets }),
  stopAllEffects: () => call<BatchReport>("stop_all_effects"),
  watchLiveOutput: (targets: OutputTarget[]) => call<null>("watch_live_output", { targets }),
  groups: () => call<GroupsView>("groups"),
  createGroup: (name: string, targets: OutputTarget[]) => call<GroupsView>("create_group", { name, targets }),
  renameGroup: (id: GroupId, name: string) => call<GroupsView>("rename_group", { id, name }),
  addGroupMembers: (id: GroupId, targets: OutputTarget[]) => call<GroupsView>("add_group_members", { id, targets }),
  removeGroupMember: (id: GroupId, target: OutputTarget) => call<GroupsView>("remove_group_member", { id, target }),
  deleteGroup: (id: GroupId) => call<GroupsView>("delete_group", { id }),
  setup: () => call<SetupStoreView>("setup"),
  saveSetup: (setup: Setup) => call<SetupStoreView>("save_setup", { setup }),
  applyAcross: (view: ViewId, flow: Flow, effect: EffectInstance, targets: OutputTarget[] | null) =>
    call<AcrossReport>("apply_across", { view, flow, effect, targets }),
  previewAcross: (request: AcrossPreviewRequest) => call<AcrossPreview>("preview_across", { request }),
  onLive: (handler: (live: LiveOutput) => void): Promise<UnlistenFn> =>
    listen<LiveOutput>(LIVE_EVENT, (event) => handler(event.payload)),
  selectMode: (device: DeviceRef, mode: number) => call<null>("select_mode", { device, mode }),
  declareExternal: (device: DeviceRef, label: string) => call<null>("declare_external", { device, label }),
  clearExternal: (device: DeviceRef) => call<null>("clear_external", { device }),
  setFrameRate: (frameRate: number) => call<number>("set_frame_rate", { frameRate }),
  previewEffect: (request: PreviewRequest) => call<PreviewBatch>("preview_effect", { request }),
  profileStore: () => call<StoreView>("profile_store"),
  profileDetails: (id: ProfileId) => call<ProfileDetails>("profile_details", { id }),
  createProfile: (name: string) => call<Saved>("create_profile", { name }),
  previewProfileUpdate: (id: ProfileId) => call<Saved>("preview_profile_update", { id }),
  updateProfile: (id: ProfileId) => call<Saved>("update_profile", { id }),
  renameProfile: (id: ProfileId, name: string) => call<null>("rename_profile", { id, name }),
  duplicateProfile: (id: ProfileId) => call<ProfileId>("duplicate_profile", { id }),
  deleteProfile: (id: ProfileId) => call<null>("delete_profile", { id }),
  applyProfile: (id: ProfileId) => call<ApplyOutcome>("apply_profile", { id }),
  exportProfile: (id: ProfileId) => call<ExportReport | null>("export_profile", { id }),
  importProfile: () => call<ImportReport | null>("import_profile"),
  setDeviceName: (device: DeviceRef, name: string | null) => call<null>("set_device_name", { device, name }),
  setZoneName: (device: DeviceRef, zone: number, name: string | null) =>
    call<null>("set_zone_name", { device, zone, name }),
  dismissStoreNotice: (notice: StoreNotice) => call<boolean>("dismiss_store_notice", { notice }),
  automation: () => call<AutomationView>("automation"),
  setRule: (trigger: Trigger, enabled: boolean, action: RuleAction) =>
    call<AutomationView>("set_rule", { trigger, enabled, action }),
  lifecycle: () => call<LifecycleView>("lifecycle"),
  changeSetting: (change: AppSettingChange) => call<LifecycleView>("change_setting", { change }),
  answerClose: (choice: CloseAction | null) => call<null>("answer_close", { choice }),
  quit: () => call<null>("quit"),
  onLifecycle: (handler: (view: LifecycleView) => void): Promise<UnlistenFn> =>
    listen<LifecycleView>(LIFECYCLE_EVENT, (event) => handler(event.payload)),
  onView: (handler: (view: ShellView) => void): Promise<UnlistenFn> =>
    listen<ShellView>(VIEW_EVENT, (event) => handler(event.payload)),
};
