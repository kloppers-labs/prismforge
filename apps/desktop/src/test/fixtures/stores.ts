// Generated from real sessions by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.
import type { StoreView } from "../../lib/api/bindings/StoreView";

export const storeEmpty: StoreView = {
  "kind": "Demo",
  "folder": "store-folder/demo-store",
  "summary": {
    "profiles": [],
    "unreadable": [],
    "apply_on_connect": null,
    "active": null,
    "unsaved_changes": null,
    "notices": []
  },
  "names": [],
  "name_limit": 60
};

export const storeSaved: StoreView = {
  "kind": "Demo",
  "folder": "store-folder/demo-store",
  "summary": {
    "profiles": [
      {
        "id": "p-0000000000000001",
        "name": "Evening",
        "devices": 4,
        "effects": 2,
        "modes": 1,
        "external": 1,
        "resolution": {
          "resolved": 4,
          "unavailable": 0,
          "missing": 0,
          "ambiguous": 0
        }
      }
    ],
    "unreadable": [],
    "apply_on_connect": null,
    "active": "p-0000000000000001",
    "unsaved_changes": 0,
    "notices": []
  },
  "names": [],
  "name_limit": 60
};

export const storeUnsaved: StoreView = {
  "kind": "Demo",
  "folder": "store-folder/demo-store",
  "summary": {
    "profiles": [
      {
        "id": "p-0000000000000001",
        "name": "Evening",
        "devices": 4,
        "effects": 2,
        "modes": 1,
        "external": 1,
        "resolution": {
          "resolved": 4,
          "unavailable": 0,
          "missing": 0,
          "ambiguous": 0
        }
      }
    ],
    "unreadable": [],
    "apply_on_connect": null,
    "active": "p-0000000000000001",
    "unsaved_changes": 4,
    "notices": []
  },
  "names": [],
  "name_limit": 60
};

export const storeNamed: StoreView = {
  "kind": "Demo",
  "folder": "store-folder/demo-store",
  "summary": {
    "profiles": [
      {
        "id": "p-0000000000000001",
        "name": "Evening",
        "devices": 5,
        "effects": 2,
        "modes": 1,
        "external": 1,
        "resolution": {
          "resolved": 5,
          "unavailable": 0,
          "missing": 0,
          "ambiguous": 0
        }
      }
    ],
    "unreadable": [],
    "apply_on_connect": null,
    "active": "p-0000000000000001",
    "unsaved_changes": 0,
    "notices": []
  },
  "names": [
    {
      "device": "demo-motherboard",
      "name": null,
      "zones": [
        {
          "zone": 1,
          "name": "Front fans"
        }
      ]
    },
    {
      "device": "demo-graphics-card",
      "name": "Desk card",
      "zones": []
    }
  ],
  "name_limit": 60
};

export const storeTwo: StoreView = {
  "kind": "Demo",
  "folder": "store-folder/demo-store",
  "summary": {
    "profiles": [
      {
        "id": "p-0000000000000001",
        "name": "Evening",
        "devices": 5,
        "effects": 2,
        "modes": 1,
        "external": 1,
        "resolution": {
          "resolved": 5,
          "unavailable": 0,
          "missing": 0,
          "ambiguous": 0
        }
      },
      {
        "id": "p-0000000000000002",
        "name": "Evening (2)",
        "devices": 5,
        "effects": 2,
        "modes": 1,
        "external": 1,
        "resolution": {
          "resolved": 3,
          "unavailable": 0,
          "missing": 0,
          "ambiguous": 2
        }
      }
    ],
    "unreadable": [],
    "apply_on_connect": null,
    "active": "p-0000000000000001",
    "unsaved_changes": 0,
    "notices": []
  },
  "names": [
    {
      "device": "demo-motherboard",
      "name": null,
      "zones": [
        {
          "zone": 1,
          "name": "Front fans"
        }
      ]
    },
    {
      "device": "demo-graphics-card",
      "name": "Desk card",
      "zones": []
    }
  ],
  "name_limit": 60
};

export const storeOffline: StoreView = {
  "kind": "Real",
  "folder": "store-folder/store",
  "summary": {
    "profiles": [
      {
        "id": "p-0000000000000003",
        "name": "Desk",
        "devices": 1,
        "effects": 1,
        "modes": 0,
        "external": 0,
        "resolution": null
      }
    ],
    "unreadable": [],
    "apply_on_connect": "p-0000000000000003",
    "active": null,
    "unsaved_changes": null,
    "notices": []
  },
  "names": [],
  "name_limit": 60
};

export const storeRecovered: StoreView = {
  "kind": "Demo",
  "folder": "store-folder/demo-store",
  "summary": {
    "profiles": [
      {
        "id": "p-0000000000000004",
        "name": "Night",
        "devices": 1,
        "effects": 1,
        "modes": 0,
        "external": 0,
        "resolution": {
          "resolved": 1,
          "unavailable": 0,
          "missing": 0,
          "ambiguous": 0
        }
      }
    ],
    "unreadable": [],
    "apply_on_connect": null,
    "active": null,
    "unsaved_changes": null,
    "notices": [
      {
        "file": "devices.json",
        "event": {
          "StartedEmpty": {
            "main": {
              "NotJson": {
                "detail": "expected value at line 1 column 1"
              }
            },
            "backup": null,
            "set_aside": [
              "devices.json.unreadable-1"
            ]
          }
        }
      },
      {
        "file": "profiles/p-0000000000000004.json",
        "event": {
          "RestoredFromBackup": {
            "problem": {
              "NotJson": {
                "detail": "expected value at line 1 column 1"
              }
            },
            "set_aside": "p-0000000000000004.json.unreadable-1",
            "rewritten": true
          }
        }
      }
    ]
  },
  "names": [],
  "name_limit": 60
};
