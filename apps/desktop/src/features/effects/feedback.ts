import type { Failure } from "../../lib/api/client";
import type { OutputTarget } from "../../lib/api/bindings/OutputTarget";
import type { RestoreOutcome } from "../../lib/api/bindings/RestoreOutcome";
import type { ShellView } from "../../lib/api/bindings/ShellView";
import { canStop, outputOf, runningOn } from "./present";

export type EffectFeedback =
  | { kind: "applied"; effect: string; label: string; target: OutputTarget; targetLabel: string; revision: number }
  | { kind: "stopped"; outcome: RestoreOutcome | null; target: OutputTarget; revision: number }
  | { kind: "failed"; error: Failure };

export type ModeFeedback =
  | { kind: "selected"; device: string; mode: number; name: string; server: string; revision: number }
  | { kind: "failed"; error: Failure };

export type FrameRateFeedback =
  | { kind: "set"; requested: number; used: number; revision: number }
  | { kind: "failed"; error: Failure };

/** A command result stays on screen until a newer view contradicts it. */
function contradicted(view: ShellView, revision: number, holds: (session: NonNullable<ShellView["session"]>) => boolean): boolean {
  return view.revision > revision && (view.session === null || !holds(view.session));
}

export function currentEffect(feedback: EffectFeedback | null, view: ShellView): EffectFeedback | null {
  if (feedback === null || feedback.kind === "failed") {
    return feedback;
  }
  if (feedback.kind === "applied") {
    const gone = contradicted(view, feedback.revision, (session) => runningOn(session, feedback.target)?.effect.effect === feedback.effect);
    return gone ? null : feedback;
  }
  return contradicted(view, feedback.revision, (session) => !canStop(session, feedback.target)) ? null : feedback;
}

export function currentMode(feedback: ModeFeedback | null, view: ShellView): ModeFeedback | null {
  if (feedback === null || feedback.kind === "failed") {
    return feedback;
  }
  const gone = contradicted(view, feedback.revision, (session) => {
    const owner = outputOf(session, feedback.device)?.owner;
    return typeof owner === "object" && "HardwareMode" in owner && owner.HardwareMode.mode === feedback.mode;
  });
  return gone ? null : feedback;
}

export function currentFrameRate(feedback: FrameRateFeedback | null, view: ShellView): FrameRateFeedback | null {
  if (feedback === null || feedback.kind === "failed") {
    return feedback;
  }
  return contradicted(view, feedback.revision, (session) => session.output.frame_rate === feedback.used) ? null : feedback;
}
