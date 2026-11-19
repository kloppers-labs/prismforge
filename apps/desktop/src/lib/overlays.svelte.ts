/** Counts open modal dialogs, and names the one open popover or menu, so the shell can keep them from overlapping. */
export const overlays = $state<{ dialogs: number; popover: string | null }>({ dialogs: 0, popover: null });
