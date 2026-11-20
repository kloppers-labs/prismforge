// Generated from real sessions by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.
import type { AcrossReport } from "../../lib/api/bindings/AcrossReport";

export const acrossDemoTower: AcrossReport = {
  "report": {
    "results": [
      {
        "target": {
          "device": "demo-motherboard",
          "zone": 1
        },
        "outcome": "Applied"
      },
      {
        "target": {
          "device": "demo-motherboard",
          "zone": 2
        },
        "outcome": "Applied"
      },
      {
        "target": {
          "device": "demo-motherboard",
          "zone": 0
        },
        "outcome": "Applied"
      },
      {
        "target": {
          "device": "demo-memory-1",
          "zone": null
        },
        "outcome": "Applied"
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
          "device": "demo-graphics-card",
          "zone": null
        },
        "outcome": "Applied"
      }
    ]
  },
  "excluded": [],
  "not_placed": [
    {
      "target": {
        "device": "demo-motherboard",
        "zone": 1
      },
      "leds": 22,
      "of": 30
    }
  ]
};

export const acrossDemoSkipped: AcrossReport = {
  "report": {
    "results": [
      {
        "target": {
          "device": "demo-motherboard",
          "zone": 1
        },
        "outcome": "Applied"
      },
      {
        "target": {
          "device": "demo-motherboard",
          "zone": 2
        },
        "outcome": "Applied"
      },
      {
        "target": {
          "device": "demo-memory-1",
          "zone": null
        },
        "outcome": "Applied"
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
          "device": "demo-graphics-card",
          "zone": null
        },
        "outcome": {
          "Skipped": {
            "error": {
              "Session": {
                "IdentifyInProgress": {
                  "device": "demo-graphics-card"
                }
              }
            }
          }
        }
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
  },
  "excluded": [
    {
      "output": "o-board-edge",
      "target": {
        "device": "demo-motherboard",
        "zone": 0
      },
      "label": "Demo Motherboard",
      "zone_name": "Board logo",
      "state": "Changed",
      "problem": null
    }
  ],
  "not_placed": [
    {
      "target": {
        "device": "demo-motherboard",
        "zone": 1
      },
      "leds": 22,
      "of": 30
    }
  ]
};

export const acrossRealTower: AcrossReport = {
  "report": {
    "results": [
      {
        "target": {
          "device": "dev-6f970707d354ee85",
          "zone": 0
        },
        "outcome": "Applied"
      },
      {
        "target": {
          "device": "dev-6f970707d354ee85",
          "zone": 1
        },
        "outcome": "Applied"
      },
      {
        "target": {
          "device": "dev-420d41cc43011cb2",
          "zone": null
        },
        "outcome": "Applied"
      }
    ]
  },
  "excluded": [],
  "not_placed": [
    {
      "target": {
        "device": "dev-6f970707d354ee85",
        "zone": 0
      },
      "leds": 28,
      "of": 36
    }
  ]
};
