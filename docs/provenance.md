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
