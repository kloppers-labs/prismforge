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
  "openrgb": "CannotTell",
  "notices": []
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
  "openrgb": "CannotTell",
  "notices": []
};

export const demoEventsFailed: AutomationView = {
  "store": "Demo",
  "support": {
    "Failed": {
      "detail": "the event window could not be created: Access is denied."
    }
  },
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
  "openrgb": "CannotTell",
  "notices": []
};

export const openRgbSleepFailed: AutomationView = {
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
          "Failed": {
            "failure": {
              "TimedOut": {
                "limit_ms": 1000
              }
            }
          }
        },
        "seen": false
      },
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
  },
  "notices": []
};

export const openRgbRunNotWritten: AutomationView = {
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
      "enabled": true,
      "action": "LightsOff",
      "profile": null,
      "problem": null,
      "last_run": {
        "trigger": "Quits",
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
        },
        "seen": false
      },
      "openrgb_also": []
    }
  ],
  "profiles": [
    {
      "id": "p-0000000000000102",
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
  },
  "notices": [
    {
      "file": "rule-runs.json",
      "event": {
        "WriteFailed": {
          "failure": {
            "file": "rule-runs.json",
            "step": "Replace",
            "detail": "Access is denied. (os error 5)"
          }
        }
      }
    }
  ]
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
  },
  "notices": []
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
        },
        "seen": false
      },
      "openrgb_also": []
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
      "last_run": {
        "trigger": "Wakes",
        "at_ms": 1795000000000,
        "outcome": {
          "Applied": {
            "profile": "p-0000000000000103",
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
            "missing": [],
            "not_kept": null
          }
        },
        "seen": false
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
  },
  "notices": []
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
          "profile": "p-0000000000000104"
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
  },
  "notices": []
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
          "profile": "p-0000000000000105"
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
      "id": "p-0000000000000105",
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
  },
  "notices": []
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
          "profile": "p-0000000000000106"
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
            "profile": "p-0000000000000106",
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
            "missing": [],
            "not_kept": null
          }
        },
        "seen": false
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
      "id": "p-0000000000000106",
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
  },
  "notices": []
};
