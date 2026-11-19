export function isMac(): boolean {
  return /Mac/i.test(navigator.userAgent);
}

export function quitShortcut(): string {
  return isMac() ? "Command-Q" : "Ctrl+Q";
}

export function copyShortcut(): string {
  return isMac() ? "Command-C" : "Ctrl+C";
}

export function trayName(): string {
  return isMac() ? "the menu bar" : "the system tray";
}
