// Generated from real sessions by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.
import type { AutomationView } from "../../lib/api/bindings/AutomationView";

export const demoNoRules: AutomationView = {
  "store": "Demo",
  "support": "Available",
  "rules": [
    {
      "trigger": "Starts",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Connects",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Sleeps",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Wakes",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "SessionEnds",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Quits",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    }
  ],
  "profiles": [
    {
      "id": "p-0000000000000100",
      "name": "Demo glow"
    }
  ],
  "openrgb": "CannotTell"
};

export const demoNotOnThisSystem: AutomationView = {
  "store": "Demo",
  "support": "NotOnThisSystem",
  "rules": [
    {
      "trigger": "Starts",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Connects",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Sleeps",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Wakes",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "SessionEnds",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Quits",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    }
  ],
  "profiles": [
    {
      "id": "p-0000000000000100",
      "name": "Demo glow"
    }
  ],
  "openrgb": "CannotTell"
};

export const openRgbSleepAndWake: AutomationView = {
  "store": "Real",
  "support": "Available",
  "rules": [
    {
      "trigger": "Starts",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Connects",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Sleeps",
      "enabled": true,
      "action": "LightsOff",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Wakes",
      "enabled": true,
      "action": {
        "ApplyProfile": {
          "profile": "p-0000000000000101"
        }
      },
      "profile": {
        "Ready": {
          "name": "Evening"
        }
      },
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "SessionEnds",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Quits",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    }
  ],
  "profiles": [
    {
      "id": "p-0000000000000101",
      "name": "Evening"
    }
  ],
  "openrgb": {
    "Read": {
      "profiles": {
        "open": {
          "enabled": false,
          "name": ""
        },
        "exit": {
          "enabled": false,
          "name": ""
        },
        "resume": {
          "enabled": false,
          "name": ""
        },
        "suspend": {
          "enabled": false,
          "name": ""
        },
        "service_startup": {
          "enabled": false,
          "name": ""
        },
        "service_shutdown": {
          "enabled": false,
          "name": ""
        }
      }
    }
  }
};

export const openRgbAfterSleepAndWake: AutomationView = {
  "store": "Real",
  "support": "Available",
  "rules": [
    {
      "trigger": "Starts",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Connects",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Sleeps",
      "enabled": true,
      "action": "LightsOff",
      "profile": null,
      "problem": null,
      "last_run": {
        "trigger": "Sleeps",
        "at_ms": 1795000000000,
        "outcome": {
          "LightsOff": {
            "report": {
              "devices": [
                {
                  "device": "dev-e87858a20206b8d3",
                  "name": "Fixture Keyboard",
                  "outcome": {
                    "Skipped": {
                      "reason": "NoDirectOrOffMode"
                    }
                  }
                },
                {
                  "device": "dev-5ae0445e22cbba19",
                  "name": "Fixture Strip",
                  "outcome": "Black"
                }
              ]
            }
          }
        }
      },
      "openrgb_also": []
    },
    {
      "trigger": "Wakes",
      "enabled": true,
      "action": {
        "ApplyProfile": {
          "profile": "p-0000000000000101"
        }
      },
      "profile": {
        "Ready": {
          "name": "Evening"
        }
      },
      "problem": null,
      "last_run": {
        "trigger": "Wakes",
        "at_ms": 1795000000000,
        "outcome": {
          "Applied": {
            "profile": "p-0000000000000101",
            "name": "Evening",
            "report": {
              "frame_rate": {
                "Applied": {
                  "in_use": 30
                }
              },
              "items": [
                {
                  "device": "dev-5ae0445e22cbba19",
                  "label": "Fixture Strip",
                  "target": "dev-5ae0445e22cbba19",
                  "item": {
                    "Effect": {
                      "zone": null,
                      "effect": "rainbow-wave"
                    }
                  },
                  "outcome": "Applied"
                }
              ]
            },
            "missing": []
          }
        }
      },
      "openrgb_also": []
    },
    {
      "trigger": "SessionEnds",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Quits",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    }
  ],
  "profiles": [
    {
      "id": "p-0000000000000101",
      "name": "Evening"
    }
  ],
  "openrgb": {
    "Read": {
      "profiles": {
        "open": {
          "enabled": false,
          "name": ""
        },
        "exit": {
          "enabled": false,
          "name": ""
        },
        "resume": {
          "enabled": false,
          "name": ""
        },
        "suspend": {
          "enabled": false,
          "name": ""
        },
        "service_startup": {
          "enabled": false,
          "name": ""
        },
        "service_shutdown": {
          "enabled": false,
          "name": ""
        }
      }
    }
  }
};

export const openRgbDeletedProfile: AutomationView = {
  "store": "Real",
  "support": "Available",
  "rules": [
    {
      "trigger": "Starts",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Connects",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Sleeps",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Wakes",
      "enabled": true,
      "action": {
        "ApplyProfile": {
          "profile": "p-0000000000000102"
        }
      },
      "profile": "Missing",
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "SessionEnds",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Quits",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    }
  ],
  "profiles": [],
  "openrgb": {
    "Read": {
      "profiles": {
        "open": {
          "enabled": false,
          "name": ""
        },
        "exit": {
          "enabled": false,
          "name": ""
        },
        "resume": {
          "enabled": false,
          "name": ""
        },
        "suspend": {
          "enabled": false,
          "name": ""
        },
        "service_startup": {
          "enabled": false,
          "name": ""
        },
        "service_shutdown": {
          "enabled": false,
          "name": ""
        }
      }
    }
  }
};

export const openRgbAlsoActs: AutomationView = {
  "store": "Real",
  "support": "Available",
  "rules": [
    {
      "trigger": "Starts",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Connects",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Sleeps",
      "enabled": true,
      "action": "LightsOff",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": [
        {
          "setting": "Suspend",
          "profile": "Dark"
        }
      ]
    },
    {
      "trigger": "Wakes",
      "enabled": true,
      "action": {
        "ApplyProfile": {
          "profile": "p-0000000000000103"
        }
      },
      "profile": {
        "Ready": {
          "name": "Evening"
        }
      },
      "problem": null,
      "last_run": null,
      "openrgb_also": [
        {
          "setting": "Resume",
          "profile": "Day"
        }
      ]
    },
    {
      "trigger": "SessionEnds",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Quits",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    }
  ],
  "profiles": [
    {
      "id": "p-0000000000000103",
      "name": "Evening"
    }
  ],
  "openrgb": {
    "Read": {
      "profiles": {
        "open": {
          "enabled": false,
          "name": ""
        },
        "exit": {
          "enabled": false,
          "name": ""
        },
        "resume": {
          "enabled": true,
          "name": "Day"
        },
        "suspend": {
          "enabled": true,
          "name": "Dark"
        },
        "service_startup": {
          "enabled": false,
          "name": ""
        },
        "service_shutdown": {
          "enabled": false,
          "name": ""
        }
      }
    }
  }
};

export const openRgbMigratedConnects: AutomationView = {
  "store": "Real",
  "support": "Available",
  "rules": [
    {
      "trigger": "Starts",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Connects",
      "enabled": true,
      "action": {
        "ApplyProfile": {
          "profile": "p-0000000000000104"
        }
      },
      "profile": {
        "Ready": {
          "name": "Evening"
        }
      },
      "problem": null,
      "last_run": {
        "trigger": "Connects",
        "at_ms": 1795000000000,
        "outcome": {
          "Applied": {
            "profile": "p-0000000000000104",
            "name": "Evening",
            "report": {
              "frame_rate": {
                "Applied": {
                  "in_use": 30
                }
              },
              "items": [
                {
                  "device": "dev-5ae0445e22cbba19",
                  "label": "Fixture Strip",
                  "target": "dev-5ae0445e22cbba19",
                  "item": {
                    "Effect": {
                      "zone": null,
                      "effect": "rainbow-wave"
                    }
                  },
                  "outcome": "Applied"
                }
              ]
            },
            "missing": []
          }
        }
      },
      "openrgb_also": []
    },
    {
      "trigger": "Sleeps",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Wakes",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "SessionEnds",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    },
    {
      "trigger": "Quits",
      "enabled": false,
      "action": "Nothing",
      "profile": null,
      "problem": null,
      "last_run": null,
      "openrgb_also": []
    }
  ],
  "profiles": [
    {
      "id": "p-0000000000000104",
      "name": "Evening"
    }
  ],
  "openrgb": {
    "Read": {
      "profiles": {
        "open": {
          "enabled": false,
          "name": ""
        },
        "exit": {
          "enabled": false,
          "name": ""
        },
        "resume": {
          "enabled": false,
          "name": ""
        },
        "suspend": {
          "enabled": false,
          "name": ""
        },
        "service_startup": {
          "enabled": false,
          "name": ""
        },
        "service_shutdown": {
          "enabled": false,
          "name": ""
        }
      }
    }
  }
};
