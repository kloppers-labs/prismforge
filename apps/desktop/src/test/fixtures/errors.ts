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

export const invalidName: CommandError = {
  "Store": {
    "InvalidName": {
      "problem": "Empty"
    }
  }
};

export const notActive: CommandError = {
  "Store": {
    "NotActive": {
      "id": "p-0000000000000002",
      "active": "p-0000000000000001"
    }
  }
};

export const nameTaken: CommandError = {
  "Store": {
    "NameTaken": {
      "name": "Evening"
    }
  }
};

export const profileApplying: CommandError = "ProfileApplying";

export const sessionBusy: CommandError = {
  "Store": {
    "SessionUnavailable": {
      "error": "Busy"
    }
  }
};

export const settingNoAddress: CommandError = {
  "Setting": "NoAddress"
};

export const settingTrayUnavailable: CommandError = {
  "Setting": "TrayUnavailable"
};

export const settingStartEntryRefused: CommandError = {
  "Setting": {
    "StartEntry": {
      "wanted": true,
      "detail": "Access is denied."
    }
  }
};

export const settingStartEntryNotKept: CommandError = {
  "Setting": {
    "StartEntry": {
      "wanted": true,
      "detail": null
    }
  }
};

export const settingsInUse: CommandError = {
  "StoreUnavailable": "InUse"
};

export const errorSetupLedBounds: CommandError = {
  "Store": {
    "InvalidSetup": {
      "problem": {
        "ExceedsOutput": {
          "output": "o-cooler",
          "needed": 17,
          "available": 16
        }
      }
    }
  }
};

export const errorPlanUnknownView: CommandError = {
  "Plan": {
    "UnknownView": {
      "view": "v-gone"
    }
  }
};
