// Generated from real sessions by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.
import type { Saved } from "../../lib/api/bindings/Saved";

export const savedNew: Saved = {
  "id": "p-0000000000000001",
  "changes": [
    {
      "device": "demo-graphics-card",
      "label": "Demo Graphics Card",
      "change": {
        "Added": {
          "now": {
            "Effect": {
              "zone": null,
              "effect": "rainbow-wave"
            }
          }
        }
      }
    },
    {
      "device": "demo-memory-1",
      "label": "Demo Memory Module",
      "change": {
        "Added": {
          "now": {
            "Effect": {
              "zone": null,
              "effect": "rainbow-wave"
            }
          }
        }
      }
    },
    {
      "device": "demo-light-bar",
      "label": "Demo Light Bar",
      "change": {
        "Added": {
          "now": {
            "HardwareMode": {
              "index": 1,
              "name": "Breathing"
            }
          }
        }
      }
    },
    {
      "device": "demo-memory-2",
      "label": "Demo Memory Module",
      "change": {
        "Added": {
          "now": {
            "External": {
              "label": "OpenRGB Effects plugin"
            }
          }
        }
      }
    }
  ],
  "frame_rate": null
};

export const previewChanges: Saved = {
  "id": "p-0000000000000001",
  "changes": [
    {
      "device": "demo-graphics-card",
      "label": "Demo Graphics Card",
      "change": {
        "Replaced": {
          "was": {
            "Effect": {
              "zone": null,
              "effect": "rainbow-wave"
            }
          },
          "now": {
            "Effect": {
              "zone": null,
              "effect": "moving-highlight"
            }
          }
        }
      }
    },
    {
      "device": "demo-memory-1",
      "label": "Demo Memory Module",
      "change": {
        "Removed": {
          "was": {
            "Effect": {
              "zone": null,
              "effect": "rainbow-wave"
            }
          }
        }
      }
    },
    {
      "device": "demo-motherboard",
      "label": "Demo Motherboard",
      "change": {
        "Added": {
          "now": {
            "Effect": {
              "zone": 1,
              "effect": "solid"
            }
          }
        }
      }
    }
  ],
  "frame_rate": {
    "was": 30,
    "now": 45
  }
};

export const savedChanges: Saved = {
  "id": "p-0000000000000001",
  "changes": [
    {
      "device": "demo-graphics-card",
      "label": "Demo Graphics Card",
      "change": {
        "Replaced": {
          "was": {
            "Effect": {
              "zone": null,
              "effect": "rainbow-wave"
            }
          },
          "now": {
            "Effect": {
              "zone": null,
              "effect": "moving-highlight"
            }
          }
        }
      }
    },
    {
      "device": "demo-memory-1",
      "label": "Demo Memory Module",
      "change": {
        "Removed": {
          "was": {
            "Effect": {
              "zone": null,
              "effect": "rainbow-wave"
            }
          }
        }
      }
    },
    {
      "device": "demo-motherboard",
      "label": "Demo Motherboard",
      "change": {
        "Added": {
          "now": {
            "Effect": {
              "zone": 1,
              "effect": "solid"
            }
          }
        }
      }
    }
  ],
  "frame_rate": {
    "was": 30,
    "now": 45
  }
};

export const previewNone: Saved = {
  "id": "p-0000000000000001",
  "changes": [],
  "frame_rate": null
};
