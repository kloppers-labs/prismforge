// Generated from real sessions by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.
import type { CommandError } from "../../lib/api/bindings/CommandError";

export const invalidBreathing: CommandError = {
  "InvalidEffect": {
    "InvalidValues": {
      "errors": [
        {
          "control": "brightness",
          "problem": {
            "OutOfRange": {
              "min": 0,
              "max": 100
            }
          }
        },
        {
          "control": "speed",
          "problem": {
            "OutOfRange": {
              "min": 0,
              "max": 120
            }
          }
        }
      ]
    }
  }
};

export const invalidGradientPalette: CommandError = {
  "InvalidEffect": {
    "InvalidValues": {
      "errors": [
        {
          "control": "palette",
          "problem": {
            "PaletteLength": {
              "min": 2,
              "max": 8
            }
          }
        }
      ]
    }
  }
};

export const outputConflict: CommandError = {
  "Session": {
    "OutputConflict": {
      "device": "demo-graphics-card",
      "owner": {
        "External": {
          "label": "OpenRGB Effects plugin"
        }
      }
    }
  }
};

export const noEffect: CommandError = {
  "NoEffect": {
    "target": {
      "device": "demo-motherboard",
      "zone": null
    }
  }
};
