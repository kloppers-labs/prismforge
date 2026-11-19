import { api, type Failure } from "../../lib/api/client";
import type { AppSettingChange } from "../../lib/api/bindings/AppSettingChange";
import type { CloseAction } from "../../lib/api/bindings/CloseAction";
import type { LifecycleView } from "../../lib/api/bindings/LifecycleView";
import type { SettingsView } from "../../lib/api/bindings/SettingsView";
import { nativeWindow } from "../../lib/window.svelte";

export type SettingKey = "OnClose" | "StartWithSystem" | "StartHidden" | "ConnectAtStart";

export type SettingFailure = { key: SettingKey | "close" | "quit"; error: Failure };

function keyOf(change: AppSettingChange): SettingKey {
  return Object.keys(change)[0] as SettingKey;
}

/** The lifecycle view from Rust; every switch shows its state, and a refusal is shown once, never retried. */
export class LifecycleState {
  view = $state<LifecycleView | null>(null);
  pending = $state<SettingKey | "close" | "quit" | null>(null);
  failure = $state<SettingFailure | null>(null);
  dismissedStop = $state(0);

  get settings(): SettingsView | null {
    const settings = this.view?.settings;
    return settings && "Ready" in settings ? settings.Ready.settings : null;
  }

  get trayAvailable(): boolean {
    return this.view?.tray === "Available";
  }

  get trayStop() {
    const stop = this.view?.tray_stop ?? null;
    return stop && stop.sequence > this.dismissedStop ? stop : null;
  }

  async load() {
    const current = await api.lifecycle();
    if (current.ok) {
      this.accept(current.value);
    }
  }

  accept(view: LifecycleView) {
    if (this.view !== null && view.revision <= this.view.revision) {
      return;
    }
    this.view = view;
    nativeWindow.visible = view.window_visible;
  }

  async change(change: AppSettingChange) {
    const key = keyOf(change);
    this.pending = key;
    this.failure = null;
    const answer = await api.changeSetting(change);
    this.pending = null;
    if (answer.ok) {
      this.accept(answer.value);
    } else {
      this.failure = { key, error: answer.error };
    }
  }

  async answerClose(choice: CloseAction | null) {
    this.pending = "close";
    this.failure = null;
    const answer = await api.answerClose(choice);
    this.pending = null;
    if (!answer.ok) {
      this.failure = { key: "close", error: answer.error };
    }
  }

  async quit() {
    if (this.pending === "quit" || this.view?.quitting) {
      return;
    }
    this.pending = "quit";
    const answer = await api.quit();
    if (!answer.ok) {
      this.pending = null;
      this.failure = { key: "quit", error: answer.error };
    }
  }

  dismissStop() {
    this.dismissedStop = this.view?.tray_stop?.sequence ?? this.dismissedStop;
  }
}
