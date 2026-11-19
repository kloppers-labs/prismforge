# PrismForge setup views: geometry model, effect mapping, views, editor interactions

Research for PrismForge (GPL-3.0-or-later). Concepts and behaviour only, in my own words. No code, artwork, device images or component definitions were copied. Access date for all sources: 2026-11-19. "(inference)" marks my own reasoning. "(not confirmed)" marks a claim I could not verify from a primary source. Sections 1 to 4 report findings. The last section is my recommendation.

## Summary

Open systems describe LED space in three ways: a grid of cells (WLED, OpenRGB matrix maps, Hyperion matrix), a fixed-range coordinate per LED (QMK, Lightloom), or a parametric model with start corner and direction (xLights, Hyperion classic layout). None of them describes splitters. Daisy chains appear only as "one long run of LEDs, then split by the user". OpenRGB exposes zone LED counts, zone types, matrix maps and segments, but no positions and no topology, so the user must declare which components hang off which channel and whether outputs are mirrored or chained. For effects, the pure function (parameters, time, position, seed) fits QMK's own approach. The weak point is a sparse LED set: a highlight narrower than the LED pitch will flicker, so the sampler needs a footprint filter and effects need a minimum feature width. A flat 2D schematic is the honest view. No usability study I found compares schematic and realistic configuration views, so that choice rests on cost and truthfulness, not evidence. For a few thousand small shapes at 30 Hz, no vendor guidance gives a threshold, so I recommend Canvas 2D for the LED layer with an SVG overlay for outlines and handles, and a measurement on the three WebViews. A large live preview needs a flash limiter, because the WCAG area exception (about 341 by 256 CSS px) is small. Editor conventions (15 degree rotation snap, grid snap with a hold-to-disable key, one undo step per drag) are well established.

---

## 1. Data model for physical lighting geometry

### 1a. How open systems describe LED position

| System | Coordinate model and units | Order and direction | Zones / components | Mirroring / splitters / repeats | Licence | Source |
|---|---|---|---|---|---|---|
| WLED ledmap | Grid. `ledmap.json` has a `map` array of integers plus optional `width` and `height`. In 2D, each cell holds the wiring index of the LED placed there. `-1` marks a gap. Zero-based. | Order is the wiring index. Serpentine is a map where alternate rows run backward. A 1D map can reorder any run. | Several maps can exist (`ledmapx.json`) and a preset picks one. Segments split one strip by start and stop. | None in the pages read. Segment "grouping" makes N adjacent LEDs share one colour, which is down-sampling, not mirroring. Mirror on segments: (not confirmed). | WLED code EUPL-1.2. Docs repo MIT. | https://kno.wled.ge/advanced/mapping/ , https://kno.wled.ge/features/segments/ , https://kno.wled.ge/advanced/2d-1d-Mixed-Setup/ |
| WLED 2D setup | Matrix width and height, first-LED corner, serpentine flag. A 1D strip can follow a matrix in the same LED range. | Corner plus serpentine flag, set in the LED preferences. | Mixed matrix plus strip uses segments with a start X at or above the matrix LED count. A bus-level "reversed" option exists for chain order. | None. | As above | https://kno.wled.ge/advanced/2d-1d-Mixed-Setup/ . The main 2D setup page returned 404, so first-corner and serpentine options are from a search snippet of community pages (not confirmed): https://wled.discourse.group/t/ws2812-16x16-matrix-orientation/7234 |
| Hyperion.ng layout | Per-LED rectangle on the picture: `hmin`, `hmax`, `vmin`, `vmax`, each 0.0 to 1.0, fractions of the image. Resolution independent. | List order is LED order. A "classic" generator takes counts for top, bottom, left, right, a start `position` offset, `reverse`, gap length and position, overlap, and depths. A "matrix" generator takes horizontal and vertical counts, cabling `snake` or `parallel`, direction, and a start corner (top-left, top-right, bottom-left, bottom-right). | One layout per instance. Per-LED `colorOrder` override. | None. | Hyperion.ng and docs MIT. | https://docs.hyperion-project.org/user/advanced/Advanced.html (docs source: https://raw.githubusercontent.com/hyperion-project/hyperion.docs/main/docs/user/advanced/Advanced.md ); generator fields: https://raw.githubusercontent.com/hyperion-project/hyperion.ng/master/libsrc/hyperion/schema/schema-ledConfig.json |
| QMK RGB Matrix | Per-LED point. `g_led_config` stores optional key-matrix row and column, an x and y position, and flags. x runs 0 to 224 and y 0 to 64 by default, centre (112, 32). Positions up to 255 are allowed. Units are abstract, not millimetres. | Order is the array index. No direction field. | Flags mark an LED as modifier, underglow, key light or indicator. Effects can target a flag group. | None. | qmk_firmware GPL-2.0 (API SPDX `GPL-2.0`). Docs licence checked only as part of that repo. | https://docs.qmk.fm/features/rgb_matrix |
| xLights | Parametric models on a 2D layout canvas (arch, candy cane, circle, custom, matrix, single line, spinner, star, tree, window frame, wreath, poly line). Custom model: a grid of node numbers typed into cells. Whether two nodes may share a cell: (not confirmed). 3D positioning: (not confirmed; there is a depth setting for 3D custom models). | Single line: a start location sets run direction, and "Swap Start/End" reverses it. Circle: start at top or bottom, outer or inner ring, clockwise or counter-clockwise, default bottom-outer clockwise. Matrix: start corner, horizontal or vertical, strands per string (zig-zag folds), and "Don't Zig Zag". | Models hold strings of nodes. A model can join several model groups. Group effects render on one shared grid ("default") or per member model. A "Minimal Grid" option sizes the buffer to the group. | None found in the pages read. A model in several groups is closest. | xLights GPL-3.0. Manual licence: (not confirmed; the manual repo was not found). | https://manual.xlights.org/xlights/chapters/chapter-four-tabs/models.md , .../models/single-line-model.md , .../models/circle-model.md , .../models/matrix-model.md , .../models/custom-model.md , .../model-groups.md |
| OpenRGB core (from the earlier file) | No positions. Zone types: single, linear, matrix, linear loop, matrix loop X and Y, segmented. `matrix_map` is height, width and a row-major vector of LED indexes. A zone has LED count, min and max, flags and optional segments. Segments have name, type, start index, LED count and an optional matrix map. | Order is the LED index. Direction is not stored. "Reverse" lives in the Effects plugin per zone, not in core. | Zones are the unit. Segments split a zone. Flags say whether size, name, type, matrix map or segments may be set by hand. | None. | GPL-2.0-or-later | `openrgb-behaviour.md` sections 3.1 and 4.2 (OpenRGB tag `release_1.1`, commit `8e16de27858cf2be6af54a33ce003df74cb9fd3e`) |
| OpenRGB Visual Map plugin (from the earlier file) | One virtual matrix device. Each zone is placed on a grid up to 1024 by 1024 by x, y, scale, LED spacing and a shape (horizontal line, vertical line, custom per-LED cells). | Order is the zone's LED order. `reverse` flips it on line shapes. | One entry per zone. | Several real LEDs in one cell are shown as "Multiple LEDs (n)". This is overlap, not a splitter. | GPL-2.0-or-later | `openrgb-behaviour.md` section 4.1 |
| Lightloom (prior art, concepts only) | Fixed canvas 320 by 200 units, origin top-left, y down (layout schema). A placement has a top-left position, a scale per axis (canvas units per LED grid unit), clockwise rotation 0 to 359 about the top-left corner, flip X and flip Y, enabled, brightness and a forced-colour mode. A component has integer LED cell coordinates inside a width by height buffer, a per-LED name array, and a wire-order mapping array. | The wire-order array maps each LED to its slot in the output stream. Library files draw rings clockwise from the top and note "if a fan lights in the wrong order, the fix is a corrected mapping". | Per README, an ARGB header lets the user say what is attached (fans, strips, LED counts) and each component gets its own place on the canvas. Several named layouts can exist. | None in the schemas I read. Chains are described only in the README, as the list of components on a header. | See the licence block below. | https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/README.md , `.../schemas/layout.schema.json` , `.../schemas/component.schema.json` , `.../components/README.md` , `.../LICENSE-EXCEPTION.md` (all at commit `b8b482b6597fc03782ab4f804cede7dcf654c673`) |

Lightloom licence statements that apply (commit `b8b482b6597fc03782ab4f804cede7dcf654c673`):

- The repository is GPL-3.0-or-later (`LICENSE`, and the README statement). The two schema files carry an SPDX comment `GPL-3.0-or-later`.
- `LICENSE-EXCEPTION.md` adds a section 7 permission: add-ons that interact only through the plugin, effect, file-format (as described in `schemas/`) and local-API interfaces may use any licence. It says copying Lightloom's own code makes the result GPL.
- `components/README.md` says the built-in component JSON files are CC0-1.0, drawn from public product pages, and generated by a script. It also says each LED count carries a "To verify" mark where it rests on one source.
- Starter effects are CC0-1.0 (README).

Provenance cautions about Lightloom (important for PrismForge):

1. Its component schema states it is "compatible with the SignalRGB 2.5 component library" and uses that product's field names. Treat that format as tainted for our purposes. PrismForge should not adopt its field names, and should not load other products' component files.
2. The repository contains `docs/host-api/components-channels.md`, which describes itself as generated from static analysis of the proprietary SignalRGB application. I read only the first lines and a keyword search to find out what it was. I did not use any of it. PrismForge contributors should not read it either. (The file is part of the repo, not the schema files I cite.)
3. Its Lian Li UNI FAN SL120 entry says 16 LEDs and is marked "To verify". Lian Li's own page says 32 per fan (see 1b). This is a live example of why the user must be able to correct LED counts.
4. Licence history is not clean: the repo has both `docs/adr/0006-license-apache-mit.md` and `docs/adr/0010-license-gpl-3.md`. The pinned commit's `LICENSE` is GPL-3.0 (GitHub API SPDX `GPL-3.0`).

SignalRGB (general, publicly described functionality only): its public docs say a layout is a canvas where each device is placed to match its real position, and that without a layout each device runs the effect independently. I did not read or record file formats, assets or UI detail. Source: https://docs.signalrgb.com/guides/device-configuration/about-layouts/ (proprietary product; used only to confirm that this category of product offers 2D layouts).

Reading across the table:

- Per-LED coordinates win for irregular shapes. QMK and Lightloom store a point per LED. Hyperion stores a rectangle per LED, which is the same idea with a footprint.
- Parametric shapes win for common shapes and for the first-LED question. xLights and Hyperion's generators ask for start corner and direction, which is what a user can actually observe.
- Order and direction are always separate from position. WLED and Lightloom keep a wiring map. xLights and Hyperion keep a start location plus a reverse switch.
- Nobody models passive splitters. The closest feature is the group in xLights (one model in several groups) and "hide members" in Visual Map.

### 1b. Geometry classes a PC or desk setup needs

| Class | Parameters that define it | What can be read from OpenRGB (earlier file) | What the user must supply |
|---|---|---|---|
| Linear strip | LED count, length, direction (which end is LED 1), folds (zig-zag) with fold count and fold pitch. Straight run is folds = 1. | Zone type linear, LED count, min and max if resizable. | Physical length or placement, direction, fold count if any. |
| Ring (fan) | Rings: list of {LED count, relative radius}. Start angle of LED 1. Direction clockwise or counter-clockwise. Order of rings in the wire (outer first or inner first). Hub LEDs are a small inner ring. Front or rear face for double-sided fans. | Zone type linear loop is a hint. LED count. | Ring split, start angle, direction, which face is shown. |
| Matrix | Rows, columns, cell pitch, optional gap mask, start corner, serpentine or parallel wiring. Per-key positions come from the matrix map. | Matrix map (height, width, index per cell, with "no LED" cells) and zone type matrix. | Usually nothing for keyboards. Physical size and placement. Gaps for custom grids. |
| Rectangle perimeter (monitor backlight) | LED count per side (top, bottom, left, right), start corner, direction, optional gap at a corner, side length ratio. Hyperion's classic layout uses the same inputs. | Zone type linear and a total LED count only. | Counts per side, start corner, direction, aspect ratio. |
| Single point (logo) | One LED, position. | Zone type single. | Placement. |
| Free polyline | Ordered points with a LED count, LEDs placed at equal arc-length spacing, or explicit per-LED points. xLights has a poly line model. | LED count. | The points. |
| Unknown or unplaced | LED count only. No position. Excluded from "across layout" effects. | Everything OpenRGB reports. | Nothing, until it is placed. |

Examples of real LED counts (examples only, not the owner's hardware; each needs verification per product):

| Example | Count and arrangement | Source and confidence |
|---|---|---|
| Corsair VENGEANCE RGB RS DDR4 module | 6 LEDs in one light pipe per module | Corsair product page, opened: https://www.corsair.com/us/en/p/memory/cmg32gx4m2e3200c16/vengeance-rgb-rs-32gb-2-x-16gb-ddr4-dram-3200mhz-c16-memory-kit-cmg32gx4m2e3200c16 |
| Corsair iCUE QL120 fan | 34 LEDs in four loops: front 12 frame and 4 hub, rear 12 frame and 6 hub | Press and review pages, not the manufacturer page: https://bit-tech.net/news/tech/cooling/corsair-launches-icue-ql-rgb-fans-34-leds-apiece/1 and https://legitreviews.com/corsair-ql-series-ql120-rgb-led-fan-kit-review_217257/2 (from a search snippet; not opened in full) |
| NZXT F120 RGB Duo fan | 20 LEDs over two rings. Per-ring split not found. | NZXT listing via search snippet: https://nzxt.com/products/f120-rgb-duo (not opened in full) |
| Lian Li UNI FAN SL120 | 32 LEDs per fan, described as 16 per side. Lian Li's own page lists 32. | Search snippets of https://lian-li.com/product/uni-fan-sl/ and https://www.vortez.net/news_story/lian_li_introduces_uni_fan_sl120_cooling_fan.html (not opened in full). Disagrees with Lightloom's 16. |
| Adafruit digital RGB strips | Sold at 30, 60 and 144 LEDs per metre | Adafruit guide via snippet: https://learn.adafruit.com/adafruit-neopixel-uberguide/neopixel-strips |
| ASUS addressable header limit | Up to 120 LEDs per header in Gen1 mode, 500 in Gen2 mode per one support FAQ. Another board manual suggests a shared 500 cap. Varies by board. | https://www.asus.com/jp/support/faq/1056087/ (snippet) and the TUF B650M-PLUS manual (snippet). The limit matters because a long daisy chain can exceed it. |
| Passive splitter cables | Retail listings say one header drives all connected fans with one shared setting. | Retail listings, weak sources: https://www.memoryexpress.com/Products/MX00113030 (snippet) |

### 1c. Mirrored outputs and daisy chains

The two physical situations differ in what data reaches the LEDs.

- Daisy chain: one data line passes through fan 1, then fan 2, then fan 3. The controller sees one zone of N LEDs, and N is the sum. Each fan shows its own slice. This is the case "one zone split into segments, each assigned to a component".
- Passive splitter: one data line is forked to several fans. Each fan receives the same stream, so they show identical colours. This is the case "one channel shown as several components that always show the same thing". If the branches have different LED counts, I expect the shorter fan to show only the first LEDs of the stream (inference from how the data line works; not confirmed from a source, measure it).

A truthful model needs both, and the combination (a splitter whose branches are each a chain). I describe it as "outputs with branches" in the recommendation.

What OpenRGB exposes (earlier file section 4.2):

- Zone LED count, with min and max where resizable, and a flag saying whether the size may be set by hand.
- Segments: a named sub-range of a zone with start index and LED count, and flags marking whether segments may be configured by hand. The Effects plugin treats one segment like one zone.
- Zone types and matrix maps.
- Not exposed: position, rotation, what is plugged in, whether a splitter exists, which direction a fan wires.

What the user must declare by hand: which component sits on which zone or segment, in what order, whether branches are mirrored, and each component's direction and LED 1 position. PrismForge can pre-fill from OpenRGB's segments when they exist and from the zone LED count when they do not. The PrismForge file should keep its own slice ranges, so it works with zones that OpenRGB cannot segment.

What PrismForge can check automatically:

- Sum of chain LED counts against the zone or slice LED count (warn on mismatch, suggest a resize if the zone is resizable).
- Branch lengths of a splitter (warn when unequal).
- Header limits are board-specific and cannot be read, so give a gentle warning above 120 LEDs only as a hint (inference).

---

## 2. Mapping effects onto a 2D layout

### 2a. Sampling approaches

| Approach | Used by | Notes |
|---|---|---|
| Point sampling at each LED coordinate | QMK RGB Matrix: effects work from each LED's x and y, with centre-based and axis-based effects (https://docs.qmk.fm/features/rgb_matrix ). Lightloom's layout schema: each LED samples the canvas at its transformed position (https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/schemas/layout.schema.json ). | Cheapest. Matches a pure function of position. Prone to aliasing when features are thinner than the LED pitch. |
| Area (box) sampling of a rendered canvas | OpenRGB Visual Map: each LED owns a rectangle of canvas cells and takes the area-weighted average (earlier file 4.1). Hyperion: each LED has a rectangle on the picture and takes the mean colour (also a dominant-colour and a "uniform" mode in the code header): https://raw.githubusercontent.com/hyperion-project/hyperion.ng/master/include/hyperion/ImageToLedsMap.h (MIT). | Filters thin features by averaging. Needs a rendered canvas, so the effect must be rasterised first. |
| Supersampling | Not found as a feature in these systems. (not confirmed) | Standard graphics technique: several sample points inside one footprint, then average. It is the point-sample version of box filtering. |
| Shared buffer per group | xLights: a group effect renders to one shared grid, or per member model. "Minimal Grid" sizes the buffer to the group, because large grids render slowly. https://manual.xlights.org/xlights/chapters/chapter-four-tabs/model-groups.md | The group's bounding box is the effect domain. I reuse this idea. |
| Per-zone independent rendering | OpenRGB Effects plugin (earlier file 3.3): each zone renders from its own index 0 with a shared per-effect time. | This is today's PrismForge behaviour. |

Aliasing facts: sampling is unambiguous only if the sampling rate exceeds twice the highest frequency in the signal. Components above that limit appear as false slower patterns. A low-pass filter before sampling reduces this. Source (secondary): https://en.wikipedia.org/wiki/Aliasing .

What this means for sparse LEDs (inference from the sampling facts):

- A moving highlight narrower than about two LED pitches will light different LEDs at different times, or none. It looks like flicker or skipping.
- Standard remedies: (1) enforce a minimum feature width of about two local pitches in the effect (widen the highlight, do not shrink the LEDs); (2) filter by averaging over each LED's footprint, either analytically for simple profiles or by 4 to 9 sample points inside the footprint; (3) temporal smoothing as a last resort, because it blurs motion.
- WLED's segment "grouping" is a blunt form of remedy (1): it makes N neighbouring LEDs one virtual LED, so features cannot be narrower than a group. https://kno.wled.ge/features/segments/
- Whether averaging should be done in linear light or in the encoded colour space is a visual choice I did not source. Test both on real LEDs.

### 2b. Coordinate conventions that keep effects resolution independent

| Topic | Finding |
|---|---|
| Fixed canvas versus content bounds | Lightloom fixes a 320 by 200 canvas. QMK fixes about 224 by 64. Visual Map uses a user-sized grid up to 1024 by 1024. xLights lets the group buffer follow the content ("Minimal Grid") or the full preview. Fixed canvases keep effect scale stable when devices are added. Content-fit domains keep an effect covering exactly the chosen targets. |
| Aspect ratio | Per-axis normalisation (0 to 1 on x and on y) distorts circles and angles on wide layouts. Normalising on the longer side keeps shapes and angles true. (inference) |
| Direction and angle parameters | Angles are meaningful only in a space with equal axis scales, which supports longer-side normalisation. QMK expresses effects as axis, centre, or per-key spread. |
| Local space versus scene space | Today: position is the normalised LED index in the device or zone. Scene mode: position is a 2D point. Both can feed one effect function if the effect receives a position struct with both fields. |
| Generalising a 1D wave | Axis wave: project the point onto a direction vector, `s = dot(p - origin, d) / extent`. Radial: `r = |p - c| / extent`. Angular: `theta = atan2(...)`. A 1D index is the special case `s = index / (count - 1)` with `d = x`. Effects should declare which projection they use. (inference) |

### 2c. Phase and continuity across devices, and the "apply" meaning

- Phase: if every device is sampled with the same scene time `t` inside one render pass, they are in phase by construction. In OpenRGB's Effects plugin, one effect instance shares one progress counter across its zones, so zones inside one effect are in phase, but separate effect instances are not synchronised (earlier file 3.3).
- Residual skew comes from transport: per-controller update queues and per-device latency. I found no source that quantifies it. It must be measured on the owner's hardware.
- Continuity across gaps: a spatial function is continuous in the gaps by definition, since it is defined everywhere. Gaps only decide which parts of it are visible. A wave crossing from the case to the desk only looks continuous if the layout's relative positions are plausible. That is a layout truth issue, not a maths issue.
- Without a layout: independent per device (each zone from index 0). With a layout: one picture across the placed targets.
- How products present the choice, in general terms: SignalRGB's public docs say devices run effects independently without a layout and flow as one picture with one (source above). Corsair's public guide describes "Lighting Link" as an effect applied across a chosen set of devices, which it says needs the software running and is not stored on the hardware: https://www.corsair.com/explorer/gamer/accessories/how-to-set-up-lighting-link-in-icue/ (vendor material; search snippet). xLights lets a group render combined or per model. OpenRGB's Effects plugin renders per zone. So the choice is common, but I found no product that labels the two modes side by side. (inference)

### 2d. Performance

I found no benchmark of per-LED pure-function evaluation. Arithmetic with an assumed cost: 3,000 LEDs at 60 Hz is 180,000 evaluations per second. At an assumed 1 microsecond each, that is 0.18 s of CPU per second, about 18 per cent of one core. At 100 nanoseconds each it is under 2 per cent. The 1 microsecond figure is an assumption, not a measurement. The footprint filter multiplies the cost by the sample count. This must be measured in release builds on the target machines. The bigger cost is more likely OpenRGB transport and the preview rendering (see 3d).

---

## 3. Views: PC view and desk view

### 3a. Schematic versus realistic; templates versus blank

- I found no direct usability study of schematic versus realistic views in configuration tools. Academic results I found are about learning, are mixed, and one says there is no clear optimal degree of realism. A paper on engineered-system displays says little research exists on how people read such diagrams. Searched pages: https://link.springer.com/10.1007/s10648-021-09638-1 and https://typeset.io/pdf/diagrammatic-displays-for-engineered-systems-effects-on-4e2i5yw0jt.pdf (snippets only; not opened). So treat the schematic choice as an engineering decision.
- Honesty argument (inference): a schematic does not pretend to know the physical size or the exact fan model. A realistic or 3D render of an unknown case and unknown fans would imply knowledge the app does not have. The UX file already recommends a flat setup canvas with natural mapping (https://www.nngroup.com/articles/natural-mappings/) and a list twin.
- Templates: defaults reduce effort and guide users (https://www.nngroup.com/articles/the-power-of-defaults/). The UX file recommends "Mid tower" and "Desk" templates that stay editable, with "Reset to template". I add (inference): a template should place unbound component slots, not bind devices silently.
- Blank-canvas prior art: SignalRGB's public docs suggest starting with every device hidden and unhiding one at a time. This is a workflow, not evidence it is better.
- Library prior art: xLights has "Download/Import Models" (https://manual.xlights.org/xlights/chapters/chapter-four-tabs/download-import-models.md ).

### 3b. Minimal first component library (generic shapes, original to PrismForge)

| Component | Geometry parameters | User supplies | Readable from OpenRGB |
|---|---|---|---|
| Strip | LEDs, length, folds, direction | Direction, folds, where it sits | Zone LED count, type linear |
| Ring fan | Rings (LED count, radius), start angle, direction, face | Ring split, start angle, direction | Total LED count, loop type hint |
| Dual-ring fan | Same as ring, two rings (outer and inner), plus optional hub | Which ring is first in the wire | Total LED count |
| Bar (RAM, light bar) | LEDs, length, direction | Direction (usually obvious) | LED count, type linear |
| Block (GPU, cooler top plate) | Strip, ring, or small matrix inside a rectangle outline | Inner shape, LED count, direction | LED count, maybe matrix map |
| Matrix (keyboard, panel) | Rows, columns, gap mask, per-key positions | Usually nothing | Matrix map, LED names |
| Perimeter (monitor backlight) | Counts per side, start corner, direction, aspect ratio | All of them | Total LED count |
| Point (logo) | One LED | Position | Zone type single |
| Outline-only shapes (case, desk, monitor, PC icon) | Rectangle, size | Size and label | Nothing. Non-lit, drawn for context. |
| Free polyline | Points, LED count | Points | LED count |

### 3c. Several views of the same hardware

- Figma's component model is a general prior art for "one definition, many placements": a main component, linked instances, and overrides. https://help.figma.com/hc/en-us/articles/360038662654-Guide-to-components-in-Figma (proprietary tool; concept only).
- xLights lets one model belong to several groups, and a group effect can render on one shared grid or per member. https://manual.xlights.org/xlights/chapters/chapter-four-tabs/model-groups.md
- Lightloom keeps several named layouts and its layout schema places each endpoint once per layout. https://github.com/MrMoraleDev/Lightloom/blob/b8b482b6597fc03782ab4f804cede7dcf654c673/schemas/layout.schema.json

The two candidate models:

| Model | Truth | Cost |
|---|---|---|
| One global scene with groups | One source of truth for position. Physical scale is implied, which is a problem: a PC is tiny on a desk, so an effect over the desk barely varies across the PC. | PC view needs zoom into a group. Desk view needs a collapsed drawing for the group. |
| Separate layouts that reference the same components, and a layout can embed another layout as a rigid item | Each view has its own honest placement. No false claim about physical scale. Effects need a stated domain. | A component can appear in more than one layout, so sampling needs a rule (see recommendation). |

### 3d. Rendering technology

- MDN's canvas tutorials say: pre-render repeated drawing, use integer coordinates, batch calls, layer several canvases by update rate, and use `requestAnimationFrame`. https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API/Tutorial/Optimizing_canvas and https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API/Tutorial/Basic_animations
- I found no MDN, web.dev or vendor page that gives element-count thresholds for SVG versus canvas. The MDN SVG-in-HTML page does not discuss performance.
- One academic poster compared SVG, Canvas and WebGL on tree graphs: WebGL was faster, SVG was on par with Canvas, and all dropped below smooth frame rates above about 400 nodes, which the poster says is about 8,000 graphical elements. It measured interaction frame rates in its own setup, not 30 Hz colour updates. https://mt.inf.tu-dresden.de/cnt/uploads/Horak-2018-Graph-Performance-Poster.pdf (opened locally as text). Secondary opinion pieces give thresholds from 1,000 to 5,000 elements and disagree (search results: https://smus.com/canvas-vs-svg-performance/ , https://www.jointjs.com/blog/svg-versus-canvas). Treat them as unreliable.
- Tauri 2 uses WebView2 on Windows, WKWebView on macOS and webkit2gtk on Linux, so performance differs by platform: https://v2.tauri.app/reference/webview-versions/
- Canvas accessibility: MDN says canvas needs fallback content inside the element, and that screen readers read it: https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API/Tutorial/Basic_usage . WCAG says charts and diagrams need a short text alternative plus a long description with the data in a table where practical: https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html . The UX file already requires a list twin and DOM or SVG items for canvas objects.
- Inference: a few thousand LED dots is small for canvas and borderline for SVG when every dot's fill changes 30 times a second, because each change touches the DOM. Measure.

### 3e. Photosensitivity

From WCAG 2.2 SC 2.3.1 (https://www.w3.org/WAI/WCAG22/Understanding/three-flashes-or-below-threshold.html ):

- A general flash is a pair of opposing changes in relative luminance, each at least 0.10 of the 0 to 1 scale, where the darker state is below 0.80.
- A red flash is a pair of opposing transitions involving saturated red (one state with R/(R+G+B) at least 0.8, and a chromaticity difference above 0.2 in CIE 1976 u'v').
- Content passes if there are at most three general flashes and at most three red flashes in any one second, or if the flashing area is small.
- The area reference is about 341 by 256 CSS pixels for the 10 degree field, and the allowed area is 25 per cent of that, about 21,800 CSS pixels (the page's own calculation reads as 87,296 px in total). Content is judged at the largest scale users can view it.
- AAA SC 2.3.2 (https://www.w3.org/WAI/WCAG22/Understanding/three-flashes.html ) allows no more than three flashes per second at any size.
- Fine balanced patterns (a checkerboard with squares under 0.1 degree) are excluded from the definition of a flash.
- ITU-R BT.1702 is the television guidance behind this. I did not confirm its rate and area figures: https://www.itu.int/dms_pubrec/itu-r/rec/bt/R-REC-BT.1702-3-202311-I!!PDF-E.pdf (not opened).
- WCAG covers web content, not physical LEDs. A preview of a strobing effect is web content, so a large preview falls under it. The UX file already sets a three-flashes-per-second target and a reduced-motion pause.

What follows for a preview (inference):

- A large preview of a full-screen flash is above the area exception, so the three-per-second rule is the real limit.
- Add a preview limiter: measure the luminance of the preview area, and if it would exceed three reversals per second, blend towards a held frame. Label it "Preview softened" so it does not lie about the hardware.
- Start paused when the OS asks for reduced motion (UX file). Apply the same cap to any effect marked "flashing".
- Do not rely on the fine-pattern exception, because a dense LED preview is not a proven balanced pattern.

---

## 4. Interaction conventions for the layout editor

| Topic | Convention and source | Suggestion for PrismForge |
|---|---|---|
| Snapping and grid | Lightloom's layout has a snap grid in canvas units, default 4 (its schema). Visual Map has snap-to-grid and grid size (earlier file 4.1). Figma snaps to objects, edges, centres and the pixel grid, and Ctrl temporarily turns it off: https://help.figma.com/hc/en-us/articles/360039956914-Adjust-alignment-rotation-and-position and https://help.figma.com/hc/en-us/articles/360041065034-Adjust-your-zoom-and-view-options | Grid and object snap on by default. A held modifier disables it. A toolbar toggle makes it visible. Snap distance in screen pixels, not scene units. (inference) |
| Alignment and distribution | Figma: align several layers to each other, distribute evenly keeping outer layers fixed, Alt plus letter shortcuts. xLights shows guide lines when a model aligns on an axis: https://manual.xlights.org/xlights/chapters/chapter-four-tabs/models.md | Align (six) and distribute (two) buttons for a multi-selection, plus live guide lines while dragging. All must work without a drag, per WCAG 2.5.7 (UX file). |
| Rotation in fixed increments | Figma: Shift snaps rotation to 15 degrees, numeric entry, pivot at centre. Inkscape (search snippet, page not opened): Ctrl snaps to 15 degrees by default, and `[` and `]` rotate by the snap angle. Lightloom allows 0 to 359 degrees about the top-left corner. Visual Map allows 90 degree steps only (earlier file). | Default snap 15 degrees, numeric field, quick buttons for 90 degrees. Rotate about the centre. Keep rotation and flip as separate transforms, as Figma does. |
| Mirroring (view) | Figma: separate flip horizontal and vertical (Shift+H, Shift+V). Lightloom: `flipX`, `flipY`. | Offer "Flip view" in the inspector. Keep it separate from "Reverse LED direction", which changes the electrical order. Explain the difference in one line. |
| Grouping | xLights model groups (many-to-many). Figma groups and frames. | Groups are spatial containers with a transform. Limit nesting to two levels. |
| Z-order | Figma has bring forward and send backward (shortcuts not confirmed on the pages I read). | Bring forward and send backward buttons plus `]` and `[`. Only matters where outlines overlap LEDs. |
| Duplicate | Common shortcut Ctrl+D (not confirmed in the pages read). | A duplicate has no output binding. It must be assigned, so duplicating never creates a hidden mirror. |
| Marquee selection | Figma has a keyboard box-selection mode (Ctrl+Space, then hold Ctrl and use arrows): https://help.figma.com/hc/en-us/articles/360040328653-Use-Figma-products-with-a-keyboard . Mouse marquee on empty canvas is standard in Inkscape (search snippet). | Drag on empty canvas to marquee. Shift adds. Provide the list view as the keyboard alternative (UX file). |
| Zoom and pan limits | Figma: zoom to fit Shift+1, to selection Shift+2, wheel with Ctrl, the zoom value is shown and can be typed. Zoom limits are not documented on the pages I read. | Fit and Zoom to selection with shortcuts. Clamp zoom (suggest 10 per cent to 800 per cent) and clamp panning to the scene extent plus a margin. The numbers are my suggestion. |
| Coordinate readouts | Figma shows X and Y from the top-left in the design panel, accepts maths expressions, and keeps X and Y relative to the unrotated corner after rotation. | Inspector with X, Y, size, rotation. Cursor coordinates in the status line. Use the item centre as the reference, to avoid the rotated-corner confusion. (inference) |
| Nudging | Figma: arrows move 1, Shift moves 10 by default, both configurable. Inkscape (snippet): arrows move 2 px, Shift moves ten times that. | Arrow nudges one grid step, Shift nudges ten. |
| "Which LED is first" marker | xLights marks the start of a single-line model with a green handle and the end with a blue handle, offers a wiring view, and has numbered nodes in a node layout dialog (models page above). The UX file recommends a filled dot with the number 1 and an arrow along the path. | Always draw LED 1 as a filled dot with "1", an arrow along the path, and the LED count. Show LED numbers on hover or in a "Show numbers" toggle. |
| Identify on hardware | The UX file recommends an Identify action that restores prior state. | Component-level "Identify": light that component's LEDs with a slow chase from LED 1, other LEDs dark, no strobing, with a visible stop control. The chase shows both order and direction. It needs a Direct-mode connection, so it must report failure honestly (UX file). |
| Undo and redo for drags | tldraw: a history mark placed at the start of a gesture, changes batch until the mark flushes, one undo reverts the whole drag, and "bail" cancels without adding a redo entry: https://tldraw.dev/sdk-features/history . A spec by Grida and a design-editor note describe begin, transient updates, and commit (search snippets; not opened). | One undo step per completed drag, rotate, or numeric edit (commit on Enter or blur). Escape cancels a drag and leaves no redo entry. Undo restores the selection with the change. Template apply is one step. |

---

## 5. Recommended model for PrismForge (my recommendation)

Everything in this section is a recommendation. None of it is copied from a source. Names and numbers are my own.

### 5.1 The data model in plain words

- A **component** is a physical lit thing with a geometry: strip, ring fan, bar, block, matrix, perimeter, point, or polyline. Geometry is parametric. Local LED points are computed from the parameters, not stored, except for polylines. Local space is unit-less and normalised on the component's longer side.
- An **output** is one controllable channel: a slice of one OpenRGB zone or segment (device match, zone index, first LED, LED count). The device match uses name, serial and location, as the earlier file shows other plugins do.
- An output has **branches**. A branch is an ordered list of components, which is a daisy chain. One branch is a plain chain. Several branches is a passive splitter, and every branch receives the same stream from LED 0 of the output.
- A **layout** is a named schematic placement: a fixed extent, and items. An item is either a component placement (position, size, rotation in 15 degree steps, flip, z) or another layout embedded as a rigid group (position, size, rotation).
- A **component never needs a placement to exist**. Unplaced components keep working in per-device mode.
- The PC view and the desk view are two layouts. The desk layout embeds the case layout as a group. Components are defined once.

Why branches and not a mirror flag: a branch list states the physics (same stream, different fans) and handles chains inside a splitter with one structure. A mirror flag cannot say how many LEDs each branch uses.

### 5.2 Illustrative JSON sketch (my own invention, not a file format commitment)

```json
{
  "version": 1,
  "components": [
    { "id": "fan-a", "name": "Front fan 1", "geometry": {
        "kind": "ring",
        "rings": [ { "leds": 12, "radius": 1.0 }, { "leds": 8, "radius": 0.55 } ],
        "ledOneAngleDeg": 0, "direction": "cw", "wireOrder": ["outer", "inner"] } },
    { "id": "fan-b", "name": "Front fan 2", "geometry": { "kind": "ring",
        "rings": [ { "leds": 12, "radius": 1.0 } ], "ledOneAngleDeg": 0, "direction": "cw" } },
    { "id": "ram-1", "name": "RAM slot 2", "geometry": {
        "kind": "bar", "leds": 6, "length": 1.0, "direction": "up" } }
  ],
  "outputs": [
    { "id": "out-1",
      "source": { "device": { "name": "Example board", "serial": "", "location": "example" },
                  "zone": 1, "firstLed": 0, "ledCount": 32 },
      "branches": [ [ "fan-a", "fan-b" ] ] },
    { "id": "out-2",
      "source": { "device": { "name": "Example board", "serial": "", "location": "example" },
                  "zone": 2, "firstLed": 0, "ledCount": 6 },
      "branches": [ [ "ram-1" ] ] }
  ],
  "layouts": [
    { "id": "case", "name": "PC view", "extent": { "w": 160, "h": 100 },
      "items": [
        { "component": "fan-a", "x": 20, "y": 30, "size": 24, "rotationDeg": 0, "flip": false, "z": 1 },
        { "component": "fan-b", "x": 50, "y": 30, "size": 24, "rotationDeg": 0, "flip": false, "z": 1 },
        { "component": "ram-1", "x": 90, "y": 20, "size": 18, "rotationDeg": 90, "flip": false, "z": 2 }
      ] },
    { "id": "desk", "name": "Desk view", "extent": { "w": 160, "h": 100 },
      "items": [
        { "layout": "case", "x": 100, "y": 10, "size": 40, "rotationDeg": 0, "flip": false, "z": 1 }
      ] }
  ]
}
```

The numbers in the sketch (12 LEDs, 8 LEDs, 6 LEDs) are placeholders. They are not the owner's hardware.

### 5.3 The sampling rule

1. For an "across layout" run, pick the **domain**: the chosen layout, or a chosen set of its items. The domain bounding box, normalised on its longer side, defines scene position `p` in 0 to 1 (longer side) and a matching shorter side.
2. For every LED of every **primary** component in the domain, compute the scene point: local LED point, then the item transform (and any embedding transforms), then domain normalisation.
3. Evaluate the effect at `(params, t, p, seed)` with a footprint of half the local LED pitch. If the effect offers an analytic area average, use it. Otherwise average 5 samples inside the footprint (centre plus four offsets).
4. The effect's feature width is raised to at least two local pitches (the maximum pitch among the LEDs it covers).
5. Write the colour to the output LED index through the output's branch layout. The primary is the first branch. Other branches are display-only: they show the same output colours and add no samples.
6. Seed per LED comes from the output id and LED index, so random effects match on mirrored branches and stay stable across runs.
7. A component placed more than once inside one domain is a validation error. Embedding picks the placement from the chosen layout.

The numbers (2 pitches, 5 samples) are starting values to tune on real hardware.

### 5.4 The two apply modes

- **Per device** (today's behaviour): position is the normalised index inside each zone or component. No layout needed. Each target is in phase because of the shared clock, but each shows its own full cycle.
- **Across layout**: one picture over the chosen domain, same shared clock `t`. Targets without a placement are listed ("2 devices have no place in this layout") and the user chooses to skip them or run them per device. The UI must say which mode is active at all times, with two cues (UX file on modes).

### 5.5 First component library

Strip, ring fan, dual-ring fan, bar, block, matrix, perimeter, point, free polyline, plus unlit outline shapes (case, desk, monitor, PC). Ship two editable templates: "Mid tower" and "Desk" (unbound slots).

### 5.6 What the user must declare

- Which component is on which output, in order.
- Splitter or chain for each output, and the LED count of each component.
- Direction and LED 1 position of each ring or strip.
- Placement in each layout (drag, click-to-place, or numeric).
- That a fan is dual-sided or shows a particular face.

PrismForge can pre-fill LED counts, matrix maps, and (where present) segments from OpenRGB, and can propose a geometry from the zone type.

### 5.7 What must be validated on real hardware

1. The first-LED position and direction of the owner's fans, RAM, GPU and cooler plate, using the chase "Identify".
2. Hub and ring wire order of the dual-ring and hub-ring fans. Do not trust any catalogue (the Lightloom SL120 mismatch is a warning).
3. Splitter behaviour: do branches with different LED counts show the first LEDs of the stream (my expectation), and do all branches stay in step.
4. Daisy-chain behaviour: LED counts per fan, and whether the zone can be resized to the sum.
5. The effect of the footprint filter and minimum feature width on a real fan at 30 and 60 Hz. Check linear-light versus encoded-space averaging.
6. Cross-device phase: measure skew between devices on different controllers.
7. Preview rendering speed at the maximum LED count, on WebView2, WKWebView and webkit2gtk.
8. Per-evaluation cost of the effect functions in a release build.
9. Header limits on the owner's motherboard (board-specific).
10. Direct-mode behaviour when OpenRGB's own profiles or other programs also write.

### 5.8 Rendering recommendation

Canvas 2D for the LED layer (one canvas, redrawn at 30 Hz, with integer coordinates and a pre-rendered static layer for outlines), and SVG overlay for component outlines, labels, handles and selection. Add the list twin with positions, LED counts and the output binding (UX file and WAI). If the measurement in 5.7 item 7 shows SVG alone is fast enough on all three WebViews, SVG for everything is simpler and more accessible. Do not use WebGL for the first release. WebGL was fastest in the one study I found, but a few thousand dots do not justify its cost, and text is harder (the poster notes blurry text). (inference)

---

## Sources

All accessed 2026-11-19. Licence given where I found it.

Open systems:

- https://kno.wled.ge/advanced/mapping/ , https://kno.wled.ge/features/segments/ , https://kno.wled.ge/advanced/2d-1d-Mixed-Setup/ , https://kno.wled.ge/features/effects/ - WLED docs (MIT, https://github.com/wled/WLED-Docs). WLED code: EUPL-1.2 (GitHub API).
- https://docs.hyperion-project.org/user/advanced/Advanced.html - Hyperion docs (MIT). https://raw.githubusercontent.com/hyperion-project/hyperion.ng/master/libsrc/hyperion/schema/schema-ledConfig.json and https://raw.githubusercontent.com/hyperion-project/hyperion.ng/master/include/hyperion/ImageToLedsMap.h - Hyperion.ng (MIT).
- https://docs.qmk.fm/features/rgb_matrix - QMK. qmk_firmware GPL-2.0 (GitHub API).
- https://manual.xlights.org/xlights/llms.txt and the `chapter-four-tabs` pages named above - xLights manual. xLights code GPL-3.0 (GitHub API). Manual licence not confirmed.
- https://github.com/MrMoraleDev/Lightloom at commit `b8b482b6597fc03782ab4f804cede7dcf654c673`: `README.md`, `schemas/layout.schema.json`, `schemas/component.schema.json`, `components/README.md`, `LICENSE-EXCEPTION.md`, `docs/adr/` file names. GPL-3.0-or-later with add-on exception. Components CC0-1.0 per `components/README.md`. I looked at the first lines of `docs/host-api/components-channels.md` only to identify it, and did not use it.
- OpenRGB 1.1 and plugins: the earlier file `openrgb-behaviour.md` (GPL-2.0-or-later).
- https://docs.signalrgb.com/guides/device-configuration/about-layouts/ - SignalRGB, proprietary; used only to confirm that such products offer 2D layouts (general description).
- https://www.corsair.com/explorer/gamer/accessories/how-to-set-up-lighting-link-in-icue/ - vendor guide, search snippet; general description only.

Hardware examples:

- https://www.corsair.com/us/en/p/memory/cmg32gx4m2e3200c16/vengeance-rgb-rs-32gb-2-x-16gb-ddr4-dram-3200mhz-c16-memory-kit-cmg32gx4m2e3200c16 (opened).
- https://bit-tech.net/news/tech/cooling/corsair-launches-icue-ql-rgb-fans-34-leds-apiece/1 , https://legitreviews.com/corsair-ql-series-ql120-rgb-led-fan-kit-review_217257/2 , https://nzxt.com/products/f120-rgb-duo , https://lian-li.com/product/uni-fan-sl/ , https://www.vortez.net/news_story/lian_li_introduces_uni_fan_sl120_cooling_fan.html , https://learn.adafruit.com/adafruit-neopixel-uberguide/neopixel-strips , https://www.asus.com/jp/support/faq/1056087/ , https://www.memoryexpress.com/Products/MX00113030 (all from search results, not opened in full).

Views, rendering and accessibility:

- https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API/Tutorial/Optimizing_canvas , .../Basic_animations , .../Basic_usage (opened; MDN content is CC-BY-SA).
- https://mt.inf.tu-dresden.de/cnt/uploads/Horak-2018-Graph-Performance-Poster.pdf (opened locally).
- https://smus.com/canvas-vs-svg-performance/ , https://www.jointjs.com/blog/svg-versus-canvas (snippets only).
- https://v2.tauri.app/reference/webview-versions/ (opened).
- https://www.w3.org/WAI/WCAG22/Understanding/three-flashes-or-below-threshold.html , https://www.w3.org/WAI/WCAG22/Understanding/three-flashes.html , https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html , https://www.w3.org/WAI/WCAG22/Understanding/dragging-movements.html (opened).
- https://www.itu.int/dms_pubrec/itu-r/rec/bt/R-REC-BT.1702-3-202311-I!!PDF-E.pdf (search listing only).
- https://www.nngroup.com/articles/the-power-of-defaults/ , https://www.nngroup.com/articles/natural-mappings/ (the latter via the UX file).
- https://en.wikipedia.org/wiki/Aliasing (secondary; opened).
- https://link.springer.com/10.1007/s10648-021-09638-1 , https://typeset.io/pdf/diagrammatic-displays-for-engineered-systems-effects-on-4e2i5yw0jt.pdf (snippets only).

Editor conventions:

- https://help.figma.com/hc/en-us/articles/360039956914-Adjust-alignment-rotation-and-position , https://help.figma.com/hc/en-us/articles/360041065034-Adjust-your-zoom-and-view-options , https://help.figma.com/hc/en-us/articles/360040328653-Use-Figma-products-with-a-keyboard , https://help.figma.com/hc/en-us/articles/360038662654-Guide-to-components-in-Figma - proprietary tool documentation, cited for conventions only.
- https://tldraw.dev/sdk-features/history (opened; tldraw's own licence was not checked).
- Inkscape rotation and nudge conventions: search snippets of https://inkscape.org/doc/keys091.html (the page itself returned 403).
- https://grida.co/docs/wg/feat-history/ , https://docuslice.com/docs/undo-and-redo/ (snippets only).
- The earlier file `ux-research.md` for WCAG 2.5.7, 2.5.8, list twin, modes, identify.

## Open points

- WLED main 2D setup page and the Inkscape keys page were not readable. WLED first-LED corner and serpentine options come from search snippets of community pages. WLED segment mirror: not confirmed.
- xLights manual licence, 3D positioning, shared node cells and any mirroring feature: not confirmed.
- Lightloom: I did not read its engine code or non-schema docs for chains or mirrors. I read nothing from its reverse-engineering document beyond identifying it. The shape of a chain (ordered component list per header) comes from its README only.
- Several LED counts (QL120 split, NZXT Duo split, SL120 count, Adafruit densities, ASUS limits) come from search snippets, not opened manufacturer pages. They are examples only.
- Passive splitter behaviour with unequal branches is my inference.
- No usability study of schematic versus realistic configuration views, and no authoritative SVG versus canvas threshold. Both need an owner decision plus measurement.
- No benchmark of per-LED effect cost. Measure.
- ITU-R BT.1702 figures not opened.
- Ctrl+D (duplicate), z-order shortcuts and Figma zoom limits are not confirmed from the pages read. The suggested zoom clamps, snap distances and the 2 pitch and 5 sample numbers are my starting values.
- Group naming collision: the UX file already has "group" as a saved set of targets. A layout group (spatial) and a target group (saved set) should be unified or clearly separated. Owner decision.
- Licence: GPL-3.0-or-later PrismForge may read concepts from all sources above. Do not copy Lightloom's component JSON content even though it is CC0 unless provenance of each file is checked, because its format is declared SignalRGB-compatible.
