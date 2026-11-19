import type { AppState } from "../state/app.svelte";
import { summarise } from "../../features/connection/wording";
import { ended } from "./status";

export type NoticeKind = "problem" | "lifecycle" | "connection" | "repaired" | "trayStop" | "connectApply";

/** Every notice the shell holds now, problems first; a notice leaves only when its data does or the user dismisses it. */
export function activeNotices(app: AppState): NoticeKind[] {
  const session = app.session;
  const failure = app.lifecycle.failure;
  const store = app.profiles.store;
  const kinds: (NoticeKind | false)[] = [
    app.problem !== null && "problem",
    failure !== null && (failure.key === "close" || failure.key === "quit") && "lifecycle",
    session !== null && !ended(session) && session.devices.length > 0 && summarise(session).reasons.length > 0 && "connection",
    store !== null && store.summary.notices.length > 0 && app.screen !== "profiles" && "repaired",
    app.lifecycle.trayStop !== null && "trayStop",
    !app.connectApplyDismissed && app.view.connect_apply !== null && "connectApply",
  ];
  return kinds.filter((kind): kind is NoticeKind => kind !== false);
}
