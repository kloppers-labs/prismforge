// Generated from real sessions by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.
import type { LifecycleView } from "../../lib/api/bindings/LifecycleView";

export const lifecycleStartPath: LifecycleView = {
  "revision": 5,
  "settings": {
    "Ready": {
      "settings": {
        "on_close": null,
        "start_hidden": false,
        "connect_at_start": true,
        "address": {
          "host": "127.0.0.1",
          "port": 6742
        }
      },
      "notices": []
    }
  },
  "tray": "Available",
  "start_with_system": "Registered",
  "machine_start_entry": false,
  "window_visible": true,
  "quitting": false,
  "close_prompt": false,
  "tray_stop": null,
  "start_profile": {
    "Marked": {
      "name": "Evening"
    }
  },
  "folder": "store-folder"
};

export const lifecycleFresh: LifecycleView = {
  "revision": 2,
  "settings": {
    "Ready": {
      "settings": {
        "on_close": null,
        "start_hidden": false,
        "connect_at_start": false,
        "address": null
      },
      "notices": []
    }
  },
  "tray": "Available",
  "start_with_system": "NotRegistered",
  "machine_start_entry": false,
  "window_visible": true,
  "quitting": false,
  "close_prompt": false,
  "tray_stop": null,
  "start_profile": "NotMarked",
  "folder": "store-folder"
};

export const lifecycleAsking: LifecycleView = {
  "revision": 5,
  "settings": {
    "Ready": {
      "settings": {
        "on_close": null,
        "start_hidden": false,
        "connect_at_start": false,
        "address": null
      },
      "notices": []
    }
  },
  "tray": "Available",
  "start_with_system": "NotRegistered",
  "machine_start_entry": false,
  "window_visible": true,
  "quitting": false,
  "close_prompt": true,
  "tray_stop": null,
  "start_profile": "NotMarked",
  "folder": "store-folder"
};

export const lifecycleChosen: LifecycleView = {
  "revision": 7,
  "settings": {
    "Ready": {
      "settings": {
        "on_close": "Background",
        "start_hidden": true,
        "connect_at_start": true,
        "address": {
          "host": "192.168.1.20",
          "port": 6742
        }
      },
      "notices": []
    }
  },
  "tray": "Available",
  "start_with_system": "Registered",
  "machine_start_entry": false,
  "window_visible": true,
  "quitting": false,
  "close_prompt": false,
  "tray_stop": null,
  "start_profile": "NotMarked",
  "folder": "store-folder"
};

export const lifecycleHidden: LifecycleView = {
  "revision": 9,
  "settings": {
    "Ready": {
      "settings": {
        "on_close": "Background",
        "start_hidden": true,
        "connect_at_start": true,
        "address": {
          "host": "192.168.1.20",
          "port": 6742
        }
      },
      "notices": []
    }
  },
  "tray": "Available",
  "start_with_system": "Registered",
  "machine_start_entry": false,
  "window_visible": false,
  "quitting": false,
  "close_prompt": false,
  "tray_stop": null,
  "start_profile": "NotMarked",
  "folder": "store-folder"
};

export const lifecycleQuitChosen: LifecycleView = {
  "revision": 3,
  "settings": {
    "Ready": {
      "settings": {
        "on_close": "Quit",
        "start_hidden": false,
        "connect_at_start": false,
        "address": null
      },
      "notices": []
    }
  },
  "tray": "Available",
  "start_with_system": "NotRegistered",
  "machine_start_entry": false,
  "window_visible": true,
  "quitting": false,
  "close_prompt": false,
  "tray_stop": null,
  "start_profile": "NotMarked",
  "folder": "store-folder"
};

export const lifecycleNoTray: LifecycleView = {
  "revision": 2,
  "settings": {
    "Ready": {
      "settings": {
        "on_close": null,
        "start_hidden": false,
        "connect_at_start": false,
        "address": null
      },
      "notices": []
    }
  },
  "tray": {
    "Unavailable": {
      "detail": "The system has no tray area."
    }
  },
  "start_with_system": "NotRegistered",
  "machine_start_entry": false,
  "window_visible": true,
  "quitting": false,
  "close_prompt": false,
  "tray_stop": null,
  "start_profile": "NotMarked",
  "folder": "store-folder"
};

export const lifecycleEntryUnknown: LifecycleView = {
  "revision": 4,
  "settings": {
    "Ready": {
      "settings": {
        "on_close": null,
        "start_hidden": false,
        "connect_at_start": false,
        "address": null
      },
      "notices": []
    }
  },
  "tray": "Available",
  "start_with_system": {
    "Unknown": {
      "detail": "No home folder was found."
    }
  },
  "machine_start_entry": false,
  "window_visible": true,
  "quitting": false,
  "close_prompt": false,
  "tray_stop": null,
  "start_profile": "NotMarked",
  "folder": "store-folder"
};

export const lifecycleMachineEntry: LifecycleView = {
  "revision": 2,
  "settings": {
    "Ready": {
      "settings": {
        "on_close": null,
        "start_hidden": false,
        "connect_at_start": false,
        "address": null
      },
      "notices": []
    }
  },
  "tray": "Available",
  "start_with_system": "NotRegistered",
  "machine_start_entry": true,
  "window_visible": true,
  "quitting": false,
  "close_prompt": false,
  "tray_stop": null,
  "start_profile": "NotMarked",
  "folder": "store-folder"
};

export const lifecycleQuitting: LifecycleView = {
  "revision": 3,
  "settings": {
    "Ready": {
      "settings": {
        "on_close": null,
        "start_hidden": false,
        "connect_at_start": false,
        "address": null
      },
      "notices": []
    }
  },
  "tray": "Available",
  "start_with_system": "NotRegistered",
  "machine_start_entry": false,
  "window_visible": false,
  "quitting": true,
  "close_prompt": false,
  "tray_stop": null,
  "start_profile": "NotMarked",
  "folder": "store-folder"
};

export const lifecycleInUse: LifecycleView = {
  "revision": 2,
  "settings": {
    "Unavailable": {
      "reason": "InUse"
    }
  },
  "tray": "Available",
  "start_with_system": "NotRegistered",
  "machine_start_entry": false,
  "window_visible": true,
  "quitting": false,
  "close_prompt": false,
  "tray_stop": null,
  "start_profile": {
    "Unavailable": {
      "reason": "InUse"
    }
  },
  "folder": "store-folder"
};

export const lifecycleRecovered: LifecycleView = {
  "revision": 2,
  "settings": {
    "Ready": {
      "settings": {
        "on_close": "Background",
        "start_hidden": false,
        "connect_at_start": false,
        "address": null
      },
      "notices": [
        {
          "file": "app-settings.json",
          "event": {
            "RestoredFromBackup": {
              "problem": {
                "NotJson": {
                  "detail": "EOF while parsing an object at line 1 column 10"
                }
              },
              "set_aside": "app-settings.json.unreadable-1",
              "rewritten": true
            }
          }
        }
      ]
    }
  },
  "tray": "Available",
  "start_with_system": "NotRegistered",
  "machine_start_entry": false,
  "window_visible": true,
  "quitting": false,
  "close_prompt": false,
  "tray_stop": null,
  "start_profile": "NotMarked",
  "folder": "store-folder"
};

export const lifecycleTrayStopped: LifecycleView = {
  "revision": 4,
  "settings": {
    "Ready": {
      "settings": {
        "on_close": null,
        "start_hidden": false,
        "connect_at_start": false,
        "address": null
      },
      "notices": []
    }
  },
  "tray": "Available",
  "start_with_system": "NotRegistered",
  "machine_start_entry": false,
  "window_visible": false,
  "quitting": false,
  "close_prompt": false,
  "tray_stop": {
    "sequence": 1,
    "stops": [
      {
        "device": "demo-graphics-card",
        "label": "Demo Graphics Card",
        "result": {
          "Stopped": {
            "restore": "Restored"
          }
        }
      },
      {
        "device": "demo-motherboard",
        "label": "Demo Motherboard",
        "result": {
          "Stopped": {
            "restore": "Restored"
          }
        }
      }
    ]
  },
  "start_profile": "NotMarked",
  "folder": "store-folder"
};

export const lifecycleTrayStopBusy: LifecycleView = {
  "revision": 3,
  "settings": {
    "Ready": {
      "settings": {
        "on_close": null,
        "start_hidden": false,
        "connect_at_start": false,
        "address": null
      },
      "notices": []
    }
  },
  "tray": "Available",
  "start_with_system": "NotRegistered",
  "machine_start_entry": false,
  "window_visible": true,
  "quitting": false,
  "close_prompt": false,
  "tray_stop": {
    "sequence": 1,
    "stops": [
      {
        "device": "demo-graphics-card",
        "label": "Demo Graphics Card",
        "result": {
          "Failed": {
            "error": {
              "Session": "Busy"
            }
          }
        }
      }
    ]
  },
  "start_profile": "NotMarked",
  "folder": "store-folder"
};
