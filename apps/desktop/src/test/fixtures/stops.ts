// Generated from real sessions by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.
import type { RestoreOutcome } from "../../lib/api/bindings/RestoreOutcome";

export const stopRestored: RestoreOutcome | null = "Restored";

export const stopOtherZonesStillRun: RestoreOutcome | null = null;

export const stopPreviousStateUnknown: RestoreOutcome | null = {
  "PartlyRestored": {
    "failure": "PreviousStateUnknown"
  }
};

export const stopNotRestoredConnectionLost: RestoreOutcome | null = {
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
};

export const stopNotRestoredRefused: RestoreOutcome | null = {
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
};
