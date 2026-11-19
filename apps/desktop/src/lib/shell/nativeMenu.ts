import { isTextField } from "./menu.svelte";

export type Build = "production" | "development";

/** Whether the WebView's own menu (Reload, Inspect, Back) may show where no PrismForge menu opened. */
export function nativeMenu(target: EventTarget | null, build: Build): "allow" | "suppress" {
  return isTextField(target) || build === "development" ? "allow" : "suppress";
}

export function guardNativeMenu(scope: Window, build: Build): () => void {
  const listener = (event: MouseEvent) => {
    if (!event.defaultPrevented && nativeMenu(event.target, build) === "suppress") {
      event.preventDefault();
    }
  };
  scope.addEventListener("contextmenu", listener);
  return () => scope.removeEventListener("contextmenu", listener);
}

export function currentBuild(): Build {
  return import.meta.env.PROD ? "production" : "development";
}
