import { mount, tick } from "svelte";
import App from "../App.svelte";
import "../app.css";
import PreviewBadge from "./PreviewBadge.svelte";
import { DEFAULT_SCENARIO, SCENARIOS } from "./scenarios";
import { installTransport } from "./transport";

const params = new URLSearchParams(window.location.search);
const key = params.get("state") ?? DEFAULT_SCENARIO;
const scenario = SCENARIOS[key] ?? SCENARIOS[DEFAULT_SCENARIO]!;
installTransport(scenario);

const target = document.getElementById("app");
if (!target) {
  throw new Error("missing #app element");
}
mount(App, { target });
mount(PreviewBadge, { target: document.body, props: { current: key } });

function wait(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function button(scope: ParentNode, name: string): HTMLButtonElement | null {
  return [...scope.querySelectorAll<HTMLButtonElement>("button")].find((candidate) => candidate.textContent?.trim() === name) ?? null;
}

async function arrange() {
  await wait(150);
  const rail = document.querySelector("nav[aria-label='Main']");
  const area = params.get("area") ?? scenario.area;
  if (rail && area) {
    button(rail, area)?.click();
  }
  if (rail && (scenario.diagnostics || params.has("diagnostics"))) {
    button(rail, "Diagnostics")?.click();
  }
  if (scenario.rescan) {
    document.querySelector<HTMLButtonElement>("button[aria-controls='connection-panel']")?.click();
    await tick();
    await wait(50);
    button(document, "Rescan devices")?.click();
  }
  if (rail && scenario.area === "Effects") {
    await effects();
  }
  if (rail && scenario.area === "Setup") {
    await setup();
  }
  if (rail && scenario.area === "Settings" && scenario.scrollTo) {
    await wait(200);
    document.getElementById(scenario.scrollTo)?.scrollIntoView({ block: "start" });
  }
  if (rail && scenario.area === "Automation" && scenario.rule) {
    await wait(200);
    document.getElementById(`rule-${scenario.rule}`)?.scrollIntoView({ block: "start" });
  }
  if (params.has("open")) {
    await wait(100);
    document.querySelector<HTMLButtonElement>(`button[aria-controls='${params.get("open")}-panel']`)?.click();
  }
}

/** Checks targets, picks a group and an effect, applies or opens the live view, as the scenario asks. */
async function effects() {
  await wait(150);
  for (const key of scenario.select ?? []) {
    document.querySelector<HTMLElement>(`[data-target="${key}"]`)?.dispatchEvent(new MouseEvent("click", { bubbles: true, ctrlKey: true }));
    await tick();
  }
  if (scenario.group) {
    const row = [...document.querySelectorAll<HTMLElement>(".groups li")].find((item) => item.querySelector(".name")?.textContent === scenario.group);
    if (row) {
      button(row, "Select")?.click();
    }
  }
  await wait(50);
  if (scenario.effect) {
    const radio = [...document.querySelectorAll<HTMLInputElement>("input[type='radio']")].find((input) => input.closest("label")?.textContent?.includes(scenario.effect ?? ""));
    radio?.click();
  }
  await wait(100);
  if (scenario.groupOpen) {
    const row = [...document.querySelectorAll<HTMLElement>(".groups li")].find((item) => item.querySelector(".name")?.textContent === scenario.groupOpen);
    row?.querySelector<HTMLElement>("summary")?.click();
    await wait(50);
    row?.scrollIntoView({ block: "center" });
  }
  if (scenario.apply) {
    [...document.querySelectorAll<HTMLButtonElement>("button.primary")].find((candidate) => candidate.textContent?.startsWith("Apply"))?.click();
  }
  if (scenario.stage === "live") {
    button(document, "Live output")?.click();
  }
  if (scenario.select && !scenario.apply && !scenario.groupOpen) {
    await wait(100);
    document.getElementById("preview-title")?.scrollIntoView({ block: "start" });
  }
}

async function setup() {
  await wait(200);
  if (scenario.setupTab) {
    [...document.querySelectorAll<HTMLElement>("[role='tab']")].find((tab) => tab.textContent?.trim() === scenario.setupTab)?.click();
    await wait(50);
  }
  if (scenario.setupTemplate) {
    button(document, `Add the ${scenario.setupTemplate} template`)?.click();
    await wait(50);
  }
  for (const [index, id] of (scenario.setupSelect ?? []).entries()) {
    document.querySelector<HTMLElement>(`[data-row="${id}"]`)?.dispatchEvent(new MouseEvent("click", { bubbles: true, ctrlKey: index > 0 }));
    await tick();
  }
  if (scenario.setupSelect?.length && !scenario.scrollTo && scenario.label.includes("list twin")) {
    document.querySelector<HTMLElement>(`[data-row="${scenario.setupSelect[0]}"]`)?.scrollIntoView({ block: "start" });
  }
  if (scenario.setupEffect) {
    const select = document.getElementById("across-effect") as HTMLSelectElement | null;
    if (select) {
      select.value = scenario.setupEffect;
      select.dispatchEvent(new Event("change", { bubbles: true }));
    }
    await wait(100);
  }
  if (scenario.stage === "live") {
    button(document, "Live output")?.click();
  }
  if (scenario.setupApply) {
    [...document.querySelectorAll<HTMLButtonElement>("button.primary")].find((candidate) => candidate.textContent?.startsWith("Apply across"))?.click();
    await wait(800);
  }
  if (scenario.scrollTo) {
    await wait(100);
    const name = scenario.scrollTo;
    const region = [...document.querySelectorAll<HTMLElement>("section, aside, fieldset, [role='status']")].find(
      (each) => each.getAttribute("aria-label") === name || each.querySelector("h2, legend")?.textContent === name,
    );
    const inner = name === "Components and editor" ? document.querySelector<HTMLElement>(".inspector") : null;
    (inner ?? region)?.scrollIntoView({ block: "start" });
  }
}

void arrange();
