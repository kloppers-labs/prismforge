# 0005: The window shell and the design tokens

Date: 2026-11-19. Status: accepted. Task 7 of `docs/plans/prismforge-plan-2-experience.md`.

## Context

The owner used PrismForge for the first time and said that the navigation bar needs work. The audit (`docs/research/ux-audit.md`) measured the old top bar: controls overlap at 920 px and, with unsaved changes, at the default width of 1200 px (N1). Navigation items appeared, disappeared and were renamed with the connection state (N2). Global state was spread over up to five stacked strips (N5, G1). The Diagnostics drawer covered the header and the focused control (G2, A2). Input borders and the current-item cue failed the 3:1 contrast rule (section 6.1). There were no tokens beyond five greys (section 5).

The code is in `apps/desktop/src/lib/shell` (rail, status strip, chips, notice slot, keyboard, icons), `apps/desktop/src/lib/tokens.css`, `apps/desktop/src/lib/Notice.svelte` and `apps/desktop/src/App.svelte`.

## Decisions

1. **A left rail for the areas and one status strip for state.** This is option C of the audit (section 2.6), and decision 1 of plan 2. Microsoft recommends left navigation for five to ten areas, and the plan has seven (`ux-research.md` section 1). The rail holds only navigation. The strip sits at the top of the page, where the eye rests, and keeps its words at every width.
2. **The rail never changes with the state.** Devices, Effects and Profiles are in the main group; Diagnostics and Settings are at the foot, Settings last. The same items stay in the same order when PrismForge is connected, disconnected, retrying or in demo mode (`ux-research.md` section 1, Shneiderman). An area that needs a connection shows its own empty state with the way to connect. The current area stays marked after a disconnect.
3. **Two rail widths, labels always shown.** 184 px from 1100 px up; 72 px below, with the icon above an 11 px label. This keeps 588 px for the page at 920 px beside a 260 px list (audit 2.6). The research guide suggests a wide rail down to 920 px; the audit's measurement wins, and labels stay, so icon-only navigation never happens.
4. **The current area has three cues.** A filled background, a 3 px accent bar (7.98:1 on the rail) and a bold label. The bar and the weight do not depend on colour. The button has `aria-current="page"`.
5. **The connection form lives on Devices.** It is the not-connected state of Devices, the first area, so the first run opens on it. The connection chip's panel has "Connect to OpenRGB", which opens Devices and puts focus in the Host field, and "Start demo mode". Effects without a connection shows the same two buttons. A form hidden behind a chip would make the first step harder to find.
6. **Global state and its actions sit in the strip.** The connection chip shows the state word and the server ("OpenRGB 127.0.0.1:6742" or "Demo backend"); its panel gives the reason in words and the actions of that state: Rescan devices and Disconnect, Stop retrying, Disconnect while connecting, or Connect and Start demo mode. When closing the window keeps PrismForge running, the panel says so. The profile chip shows the active profile and its unsaved changes ("unknown" when Rust reports `null`), "Applying Evening" during an apply, and the profile set to apply on connect when none is active; its panel has Save changes and Open Profiles. Disconnect is now two steps away from Rescan, which removes the adjacency the audit found (N6).
7. **Panels follow the WAI-ARIA disclosure pattern.** The chip is a button with `aria-expanded` and `aria-controls`. Opening moves focus to the first action; arrow keys, Home and End move between actions; Escape closes and returns focus to the chip; a click outside or focus leaving closes it. Escape in a panel does not also close Diagnostics.
8. **Demo mode is marked on every screen.** A "Demo mode" badge with a stripe pattern and the words "simulated devices", plus a 3 px striped edge under the strip, replace the 39 px banner. A screen reader hears "Nothing is sent to hardware." The marker never appears for OpenRGB.
9. **One notice slot.** Command problems, a failed close or quit, connection problems, repaired files, the tray stop report and the apply-on-connect result appear in one place under the strip. Problems come first. With more than one, only the first shows, with "Show all N notices". Each one that the data allows can be dismissed. A connection problem has no Dismiss: it stays until the connection changes. Long lists (the apply report, a tray stop that stopped everything) are behind "Details".
10. **Diagnostics docks.** It opens beside the page, below the strip, so it covers neither the focused control nor the demo marker (WCAG 2.4.11). When it opens or closes, the page control that had focus is scrolled into view again. Below 1100 px it covers the page, and the page and notices become `inert`, so focus cannot land under it; choosing an area closes it.
11. **Keyboard.** Ctrl+1 to Ctrl+3 (Command on macOS) open the areas in rail order; Ctrl+, opens Settings. They do nothing while focus is in a field or a dialog is open. F6 and Shift+F6 move between the rail, the strip, the notices, the page and Diagnostics. "Skip to content" is the first Tab stop. The landmarks are `nav` "Main", the "Status" region, the "Notices" region and `main`.
12. **Design tokens in one file.** `src/lib/tokens.css` holds the roles of the audit's section 5.3: surfaces, lines (`--line-subtle` for dividers, `--line-control` for control boundaries), text (`--text`, `--text-dim`, `--text-faint`), accent and focus, state colours with their surfaces and lines, demo colours, the spectrum; spacing on a 4 px base; five type sizes plus an 11 px size for compact rail labels; three weights; radii; three shadows; two durations and one easing; four layers; control and rail sizes. Buttons, inputs, the switch, the dialog and the notice use them. Component styles use the same values by token. `--text-faint` moved to #8b91a1 and controls got `--line-control`, which clear the contrast failures of section 6.1. Button text no longer inherits a dim colour.
13. **Icons are PrismForge's own.** Five line icons on a 20 px grid with a 1.6 px round stroke, drawn for this project in `lib/shell/Icon.svelte`: a module with three LEDs (Devices), a prism that splits light (Effects), the hexagon of the mark with lines (Profiles), a trace in a frame (Diagnostics) and three sliders (Settings). Every icon has a visible label. No icon library is used.

## How new areas are added

Setup and Automation are not in the rail yet; the plan forbids placeholder areas. Each one is added in three steps:

1. Add its `Screen` value in `lib/state/app.svelte.ts` and its page in `App.svelte`.
2. Add one entry to `MAIN_AREAS` in `lib/shell/areas.ts`, in the plan's order: Devices, Effects, Setup, Profiles, Automation. The rail, the shortcuts and the tests that compare the rail with the area list follow.
3. Draw its icon in `lib/shell/Icon.svelte`.

The shortcuts follow the rail order, so Profiles moves from Ctrl+3 to Ctrl+4 when Setup arrives. The compact rail has room for both: seven items of about 54 px fit in 600 px.

## Verification

- `src/lib/shell/shell.test.ts`: 38 interface tests through the real components with only the Tauri transport mocked, on fixtures generated from Rust.
- Screenshots of the interface preview (`pnpm preview:ui`) at 1280 × 800, 920 × 600 and the 200 % equivalents, in Chromium on macOS. They are design evidence only.
- The layout has no fixed widths that force an overlap: the strip wraps its chips, the place text can shorten while the state word cannot, and the rail scrolls when the window is short.

## Not verified

- Native window chrome, minimum size and resize in Tauri.
- WebView2 rendering on Windows: Segoe UI weights, scrollbars and forced colours.
- A screen reader pass with Narrator, NVDA or VoiceOver. Announcements are checked from the markup only.
- Focus rings in the screenshots: the preview tab does not hold system focus, so `:focus-visible` is not drawn there.
- The macOS app menu changes made with this task (View and the Window menu id); Tauri's mock runtime has no native menu.
