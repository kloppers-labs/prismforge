// Generated from real sessions by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.
import type { IdentifyOutcome } from "../../lib/api/bindings/IdentifyOutcome";

export const restored: IdentifyOutcome = {
  "device": "demo-motherboard",
  "zone": 1,
  "light_failure": null,
  "restore": "Restored"
};

export const unchangedStaleInventory: IdentifyOutcome = {
  "device": "demo-graphics-card",
  "zone": null,
  "light_failure": "StaleInventory",
  "restore": "Unchanged"
};

export const partlyRestoredPreviousStateUnknown: IdentifyOutcome = {
  "device": "demo-graphics-card",
  "zone": null,
  "light_failure": null,
  "restore": {
    "PartlyRestored": {
      "failure": "PreviousStateUnknown"
    }
  }
};

export const notRestoredConnectionLost: IdentifyOutcome = {
  "device": "demo-graphics-card",
  "zone": null,
  "light_failure": null,
  "restore": {
    "NotRestored": {
      "failure": {
        "ConnectionLost": {
          "reason": {
            "ConnectionLost": {
              "cause": "Timeout"
            }
          }
        }
      }
    }
  }
};

export const notRestoredRefused: IdentifyOutcome = {
  "device": "dev-1052f35afd339289",
  "zone": null,
  "light_failure": null,
  "restore": {
    "NotRestored": {
      "failure": {
        "Refused": {
          "error": {
            "Rejected": {
              "operation": "packet 1101",
              "status": 5
            }
          }
        }
      }
    }
  }
};

export const unchangedLightRefused: IdentifyOutcome = {
  "device": "dev-1052f35afd339289",
  "zone": null,
  "light_failure": {
    "Rejected": {
      "operation": "packet 1100",
      "status": 5
    }
  },
  "restore": "Unchanged"
};
