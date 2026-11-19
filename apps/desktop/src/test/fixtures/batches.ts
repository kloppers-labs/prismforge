// Generated from real sessions by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.
import type { BatchReport } from "../../lib/api/bindings/BatchReport";

export const batchAppliedWithSkips: BatchReport = {
  "results": [
    {
      "target": {
        "device": "demo-graphics-card",
        "zone": null
      },
      "outcome": "Applied"
    },
    {
      "target": {
        "device": "demo-memory-1",
        "zone": null
      },
      "outcome": {
        "Skipped": {
          "error": {
            "Session": {
              "OutputConflict": {
                "device": "demo-memory-1",
                "owner": {
                  "External": {
                    "label": "OpenRGB Effects plugin"
                  }
                }
              }
            }
          }
        }
      }
    },
    {
      "target": {
        "device": "demo-memory-2",
        "zone": null
      },
      "outcome": "Applied"
    },
    {
      "target": {
        "device": "demo-motherboard",
        "zone": 1
      },
      "outcome": {
        "Covered": {
          "by": {
            "device": "demo-motherboard",
            "zone": null
          }
        }
      }
    },
    {
      "target": {
        "device": "demo-motherboard",
        "zone": null
      },
      "outcome": "Applied"
    },
    {
      "target": {
        "device": "demo-light-bar",
        "zone": null
      },
      "outcome": {
        "Skipped": {
          "error": {
            "Session": {
              "Backend": {
                "DirectModeUnavailable": {
                  "device": "demo-light-bar"
                }
              }
            }
          }
        }
      }
    }
  ]
};

export const batchStopped: BatchReport = {
  "results": [
    {
      "target": {
        "device": "demo-graphics-card",
        "zone": null
      },
      "outcome": {
        "Stopped": {
          "restore": "Restored"
        }
      }
    },
    {
      "target": {
        "device": "demo-memory-1",
        "zone": null
      },
      "outcome": {
        "Skipped": {
          "error": {
            "NoEffect": {
              "target": {
                "device": "demo-memory-1",
                "zone": null
              }
            }
          }
        }
      }
    }
  ]
};

export const stopAllReport: BatchReport = {
  "results": [
    {
      "target": {
        "device": "demo-memory-2",
        "zone": null
      },
      "outcome": {
        "Stopped": {
          "restore": "Restored"
        }
      }
    },
    {
      "target": {
        "device": "demo-motherboard",
        "zone": null
      },
      "outcome": {
        "Stopped": {
          "restore": "Restored"
        }
      }
    }
  ]
};
