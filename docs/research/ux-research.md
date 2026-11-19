# PrismForge UX research guide

Access date for all sources: 2026-11-19. Items marked "(inference)" are my reasoning from a source, not something the source states.

## Summary

PrismForge controls physical lights, so most UX risk comes from one gap: what the screen shows versus what the hardware does. The research points to six moves. First, use a stable left navigation rail with icons and labels, because you have six or seven top-level areas and Microsoft recommends left navigation for 5-10 areas. Second, keep status global and always visible (connection, demo mode, active profile, unsaved changes) and keep diagnostics out of the main flow. Third, treat every multi-device action as a batch with a per-device result list and a visible revert. Fourth, make "previewing" and "live on the device" two clearly different, always-labelled states. Fifth, build the setup view as a list-plus-canvas editor with a keyboard and single-pointer path for every drag, as WCAG 2.2 SC 2.5.7 requires. Sixth, make automation a plain "When X, do Y" rule with honest wording about what happens while the app is closed. Treat all dark-theme work as WCAG 2.2 AA: 4.5:1 text, 3:1 controls and focus, 24 px minimum targets, and no more than three flashes per second in anything on screen.

---

## 1. Navigation for a desktop tool with 5-7 top-level areas

| Principle | Source URL | What PrismForge should do |
|---|---|---|
| Use left navigation for 5-10 equally important areas; use top navigation for 5 or fewer. Left navigation costs width. | https://learn.microsoft.com/en-us/windows/apps/design/controls/navigationview | Use a left rail. Planned areas (Connection, Devices, Effects, Setup, Profiles, Automation, Settings) are 6-7. Fold Connection into the status area plus a first-run state so the rail holds about six items. |
| Left navigation scales as areas are added, and icons alone are not enough. Use icon plus text, left-aligned, keyword first. Do not hide desktop navigation behind a hamburger. | https://www.nngroup.com/articles/vertical-nav/ | Every rail item shows an icon and a visible label. No hover-only labels. Do not start collapsed. |
| Icons without labels go unused. Always show a visible label. | https://www.nngroup.com/articles/icon-usability/ | If you offer a collapsed icon-only rail, make it a user choice, give each item a tooltip and an accessible name, and keep the default expanded. |
| Show the current location. NN/G calls missing location cues probably the most common menu mistake. | https://www.nngroup.com/articles/menu-design/ | Mark the active item with at least two cues (fill plus an edge bar plus bold label). Set `aria-current="page"`. Test the cue at 3:1 against the rail background (WCAG 1.4.11). |
| Windows default adaptive behaviour: expanded pane at 1008 px and wider, icon-only at 641-1007 px, menu button only at 640 px and below. | https://learn.microsoft.com/en-us/windows/apps/design/controls/navigationview | The Windows default would collapse the rail at your 920 px floor. Override it: keep the rail expanded (about 200-220 px) down to 920 px. Collapse to icon-only only below about 800 CSS px, which also covers 200% display scaling (inference). |
| Apple: do not hide the sidebar by default, allow show/hide, avoid critical actions at the bottom (windows move). | https://developer.apple.com/design/human-interface-guidelines/sidebars | Offer a collapse control. Do not put Quit or other critical actions only at the bottom of the rail. |
| App settings are the last navigation item, pinned to the bottom. | https://learn.microsoft.com/en-us/windows/apps/design/app-settings/guidelines-for-app-settings | Pin Settings at the bottom of the rail. Put Diagnostics next to it as a utility item that opens the existing drawer. |
| Landmarks help assistive technology. Use `nav` and `main`; keep landmarks few. | https://www.w3.org/WAI/ARIA/apg/patterns/landmarks/ | Wrap the rail in `<nav aria-label="Main">`. Do not reorder or hide it per state. |
| Keep terms, layout and behaviour consistent; keep users in control and avoid surprises. | https://www.cs.umd.edu/users/ben/goldenrules.html | Never add, remove or reorder rail items by state (connected, demo, no devices). Show a disabled-with-reason or an empty state inside the page instead. |
| Shortcuts serve experts (flexibility and efficiency). | https://www.nngroup.com/articles/ten-usability-heuristics/ | Add Ctrl+1 to Ctrl+6 for the main areas. Show them in tooltips. |

Where global items belong (my recommendation, built from the sources above):

- **Global status** (connection, demo, active profile, unsaved changes): one persistent strip at the top of the content area or at the rail foot. It is the same on every screen. Pair each state with text, not colour only (WCAG 1.4.1).
- **Global actions:** Settings and Diagnostics in the rail foot. Quit lives in the tray menu and in Settings (see topic 8).
- **Do not** use the nav to show state, such as badges that change which items exist.

---

## 2. Information architecture for hardware configuration

| Principle | Source URL | What PrismForge should do |
|---|---|---|
| Show only the few most important options first; put specialised options on a secondary display. More than two levels performs poorly. | https://www.nngroup.com/articles/progressive-disclosure/ | Devices: row shows name, type, status, identify. Zones, LED counts, capabilities and hardware modes sit in an expandable detail panel. Never go deeper than two levels. |
| Fewer, clearer choices speed decisions; do not simplify into abstraction. | https://lawsofux.com/hicks-law/ | Effects: show six effects as a visible list, but group the many parameters into "Basic" (colour, speed, brightness) and "More". |
| Natural mapping: controls should correspond to the physical thing they affect, for example on-screen layout that mirrors the real setup. | https://www.nngroup.com/articles/natural-mappings/ | The setup view (case and desk) is the strongest mapping tool. In the device list, show an icon of the device type and the name the user chose, with the OpenRGB name in secondary text. |
| Use domain language and match real-world conventions. | https://www.nngroup.com/articles/ten-usability-heuristics/ | Say "Strip", "Fan", "Keyboard", not controller or zone IDs. Keep raw IDs in Diagnostics only. |
| Empty states should show accurate status, teach, and give a direct path. Do not show "No devices" while still loading. | https://www.nngroup.com/articles/empty-state-interface-design/ | First run: a single "Connect to OpenRGB" step with the three outcomes (found, not running, no devices). Devices empty state offers "Rescan" and "Try demo mode". Show a progress indicator, not "none found", during the scan. |
| Show only what helps now; keep rarely used information out of the main path. | https://www.nngroup.com/articles/usability-heuristics-complex-applications/ | Diagnostics stays a drawer, opened from the rail foot or from an error's "Details" link. It never appears inline in Devices or Effects. |
| Reversible, visible, identifiable actions reduce anxiety. | https://www.cs.umd.edu/users/ben/goldenrules.html | Give every device an Identify action (flash a distinct pattern for a few seconds, then restore the prior state). Show a visible "Identifying..." state and a stop control. |

Task-first versus object-first (inference): keep **Devices** as the object-first inventory and **Effects/Profiles/Automation** as task-first areas. The **Setup** view is a spatial lens on the same objects, not a third data model. Users should be able to select the same devices from the list, the canvas, or the effect screen's target picker.

Naming and grouping: let users rename devices and create named groups (for example "Case fans"). A group is a saved set of targets, not a new device. Smart-home research suggests the abstraction an app picks shapes how users think about it (Clark, Newman and Dutta, https://web.eecs.umich.edu/~prabal/pubs/papers/clark17abstractions.pdf; not opened, snippet only). A critique of one vendor's home app says too many layered concepts (rooms, zones, groups, scenes) confuse users (snippet only). So keep PrismForge to three concepts: device, group, profile.

---

## 3. Selecting several physical targets and applying one action

| Principle | Source URL | What PrismForge should do |
|---|---|---|
| Bulk actions need a select-all control, a context-sensitive action bar, and confirmed outcomes with undo. | https://www.nngroup.com/videos/bulk-actions-design-guidelines/ | Effects target picker: checkbox list, "Select all" at the top, and an action bar ("Apply to 4 devices", "Stop") that appears once at least one target is selected. |
| A parent checkbox shows the indeterminate state when some children are selected. Users must not set indeterminate directly. | https://learn.microsoft.com/en-us/windows/apps/design/controls/checkbox | Group rows show a mixed state when only some member devices are targeted. Clicking a mixed group selects all, then clears (not mixed). |
| Multi-select listbox keys: Space toggles, Shift+Arrow extends, Shift+Space selects range, Ctrl+Shift+Home/End, Ctrl+A selects all. Set `aria-multiselectable`. | https://www.w3.org/WAI/ARIA/apg/patterns/listbox/ | Support these keys in the target list. Also keep visible checkboxes, so Shift/Ctrl click is a shortcut and never the only way. |
| Focus and selection are different and must look different. Do not make selection follow focus when focusing triggers slow work. | https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/ | Moving focus down the list must not send anything to hardware. Only Apply does. |
| Show system status and give informative feedback proportional to the action. A major action needs a substantive response. | https://www.cs.umd.edu/users/ben/goldenrules.html | After Apply, show a result list: one row per device with Applied, Skipped (another program controls it), or Failed (reason, next step). Summarise: "3 of 4 applied". |
| Errors should be specific, plain and constructive. | https://www.nngroup.com/articles/error-message-guidelines/ | Partial failure never shows a green success banner. Show a warning state with a "Retry failed" action and keep the user's selection and parameters. |
| Status messages must be announced without moving focus. | https://www.w3.org/WAI/WCAG22/Understanding/status-messages.html | Announce "3 of 4 applied, 1 failed" in a `role="status"` region. Use `role="alert"` only for failures needing attention. |
| Undo is the safer net; confirm only serious or irreversible actions. | https://www.nngroup.com/articles/confirmation-dialog/ | Applying a lighting effect is easy to reverse, so do not confirm it. Offer "Revert" (restore the previous assignments PrismForge applied) in the result bar for about 10 seconds and in the Effects header afterwards. |

Undo expectations (inference): PrismForge can reliably undo only what it knows. It cannot always read back a device's own stored state. State this honestly: "Revert restores the last effects PrismForge applied. It does not restore settings stored on the device itself."

---

## 4. Live preview and direct manipulation for parameter editing

| Principle | Source URL | What PrismForge should do |
|---|---|---|
| Direct manipulation means visible objects and immediate, reversible effects. It is weak for bulk or precise work. | https://www.nngroup.com/articles/direct-manipulation/ | Parameter changes update the preview strip with no Apply step. Keep Apply for the hardware. |
| Response times: 0.1 s feels instant; 1 s keeps flow; beyond that show progress. | https://www.nngroup.com/articles/response-times-3-important-limits/ | The strip must update within 100 ms of a change. If a hardware send takes longer, show a "Sending..." indicator on the device row. |
| Sliders suit coarse, instant-feedback adjustment. Pair them with a synced numeric field, a default tick and a reset. | https://www.nngroup.com/articles/sliders-knobs/ | Every numeric parameter (speed, brightness) gets slider plus number field plus reset-to-default. |
| Slider keyboard behaviour: arrows, Home/End, PageUp/PageDown, `aria-valuenow`, `aria-valuetext`. | https://www.w3.org/WAI/ARIA/apg/patterns/slider/ | Use native or ARIA sliders. Give `aria-valuetext` units ("40 percent", "2.5 seconds"). |
| Preview before committing when a change is broad or hard to verify. | https://www.nngroup.com/articles/user-mistakes/ | Preview strip is always local (a simulation). Hardware changes only on Apply, or in an opt-in "Live on devices" mode. |
| Mode errors happen when users forget the active mode. Use at least two visual indicators, name modes, and avoid shared controls where errors are costly. | https://www.nngroup.com/articles/modes/ | Draft (strip only) and Live (devices follow edits) get different header colour plus a text badge plus an icon. Live mode needs one explicit toggle, auto-reverts on leaving the screen, and shows "Live on 3 devices". |
| Colour-only distinctions fail users; pair with text, shape or pattern. | https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html | Colour picker shows hex and RGB or HSV fields. Name colours or show a text value. Do not depend on hue to tell "preview" from "live". |
| A colour wheel or drag must have a single-pointer alternative, for example click a position or enter numeric values. | https://www.w3.org/WAI/WCAG22/Understanding/dragging-movements.html | Colour picker offers click-to-set plus numeric fields plus a preset swatch row. |
| Auto-moving content longer than 5 seconds needs a pause, stop or hide control. | https://www.w3.org/WAI/WCAG22/Understanding/pause-stop-hide.html | The preview strip animates by nature. Add a Pause control (shows a static frame) and apply it by default when the OS asks for reduced motion. |
| `prefers-reduced-motion` reports the OS setting. | https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-reduced-motion | Honour it for UI transitions. For the strip, start paused and show a "Play preview" button. |

Debouncing (inference, not from a source): update the on-screen strip on every input event. Send to hardware at a capped rate (for example about 20-30 updates per second) with a final "trailing" send on release, so the device always ends at the last value. Test it: dragging a slider for 3 seconds must produce no more than the cap and must end at the final value.

---

## 5. Spatial and visual layout editors

| Principle | Source URL | What PrismForge should do |
|---|---|---|
| Any drag needs a single-pointer, non-drag alternative (AA). Examples: click to place, move buttons, numeric fields. | https://www.w3.org/WAI/WCAG22/Understanding/dragging-movements.html | Each item has an inspector with X, Y, rotation (0/90/180/270), mirror, scale. Add "Move" arrow buttons. Click a device, then click a canvas spot to place it. |
| All functionality must work from a keyboard, except genuinely path-dependent input. Dragging an object to a location is not exempt. | https://www.w3.org/WAI/WCAG22/Understanding/keyboard.html | Tab to each item, arrow keys nudge, Shift+Arrow nudges more, R rotates, M mirrors, Delete removes, Ctrl+Z/Ctrl+Y undo/redo. |
| Drag-and-drop needs handles, a grabbed state, a drop preview, magnetism, and arrow-key precision. Keyboard: Tab to the handle, Space to grab. | https://www.nngroup.com/articles/drag-drop/ | Use visible handles at least 24 px. Show a ghost and a snap highlight. Standard move and resize cursors. |
| Target size minimum is 24 by 24 CSS px (AA). | https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html | Rotation, resize and mirror handles are at least 24 px with spacing. Use 32 px where space allows. |
| Larger, closer targets are faster to hit. | https://lawsofux.com/fittss-law/ | Put the toolbar (select, pan, zoom, fit, snap, undo) at one edge. Do not scatter controls. |
| Example of general canvas conventions: zoom-to-fit (Shift+1), zoom-to-selection (Shift+2), modifier-wheel zoom. | https://help.figma.com/hc/en-us/articles/360041065034-Adjust-your-zoom-and-view-options | Provide Fit and Zoom to selection with shortcuts and buttons. Ctrl+wheel zooms. Show the zoom level. |
| Example: canvas operable by keyboard (arrow keys pan, keyboard box selection, toolbar focus). The page does not describe nudging selected objects. | https://help.figma.com/hc/en-us/articles/360040328653-Use-Figma-products-with-a-keyboard | A reviewer should check the editor with no mouse. Add a shortcuts panel. |
| Charts and diagrams need a text alternative with the data in a table. | https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html | The setup view always has a "List view" twin: a table of devices, positions and rotation. Make canvas items real DOM or SVG elements with names (inference), not pixels in one `<canvas>`. |
| Make the active mode obvious and name modes. | https://www.nngroup.com/articles/modes/ | Select, Pan and Place tools show the active tool with two cues. Space-hold to pan is a spring-loaded mode, which NN/G treats as low-risk. |
| Support reversal and easy exit. | https://www.cs.umd.edu/users/ben/goldenrules.html | Undo/redo for every layout edit, including template apply. Keep at least 50 steps (inference). |

Direction and order: for each strip, fan or zone, draw an arrow along the LED path and mark LED 1 with a filled dot and the number "1". Offer "Reverse direction" as a one-click action and as a mirror toggle, and show the result live on the real device. Wrong LED order is the most likely mapping error (inference).

Templates versus free layout: start with templates ("Mid tower", "Desk"), which place common shapes, then allow free editing. Mark template items as editable, not locked. Add "Reset to template".

---

## 6. Status, feedback and trust for software that controls hardware

| Principle | Source URL | What PrismForge should do |
|---|---|---|
| Visibility of system status with timely feedback. | https://www.nngroup.com/articles/ten-usability-heuristics/ | One global status strip: Connected / Connecting / Not connected / Demo, active profile, unsaved changes. Same position on every screen. |
| Indicators are passive and contextual; notifications report events; avoid noise. Do not use passive toasts for errors. | https://www.nngroup.com/articles/indicators-validations-notifications/ | Use indicators for steady state, a result panel for batch outcomes, and a persistent inline message for errors. No auto-dismissing error toasts. |
| Direct (software-driven) mode does not save to device memory. Static and some other modes may save. Built-in modes are generated by device firmware and run without the PC. Some devices fall back to built-in effects if the data stream stops. | https://gitlab.com/CalcProgrammer1/OpenRGB/-/raw/master/Documentation/Common-Modes.md | Label each effect as "Software (PrismForge drives it)" or "Hardware mode (stored on the device)". Warn that software effects stop when PrismForge closes or disconnects, device-dependent. |
| OpenRGB's "Save To Device" writes to device memory on supported controllers (release notes, via search snippet; not opened in full). | https://www.softpedia.com/progChangelog/OpenRGB-Changelog-263197.html | Treat save-to-device as a deliberate, labelled action: "Save to device memory", with a one-line consequence and a confirmation (it is the hardest action to reverse). |
| Windows lets a foreground app take control, offers a background priority list, and troubleshoots "no effect" as a higher-priority app. | https://support.microsoft.com/windows/8e8f22e3-e820-476c-8f9d-9ffc7b6ffcd2 | "Another program controls this device" must say what happened (PrismForge's changes are not visible), why (a higher-priority program holds it), and what to do (close it, or check Windows Dynamic Lighting priority). |
| Error messages: specific problem, plain language, suggest a fix, keep user input. | https://www.nngroup.com/articles/error-message-guidelines/ | Use the structure What happened / Why / What to do, in that order. Put technical codes under "Details". |
| Say that an error occurred, and suggest a correction where the system knows one. | https://www.w3.org/WAI/WCAG22/Understanding/error-suggestion.html | Same structure. Where PrismForge can offer a fix ("Restart connection"), make it a button. |
| Confirm serious or irreversible actions only; label buttons with the outcome; frequent prompts train users to click through. | https://www.nngroup.com/articles/confirmation-dialog/ | Confirm: delete profile, reset all settings, save to device memory. Do not confirm apply, stop, rename or identify. Use "Delete profile" and "Keep profile", not Yes/No. |
| In a destructive dialog, focus the least destructive button. | https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/ | Initial focus on Cancel or Keep. Escape cancels. Focus returns to the trigger. |
| Status messages need a role so assistive technology announces them. | https://www.w3.org/WAI/WCAG22/Understanding/status-messages.html | Use `role="status"` for success and progress and `role="alert"` for failures. |
| Windows overlay icons for warning, error, disabled, offline. Do not use pure red, yellow or green in base icons; reserve for status. | https://learn.microsoft.com/en-us/windows/win32/uxguide/winenv-notification | Use one status vocabulary in app and tray: connected, connecting, demo, warning, error, offline. |

Demo mode (inference): make it impossible to confuse with real. Show a persistent "Demo: no real devices" badge in the status strip, a different accent on the strip, and "(demo)" on every device name and result row. Never let demo write to a profile that real devices load without a visible marker.

---

## 7. Automation and system-event triggers

| Principle | Source URL | What PrismForge should do |
|---|---|---|
| A rule is trigger, optional condition, action. A plain-language scenario explains timing. | https://www.home-assistant.io/docs/automation/basics/ | Present rules as a sentence with dropdowns: "When [PC wakes from sleep] apply [Evening profile]". Hide conditions by default; add them under "Only if..." (progressive disclosure). |
| Overlap handling must be defined: single, restart, queued, parallel. | https://www.home-assistant.io/docs/automation/modes/ | Do not expose these modes. Pick one behaviour: the newest event wins and restarts. Say so in one line under the rule. |
| Windows sends suspend and resume power events; the API cannot tell which low-power state. | https://learn.microsoft.com/en-us/windows/win32/power/wm-powerbroadcast | Offer "PC goes to sleep" and "PC wakes up". The page gives no timing guarantee for the suspend handler, so test reliability before promising "lights off on sleep" (inference). |
| Windows sends session notifications (the API page lists registration; lock and unlock details are in the linked message page, not opened). | https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/nf-wtsapi32-wtsregistersessionnotification | Offer "PC locks" and "PC unlocks" only after verifying the event names against the WM_WTSSESSION_CHANGE page (not opened). |
| Windows Dynamic Lighting shows an explicit priority list that the user can drag. | https://support.microsoft.com/windows/8e8f22e3-e820-476c-8f9d-9ffc7b6ffcd2 | If two rules match one event, show them in an ordered list with a visible winner, not hidden precedence. Start simpler: allow one rule per event (inference). |
| Keep users in control; avoid surprises. | https://www.cs.umd.edu/users/ben/goldenrules.html | Every rule has an on/off switch, a "Test now" button, and a "Last ran" time. A history of the last 20 automatic changes sits in the Automation screen. |
| Direct-mode devices may fall back to built-in effects when software stops. | https://gitlab.com/CalcProgrammer1/OpenRGB/-/raw/master/Documentation/Common-Modes.md | Put a status line on the screen: "Rules run only while PrismForge is running (in the tray or open)". Link to the background setting. If background running is off, show a warning on the page. |

Defaults and reversibility (inference):

- No rule is on by default. First-run suggests two templates, "Lights off when PC sleeps" and "Apply my profile at startup", and the user enables them.
- Every "turn lights off" rule offers "Then restore the previous lighting when the PC wakes". Default to on.
- Move "Apply profile on connect" out of Profiles into Automation as a rule ("When PrismForge connects to OpenRGB"), so there is one place for automatic behaviour.
- Say what happens while closed, in plain words: "When PrismForge is closed, your devices show their own built-in lighting. Close-to-tray keeps rules running."

---

## 8. Background and tray applications; start with system

| Principle | Source URL | What PrismForge should do |
|---|---|---|
| Notification area is for status and notifications; keep users in control: let them remove the icon, choose notifications, suspend, and quit. Written for Windows 7; the page says much still applies. | https://learn.microsoft.com/en-us/windows/win32/uxguide/winenv-notification | Tray icon shows state (connected, demo, error overlay). Tooltip: "PrismForge, Connected, Evening profile". Do not flash or animate the icon. |
| Left click shows something useful; right click shows the menu with default command bold; menu order puts Options and Exit last; Exit ends the session and removes the icon. | https://learn.microsoft.com/en-us/windows/win32/uxguide/winenv-notification | Left click opens a small flyout: status, active profile, Pause lighting, Open. Right-click menu: Open PrismForge, Pause/Resume, Profiles, Settings, Quit PrismForge. |
| Minimising to the tray should be opt-in; the page says to use Minimize, not Close, for it. | https://learn.microsoft.com/en-us/windows/win32/uxguide/winenv-notification | Many apps use close-to-tray today, so this is a judgement call. If Close hides to tray, ask once on first close ("Keep PrismForge running in the background?") and remember the answer. Make it a Settings switch. |
| Packaged Windows startup tasks need user consent for UWP apps. Once enabled, the user controls it in Settings > Startup and Task Manager. If the user disabled it there, the app cannot override. | https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.startuptask?view=winrt-26100 | Start-with-system is opt-in, off by default. Read the real state each time Settings opens. If Windows has it disabled, show "Turned off in Windows Startup settings" with a link, not a switch that silently fails. (PrismForge uses Tauri, which is not a packaged UWP app; the page describes the OS behaviour, not your exact API.) |
| Tauri autostart plugin offers `enable`, `disable`, `isEnabled`. | https://v2.tauri.app/plugin/autostart/ | Bind the switch to `isEnabled()`, not to a stored boolean. |
| Tauri tray: menu on left and right click, click events, left-click menu can be turned off, tray events unsupported on Linux. | https://v2.tauri.app/learn/system-tray/ | Use the left-click flyout on Windows. Test the tray on each platform before shipping it. |
| macOS menu bar extras: let people, not the app, decide to show them; do not rely on them; use a menu, not a popover; also expose in the Dock menu. | https://developer.apple.com/design/human-interface-guidelines/the-menu-bar (read via Apple's data feed) | For the macOS port: tray is a Settings option, offer the choice at setup, and keep every feature reachable from the main window. |
| macOS Login Items and Extensions lets people control login items and background activity; apps may keep running after quit. | https://support.apple.com/guide/mac-help/change-login-items-extensions-settings-mtusr003/mac and https://support.apple.com/en-us/125671 | Document it. When the user quits, quit fully. Describe the setting in the help text. |

Quit discoverability: Quit appears in (1) the tray menu, (2) Settings under "Background", and (3) a keyboard shortcut shown in the tray menu. Closing the window while background running is on shows a one-time toast: "PrismForge is still running in the tray. Quit from the tray menu." Test that a new user can find Quit in under 10 seconds.

---

## 9. Settings screens

| Principle | Source URL | What PrismForge should do |
|---|---|---|
| Settings are low-frequency options. Workflow commands do not belong there. Keep few settings, smart defaults, group under headers, one column, apply immediately. | https://learn.microsoft.com/en-us/windows/apps/design/app-settings/guidelines-for-app-settings | Settings holds: Background running, Start with system, Theme, Reduce motion, About. Move "Apply on connect" to Automation and "Reset" to a danger group. |
| Add a description; disable with an explanation, not silently. Use toggles for on/off. | https://learn.microsoft.com/en-us/windows/apps/design/app-settings/guidelines-for-app-settings | Each switch has a one-line consequence below it ("Keeps your lighting rules running when you close the window"). |
| Toggles take effect immediately; label states what happens when on; do not mix toggles with submit forms. | https://www.nngroup.com/articles/toggle-switch-guidelines/ | No Save button on the Settings page. A toggle that fails (for example Windows blocks startup) snaps back and explains. |
| A switch's accessible label must not change with its state. | https://www.w3.org/WAI/ARIA/apg/patterns/switch/ | Keep the label "Start with system". Show state in the control, not by renaming. |
| Check boxes are for status that can be submitted later; switches are for immediate actions. | https://learn.microsoft.com/en-us/windows/apps/design/controls/checkbox | Use checkboxes only where a form has an Apply. |
| Confirm only serious actions; label with the outcome. | https://www.nngroup.com/articles/confirmation-dialog/ | Group "Danger zone": Reset settings, Clear all profiles, Forget devices. Each opens a dialog naming what will be removed and has focus on Cancel. |
| Use a disclosure for advanced items. | https://www.w3.org/WAI/ARIA/apg/patterns/disclosure/ | "Advanced" section (log level, OpenRGB host and port) in a disclosure with `aria-expanded`. Diagnostics stays separate. |

---

## 10. Dark theme and accessibility for a lighting app

| Principle | Source URL | What PrismForge should do |
|---|---|---|
| Text contrast: 4.5:1 normal, 3:1 large (18 pt, or 14 pt bold). Inactive controls are exempt. | https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html | Check every text and background pair in the dark palette, including secondary grey text and disabled-looking helper text. Do not round: 4.499 fails. |
| UI components and states need 3:1 against adjacent colours, including focus and selected states. | https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html | Switch tracks, slider thumbs, checkbox borders, focus rings and the active rail marker need 3:1. The page does not say whether colour swatches are exempt; give swatches a 3:1 outline (inference). |
| Colour must not be the only cue. | https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html | Colour is the content in swatches and the preview, so label them with hex or a name. Colour is state in status pills: add text and an icon. |
| Focus must not be fully hidden by author content. | https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum.html | The status strip, drawer and any sticky action bar must not cover the focused control. Test Tab through every screen with the Diagnostics drawer open. |
| Keep a visible focus indicator; selected must look different from focused. | https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/ | One focus ring style across the app, tested on every surface colour. |
| No more than three flashes in any second, or below the thresholds (Level A). Reference area 341 by 256 CSS px. | https://www.w3.org/WAI/WCAG22/Understanding/three-flashes-or-below-threshold.html | The on-screen preview strip must not flash more than three times per second, whatever the effect. Do not rely on the small-area exception. |
| The stricter AAA version bans more than three flashes per second regardless of size. | https://www.w3.org/WAI/WCAG22/Understanding/three-flashes.html (via search result; not opened in full) | Use it as the internal target for the UI. |
| Reduce animation on request. | https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html | Honour the OS setting for transitions. Offer an in-app "Reduce motion" switch too. |
| Pointer targets are at least 24 by 24 CSS px (AA), with spacing exceptions. | https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html | Icon buttons, checkboxes, canvas handles: 24 px minimum, 32 px target. |
| Reflow: usable at 320 CSS px width, but two-dimensional content such as diagrams is excepted. Text resizes to 200%. | https://www.w3.org/WAI/WCAG22/Understanding/reflow.html | Test at 200% Windows display scaling: a 920 px window becomes about 460 CSS px. The rail collapses, the setup canvas may scroll in two directions, everything else stays single column. |

Strobing effects (inference): WCAG covers web content. It does not cover physical LEDs, so there is no standard threshold for a strobe on a fan. Be conservative anyway. Cap strobe frequency at or below 3 Hz by default. Offer a "Allow fast strobing" setting that is off by default and shows a short warning. Label strobe-type effects "Flashing" in the effect list. Apply the cap to saved and imported profiles too.

---

## 11. Heuristic checklist for reviewers (run per screen)

1. **Where am I?** The rail shows the current area with two cues, and the screen title matches the rail label.
2. **Stable navigation.** No rail item appears, disappears or moves because of state. Empty and disabled states explain themselves.
3. **Global status.** The status strip shows connection, demo, active profile and unsaved changes, with text for each.
4. **Real versus simulated.** Demo mode is unmistakable on this screen, including results and previews.
5. **Software versus device.** Every effect and setting says whether PrismForge drives it or the device stores it.
6. **Preview versus live.** The screen states whether changes reach hardware now, using at least two cues.
7. **Mapping.** A user can tell which physical device a row or canvas item is (name, type, Identify, position).
8. **Batch outcome.** Multi-device actions show one result per device, with honest partial failure and a Retry or Revert.
9. **Reversibility.** Easy actions have undo or revert. Only hard-to-reverse actions (delete, save to device) ask for confirmation, with outcome-labelled buttons.
10. **Errors.** Each error says what happened, why, and what to do, and keeps the user's input. No error is a toast only.
11. **Keyboard.** Every function works without a mouse. Every drag has a single-pointer alternative (WCAG 2.5.7). Focus is visible and never hidden by the drawer.
12. **Contrast and colour.** Text 4.5:1, controls and focus 3:1, no colour-only meaning, colour swatches labelled.
13. **Motion and flashing.** The preview pauses, honours reduced motion, and never flashes more than three times per second.
14. **Targets and scaling.** Targets are at least 24 px. The screen works at 920 px and at 200% display scaling.
15. **Automatic behaviour.** Anything the app does on its own (rules, startup, tray) is visible, can be switched off, and states whether it works while the app is closed.

---

## Sources

All accessed 2026-11-19.

- https://www.nngroup.com/articles/ten-usability-heuristics/ - Nielsen's ten heuristics; status, user control, error prevention, flexibility.
- https://www.nngroup.com/articles/usability-heuristics-complex-applications/ - heuristics applied to complex apps; live preview, in-context help.
- https://www.nngroup.com/articles/vertical-nav/ - left navigation: scalability, labels, no hamburger on desktop.
- https://www.nngroup.com/articles/menu-design/ - menu guidelines, including current-location indication.
- https://www.nngroup.com/articles/icon-usability/ - icons need visible text labels.
- https://www.nngroup.com/articles/progressive-disclosure/ - progressive disclosure, two levels maximum.
- https://www.nngroup.com/articles/empty-state-interface-design/ - empty-state design.
- https://www.nngroup.com/articles/natural-mappings/ - natural mapping between controls and the real world.
- https://www.nngroup.com/articles/modes/ - mode errors; two visual indicators; spring-loaded modes.
- https://www.nngroup.com/articles/user-mistakes/ - slips versus mistakes; preview before committing.
- https://www.nngroup.com/articles/direct-manipulation/ - direct manipulation definition and limits.
- https://www.nngroup.com/articles/drag-drop/ - drag-and-drop feedback, handles, keyboard alternative.
- https://www.nngroup.com/articles/sliders-knobs/ - sliders with numeric fields; 0.1 s response.
- https://www.nngroup.com/articles/response-times-3-important-limits/ - 0.1 s, 1 s, 10 s limits.
- https://www.nngroup.com/articles/confirmation-dialog/ - when to confirm; prefer undo; wording.
- https://www.nngroup.com/articles/error-message-guidelines/ - error message structure and tone.
- https://www.nngroup.com/articles/indicators-validations-notifications/ - indicators, validations, notifications; noise.
- https://www.nngroup.com/articles/toggle-switch-guidelines/ - toggles take effect immediately; labels.
- https://www.nngroup.com/videos/bulk-actions-design-guidelines/ - three bulk-action guidelines (video summary page).
- https://www.cs.umd.edu/users/ben/goldenrules.html - Shneiderman's eight golden rules.
- https://lawsofux.com/fittss-law/ - Fitts's law summary (secondary source).
- https://lawsofux.com/hicks-law/ - Hick's law summary (secondary source).
- https://learn.microsoft.com/en-us/windows/apps/design/controls/navigationview - top versus left navigation, breakpoints.
- https://learn.microsoft.com/en-us/windows/apps/design/app-settings/guidelines-for-app-settings - settings guidance; immediate apply; settings last in nav.
- https://learn.microsoft.com/en-us/windows/apps/design/controls/checkbox - check boxes, indeterminate parent.
- https://learn.microsoft.com/en-us/windows/win32/uxguide/winenv-notification - notification area guidance (Windows 7-era guide).
- https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.startuptask?view=winrt-26100 - startup task consent and user control.
- https://learn.microsoft.com/en-us/windows/win32/power/wm-powerbroadcast - suspend and resume power events.
- https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/nf-wtsapi32-wtsregistersessionnotification - session notification registration.
- https://support.microsoft.com/windows/8e8f22e3-e820-476c-8f9d-9ffc7b6ffcd2 - Windows Dynamic Lighting controls and app priority.
- https://developer.apple.com/design/human-interface-guidelines/sidebars - Apple sidebar guidance (read through Apple's JSON data feed).
- https://developer.apple.com/design/human-interface-guidelines/the-menu-bar - menu bar extras guidance (read through Apple's JSON data feed).
- https://support.apple.com/guide/mac-help/change-login-items-extensions-settings-mtusr003/mac - macOS login items settings.
- https://support.apple.com/en-us/125671 - apps running in the background after quit.
- https://www.w3.org/WAI/WCAG22/Understanding/dragging-movements.html - SC 2.5.7.
- https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html - SC 2.5.8.
- https://www.w3.org/WAI/WCAG22/Understanding/three-flashes-or-below-threshold.html - SC 2.3.1.
- https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html - SC 1.4.3.
- https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html - SC 1.4.11.
- https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html - SC 1.4.1.
- https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum.html - SC 2.4.11.
- https://www.w3.org/WAI/WCAG22/Understanding/pause-stop-hide.html - SC 2.2.2.
- https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html - SC 2.3.3.
- https://www.w3.org/WAI/WCAG22/Understanding/status-messages.html - SC 4.1.3.
- https://www.w3.org/WAI/WCAG22/Understanding/error-suggestion.html - SC 3.3.3 and relation to 3.3.1.
- https://www.w3.org/WAI/WCAG22/Understanding/error-prevention-legal-financial-data.html - SC 3.3.4 (opened; does not require confirmation for every action).
- https://www.w3.org/WAI/WCAG22/Understanding/reflow.html - SC 1.4.10 and 1.4.4.
- https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html - SC 1.1.1, charts and diagrams.
- https://www.w3.org/WAI/WCAG22/Understanding/keyboard.html - SC 2.1.1.
- https://www.w3.org/WAI/ARIA/apg/patterns/listbox/ - multi-select keys.
- https://www.w3.org/WAI/ARIA/apg/patterns/toolbar/ - toolbar keyboard model (opened; use for the canvas toolbar).
- https://www.w3.org/WAI/ARIA/apg/patterns/slider/ - slider keyboard and ARIA.
- https://www.w3.org/WAI/ARIA/apg/patterns/switch/ - switch pattern; label must not change.
- https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/ - modal dialog focus; least destructive action.
- https://www.w3.org/WAI/ARIA/apg/patterns/disclosure/ - disclosure pattern.
- https://www.w3.org/WAI/ARIA/apg/patterns/landmarks/ - landmark roles.
- https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/ - focus management, visible focus, selection versus focus.
- https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-reduced-motion - reduced-motion media feature.
- https://help.figma.com/hc/en-us/articles/360041065034-Adjust-your-zoom-and-view-options - example of general zoom conventions only.
- https://help.figma.com/hc/en-us/articles/360040328653-Use-Figma-products-with-a-keyboard - example of keyboard-operated canvas only.
- https://www.home-assistant.io/docs/automation/basics/ - trigger, condition, action structure.
- https://www.home-assistant.io/docs/automation/modes/ - overlap modes.
- https://gitlab.com/CalcProgrammer1/OpenRGB/-/raw/master/Documentation/Common-Modes.md - OpenRGB Direct versus device modes; no flash save in Direct.
- https://v2.tauri.app/plugin/autostart/ - Tauri autostart plugin.
- https://v2.tauri.app/learn/system-tray/ - Tauri tray support.

Search-snippet only (not opened in full; used with that caveat): https://www.softpedia.com/progChangelog/OpenRGB-Changelog-263197.html, https://web.eecs.umich.edu/~prabal/pubs/papers/clark17abstractions.pdf, https://www.w3.org/WAI/WCAG22/Understanding/three-flashes.html.

---

## Open points

- **Apple HIG** Settings, Launching and Undo-and-redo pages returned only their titles (JavaScript-rendered), so they are not opened. The sidebar and menu bar pages were read through Apple's JSON data feed. Re-check against the live pages.
- **Material Design 3** (navigation rail, selection, snackbar) returned only titles. Not opened, so no Material claims are used.
- **ISO 9241-110:2020** full text is paywalled. I only saw search-result summaries (principles named in 2006 and 2020 editions). I did not cite it as a source for any recommendation. Shneiderman's rules and NN/G heuristics cover the same ground.
- **Don Norman** primary text (book, jnd.org) not opened. Mapping is cited through NN/G's summary only.
- **Smart-home and IoT HCI literature** was found only as search snippets. No peer-reviewed study of "blink to identify" for physical devices was found. The Identify recommendation rests on natural mapping and general feedback principles (inference).
- **WCAG and physical LEDs:** WCAG targets web content. No standard I found sets flash limits for physical RGB hardware. The strobe cap in topic 10 is a precaution, not a requirement. Consider asking an accessibility specialist.
- **Colour swatches under SC 1.4.11:** the Understanding page does not address them. The 3:1 swatch outline is my interpretation.
- **Suspend timing:** Microsoft's page does not state how long an app has to react before sleep. Test "lights off on sleep" on real hardware. The WM_WTSSESSION_CHANGE page (lock/unlock events) was not opened.
- **Windows tray guidance** is a Windows 7-era document. The "use Minimize, not Close" advice conflicts with common modern close-to-tray behaviour. Decide with the owner and test with users.
- **Numeric values** (10-second revert window, 20-30 sends per second, 50 undo steps, 32 px targets, 800 CSS px collapse point) are my engineering suggestions, not sourced limits. Tune them in testing.
- **Hardware-state facts** (which devices fall back to built-in effects when the stream stops, which can save to flash) vary by device. Verify per supported device through OpenRGB.
- **Fitts's and Hick's laws** are cited through lawsofux.com, a secondary summary, not the original papers.
