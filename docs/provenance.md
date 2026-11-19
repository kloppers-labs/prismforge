# Provenance

This file records where PrismForge material comes from. Update it in the same change that adds outside material.

## Current state

All code in this repository is original PrismForge code. The repository contains no code, text or assets from OpenRGB, the OpenRGB Effects plugin, Lightloom, Artemis or SignalRGB.

## Built-in effects

The six built-in effects in `crates/prismforge-effects` are original PrismForge code: solid colour, breathing, gradient, rainbow wave, colour cycle and moving highlight. They were written on 2026-11-18 from general lighting behaviour and basic arithmetic. No code from OpenRGB, the OpenRGB Effects plugin, SignalRGB, Lightloom or Artemis was read, copied or translated for them. The colour wheel, the fades and the blends are standard arithmetic written for this project.

## Rules

1. Record each outside source before you use it: project, version or commit, licence, and what you took.
2. Check the licence header of each file, not only the root licence of the project.
3. Do not copy, translate or extract code, effects, artwork, icons or text from SignalRGB.
4. Do not copy Artemis or Lightloom code unless a decision record approves it.
5. Do not commit invoices, local file paths, account data, serial numbers, or hardware logs with personal data.

## Sources read to build the OpenRGB client

These sources were read on 2026-11-18. No text or code from them is in this repository. The client in `crates/prismforge-openrgb` and the fake server in `crates/prismforge-testkit` are our own code, written from the described wire format.

| Source | Version | Licence stated in the file | Use |
|---|---|---|---|
| OpenRGB `Documentation/OpenRGBSDK.md` | tag `release_1.1`, commit `8e16de27858cf2be6af54a33ce003df74cb9fd3e` | No header. The project root licence is GPL version 2. | Wire format |
| OpenRGB `NetworkProtocol.h` | same commit | `GPL-2.0-or-later` | Packet IDs and limits |
| OpenRGB `NetworkServer.cpp` | same commit | `GPL-2.0-or-later` | Server behaviour that the document does not state |
| OpenRGB `RGBController/RGBController.h`, `RGBControllerInterface.h`, `RGBController.cpp` | same commit | `GPL-2.0-or-later` | Flag values, type values, colour byte order |
| OpenRGB Effects plugin source | commit `64928a4bccd1efdd05b366de38601c96b6bd59d7` and tag `release_1.1` | `GPL-2.0-or-later` | Study of the plugin's network interface. See `docs/decisions/0001-openrgb-integration.md`. |
| `openrgb2` crate | 0.4.1 | `GPL-2.0` | Evaluation only. Not used. |

## Sources read for plan 2 (experience)

These sources were read on 2026-11-19 for the reports in `docs/research/`. No text, code, artwork or component definition from them is in this repository.

| Source | Version | Licence stated | Use |
|---|---|---|---|
| OpenRGB source | commit `8e16de27858cf2be6af54a33ce003df74cb9fd3e` | `GPL-2.0-or-later` in file headers | How it loads profiles at events, and how it shows device colours |
| OpenRGB Effects plugin source | commit `f211d08afd3382318e60f829348b25c6e15fb0ae` | `GPL-2.0-or-later` in file headers | How it previews an effect and handles several zones |
| OpenRGB Visual Map plugin source | tag `release_1.0`, commit `1dde279e` | See `docs/research/openrgb-behaviour.md` | How it places zones on a grid |
| Public documents of WLED, Hyperion, QMK and xLights | See `docs/research/setup-views-research.md` | See that file | How open systems describe LED positions |
| Public guidance from Microsoft, Apple, W3C and Nielsen Norman Group | See `docs/research/ux-research.md` | Public guidance | Interface practice |

### Material that PrismForge must not use

- **The component format and the component files of Lightloom.** The Lightloom repository says that its component format is compatible with the SignalRGB component library. It also holds a document that describes itself as made from analysis of the proprietary SignalRGB application. Do not read that document for PrismForge work. Do not adopt that format, its field names or its component files, also where a file has a CC0 label.
- **SignalRGB layouts, components, device artwork and text.** The research names SignalRGB only to confirm, from its public pages, that such products have 2D layouts.

PrismForge defines its own setup model and its own generic component shapes.

## Licence of PrismForge

PrismForge is licensed under GPL-3.0-or-later. The owner chose this licence on 2026-11-18.

A dependency must have a licence that is compatible with GPL-3.0-or-later. Code under "GPL version 2 only" is not compatible.

## Adapted material

None.

Source files of OpenRGB and the Effects plugin with the header `GPL-2.0-or-later` can be adapted into a GPL-3.0-or-later project. Before any such use:

1. Get the owner's approval.
2. Check the header of each file.
3. Record the file, the commit and the change in this section.
4. Keep the original copyright notices.

This file is an engineering record. It is not legal advice.
