/** Counts open modal dialogs, so the shell can close a covering drawer before a dialog would sit under it. */
export const overlays = $state({ dialogs: 0 });
