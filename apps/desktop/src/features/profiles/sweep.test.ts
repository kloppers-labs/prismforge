import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import type { StoreView } from "../../lib/api/bindings/StoreView";
import { fakeShell } from "../../test/backend";
import { effectHandlers } from "../../test/effects";
import { appliedPartly } from "../../test/fixtures/applies";
import { detailsEvening, detailsImported, detailsOffline } from "../../test/fixtures/profileDetails";
import { savedNew } from "../../test/fixtures/saves";
import { demoConnected } from "../../test/fixtures/sessions";
import { storeEmpty, storeOffline, storeTwo, storeUnsaved } from "../../test/fixtures/stores";
import { viewConnectApplied } from "../../test/fixtures/views";
import { current, notices } from "../../test/shell";

function handlers(store: StoreView, extra: Record<string, (args: Record<string, unknown>) => unknown> = {}) {
  return effectHandlers({
    profile_store: () => store,
    profile_details: (args: Record<string, unknown>) => (args.id === "p-0000000000000002" ? detailsImported : detailsEvening),
    ...extra,
  });
}

async function openProfiles() {
  const screens = await screen.findByRole("navigation", { name: "Main" });
  await fireEvent.click(within(screens).getByRole("button", { name: "Profiles" }));
  return screen.findByRole("navigation", { name: "Profiles" });
}

test("the_selected_profile_comes_first_and_the_save_form_waits_behind_a_button", async () => {
  fakeShell(demoConnected, handlers(storeUnsaved));
  render(App);
  const list = await openProfiles();
  await fireEvent.click(within(list).getByRole("button", { name: /^Evening/ }));
  const article = await screen.findByRole("article", { name: "Evening" });

  expect(screen.queryByRole("textbox", { name: "Profile name" })).toBeNull();
  const page = screen.getByRole("main");
  const first = [...page.querySelectorAll("article, p, section, form")].find((element) => !element.closest("nav") && !element.classList.contains("visually-hidden"));
  expect(first === article || article.contains(first ?? null)).toBe(true);
});

test("save_current_lighting_opens_from_the_list_header", async () => {
  const shell = fakeShell(demoConnected, handlers(storeEmpty, { create_profile: () => savedNew }));
  render(App);
  const list = await openProfiles();

  await fireEvent.click(within(list).getByRole("button", { name: "Save current lighting…" }));
  const dialog = await screen.findByRole("dialog", { name: "Save the current lighting" });
  await waitFor(() => expect(document.activeElement).toBe(within(dialog).getByRole("textbox", { name: "Profile name" })));
  await fireEvent.input(within(dialog).getByRole("textbox", { name: "Profile name" }), { target: { value: "Night" } });
  await fireEvent.click(within(dialog).getByRole("button", { name: "Save as new profile" }));

  expect(shell.calls.filter((call) => call.command === "create_profile").map((call) => call.args)).toEqual([{ name: "Night" }]);
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
});

test("an_apply_result_shows_under_the_profile_actions", async () => {
  fakeShell(demoConnected, handlers(storeUnsaved, { apply_profile: () => appliedPartly }));
  render(App);
  const list = await openProfiles();
  await fireEvent.click(within(list).getByRole("button", { name: /^Evening/ }));
  const article = await screen.findByRole("article", { name: "Evening" });

  await fireEvent.click(within(article).getByRole("button", { name: "Apply profile" }));

  await waitFor(() => expect(within(article).getByRole("status", { name: "Profile result" }).textContent).toMatch(/applied in part/));
});

test("profiles_with_devices_not_found_today_are_marked_in_the_list", async () => {
  fakeShell(demoConnected, handlers(storeTwo));
  render(App);
  const list = await openProfiles();

  const older = within(list).getByRole("button", { name: /^Evening \(2\)/ });
  expect(older.querySelector(".tag.caution")?.textContent).toBe("2 not found today");
});

test("offline_profiles_say_so_in_one_line_with_a_way_to_connect", async () => {
  fakeShell(null, { profile_store: () => storeOffline, profile_details: () => detailsOffline });
  render(App);
  await openProfiles();

  const line = await screen.findByText(/^Not connected\. You can still rename, duplicate, delete, export and import profiles\.$/);
  await fireEvent.click(within(line.parentElement as HTMLElement).getByRole("button", { name: "Connect to OpenRGB" }));

  expect(current()).toEqual(["Devices"]);
});

test("the_apply_on_connect_notice_names_effects_by_their_labels", async () => {
  const shell = fakeShell(demoConnected, effectHandlers());
  render(App);
  await waitFor(() => expect(shell.commands()).toContain("effect_catalog"));

  await shell.pushView({ ...viewConnectApplied, session: demoConnected, revision: 50 });
  const slot = await waitFor(() => notices() ?? Promise.reject(new Error("no notices")));
  await fireEvent.click(within(slot).getByRole("button", { name: "Details" }));

  expect(slot.textContent).not.toMatch(/the effect “/);
  expect(slot.textContent).toMatch(/Rainbow wave|Solid colour|Moving highlight|Breathing/);
});
