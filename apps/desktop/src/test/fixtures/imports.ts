// Generated from real sessions by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.
import type { ImportReport } from "../../lib/api/bindings/ImportReport";

export const importedRenamed: ImportReport = {
  "renamed_from": "Evening",
  "details": {
    "id": "p-0000000000000002",
    "name": "Evening (2)",
    "frame_rate": 45,
    "apply_on_connect": false,
    "checked": true,
    "devices": [
      {
        "reference": "device-1",
        "label": "Demo Graphics Card",
        "status": {
          "Resolved": {
            "device": "demo-graphics-card"
          }
        }
      },
      {
        "reference": "device-2",
        "label": "Demo Memory Module",
        "status": "Ambiguous"
      },
      {
        "reference": "device-3",
        "label": "Demo Light Bar",
        "status": {
          "Resolved": {
            "device": "demo-light-bar"
          }
        }
      },
      {
        "reference": "device-4",
        "label": "Demo Memory Module",
        "status": "Ambiguous"
      },
      {
        "reference": "device-5",
        "label": "Demo Motherboard",
        "status": "Missing"
      }
    ],
    "settings": [
      {
        "device": "device-4",
        "label": "Demo Memory Module",
        "zone_name": null,
        "item": {
          "External": {
            "label": "OpenRGB Effects plugin"
          }
        },
        "effect_label": null,
        "today": {
          "Skipped": {
            "reason": "Ambiguous"
          }
        }
      },
      {
        "device": "device-3",
        "label": "Demo Light Bar",
        "zone_name": null,
        "item": {
          "HardwareMode": {
            "index": 1,
            "name": "Pulse"
          }
        },
        "effect_label": null,
        "today": {
          "Skipped": {
            "reason": {
              "ModeListChanged": {
                "found": "Breathing"
              }
            }
          }
        }
      },
      {
        "device": "device-1",
        "label": "Demo Graphics Card",
        "zone_name": null,
        "item": {
          "Effect": {
            "zone": null,
            "effect": "aurora"
          }
        },
        "effect_label": null,
        "today": {
          "Skipped": {
            "reason": {
              "NotAvailable": {
                "problem": {
                  "EffectNotAvailable": {
                    "effect": "aurora"
                  }
                }
              }
            }
          }
        }
      },
      {
        "device": "device-5",
        "label": "Demo Motherboard",
        "zone_name": "Addressable Header 1",
        "item": {
          "Effect": {
            "zone": 1,
            "effect": "solid"
          }
        },
        "effect_label": "Solid colour",
        "today": {
          "Skipped": {
            "reason": "Missing"
          }
        }
      }
    ],
    "unsupported": [
      {
        "device": "device-1",
        "zone": null,
        "problem": {
          "EffectNotAvailable": {
            "effect": "aurora"
          }
        }
      }
    ]
  }
};
