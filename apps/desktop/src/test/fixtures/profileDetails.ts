// Generated from real sessions by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.
import type { ProfileDetails } from "../../lib/api/bindings/ProfileDetails";

export const detailsEvening: ProfileDetails = {
  "id": "p-0000000000000001",
  "name": "Evening",
  "frame_rate": 45,
  "apply_on_connect": false,
  "checked": true,
  "devices": [
    {
      "reference": "demo-graphics-card",
      "label": "Demo Graphics Card",
      "status": {
        "Resolved": {
          "device": "demo-graphics-card"
        }
      }
    },
    {
      "reference": "demo-memory-1",
      "label": "Demo Memory Module",
      "status": {
        "Resolved": {
          "device": "demo-memory-1"
        }
      }
    },
    {
      "reference": "demo-light-bar",
      "label": "Demo Light Bar",
      "status": {
        "Resolved": {
          "device": "demo-light-bar"
        }
      }
    },
    {
      "reference": "demo-memory-2",
      "label": "Demo Memory Module",
      "status": {
        "Resolved": {
          "device": "demo-memory-2"
        }
      }
    },
    {
      "reference": "demo-motherboard",
      "label": "Demo Motherboard",
      "status": {
        "Resolved": {
          "device": "demo-motherboard"
        }
      }
    }
  ],
  "settings": [
    {
      "device": "demo-memory-2",
      "label": "Demo Memory Module",
      "zone_name": null,
      "item": {
        "External": {
          "label": "OpenRGB Effects plugin"
        }
      },
      "effect_label": null,
      "today": "Ready"
    },
    {
      "device": "demo-light-bar",
      "label": "Demo Light Bar",
      "zone_name": null,
      "item": {
        "HardwareMode": {
          "index": 1,
          "name": "Breathing"
        }
      },
      "effect_label": null,
      "today": "Ready"
    },
    {
      "device": "demo-graphics-card",
      "label": "Demo Graphics Card",
      "zone_name": null,
      "item": {
        "Effect": {
          "zone": null,
          "effect": "moving-highlight"
        }
      },
      "effect_label": "Moving highlight",
      "today": "Ready"
    },
    {
      "device": "demo-motherboard",
      "label": "Demo Motherboard",
      "zone_name": "Addressable Header 1",
      "item": {
        "Effect": {
          "zone": 1,
          "effect": "solid"
        }
      },
      "effect_label": "Solid colour",
      "today": "Ready"
    }
  ],
  "unsupported": []
};

export const detailsImported: ProfileDetails = {
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
};

export const detailsOffline: ProfileDetails = {
  "id": "p-0000000000000003",
  "name": "Desk",
  "frame_rate": 30,
  "apply_on_connect": true,
  "checked": false,
  "devices": [
    {
      "reference": "dev-5ae0445e22cbba19",
      "label": "Fixture Strip",
      "status": "Missing"
    }
  ],
  "settings": [
    {
      "device": "dev-5ae0445e22cbba19",
      "label": "Fixture Strip",
      "zone_name": null,
      "item": {
        "Effect": {
          "zone": null,
          "effect": "rainbow-wave"
        }
      },
      "effect_label": "Rainbow wave",
      "today": {
        "Skipped": {
          "reason": "Missing"
        }
      }
    }
  ],
  "unsupported": []
};
