// Generated from real sessions by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.
import type { ApplyOutcome } from "../../lib/api/bindings/ApplyOutcome";

export const appliedPartly: ApplyOutcome = {
  "Applied": {
    "frame_rate": {
      "Applied": {
        "in_use": 45
      }
    },
    "items": [
      {
        "device": "device-4",
        "label": "Demo Memory Module",
        "target": null,
        "item": {
          "External": {
            "label": "OpenRGB Effects plugin"
          }
        },
        "outcome": {
          "Skipped": {
            "reason": "Ambiguous"
          }
        }
      },
      {
        "device": "device-3",
        "label": "Demo Light Bar",
        "target": "demo-light-bar",
        "item": {
          "HardwareMode": {
            "index": 1,
            "name": "Pulse"
          }
        },
        "outcome": {
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
        "target": "demo-graphics-card",
        "item": {
          "Effect": {
            "zone": null,
            "effect": "aurora"
          }
        },
        "outcome": {
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
        "target": "demo-motherboard",
        "item": {
          "Effect": {
            "zone": 1,
            "effect": "solid"
          }
        },
        "outcome": "Applied"
      }
    ]
  }
};

export const appliedFully: ApplyOutcome = {
  "Applied": {
    "frame_rate": {
      "Applied": {
        "in_use": 45
      }
    },
    "items": [
      {
        "device": "demo-memory-2",
        "label": "Demo Memory Module",
        "target": "demo-memory-2",
        "item": {
          "External": {
            "label": "OpenRGB Effects plugin"
          }
        },
        "outcome": "Applied"
      },
      {
        "device": "demo-light-bar",
        "label": "Demo Light Bar",
        "target": "demo-light-bar",
        "item": {
          "HardwareMode": {
            "index": 1,
            "name": "Breathing"
          }
        },
        "outcome": "Applied"
      },
      {
        "device": "demo-graphics-card",
        "label": "Demo Graphics Card",
        "target": "demo-graphics-card",
        "item": {
          "Effect": {
            "zone": null,
            "effect": "moving-highlight"
          }
        },
        "outcome": "Applied"
      },
      {
        "device": "demo-motherboard",
        "label": "Demo Motherboard",
        "target": "demo-motherboard",
        "item": {
          "Effect": {
            "zone": 1,
            "effect": "solid"
          }
        },
        "outcome": "Applied"
      }
    ]
  }
};

export const waitingForConnection: ApplyOutcome = "WaitingForConnection";
