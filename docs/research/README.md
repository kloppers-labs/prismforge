# Research

These files are research reports for `docs/plans/prismforge-plan-2-experience.md`. They were written on 2026-11-19.

| File | Content | How it was made |
|---|---|---|
| `ux-research.md` | Best practices for hardware configuration software and general HCI | Read of public sources. Each claim has a URL. |
| `ux-audit.md` | Audit of the PrismForge screens at commit `0397d7b`, with three navigation designs and a priority list | The interface was rendered in a browser with test data, and its code was read. |
| `openrgb-behaviour.md` | How OpenRGB 1.1, its Effects plugin and its Visual Map plugin handle events, previews and layouts | Read of their source code at pinned commits. |
| `setup-views-research.md` | Geometry models, sampling of effects in two dimensions, and layout editors | Read of public documents of open lighting projects. |

## How to read these files

- A research file gives facts and recommendations. It is not a decision. The plan and the decision records hold the decisions.
- Each file marks what its author could not open or could not confirm. Do not treat such a point as a fact.
- Number values for tuning are suggestions of the author, and the files mark them as such.
- The screenshots of the audit are not in this repository. The audit names them for reference only.
- No test in these files ran against a real OpenRGB server or real hardware.

## Rules for reuse

- These files describe the behaviour of other software in our own words. They contain no code and no artwork of other projects.
- Do not copy code, component definitions, artwork or text from a project that these files name. `docs/provenance.md` has the rules.
