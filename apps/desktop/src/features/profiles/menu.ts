import type { ProfileSummary } from "../../lib/api/bindings/ProfileSummary";
import type { StoreSummary } from "../../lib/api/bindings/StoreSummary";
import type { Menu, MenuItem } from "../../lib/shell/menu.svelte";
import type { ProfilesState } from "./state.svelte";

export type ProfilesMenuContext = { profiles: ProfilesState; live: boolean; applying: string | null; notConnected: string };

const WORKING = "PrismForge is still working on an earlier request.";

export function profileMenu(context: ProfilesMenuContext, profile: ProfileSummary, summary: StoreSummary): Menu {
  const { profiles, live, applying } = context;
  const id = profile.id;
  const busy = profiles.working === null ? null : WORKING;
  const marked = summary.apply_on_connect === id;
  const items: MenuItem[] = [
    {
      id: "profile.apply",
      label: "Apply profile",
      reason: applying ?? (live ? null : context.notConnected) ?? busy,
      run: async () => {
        await profiles.show(id);
        await profiles.apply(id);
      },
    },
  ];
  if (summary.active === id && live) {
    items.push({
      id: "profile.update",
      label: `Save changes to ${profile.name}`,
      reason: applying ?? busy,
      run: async () => {
        await profiles.show(id);
        await profiles.previewUpdate(id);
      },
    });
  }
  items.push(
    { id: "profile.rename", label: "Rename", run: (origin) => profiles.ask(id, "rename", origin) },
    {
      id: "profile.duplicate",
      label: "Duplicate",
      reason: busy,
      run: async () => {
        await profiles.show(id);
        await profiles.duplicate(id);
      },
    },
    { id: "profile.export", label: "Export…", run: (origin) => profiles.ask(id, "export", origin) },
    {
      id: "profile.apply-on-connect",
      label: "Apply this profile when PrismForge connects",
      checked: marked,
      run: async () => {
        await profiles.show(id);
        await profiles.setApplyOnConnect(marked ? null : id);
      },
    },
    { id: "profile.delete", label: "Delete", danger: true, reason: applying, run: (origin) => profiles.ask(id, "delete", origin) },
  );
  return { label: `Actions for ${profile.name}`, items };
}
