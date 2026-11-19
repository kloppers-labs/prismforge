# PrismForge UI/UX audit

> The screenshots that this audit names are not in the repository. The names stay for reference.

Date: 2026-11-19. Scope: the Svelte frontend on `main` (commit `0397d7b`), rendered in a browser with a fake transport. Paths below are relative to `apps/desktop/src/` unless they start with another folder. Screenshots are in `screens/`; a name such as `25-…-920` means `screens/25-effects-running-920.png`.

## 1. Executive summary

1. The app is honest, careful and consistent in tone. Its main problems are layout at small sizes, a navigation bar that carries too much, and long explanatory text that pushes the primary actions off screen.
2. **The top bar breaks at the minimum width (920 px) and, with unsaved changes, at the default width (1200 px).** "Connected" is overlapped by "Refresh devices" (`25-…-920`, `62-…-1200x780`). At 920 px with an active profile, "Disconnect" is covered by "Diagnostics" and the page scrolls sideways (`10-…-920`).
3. **Navigation items change with state.** "Devices" becomes "Connection", "Effects" disappears, and after a disconnect on the Effects screen no item is marked current (`59-…-1280`).
4. **On Effects, "Apply effect" sits 1.6 screens down at 1280×800 and 2.6 screens down at 920×600.** It is the 24th to 35th Tab stop. An 84-word lifecycle paragraph sits above it on every visit (`20`, `21`, `60`).
5. **Global status is scattered over up to five stacked strips** (top bar, demo banner, repaired banner, apply-on-connect report, connection reasons). At 920×600 they take 375 of 600 px (`50-…-920`). A profile being applied has no global indicator at all.
6. **A page-level scrollbar appears on Devices when the window is shorter than about 715–775 px, or at 1280×800 with a notice strip showing**, because a visually hidden table header escapes its scroll container (`devices/DeviceDetail.svelte:84`, `app.css:122-129`).
7. Several controls mislead: enabled buttons that look disabled (`22-…-1280`), effect cards whose hover invites a click that does nothing, hardware-mode "pills" that are not buttons, and a selection bar meant to be a spectrum that renders olive.
8. Accessibility is better than average (labels, live regions, focus return, reduced motion, non-drag alternatives). It fails WCAG 2.2 AA on control-boundary contrast (1.62:1 input borders), focus obscured by the Diagnostics drawer, text at 200%, and one text pair at 4.05:1.
9. The design system is five greys, three text colours and two radii. There is no spacing, type, elevation or motion scale; eight font weights are in use and eleven colours sit outside the tokens.
10. **Recommendation:** fix the top bar now (small), then move to a left rail plus one status strip (section 2, option C), give Effects a sticky action bar and a multi-target stage (section 4), and adopt the token set in section 5.

## 2. Navigation and information architecture

### 2.1 What is there today

The header (`App.svelte:57-91`) is one non-wrapping flex row with a 24 px gap (`App.svelte:149-157`):

| Position | Element | Source |
|---|---|---|
| 1 | Brand mark and "PrismForge" | `App.svelte:58-61` |
| 2 | Segmented control: "Devices" or "Connection", "Effects" (only with devices), "Profiles" | `App.svelte:62-74` |
| 3 | Active profile button: "Profile **Evening** [4 unsaved changes]" (only when a profile is active) | `connection/ProfileStatus.svelte:9-12` |
| 4 | Connection state: tone icon, label, place; then "Refresh devices" and "Disconnect" or "Stop retrying" | `connection/SessionBar.svelte:13-29` |
| 5 | "Diagnostics" toggle | `App.svelte:83-90` |

Below the header, up to four more full-width strips stack before the content: demo banner (`App.svelte:93-95`), repaired-files banner (`:97-99`), apply-on-connect report (`:101-103`), command error (`:105-110`) and connection reasons (`:112-121`).

There is no Settings entry on `main`, no Quit, no About or version, and no keyboard shortcut for any screen (the only key handlers are Escape and Enter in the dialog, drawer and rename fields).

### 2.2 Findings

**N1. The header overflows at the minimum width, and in one common case at the default width. (P0)**
The state block may shrink to nothing (`connection/SessionBar.svelte:40-46`), but its 74 px label cannot (`:48-51`), so the label slides under the next button. Measured in the rendered page:

| Header content | Label overlapped by "Refresh devices" | Clear from about |
|---|---|---|
| Connected, no active profile | 15 px at 920 px (`25-…-920`, `09-…-920`) | 935 px |
| Active profile | 46 px at 1040 px | 1090 px |
| Active profile with unsaved changes | **17 px at 1200 px, the default window width** (`62-…-1200x780`) | 1220 px |

- At 920 px with an active profile it gets worse: "Refresh devices" wraps to two lines, the header grows from 58 to 79 px, "Disconnect" ends at x = 928 in a 905 px header and "Diagnostics" is drawn on top of it. The document gains a horizontal scrollbar. See `10-…-920`, `15-…-920`, `38-…-920`, `50-…-920`.
- At 1280 px with unsaved changes the place text is cut to "the …", which says nothing (`10-…-1280`).
- The Settings entry that is in review adds a fourth nav item of about 85 px. That moves every threshold above up by the same amount.

**N2. Navigation items appear, disappear and rename. (P1)**
"Devices" turns into "Connection" and "Effects" is removed when there is no inventory (`App.svelte:63-70`). When the session ends while the user is on Effects, `app.screen` stays `"effects"`, the Effects item is gone, and neither remaining item has `aria-current` (`59-…-1280`; verified in the DOM: `Connection:null, Profiles:null`). After reconnecting, the user lands back on Effects without having chosen it. The research guide asks for the opposite: never add, remove or reorder items by state; show an empty state inside the page (ux-research.md §1, Shneiderman).

**N3. Current location has one weak cue. (P1)**
The current item differs by background `--ink-3` on `--ink-0` (1.40:1) plus a text colour change (`App.svelte:191-201`). WCAG 1.4.11 asks 3:1 for the visual information that identifies a state. The pages themselves have no title that repeats the nav label: Devices and Profiles show an object name as `h2`; only the empty Effects page says "Effects" (`18-…-1280`), and that heading is dimmed by inheritance (`effects/EffectsScreen.svelte:88-92`).

**N4. The segmented control looks like tabs but is a nav. (P2)**
The markup is right for navigation (`<nav aria-label="Screens">`, buttons, `aria-current="page"`). The visual is a segmented toggle, which users read as "views of one thing". With six or seven areas it will not hold. Keep the nav semantics; change the visual.

**N5. Global status is spread across five places and two vocabularies. (P1)**
- Connection: icon + label + place in the header, and the reasons again in a strip below (`07-…-920`). The label "Retrying, retry 3 next" (`connection/wording.ts:42`) is awkward and has no countdown.
- Demo mode: a 39 px striped banner on every screen (`connection/DemoBanner.svelte`). It is unmistakable, which is right (ux-research.md §6), but it costs 6.5% of a 600 px window for a state that never changes during a session.
- Active profile and unsaved changes: a header button, a tag in the profile list, and a sentence in the profile article. Good redundancy, but the header version is the first thing to be squeezed out.
- Profile being applied: **no global indicator**. `view.applying` only shows as "… is off" lines next to buttons far down each page (`33-…-1280`).
- The apply-on-connect report is a full-width strip above every screen, with up to six lines and a Dismiss button (`49-…-1280`). At 920×600 the stack leaves 215 px for content (`50-…-920`).

**N6. Global actions sit beside status with equal weight. (P1)**
"Refresh devices" and "Disconnect" are two default buttons in the header. Disconnect ends the session and, by the app's own text, stops every PrismForge effect. It is one click, has no undo and sits 8 px from Refresh (`connection/SessionBar.svelte:19-28`). "Diagnostics" is a rare support action that owns the most valuable corner of the window.

**N7. No shortcuts and a long Tab path. (P1)**
There are 6–7 Tab stops in the header before content, then one stop per device and per zone. "Apply effect" is Tab stop 24 (Rainbow wave) to 35 (Colour cycle). There is no skip link, no F6 pane cycling, no Ctrl+1…n.

**N8. Devices and Effects duplicate the device list. (P1, structural)**
Both screens are list-plus-detail with the same devices on the left (`devices/Inventory.svelte:26`, `effects/EffectsScreen.svelte:64`). Selection is not shared: choosing a device on Devices does not preselect it on Effects, and there is no "Set lighting" link from a device. Hardware modes are shown read-only on Devices (`devices/DeviceDetail.svelte:135-151`) and as actions on Effects (`effects/HardwareModes.svelte`). The global frame rate lives at the bottom of every Effects target page (`effects/EffectsScreen.svelte:56`).

**N9. Screen reader behaviour is mostly sound. (keep)**
One `nav` for screens, one `nav` for the list, one `main`; `aria-current` on the current screen and list row; the connection state is `aria-live="polite"`; the Diagnostics button has `aria-expanded`. Gaps: the demo banner is an `aside`, so it is not announced when demo mode starts; the Diagnostics button has no `aria-controls`; "Profile Evening 4 unsaved changes" is read as one run with no separator.

### 2.3 What the navigation must hold

Areas: Devices, Effects (soon multi-target with a live stage), Setup (case view and desk view), Profiles, automatic triggers (sleep, wake, startup, shutdown, and today's apply-on-connect), Settings (in review: background on close, start with the system, start hidden, connect at start), Diagnostics (drawer).
Global status: connection, demo mode, active profile, unsaved changes, profile being applied, and soon "running in the background" and "live on N devices".
Global actions: connect, disconnect, rescan, diagnostics, settings, quit.

That is six or seven areas. Microsoft's guidance is top navigation for five or fewer and left navigation for five to ten (ux-research.md §1). I agree, with one change described in option C.

Two naming notes that apply to every option:

- **"Setup" next to "Settings"** will be misread; both start with "Set" and both sound like configuration. Call the spatial area **Layout** (tabs inside: Case, Desk).
- **"Connection" should not be a destination.** It is a state plus a small form. Put it in the status area, in the empty state of Devices, and in Settings ("Connect at start", host and port).

### 2.4 Option A — left rail with a status foot

Everything global lives in the rail. The content area has no top bar.

Desktop, 1280 px (rail 208 px):

```
+----------------------+---------------------------+---------------------------------------------+
| (P) PrismForge       | Devices                 5 | Demo Motherboard            [Identify] [..] |
|                      | > Demo Memory Module      | Motherboard · 52 LEDs · 3 zones             |
| [#] Devices        < |   Demo Memory Module      |                                             |
| [~] Effects          |   Demo Motherboard        |  ...page content...                         |
| [=] Layout           |   Desk card               |                                             |
| [*] Profiles         |   Demo Light Bar          |                                             |
| [>] Automation       |                           |                                             |
|                      |                           |                                             |
| -------------------- |                           |                                             |
| (o) Connected        |                           |                                             |
|     127.0.0.1:6742   |                           |                                             |
| [*] Evening · 4 uns. |                           |                                             |
| [DEMO MODE]          |                           |                                             |
| -------------------- |                           |                                             |
| [i] Diagnostics      |                           |                                             |
| [o] Settings         |                           |                                             |
+----------------------+---------------------------+---------------------------------------------+
```

Minimum, 920 px (rail stays 200 px so the status text survives):

```
+--------------------+----------------------+--------------------------------+
| (P) PrismForge     | Devices            5 | Demo Motherboard               |
| [#] Devices      < | > Demo Memory Mod..  | Motherboard · 52 LEDs          |
| [~] Effects        |   Demo Memory Mod..  |                                |
| [=] Layout         |   Demo Motherboard   |  460 px left for the page      |
| [*] Profiles       |   Desk card          |                                |
| [>] Automation     |                      |                                |
| ------------------ |                      |                                |
| (o) Connected      |                      |                                |
| [*] Evening · 4    |                      |                                |
| [i] Diagnostics    |                      |                                |
| [o] Settings       |                      |                                |
+--------------------+----------------------+--------------------------------+
   200                  260                    460
```

Pros: one place for everything global; no vertical cost; closest to the research guide's wording (expanded rail down to 920). Scales to ten areas.
Cons: at 920 px the work area is 460 px wide once the 260 px list is kept; the effect picker and the future Layout canvas suffer most. If the rail collapses to icons to win width back, the status foot loses its text, which breaks "pair each state with text" (WCAG 1.4.1). Status at the bottom-left is far from where the eye rests, and Apple's guidance warns against critical items at the bottom of a sidebar because windows move off screen (ux-research.md §1).

### 2.5 Option B — refined two-row top bar

Row 1 is status and utilities. Row 2 is navigation tabs. No rail.

Desktop, 1280 px:

```
+------------------------------------------------------------------------------------------------+
| (P) PrismForge   (o) Connected · 127.0.0.1:6742 v   [*] Evening · 4 unsaved v   DEMO   [i] [o] |
+------------------------------------------------------------------------------------------------+
|  Devices     Effects     Layout     Profiles     Automation                                    |
|  =======                                                                                       |
+---------------------------+--------------------------------------------------------------------+
| Devices                 5 | Demo Motherboard                              [Identify] [...]     |
| > Demo Memory Module      |  ...page content...                                                |
+---------------------------+--------------------------------------------------------------------+
```

Minimum, 920 px:

```
+------------------------------------------------------------------------+
| (P)  (o) Connected v     [*] Evening · 4 v      DEMO         [i] [o]   |
+------------------------------------------------------------------------+
| Devices   Effects   Layout   Profiles   Automation                     |
| =======                                                                |
+----------------------+-------------------------------------------------+
| Devices            5 | Demo Motherboard                                |
| > Demo Memory Mod..  |  640 px left for the page                       |
+----------------------+-------------------------------------------------+
```

Pros: smallest change from today; no width cost, which helps the two-pane screens and the Layout canvas; five tabs fit at 920 px (about 500 px).
Cons: two rows cost about 84 px, 14% of the 600 px minimum height, before any notice. Top navigation is recommended for five or fewer areas, and this is already at five with Settings and Diagnostics pushed into icon-only buttons (icons without labels go unused, ux-research.md §1). A sixth area does not fit at 920 px. It keeps the "tabs" reading that N4 describes.

### 2.6 Option C — hybrid: left rail for areas, one status strip for state (recommended)

The rail holds navigation only, with Settings and Diagnostics pinned at its foot. One 40 px strip at the top of the content holds all global state as two chips that open small panels. The rail has two widths, and labels stay visible in both.

Desktop, 1280 px (rail 184 px, icon beside label):

```
+-------------------+-----------------------------------------------------------------------------+
| (P) PrismForge    | (o) Connected · OpenRGB 127.0.0.1:6742 v | [*] Evening · 4 unsaved v | DEMO |
|                   +---------------------------+-------------------------------------------------+
| [#] Devices     < | Devices                 5 | Demo Motherboard             [Identify] [...]   |
| [~] Effects       | > Demo Memory Module      | Motherboard · 52 LEDs · 3 zones                 |
| [=] Layout        |   Demo Memory Module      |                                                 |
| [*] Profiles      |   Demo Motherboard        |  ...page content...                             |
| [>] Automation    |   Desk card               |                                                 |
|                   |   Demo Light Bar          |                                                 |
|                   |                           |                                                 |
| [i] Diagnostics   |                           |                                                 |
| [o] Settings      |                           |                                                 |
+-------------------+---------------------------+-------------------------------------------------+
   184                 260–320                     at least 776
```

Minimum, 920 px (rail 72 px, icon above an 11 px label):

```
+--------+---------------------------------------------------------------+
|  (P)   | (o) Connected v     | [*] Evening · 4 unsaved v     |  DEMO   |
|        +----------------------+----------------------------------------+
|  [#]   | Devices            5 | Demo Motherboard      [Identify] [..]  |
| Devices| > Demo Memory Mod..  | Motherboard · 52 LEDs                  |
|  [~]   |   Demo Memory Mod..  |                                        |
| Effects|   Demo Motherboard   |  588 px left for the page              |
|  [=]   |   Desk card          |                                        |
| Layout |   Demo Light Bar     |                                        |
|  [*]   |                      |                                        |
|Profiles|                      |                                        |
|  [>]   |                      |                                        |
|  Auto  |                      |                                        |
|  [i]   |                      |                                        |
|  Diag  |                      |                                        |
|  [o]   |                      |                                        |
|Settings|                      |                                        |
+--------+----------------------+----------------------------------------+
   72       260                    588
```

What goes where:

| Item | Place | Behaviour |
|---|---|---|
| Devices, Effects, Layout, Profiles, Automation | Rail, top group, in this order (inventory → light it → place it → save it → automate it) | Always present, always in the same order. Without a connection each page shows its own empty state with a "Connect" button. Profiles works offline, as it does today. |
| Diagnostics, Settings | Rail foot, Settings last (ux-research.md §1, §9) | Diagnostics opens the existing drawer. Settings is a page. |
| Connection | Status strip, chip 1: tone icon + state word + server | Click opens a panel: state and reason in one sentence, next retry countdown, **Rescan devices**, **Disconnect** or **Stop retrying**, **Change server…**. Replaces the two header buttons and the reasons strip. |
| Active profile, unsaved changes, "applying…" | Status strip, chip 2 | Click opens a panel: **Save changes…**, **Apply another profile ▸**, **Open Profiles**. While a profile applies the chip shows "Applying Evening…" with a progress mark. |
| Demo mode | Status strip, right end: a fixed "DEMO" badge with the stripe pattern, plus a 3 px striped edge under the strip | Still on every screen and still unmistakable, at 3 px instead of 39 px. The full sentence moves to the chip's tooltip and to the connection panel. Add "(demo)" to result rows as the research guide suggests (§6). |
| Notices (repaired files, apply-on-connect report, command errors) | One notice slot under the strip, one line each, newest first, with "Details" and "Dismiss"; more than one collapses to "2 notices" | Errors stay until dismissed. The apply-on-connect report opens in the Profiles result panel instead of printing six lines over every screen. |
| Quit, background state | Tray menu and Settings (from the branch in review); the connection panel shows "Running in the background is on" | Do not put Quit in the rail. |
| Layout sub-views | Tabs inside the Layout page: Case, Desk | A real tablist (arrow keys), because these are views of one thing. |

Keyboard and screen reader behaviour: `<nav aria-label="Main">` with one focusable item per area and `aria-current="page"`; Ctrl+1…5 for the areas, Ctrl+, for Settings, F6 and Shift+F6 to cycle rail → list → page; a skip link "Skip to content" as the first Tab stop; the status strip is a `role="status"` region so changes are announced once; the page `h1` repeats the rail label.

Current-location cue: filled background, a 3 px accent bar on the inner edge, and the label in the strong text colour and weight. The bar carries the 3:1 contrast; the fill does not need to.

Pros:
- Holds seven areas with room for more. Items never move.
- Status keeps its words at every width, in the place users already look (top of the content).
- Vertical cost drops from 58 px (97 px in demo mode) to about 43 px.
- At 920 px the page keeps 588 px, against 460 px in option A.
- Global actions move behind the state they belong to, which removes the Refresh/Disconnect adjacency (N6).
Cons:
- Largest change: new shell, about ten icons to draw, popover panels to build and test with a keyboard.
- Two-step access to Disconnect and Rescan. That is acceptable for rare actions; add Ctrl+R for Rescan.
- The compact rail's 11 px labels are small. They must use the strong text colour and a 72 px width so "Automation" can shorten to "Auto" only below 920 px equivalents.

Where this differs from the research guide: the guide says keep the rail expanded (200–220 px) down to 920 px and collapse to icon-only below about 800 px (§1, marked there as inference). I recommend a labelled compact rail from below 1100 px instead. The reason is measured: every main screen has a second, 260 px list pane, so an expanded rail leaves 460 px for the page at 920 px. The compact rail keeps visible labels, so the guide's real concern (icon-only navigation goes unused) is still met.

### 2.7 Recommendation

**Choose option C.** Do it in two steps:

1. **Now, on the current top bar (size S):** let the place text drop first and the state label never shrink below its width; move "Refresh devices", "Disconnect" and "Diagnostics" into one "⋯" menu below 1100 px, or wrap the header into two rows; keep "Devices", "Effects", "Profiles" always present with disabled-with-reason or empty states; reset `app.screen` to `"devices"` when the session ends.
2. **With the Settings and Layout work (size L):** build the rail and status strip. Land it before adding Settings as a fourth top-bar item, because that item makes N1 worse.

## 3. Screen-by-screen findings

### 3.1 Connection (first run, connecting, failed)

Screenshots: `01`, `02`, `03`, `04`, `05`, `06`, `08`, `58`.

What works: one clear heading, sensible defaults (127.0.0.1, 6742), one primary button, a separate dashed card for demo mode, a specific error for an old server ("uses SDK protocol 5. PrismForge needs protocol 6", `05-…-920`).

| ID | Finding | Evidence | Recommendation | Pri |
|---|---|---|---|---|
| C1 | At 920 px the two cards stack and **demo mode drops below the fold**. A first-time user without OpenRGB running sees no way forward. | `01-…-920`; breakpoint at `connection/ConnectionScreen.svelte:199-203` | Keep two columns down to 920 px (about 500 + 335 px inside the page padding), or put "Try demo mode" as a secondary button beside "Connect". | P1 |
| C2 | While connecting or retrying, the page has **no action**. "Stop retrying" exists only in the top bar; there is no "Change address" and no "Try demo mode". | `03`, `04`; `connection/WaitingPanel.svelte:14-48` | Add the buttons to the panel: "Stop and change address", "Start demo mode". Show a countdown to the next retry. | P1 |
| C3 | "Retrying, retry 3 next" reads oddly, and the body repeats it: "Retry 3 comes next, with a longer wait before each retry (at most 30 seconds)". | `04`; `connection/wording.ts:42-46` | Label: "Not connected — retrying". Body: "Nothing answered at 127.0.0.1:6742. Next try in 12 s (try 3)." | P2 |
| C4 | The prerequisites paragraph is 54 words of 13 px faint text. It shows on the connection screen and in both waiting states. It mixes "what you need" with "what PrismForge will never do". | `01`, `03`, `04`; `connection/Prerequisites.svelte:8` | Keep the three bullets. Move the promise to one line with a "What PrismForge writes, and when" disclosure. | P2 |
| C5 | The inline code `--server` breaks across two lines at 920 px. | `01-…-920`; `connection/Prerequisites.svelte:5` | `white-space: nowrap` on `code`. | P2 |
| C6 | Field errors look like help. With an invalid port the help line changes its words but stays faint grey; only the 1 px border turns red; the button just disables. | `connection/ConnectionScreen.svelte:71-82`, `:186-193` | Give the message the problem colour and an icon, and announce it (`role="alert"` or `aria-live`). | P2 |
| C7 | "Connect" becomes "Connecting" with no motion or ellipsis; the disabled look is the only cue. | `02-…-1280` | Add a spinner or progress mark; the sweep in `WaitingPanel` shows the pattern already exists. | P2 |
| C8 | The global command-error strip has no dismiss and no icon text; it stays until the next command. | `06-…-1280`; `App.svelte:105-110` | Put command errors in the notice slot with "Dismiss", next to the control that caused them where possible. | P2 |
| C9 | No remembered server, no "connect at start" on `main`. | — | The Settings branch adds "connect at start"; show that choice here too, as a checkbox under the form. | P2 |

### 3.2 Devices (inventory and detail)

Screenshots: `09`, `10`, `11`, `12`, `13`, `14`, `15`, `16`, `17`, `07`, `53`.

What works: clear list rows, honest notes ("Simulated device. There is no hardware behind it."), Identify explains itself once and then folds into "How identify works" (`13-…-1280`), results state what PrismForge cannot confirm.

| ID | Finding | Evidence | Recommendation | Pri |
|---|---|---|---|---|
| D1 | **A document-level scrollbar appears on Devices.** The hidden "Name" column header is `position: absolute` inside a scroll container that is not positioned, so it extends the page. The document measured 774 px tall in a 600 px window (713 px at 1280 wide); the whole app, header included, then scrolls and a blank band follows. It happens whenever the window is shorter than that, and also at 1280×800 once a notice strip pushes the page down. Hiding the span in the DOM returned the document to 600 px. | `devices/DeviceDetail.svelte:84`, `:168-175`; `app.css:122-129`. Captures leave scrollbars out, so affected files are 15 px narrower: `09-…-920` and `11-…-920` are 905 px wide, `49-…-1280` is 1265 px wide. The ambiguous device has no rename column and is not affected (`16-…-920` is 920 px wide). | Add `position: relative` to the scrolling panes, and `overflow: hidden` on `body`. | P1 |
| D2 | The detail pane's **scrollbar sits in the middle of the window** at 1280 px, with a dead band to its right, because the scroll container itself has `max-width: 880px`. The same happens on Effects and Profiles (920 px). | `09-…-1280`, `20-…-1280`; `devices/DeviceDetail.svelte:168-175`, `effects/EffectsScreen.svelte:69-76`, `profiles/ProfilesScreen.svelte:108-115` | Let the pane fill the width and scroll; put the max-width on an inner wrapper. | P1 |
| D3 | Two "Demo Memory Module" rows are identical in name and meta. Nothing but position tells them apart. | `09-…-1280`; `devices/DeviceList.svelte:22-25` | Add a distinguishing detail when names collide (location, serial tail, or "1 of 2"), and prompt "Rename" after Identify. | P1 |
| D4 | The selection bar is a horizontal six-colour gradient squeezed into 3 px; it renders as **olive green**, not a spectrum. Same code in four files. | `09-…-1280`; `devices/DeviceList.svelte:97-107`, `effects/TargetList.svelte:156-166`, `effects/EffectPicker.svelte:62-72`, `profiles/ProfileList.svelte:112-122` | Use one accent colour, or a vertical gradient (`180deg`). Make it a shared class. | P1 |
| D5 | There is **no path from a device to its lighting**. The primary button on the page is "Identify device". To set an effect the user switches screen and finds the device again. | `11-…-1280` | Add "Set lighting" (opens Effects with this device selected) to the device header. Show the current owner and effect here. | P1 |
| D6 | Hardware modes are drawn as rounded pills with borders. They look like buttons or chips and do nothing. The `h3` "Hardware modes" is nested inside "What PrismForge can do". | `devices/DeviceDetail.svelte:135-151`, `:263-287` | Plain list or table (mode, Active, Direct). Give it its own section. | P2 |
| D7 | Button labels repeat the object: "Rename zone Addressable Header 1", "Identify zone Addressable Header 2". They wrap and dominate the table. | `13-…-1280`, `11-…-1280`; `devices/NameEditor.svelte:29`, `devices/IdentifyPanel.svelte:63` | Put "Identify" and "Rename" as short buttons in each zone row; keep the long form as `aria-label`. | P2 |
| D8 | "Last reported colours" is a 14 px strip of 18 px squares, left-aligned; for 8 LEDs it is 160 px wide. It has no text alternative (`aria-hidden`). An unavailable device shows black squares that read as "off". | `09-…-1280`, `07-…-920`; `devices/LedStrip.svelte:6-9`, `:25` | Group by zone, label each zone, add a text summary ("8 LEDs, all #ff6a2a"). For unavailable devices show a hatched placeholder. | P2 |
| D9 | The ambiguous-device note says a fix "comes in a later version". That is honest, and it reads as unfinished. | `16-…-1280`; `devices/DeviceDetail.svelte:60-65` | Offer what can be done today: "Unplug one of the two, then Rescan" or "Give one a different name in OpenRGB". | P2 |
| D10 | The list has one Tab stop per device and no arrow-key movement. With 20 devices that is 20 stops before the page. | `devices/DeviceList.svelte:19-34` | Roving tabindex with Up/Down, Home/End; keep `aria-current`. | P2 |

### 3.3 Effects

Screenshots: `18`–`33`, `54`, `56`, `57`, `60`. Section 4 holds the redesign.

What works: every effect has a one-line description; every control has a range and unit; sliders have number fields; palette order uses buttons, not drag; the preview says plainly "It is not what the device shows"; blocked states say why.

| ID | Finding | Evidence | Recommendation | Pri |
|---|---|---|---|---|
| E1 | **The primary action is far below the fold.** Order: status, picker, settings, preview, an 84-word paragraph, then "Apply effect". Measured: 1140 px down in a 704 px pane at 1280×800; 1335 px down in a 504 px pane at 920×600. | `20`, `21`, `60`; `effects/EffectEditor.svelte:40-122` | Sticky action bar at the bottom of the pane. See section 4. | P0 |
| E2 | The lifecycle paragraph (computed by PrismForge, stops on close, save as profile, saved on this computer…) shows every time, above the buttons. It will also be **wrong** once "background on close" ships: it says "Closing the window quits PrismForge". | `21-…-1280`; `effects/EffectEditor.svelte:64-70` | One line under the bar: "Runs while PrismForge is running and connected. [How effects run]". Full text in a disclosure, shown open on first apply only (the Identify panel already does this). Build the text from the real background setting. | P1 |
| E3 | **Two enabled buttons look disabled.** "Another program controls this device" and "Set frame rate" inherit the dim text colour of their sections. | `22-…-1280` (compare "Use Static"); computed colour `rgb(169,174,187)` against `rgb(236,233,226)`; `effects/ExternalControl.svelte:82-86`, `effects/FrameRate.svelte:76-80`, `app.css:70` | Set `color: var(--text)` on `button`, not `inherit`. | P1 |
| E4 | **Effect cards have a false affordance.** The whole card highlights on hover, but only the radio and the name are clickable. Clicking the description or the card corner does nothing (verified in the DOM). | `effects/EffectPicker.svelte:15-28`, `:53-55` | Wrap the whole card in the `label`. | P1 |
| E5 | **Auto-apply is a hidden mode.** When the chosen effect already runs on the target, edits go to hardware after 300 ms. Otherwise they wait for Apply. After leaving the screen the target silently needs Apply again. The only cue is one hint line beside the buttons, off screen while editing. | `26-…-1280`; `effects/EffectEditor.svelte:71-77`; `effects/state.svelte.ts` (`canAutoApply`, `leave`) | Name the mode and show it twice: a "Live on 1 device" badge in the sticky bar and a tinted bar edge; otherwise "Draft — devices unchanged" (ux-research.md §4, modes). | P1 |
| E6 | Feedback points elsewhere: "PrismForge accepted Rainbow wave… The Output line above shows whether it runs." The Output line is at the top of a 1900 px page. | `26-…-1280`; `effects/EffectEditor.svelte:111` | Show the state in the bar: "Running on Demo Graphics Card". | P1 |
| E7 | "Owner" is internal vocabulary. The target list prints "Owner: none" under every device; rows read "Owner: PrismForge effect, refused", "Owner: OpenRGB Effects plugin, as you said". | `18-…-1280`, `25-…-920`; `effects/present.ts:126-140`, `:149-161` | Say "Controlled by". Show nothing when nothing controls the device. Use a small state mark per row (running, paused, refused, other program). | P1 |
| E8 | A device handed to another program shows two "… is off" lines; the fix ("Hand back") is a screen further down. | `29-…-1280`, `30-…-1280`; `effects/present.ts:82-85` | Put "Hand back to PrismForge" inside the blocked notice. Disable or hide the effect picker while blocked. | P1 |
| E9 | "Another program controls this device" is a sentence used as a button, and "say so here" is unusual wording. | `22-…-1280`; `effects/ExternalControl.svelte:58-68` | Button: "Leave this device to another program". Heading: "Used by another program". | P2 |
| E10 | Hardware modes, another program and frame rate are **not part of choosing an effect**, yet they extend every target page by about 700 px. Frame rate is global. | `22-…-1280`, `60` | Move hardware modes and other-program control to the device ("Control" section, reachable from both screens). Move frame rate to Settings → Output. | P1 |
| E11 | The colour field shows the hex twice (input and an `output` beside it). | `23-…-1280`; `effects/controls/ColorField.svelte:37-49` | Drop the echo; use the space for recent colours. | P2 |
| E12 | A palette row is label, swatch, hex, echo, Up, Down, Remove. Eight colours make a 600 px column. | `23-…-1280`; `effects/controls/PaletteField.svelte:47-59` | One row of swatches with a selected-swatch editor below; keep Up/Down as "Move left/right" buttons (no drag needed, WCAG 2.5.7). | P2 |
| E13 | "Solid colour" starts with no colour, so the first thing the user sees is a red "Required" error, a grey preview and a disabled Apply. | `24-…-1280`; the catalog gives this colour no default (`test/fixtures/catalog.ts`) | Default to a visible colour (the last one used, or white). | P1 |
| E14 | The preview is a single line of LEDs for every zone shape. Rings and grids read as strips. | `21-…-1280`; `effects/PreviewStrip.svelte` | Draw by zone shape (the data is already in `zoneShape`). Section 4. | P2 |
| E15 | The empty state's heading is dim and the page shows only the global frame rate under it. | `18-…-1280`; `effects/EffectsScreen.svelte:51-56`, `:88-92` | A real empty state: what to do, plus "Select all lightable devices". | P2 |
| E16 | No reset-to-default per control, no presets, no "copy settings from another target". | — | Add reset per control (ux-research.md §4, sliders). | P2 |

### 3.4 Profiles

Screenshots: `34`–`48`, `61`.

What works: the list rows say what a profile holds and how many devices are found today; destructive dialog focuses "Keep profile"; the update dialog lists exact changes before saving (`40-…-1280`); the export dialog says what the file contains; the partial-apply report is specific per device (`39-…-1280`).

| ID | Finding | Evidence | Recommendation | Pri |
|---|---|---|---|---|
| P1 | **The selected profile starts 387 px down** (449 px at 920×600, in a 483 px pane). Above it: 110 words in two notes and the "Save the current lighting" form, which is about a different object. | `37-…-1280`, `61`; `profiles/ProfilesScreen.svelte:50-85` | Selected profile first. Move "Save current lighting as…" to a button in the list header and to the profile chip. | P1 |
| P2 | Results appear at the top of the page, away from the button that caused them, and can be off screen when the page is scrolled. | `39-…-1280`; `profiles/ProfilesScreen.svelte:71-72` | Show the result under the action row of the profile, and scroll it into view. | P1 |
| P3 | The apply-on-connect status is a paragraph on every visit: "No profile was marked when PrismForge connected, so none was applied by itself. A profile you mark now is not applied by itself in this session. You can still apply it yourself." | `35-…-1280`; `profiles/present.ts` (`onConnectText`) | Show it only next to the switch, and only when it changes what the switch will do. Later, move the whole feature to Automation as a rule (ux-research.md §7). | P1 |
| P4 | "Apply this profile when PrismForge connects" has two hint paragraphs that repeat each other ("It does not start PrismForge or connect by itself" / "You start PrismForge and connect it yourself"). The switch also shows an "On/Off" chip beside the track. | `37-…-1280`; `profiles/ApplyOnConnect.svelte:8-14` | One hint line. Rewrite when "start with the system" and "connect at start" land, because the two sentences become false. | P2 |
| P5 | Six equal buttons in a row: Apply, Save changes, Rename, Duplicate, Export, Delete. Delete sits beside Export with the same look. | `36-…-1280`; `profiles/ProfileArticle.svelte:111-135` | Apply (primary), Save changes (secondary), then a "More" menu with Rename, Duplicate, Export, Delete (Delete last, separated). | P2 |
| P6 | Skipped settings are explained twice with different words ("Skipped today. Identical devices…" in the list, "Skipped. Identical devices…" in the result). At 920 px the two-column row wraps each reason to four lines. | `38-…-920`, `39-…-1280`; `profiles/SettingsList.svelte:62-68` | One status word per row with the reason below it; fix what can be fixed from the row ("Rescan", "Open device"). | P2 |
| P7 | The import report and several results are plain paragraphs with no container; other notices have a bar. | `42-…-1280`; `profiles/ImportResult.svelte`, `profiles/ResultPanel.svelte` | One shared notice component (section 5). | P2 |
| P8 | The global apply-on-connect report shows raw effect ids: `the effect “moving-highlight”`, `the effect “solid”`. The catalog is only loaded when Effects or Profiles opens. | `49-…-1280`, `50-…-920`; `App.svelte:34-38`; `profiles/present.ts` (`itemText`) | Load the effect catalog when the session starts. | P1 |
| P9 | "Active in this session", "Unsaved changes: 4", "4 unsaved changes", "4 settings changed since you applied or saved it" are four phrasings of one fact. | `40-…-1280`; `profiles/ProfileList.svelte:10-15`, `connection/ProfileStatus.svelte:4-6`, `profiles/ProfileArticle.svelte:53-61` | One phrase: "4 unsaved changes". | P2 |
| P10 | Offline, the page explains at length what can and cannot be done; "Apply profile" is disabled with a 33-word reason. | `45-…-1280` | One line plus a "Connect" button. | P2 |

### 3.5 Notices, recovery and the Diagnostics drawer

Screenshots: `47`, `48`, `49`, `50`, `51`, `52`.

| ID | Finding | Evidence | Recommendation | Pri |
|---|---|---|---|---|
| G1 | Notices stack without limit and push the app down (N5). | `50-…-920` | One notice slot (section 2.6). | P1 |
| G2 | **The drawer covers the header**, including the button that opened it, the connection state and Disconnect. At 920 px it covers 52% of the window. Focus can move to covered controls. | `51-…-920`, `51-…-1280`; `diagnostics/DiagnosticsDrawer.svelte:162-178` | Start the drawer below the status strip, or make it modal (trap focus, inert background). At under 1100 px make it a full page. | P1 |
| G3 | The drawer has no "Copy diagnostics" or "Save report". Its main use is a bug report. | `diagnostics/DiagnosticsDrawer.svelte:40-159` | Add "Copy as text". Use the user's device names beside the references. | P1 |
| G4 | Recovery notices are careful and long (three sentences per file, file names with 16-digit ids). | `48-…-1280`; `profiles/present.ts` (`noticeText`) | Headline per file ("Device names were reset"), details behind "What happened". | P2 |
| G5 | Six components draw the same left-bar notice with different padding (10×14, 14×16) and three draw full-width strips with hard-coded backgrounds. | `devices/DeviceDetail.svelte:191-208`, `devices/IdentifyPanel.svelte:132-144`, `effects/EffectEditor.svelte:168-180`, `effects/FailureNotice.svelte:30-38`, `profiles/ProfilesScreen.svelte:118-127`, `profiles/RecoveryNotices.svelte:28-35`; `App.svelte:207-226` | One `Notice` component with tone, title, body, actions. | P2 |

### 3.6 Copy: where honesty should move to progressive disclosure

The wording is accurate and plain. The problem is volume and placement. Counted in the rendered Effects target page (`60`): 294 words in ten standing paragraphs, before any result and not counting control hints. Rules I recommend:

1. **One line where the action is; the rest behind "How this works".** Apply this to the Effects lifecycle text (E2), hardware modes (`effects/HardwareModes.svelte:30-34`), another program (`effects/ExternalControl.svelte:50-57`), Identify (already done), apply-on-connect (P3, P4) and prerequisites (C4).
2. **Say each promise once per screen.** "Nothing is saved to the devices" appears three times on Profiles (`profiles/SaveCurrent.svelte:31`, `profiles/ResultPanel.svelte:33`, the update dialog).
3. **Lead with the state, then the reason, then the next step** (ux-research.md §6). "Effects are off. You said OpenRGB Effects plugin controls this device. Hand it back below before you apply an effect." becomes a notice titled "Left to OpenRGB Effects plugin" with a "Hand back" button.
4. **Drop internal words:** owner, target, statement, "as you said", "marked", "armed" (in data only), "unresolved". Use: controlled by, device or zone, used by another program, "set to apply on connect", "not found today".
5. **Keep the uncertainty lines.** "PrismForge cannot confirm what the device shows now" is the right kind of honesty. Shorten it to one consistent sentence and reuse it.

## 4. The Effects flow: proposal for multi-target and live preview

### 4.1 Principles

- **Pick targets, pick a look, see it, send it.** The left pane answers "where", the centre answers "what", the stage answers "what will it look like", the bar answers "is it on the hardware".
- **Two named states, never implicit:** Draft (nothing sent) and Live (devices follow edits). ux-research.md §4 asks for two cues per mode; OpenRGB's Effects plugin has no such distinction, its effect simply runs (openrgb-behaviour.md §3.2).
- **Honest preview.** PrismForge can show what it computes and what it sends. It cannot read back what a device shows. Label accordingly.

### 4.2 Layout

```
+--------+----------------------------+-------------------------------------------------------+
| rail   | Targets        [Select all]| STAGE                              Draft · not sent   |
|        | [-] Demo Motherboard       |  Motherboard · Header 1  (30)  oooooooooooooooooooo   |
|        |     [ ] Onboard        6   |  Motherboard · Header 2  (16)  oooooooooooo           |
|        |     [x] Header 1      30   |  Desk card · Logo        (12)  ( ring drawn as ring ) |
|        |     [x] Header 2      16   |  [Pause]  [Hide]   "The pattern starts again on each" |
|        | [x] Desk card         12 ~ +-------------------------------------------------------+
|        | [ ] Memory 1           8 ! | Effect:  (Solid) (Breathing) (Gradient) (Rainbow) ... |
|        | [/] Memory 2  other prog.  | Speed      ----o--------  [ 20 ] per minute   (reset) |
|        | [/] Light Bar  no Direct   | Direction  (o) Forward  ( ) Backward                  |
|        |                            | More settings >                                       |
|        | 3 of 5 selected            |                                                       |
|        +----------------------------+-------------------------------------------------------+
|        | [ Apply to 3 targets ]  [ Stop ]      Draft — devices unchanged     How effects run |
+--------+------------------------------------------------------------------------------------+
```

At 920 px the stage and editor share one column; the stage collapses to one line per target with a "Show stage" toggle, and the action bar stays pinned.

### 4.3 Target selection

- **Checkbox tree.** One row per device; devices with more than one zone expand to zone rows. The device checkbox is tri-state: all, none, mixed (`[-]`). Clicking a mixed box selects all, then clears (ux-research.md §3). OpenRGB's Effects plugin uses the same shape (controller checkbox sets all its zones, openrgb-behaviour.md §3.1), so users who know it will recognise this.
- **"Select all" selects only what can be lit** (available, has Direct Mode, not left to another program), again matching OpenRGB. Rows that cannot be lit stay visible, disabled, with a short reason and a fix link ("no Direct Mode", "other program — Hand back", "identical device"). Do not hide them.
- **Focus is not selection.** Arrow keys move, Space toggles, Shift+Arrow extends, Ctrl+A selects all lightable. Moving focus sends nothing to hardware. Visible checkboxes stay, so modifier-click is never the only way.
- **State marks per row:** `~` running a PrismForge effect, `!` refused or paused, lock for another program. The words appear in the tooltip and in the accessible name.
- **A whole device and its zones cannot both be picked** in a way that conflicts. Selecting the device checks every zone; unchecking one zone turns the device mixed. The backend's "whole device covers its zones" rule (`effects/present.ts:33-39`) maps to this directly.
- **Count line** under the list: "3 of 5 selected · 58 LEDs".
- **Later:** saved groups ("Case fans") as one more row type. Keep to device, group, profile (ux-research.md §2).
- **Shared selection:** "Set lighting" on a device, or a selection on the future Layout canvas, opens Effects with those targets checked.

### 4.4 Effect picker and parameters with several targets

- If all selected targets run the same effect with the same values, the editor shows them.
- If they differ, the picker shows "Mixed" and the stage shows each target's current effect. Choosing an effect replaces all on Apply. A line says so: "Replaces 2 different effects on 3 targets."
- Fields whose value differs across targets show "Mixed" as placeholder; editing a field sets it for all selected targets and leaves untouched fields as they are.
- Basic settings first (colour, speed, brightness); the rest under "More settings" (ux-research.md §2, Hick's law). Each control gets a reset-to-default.

### 4.5 The stage: draft preview and live output

Two sources, one place, always labelled:

| State | What the stage draws | Label (text + icon + bar tint) |
|---|---|---|
| **Draft** | The effect computed for each selected target at its real LED count and shape, from the current settings. This is today's preview, one row per target. | "Draft — devices unchanged" (neutral bar) |
| **Live** | The frames PrismForge is sending to each target right now, from the output engine, not a second render. | "Live on 3 devices — what PrismForge sends" (accent bar edge, the spectrum line) |
| **Live with a problem** | Sent frames, with the row dimmed and a mark where the last write was refused or paused. | "Live on 2 of 3 · 1 refused" (warning) |

- **Shapes.** Draw a line as a line, a ring as a ring, a grid as a grid. Mark LED 1 with a dot and show direction with an arrow. Wrong LED order is the most likely mapping error (ux-research.md §5).
- **Shared clock.** All rows use one time base so the user sees phase. Today each target renders from its own first LED; say so in one line: "The pattern starts again on each device." This is how OpenRGB's plugin behaves too (openrgb-behaviour.md §3.3). When Layout exists, offer "Span across the layout" as an option; it needs PrismForge's own positions because OpenRGB core has none (openrgb-behaviour.md §4.2, §5 item 8).
- **Why real frames, not a second render:** OpenRGB's plugin previews by rendering the effect again on a synthetic 64×64 canvas (openrgb-behaviour.md §3.2). That shows the effect, not the output. PrismForge already previews at the target's LED count; showing sent frames in Live closes the remaining gap and makes "refused" and "paused" visible.
- **What it never claims:** the caption stays "PrismForge cannot see the device. This is what it sends." A device row can link to Identify for a physical check.
- **Controls:** Pause (required for motion longer than 5 s, WCAG 2.2.2), Hide, and the reduced-motion still frame that already exists (`effects/preview.svelte.ts:10`, `:38-40`).
- **Flash guard.** The stage is large enough to pass the WCAG 2.3.1 area threshold (section 6, A9). Cap what the stage draws at three luminance flashes per second, whatever the effect does, and mark the row "Fast flashing — shown slowed".

### 4.6 Apply, stop and results

- **Sticky action bar**, always visible: primary "Apply to 3 targets", secondary "Stop", then the state text. In Live the primary becomes "Live ✓" with a "Pause live edits" toggle; edits send after a short delay, as today.
- **Live is explicit.** After the first Apply the bar switches to Live and says so. Leaving the screen returns to Draft (the code already does this silently in `leave()`); returning shows "Draft — press Apply to continue live edits".
- **Batch result, one row per target**, in a panel above the bar: "2 of 3 applied". Rows: Applied; Skipped — left to OpenRGB Effects plugin [Hand back]; Failed — reason [Retry]. Partial success is never green (ux-research.md §3). The Profiles apply report (`39-…-1280`) is the model; reuse its component.
- **Revert.** "Revert" restores the assignments PrismForge had before this Apply. State the limit: "It does not restore what other programs or the device itself did."
- **No confirmation** for apply or stop. Both are easy to undo.
- **Stop** applies to the selected targets and reports the same way ("Stopped on 3. Earlier lighting put back on 2; 1 could not be confirmed").

### 4.7 Ownership and hand-over texts

- Per-device control moves out of the effect editor into a compact "Control" block on the device (and in the target row's menu): **PrismForge effect / Hardware mode ▾ / Another program**. Three radio-like choices explain the model better than three sections of prose.
- Blocked targets carry their fix inline (E8).
- One sentence explains conflicts, once: "If two programs write to one device, the lights flicker or the last writer wins." Keep it in the "Another program" help, not on every page.
- Windows Dynamic Lighting can also take a device (ux-research.md §6). Name it in the help text as a common "other program".

## 5. Visual design system

### 5.1 What exists

Tokens (`app.css:1-29`): four surface greys (`--ink-0…3`), one line colour, three text colours, focus, four status colours, two demo colours, two radii (6, 10 px), the spectrum gradient and the font stack. Type is the system font at 14 px / 1.5.

Identity: the hexagonal prism mark (`lib/Mark.svelte`), a spectrum hairline under the header, warm paper-white (`#ece9e2`) as the primary button colour. This is original and restrained. Keep it.

### 5.2 Findings

| ID | Finding | Evidence | Pri |
|---|---|---|---|
| S1 | **No spacing scale.** 22 distinct gap values and 37 distinct paddings, including 7, 9, 14, 18, 26 px one-offs. | counts from all component styles; e.g. `padding: 9px 10px 9px 14px`, `24px 26px`, `18px 20px` | P2 |
| S2 | **No type scale.** Sizes 26, 20, 18, 15, 14, 13, 12 px; 13 px is set 46 times by hand. | — | P2 |
| S3 | **Eight font weights** (400, 450, 500, 550, 600, 650, 700, 750). A static font such as Windows 10's Segoe UI has no in-between weights, so by the CSS matching rules these snap to the nearest real one and the hierarchy differs by machine (not verified on Windows 10). | `app.css`, `connection/DemoBanner.svelte:23`, `connection/ProfileStatus.svelte:20` | P2 |
| S4 | **Eleven colours outside the token list, in fourteen places:** tone backgrounds `#2a1d1f`, `#2a2519` (each twice), tone borders `#6a3a35`, `#6b5a33` (twice), `#7a3a33`, hover border `#4a5163` (twice), demo `#e3ad45`, `#b98a2e`, preview `#0f1014`, `#16181e`, `#ffffff`. | `App.svelte:221-225`, `connection/ConnectionScreen.svelte:122-124`, `:158`, `profiles/RepairedBanner.svelte:20`, `profiles/ProfileArticle.svelte:249`, `effects/EffectPicker.svelte:59`, `devices/DeviceList.svelte:138`, `connection/DemoBanner.svelte:17-19`, `effects/PreviewStrip.svelte:57` | P2 |
| S5 | **Surfaces are too close.** `--ink-2` on `--ink-1` is 1.12:1, `--ink-3` on `--ink-2` 1.16:1, `--line` on `--ink-1` 1.51:1. Cards, buttons and inputs are separated by lines most displays barely show. | measured, section 6 | P1 |
| S6 | **No elevation.** The dialog has no shadow; the drawer has the only one. No z-index scale (10 and 20 by hand). | `lib/Dialog.svelte:72-82`, `diagnostics/DiagnosticsDrawer.svelte:170-176` | P2 |
| S7 | **Five radii by hand** besides the two tokens: 4 px (tags, five places), 3, 2, 1 px, 999 px. | — | P2 |
| S8 | **No icon set.** Four tone icons and the mark. Every action is a text button, which is part of why rows are wide. | `lib/ToneIcon.svelte` | P2 |
| S9 | **Motion is ad hoc.** 120 ms ease (buttons, switch), 160 ms ease-out (drawer), 1.6 s linear (sweep). Reduced motion is handled globally and well. | `app.css:80`, `:131-139` | P2 |
| S10 | **Repeated patterns are copied, not shared:** list row, tag, left-bar notice, small button, selected bar, hint text. | D4, G5 | P2 |
| S11 | The **primary button is the only strong element**, and it is neutral. There is no accent colour for selection, links or "live". The spectrum is used for selection, where it fails (D4), and for loading. | — | P2 |
| S12 | The **disabled button** keeps its border and fill and only dims its text. Next to S5 and E3 this makes enabled and disabled hard to tell apart. | `22-…-1280`; `app.css:99-104` | P1 |

### 5.3 Proposed tokens

Keep the palette's character. Widen the steps, name roles, and add the missing scales. All contrast figures are computed with the WCAG formula.

```css
:root {
  /* Surfaces (unchanged values, role names) */
  --surface-0: #15171c;   /* window */
  --surface-1: #1b1e25;   /* panes, cards */
  --surface-2: #232731;   /* raised, hover */
  --surface-3: #2d323e;   /* pressed, selected fill */

  /* Lines */
  --line-subtle: #363c4a;          /* dividers only; 1.5:1, decorative */
  --line-control: #727a8c;         /* inputs, switches, checkboxes: 4.16 / 3.87 / 3.47 on surface 0 / 1 / 2 */

  /* Text */
  --text: #ece9e2;                 /* 14.8:1 on surface-0 */
  --text-dim: #a9aebb;             /* 8.1 / 7.5 / 6.7 */
  --text-faint: #8b91a1;           /* was #7f8595; now 5.7 / 5.3 / 4.7 on surface 0 / 1 / 2 */

  /* Accent and state */
  --accent: #8ab4ff;               /* selection bar, focus, links: 8.6:1 on surface-0 */
  --focus: var(--accent);
  --ok: #63d197;  --working: #8ab4ff;  --warning: #f2bd55;  --problem: #ff8576;
  --ok-surface: #1a2a24;  --warning-surface: #2a2519;  --problem-surface: #2a1d1f;
  --warning-line: #6b5a33;  --problem-line: #7a3a33;
  --demo: #f2bd55;  --demo-alt: #e3ad45;  --demo-ink: #22190a;   /* 10.1:1 and 8.5:1 */
  --spectrum: linear-gradient(90deg, #ff5a4e, #ffb340, #f4e04d, #4fd18b, #3fa9ff, #9b7bff);
  /* Use the spectrum only for "light is flowing": live bar edge, loading sweep, the mark. */

  /* Space: 4 px base */
  --space-1: 4px; --space-2: 8px; --space-3: 12px; --space-4: 16px;
  --space-5: 24px; --space-6: 32px; --space-7: 48px;

  /* Type: five sizes, three weights */
  --font-xs: 12px; --font-s: 13px; --font-m: 14px; --font-l: 18px; --font-xl: 24px;
  --weight-regular: 400; --weight-strong: 600; --weight-heading: 700;
  --leading-tight: 1.25; --leading-body: 1.5;

  /* Shape, depth, motion, layers */
  --radius-xs: 4px; --radius-s: 6px; --radius-m: 10px; --radius-pill: 999px;
  --shadow-1: 0 1px 2px rgb(0 0 0 / 0.4);
  --shadow-2: 0 8px 24px rgb(0 0 0 / 0.45);             /* dialog, popover */
  --shadow-3: -18px 0 40px rgb(0 0 0 / 0.35);           /* drawer */
  --dur-fast: 120ms; --dur-base: 160ms; --ease-out: cubic-bezier(0.2, 0, 0, 1);
  --z-sticky: 5; --z-drawer: 10; --z-dialog: 20; --z-popover: 30;

  /* Sizes */
  --control-h: 32px; --control-h-s: 28px; --target-min: 24px; --rail-w: 184px; --rail-w-compact: 72px;
}
```

Component rules that follow from it:

- **Buttons:** three kinds only. Primary (paper-white), default (surface-2 fill, `--line-control` border), quiet (no fill, text only). Disabled drops fill and border to `--line-subtle` and text to `--text-faint`. Danger is default with `--problem` text and `--problem-line` border.
- **Selected row:** `--surface-2` fill, 3 px `--accent` bar, strong weight. One class for device, target, profile and effect rows.
- **Notice:** one component. Tone bar 3 px, tone surface, icon, optional title, body, actions. Sizes: inline and strip.
- **Tag:** `--radius-xs`, `--font-xs`, tone border and an icon for warning and problem tags (colour is not the only cue).
- **Icons:** draw a small original set on a 20 px grid with the 1.6 px round stroke the tone icons already use: devices, effect, layout, profile, automation, diagnostics, settings, plus rescan, identify, rename, more. Base the shapes on the mark's hexagon and facets so the set is PrismForge's own. Do not adopt another product's glyphs.
- **Identity:** the prism is the idea. White in, colour out. Neutral, warm controls; colour appears only where light is actually being produced (stage, live edge, swatches). That is distinct from the neon-on-black look common in this category, and it should stay that way.

## 6. Accessibility (WCAG 2.2 AA)

### 6.1 Contrast, measured from the tokens

Text (needs 4.5:1; large text 3:1):

| Pair | Ratio | Result | Where |
|---|---|---|---|
| `--text` #ece9e2 on ink-0 / 1 / 2 / 3 | 14.79 / 13.76 / 12.32 / 10.58 | pass | body, headings |
| `--text-dim` #a9aebb on ink-0 / 1 / 2 / 3 | 8.08 / 7.51 / 6.73 / 5.78 | pass | meta, explanations |
| `--text-faint` #7f8595 on ink-0 | 4.86 | pass | hints in the page |
| `--text-faint` on ink-1 | 4.52 | pass, by 0.02 | hints in cards, list counts, drawer labels |
| **`--text-faint` on ink-2** | **4.05** | **fail** | LED count in a selected or hovered target row (`effects/TargetList.svelte:147-171`) |
| `--text-faint` on the problem strip #2a1d1f | 4.40 | fail if used | not used today; do not start |
| `--problem` #ff8576 on ink-0 / 1 | 7.57 / 7.04 | pass | field errors |
| `--warning` #f2bd55 on ink-1 | 9.68 | pass | "Identical device" tag |
| `--ok` #63d197 on ink-2 | 7.89 | pass | "On this target" |
| `--demo-ink` on demo stripes #f2bd55 / #e3ad45 | 10.06 / 8.53 | pass | demo banner |
| ink-0 on primary button #ece9e2 | 14.79 | pass | primary buttons |

Controls and states (needs 3:1 against adjacent colours, SC 1.4.11):

| Pair | Ratio | Result | Where |
|---|---|---|---|
| **Input border `--line` on ink-0 / ink-1** | **1.62 / 1.51** | **fail** | every text and number field; the field fill equals the page colour, so the border is the only boundary (`app.css:106-111`) |
| **Switch track border `--line` on ink-0** | **1.62** | **fail** for the track; the thumb (4.86) carries the state | `profiles/ApplyOnConnect.svelte:33-41` |
| Button fill ink-2 on ink-0, border `--line` | 1.20, 1.62 | below 3:1; the label identifies the button, so this is a weakness, not a failure | all default buttons |
| **Current nav item ink-3 on ink-0** | **1.40** | **fail** as a state cue; text colour also changes | `App.svelte:198-201` |
| Selected list row: ink-2 on ink-1 | 1.12 | relies on the 3 px bar | D4 |
| Danger button border #7a3a33 on ink-2 | 1.76 | label carries it | delete dialog |
| Focus ring #8ab4ff on ink-0 / 1 / 2 | 8.58 / 7.98 / 7.15 | pass | all controls |

Fix: the two tokens in section 5.3 (`--line-control` 3.47–4.16:1, `--text-faint` 4.74–5.69:1) clear every failing row above.

### 6.2 Other criteria

| ID | Criterion | Finding | Evidence | Pri |
|---|---|---|---|---|
| A1 | 2.4.7 Focus visible | Pass. One rule: 2 px `--focus` outline, 2 px offset (`app.css:113-116`). The colour swatch forwards focus from its hidden input (`effects/controls/ColorField.svelte:96-99`). The drawer heading suppresses its outline, which is fine for a programmatic focus target. | `53`, `54` (rings drawn by applying the app's own rule; see section 8) | keep |
| A2 | 2.4.11 Focus not obscured | **Fail with the drawer open.** The drawer is fixed from the top of the window, 480 px wide, and is not modal. Tab reaches "Refresh devices", "Disconnect" and the Diagnostics button underneath it. | `51-…-920`; `diagnostics/DiagnosticsDrawer.svelte:162-178` | P1 |
| A3 | 2.1.1 Keyboard | Pass for every control I could reach: nav, lists, radios, sliders, number and colour fields, palette order, switch, dialogs (trap, Escape, focus return), rename (Enter, Escape). | code and DOM order | keep |
| A4 | 2.4.1 / 2.4.3 Bypass and order | Order is logical. There is no bypass: 6–7 header stops, then one stop per device or zone. "Apply effect" is stop 24–35. | measured Tab order | P1 |
| A5 | 2.5.8 Target size | Pass. Default buttons 37 px high and nav buttons 29 px (measured); small buttons about 28 px and the swatch 34 px (from the CSS). Radios are 13 px but their labels extend the target. Effect cards need E4 fixed to be honest targets. | DOM measurement and CSS | keep |
| A6 | 2.5.7 Dragging | Pass today: sliders have number fields; palette order uses buttons; nothing requires a drag. **The Layout editor must keep this:** inspector fields, nudge keys, click-to-place (ux-research.md §5). | `effects/controls/RangeField.svelte`, `PaletteField.svelte:52-54` | watch |
| A7 | 1.4.4 Resize text, 1.4.10 Reflow | **Fail at 200%.** At 640×400 CSS px (a 1280×800 window at 200%) the header runs off the right edge, the page scrolls both ways, and the list keeps its 260 px minimum beside a 330 px page. Type is in px, and `tauri.conf.json` does not enable zoom hotkeys, so there is no in-app way to enlarge text. | `55`, `56`, `57`; `devices/Inventory.svelte:26` | P1 |
| A8 | 2.3.3 / reduced motion | Pass. Transitions and animations are cut globally; the preview shows one still frame and says so. | `app.css:131-139`; `effects/PreviewStrip.svelte:37` | keep |
| A9 | 2.3.1 Three flashes | **At risk.** The preview plays at 25 frames per second (`effects/preview.svelte.ts:7`). Allowed settings: Moving highlight up to 240 rounds per minute (each LED goes dark-white-dark 4 times a second), Colour cycle up to 120 rounds per minute with up to 8 colours (16 colour changes a second, saturated red allowed). The strip is about 804 × 18 px ≈ 14 500 px² at 100%, under the 21 824 px² small-area limit; at 200% zoom, or on the larger multi-target stage, it is over. The hardware itself is outside WCAG but is the bigger risk. | catalog ranges in `test/fixtures/catalog.ts`; `21-…-1280` | P1 |
| A10 | 2.2.2 Pause, stop, hide | The preview can be hidden ("Hide preview"). There is no pause. Hiding satisfies the criterion; a Pause that keeps a still frame is better. | `effects/PreviewPanel.svelte:14-16` | P2 |
| A11 | 4.1.3 Status messages | Mostly pass: results use `role="status"`, failures `role="alert"`. Gaps: field-level errors on Connection and Frame rate are not announced when they appear; the demo banner is not announced; the several `role="status"` regions on Profiles can speak at once. | `connection/ConnectionScreen.svelte:71-79`, `effects/FrameRate.svelte:59-61` | P2 |
| A12 | 1.3.1 / 4.1.2 Names and roles | Good: fieldsets and legends for radios and palettes, `aria-describedby` for hints and blocked reasons, `role="switch"` with a fixed label. Notes: slider and number field share one name ("Speed", "Speed"); the switch shows a visible "On/Off" chip hidden from assistive technology, which is fine; list selection uses `aria-current="true"` on buttons inside a `nav`, acceptable, a listbox would be richer. | `effects/controls/RangeField.svelte:30-52` | P2 |
| A13 | 1.1.1 Non-text content | The LED strips and the preview are `aria-hidden` with no text equivalent. The caption covers the preview's purpose; the "Last reported colours" strip has none. | `devices/LedStrip.svelte:9` | P2 |
| A14 | Dialogs (APG) | Good: `role="dialog"`, `aria-modal`, labelled, first focus on the safe button, Escape, focus return. Gaps: the background is not `inert`; Escape in a dialog also closes the drawer if both are open, because the drawer listens on `window` (`diagnostics/DiagnosticsDrawer.svelte:31-38`). | `lib/Dialog.svelte:21-50` | P2 |
| A15 | 1.4.1 Use of colour | Pass in general: every tone has an icon shape and words. Weak spots: the invalid-field cue is a red border plus changed help text in the same grey (C6); the "Identical device" tag differs from "No Direct Mode" by colour only. | `devices/DeviceList.svelte:133-140` | P2 |
| A16 | Page structure | No `h1` on the main screens (Devices and Profiles start at `h2`; Connection has one). Some ids repeat across components but never on one screen. | `devices/DeviceDetail.svelte:40` | P2 |

## 7. Prioritised list

Size: S up to a day, M a few days, L a week or more.

### P0 — confusing or broken

| # | Screen | Problem | Recommendation | Size |
|---|---|---|---|---|
| 1 | Top bar | Controls overlap at 920 px always, and at the default 1200 px with unsaved changes; sideways scroll at 920 px with an active profile (N1). | Step 1 of section 2.7: shrink order, overflow menu or two rows. | S |
| 2 | Effects | "Apply effect" is 1.6–2.6 screens down and the 24th–35th Tab stop (E1). | Sticky action bar with state text. | M |

### P1 — clear improvements

| # | Screen | Problem | Recommendation | Size |
|---|---|---|---|---|
| 3 | Devices | Page-level scrollbar on short windows (D1). | Position the scroll panes; hide body overflow. | S |
| 4 | All two-pane screens | Scrollbar in mid-window; dead band on the right (D2). | Scroll the pane, cap an inner wrapper. | S |
| 5 | Effects | Enabled buttons look disabled (E3, S12). | Button text colour not inherited; clearer disabled style. | S |
| 6 | Effects | Card hover invites clicks that do nothing (E4). | Whole card is the label. | S |
| 7 | Lists | Selection bar renders olive (D4). | Single accent bar, shared class. | S |
| 8 | Navigation | Items rename and vanish; no current item after disconnect (N2). | Fixed items, empty states, reset screen on session end. | S |
| 9 | Global | Raw effect ids in the apply-on-connect report (P8). | Load the catalog at session start. | S |
| 10 | Effects | "Solid colour" opens in an error state (E13). | Default colour. | S |
| 11 | Global | Notices stack to 375 px at 920×600; no "applying" indicator (N5, G1). | One notice slot; "Applying…" in the profile chip. | M |
| 12 | Effects | Hidden auto-apply mode; feedback points off screen (E5, E6). | Draft and Live states in the bar. | M |
| 13 | Effects | Lifecycle paragraph on every page, soon inaccurate (E2). | One line plus disclosure, text built from settings. | S |
| 14 | Effects | "Owner" vocabulary and "Owner: none" noise (E7). | "Controlled by", state marks. | S |
| 15 | Effects | Blocked target; fix is a screen away (E8). | "Hand back" in the notice. | S |
| 16 | Effects / Devices | Hardware modes, other program and frame rate inflate every target page (E10, N8). | Per-device Control block; frame rate to Settings. | M |
| 17 | Devices | No route from a device to its lighting; duplicate names (D5, D3). | "Set lighting"; disambiguate rows. | M |
| 18 | Profiles | Selected profile starts 387–449 px down; results far from actions (P1, P2, P3). | Profile first; save-as in the header; result under the actions. | M |
| 19 | Connection | Demo mode below the fold at 920 px; no actions while retrying (C1, C2). | Keep two columns; add actions to the waiting panel. | S |
| 20 | Diagnostics | Drawer covers the header and its own button; focus obscured; no copy (G2, G3, A2). | Start below the strip or go modal; "Copy as text". | M |
| 21 | Accessibility | Input and switch borders 1.62:1; faint text 4.05:1 on selected rows (6.1). | `--line-control`, new `--text-faint`. | S |
| 22 | Accessibility | Flash risk in fast previews and on hardware (A9). | Cap what the preview draws; label and gate fast settings. | M |
| 23 | Accessibility | Nothing scales to 200%; no zoom (A7). | Enable zoom hotkeys, a UI-scale setting, rem-based type, collapse the list pane when narrow. | M |
| 24 | Navigation | No shortcuts, no bypass (N7, A4). | Ctrl+1…n, F6, skip link, roving tabindex in lists. | M |
| 25 | Navigation | Seven areas will not fit the top bar (section 2). | Rail plus status strip, option C. | L |
| 26 | Effects | One target at a time; preview is not the output (section 4). | Checkbox tree, stage, batch results, revert. | L |

### P2 — polish

| # | Screen | Problem | Recommendation | Size |
|---|---|---|---|---|
| 27 | Design system | No spacing, type, depth, motion or layer scales; eight weights; eleven stray colours (S1–S9). | Token set in 5.3; replace literals. | M |
| 28 | Design system | Notice, tag, row, small button are copied per component (S10, G5). | Shared components. | M |
| 29 | Design system | No icons; rows are wide with text buttons (S8, D7). | Small original icon set; short row actions. | M |
| 30 | Copy | Long standing explanations; one fact phrased four ways (3.6, P9, P4, C4, G4). | One line plus disclosure; one phrase per fact. | M |
| 31 | Connection | Wording of retry label, field errors, busy state, `--server` wrap (C3, C5, C6, C7). | As listed. | S |
| 32 | Devices | Mode pills, colour strip, ambiguous-device advice (D6, D8, D9). | As listed. | S |
| 33 | Effects | Hex shown twice; palette rows; empty state; reset per control; shaped preview (E11, E12, E15, E16, E14). | As listed. | M |
| 34 | Profiles | Six equal buttons; duplicate skip wording; offline text (P5, P6, P10). | "More" menu; one status word; one line. | S |
| 35 | Accessibility | Announce field errors and demo mode; pause for the preview; text for LED strips; `inert` behind dialogs; Escape scope; page `h1` (A10–A16). | As listed. | S |
| 36 | Navigation | "Setup" beside "Settings" (2.3). | Name it "Layout". | S |

## 8. Method and limits

### How the screens were produced

- A throwaway harness in the scratch worktree (`apps/desktop/harness.html`, `apps/desktop/src/harness/main.ts`) mounts the real `App` component and installs a fake Tauri transport with `mockIPC`, the same way the tests do. Each state is one URL parameter. Data comes from the Rust-generated fixtures in `src/test/fixtures/`.
- The project's Vite dev server served it on `localhost:5347`. Screenshots were taken from a tab in Orca's embedded browser with `orca viewport` and `orca screenshot`, at 1280×800 and at the declared minimum 920×600 (`src-tauri/tauri.conf.json`: `minWidth` 920, `minHeight` 600).
- 114 PNG files: 52 states at both sizes (104), two focus-ring images, four 200% images, one navigation state after a pushed disconnect, two tall full-page images, one at the default window size 1200×780.

### Caveats on the evidence

- **Preview pixels are synthetic.** The harness generates frames in the shape Rust returns. Layout, size and captions are the real component; the colours are not the real renderer's.
- **Focus rings in `53` and `54` are drawn by applying the app's own `:focus-visible` declaration to chosen elements.** The background tab does not hold OS focus, so the browser does not match `:focus-visible` there. Tab order itself was verified with real key presses.
- **`31` is a forced state:** the fake backend refuses an apply with a conflict on a device the session shows as free. It shows the notice, not a reachable sequence.
- **`27` and `41`** show results over a session snapshot that the fake backend does not update, so surrounding hints may not match the result.
- **Capture size:** Orca leaves scrollbars out of the image. A file that is 905 px or 1265 px wide instead of 920 or 1280 had a page-level vertical scrollbar; 585 px high means a horizontal one too. Twenty files are affected; they are evidence for D1 and N1.
- **Rendering engine and OS:** Chromium on macOS. Fonts are the macOS system font, not Segoe UI Variable. Scrollbars are the classic always-visible kind on this machine. Widths on Windows will differ by a few pixels; the overflow findings have enough margin to hold.
- **200% images** use a half-size viewport at device scale 2, which is equivalent to browser zoom, not to Windows display scaling.

### What I could not check

- Native window chrome, title bar, resize behaviour and real minimum-size enforcement in Tauri on Windows and macOS.
- WebView2 rendering: Segoe UI weights, scrollbar style, native colour picker, number-field spinners, high-contrast (forced colours) mode.
- Native file dialogs for import and export, and what the app shows while they are open.
- Real hardware: timing of Identify, write latency, refused writes, what devices show after stop or disconnect, and any flashing on real LEDs.
- A screen reader pass (Narrator, NVDA, VoiceOver). Findings on announcements come from the markup.
- Whether Ctrl +/− zoom works in the built app. The config does not enable it; I did not run the app.
- The Settings screen and tray on the branch in review. I took their four settings into account from the description only.
- What `disconnect` returns in a real session. N2 was verified by pushing a view with no session while on Effects.
- Light theme, localisation and right-to-left layout: none exist.
- User testing. This is an expert review; the priorities are judgements, not observed failure rates.

## 9. Screenshot index

Each state exists as `NN-name-1280.png` and `NN-name-920.png` unless noted.

| No. | State |
|---|---|
| 01 | Connection, first run |
| 02 | Connection, Connect pressed (button busy) |
| 03 | Connecting (waiting panel) |
| 04 | Retrying, nothing answers |
| 05 | Connection failed: old server |
| 06 | Command error strip |
| 07 | Connection lost while connected: inventory kept, retrying |
| 08 | Connected, no usable devices |
| 09 | Demo inventory with demo banner |
| 10 | Demo inventory with active profile and 4 unsaved changes |
| 11 | Device detail with user-given names |
| 12 | Identify running |
| 13 | Identify finished, restored |
| 14 | Identify finished, not restored |
| 15 | Device detail, zones area |
| 16 | Ambiguous (identical) device |
| 17 | Unavailable device |
| 18 | Effects, no target |
| 19 | Effects, target selected, no effect |
| 20 | Effects, Rainbow wave selected: controls |
| 21 | Effects, preview and run section |
| 22 | Effects, hardware modes, another program, frame rate |
| 23 | Effects, palette controls (Colour cycle) |
| 24 | Effects, invalid settings (Solid colour without a colour) |
| 25 | Effect running on the target |
| 26 | Effect running, Apply accepted |
| 27 | Effect stopped, restore result |
| 28 | Output refused by the server |
| 29 | Device left to another program: blocked |
| 30 | Device left to another program: hand back |
| 31 | Apply refused with a conflict (forced) |
| 32 | Effects while not connected (paused) |
| 33 | Effects while a profile is being applied |
| 34 | Profiles, empty |
| 35 | Profiles, list of two |
| 36 | Profile detail |
| 37 | Profile with unresolved targets, top |
| 38 | Profile with unresolved targets, settings list |
| 39 | Apply result, partial |
| 40 | Update: change list dialog |
| 41 | Update: saved result |
| 42 | Import report |
| 43 | Delete dialog |
| 44 | Export dialog |
| 45 | Profiles while not connected |
| 46 | Profile folder in use by another window |
| 47 | Recovery banner on Devices |
| 48 | Recovery notices on Profiles |
| 49 | Apply-on-connect report strip |
| 50 | Worst-case stack of strips |
| 51 | Diagnostics drawer, degraded session |
| 52 | Diagnostics drawer, no session |
| 53 | Focus rings on Devices (simulated, 1280 only) |
| 54 | Focus rings on Effects (simulated, 1280 only) |
| 55 | 200%: Devices at 1280×800 |
| 56 | 200%: Effects at 1280×800 |
| 57 | 200%: Effects at 1920×1080 |
| 58 | 200%: Connection at 1280×800 |
| 59 | Navigation after a disconnect on Effects: no current item (1280 only) |
| 60 | Effects, full page, 1280×2060 |
| 61 | Profile detail, full page, 1280×1200 |
| 62 | Default window 1200×780 with active profile and unsaved changes: label overlapped |
