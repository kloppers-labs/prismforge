// Generated from real sessions by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.
import type { ShellView } from "../../lib/api/bindings/ShellView";

export const viewApplying: ShellView = {
  "revision": 18,
  "session": {
    "connection": "Connected",
    "backend": {
      "origin": "Simulated",
      "address": null,
      "protocol_version": null,
      "server_name": null
    },
    "devices": [
      {
        "identity": {
          "Matched": "demo-memory-1"
        },
        "descriptor": {
          "origin": "Simulated",
          "name": "Demo Memory Module",
          "vendor": "PrismForge Demo",
          "description": "Simulated device, no hardware",
          "version": "1.0",
          "serial": "",
          "location": "Simulated memory slot 1",
          "kind": "Memory",
          "zones": [
            {
              "name": "Module",
              "kind": "Linear",
              "led_count": 8,
              "leds_min": 8,
              "leds_max": 8,
              "matrix": null
            }
          ],
          "led_count": 8,
          "colors": [
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            }
          ],
          "capabilities": {
            "direct_mode": 0,
            "active_mode": 1,
            "modes": [
              {
                "name": "Direct",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": true,
                "mode_colors": null,
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "None"
              },
              {
                "name": "Static",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": false,
                "mode_colors": {
                  "min": 1,
                  "max": 1
                },
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "ManualSave"
              },
              {
                "name": "Breathing",
                "speed": {
                  "min": 1,
                  "max": 5
                },
                "brightness": {
                  "min": 0,
                  "max": 100
                },
                "directions": [],
                "per_led_color": false,
                "mode_colors": {
                  "min": 1,
                  "max": 2
                },
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "ManualSave"
              }
            ],
            "resizable_zones": [],
            "saves_to_device_memory": true
          }
        },
        "available": true
      },
      {
        "identity": {
          "Matched": "demo-memory-2"
        },
        "descriptor": {
          "origin": "Simulated",
          "name": "Demo Memory Module",
          "vendor": "PrismForge Demo",
          "description": "Simulated device, no hardware",
          "version": "1.0",
          "serial": "",
          "location": "Simulated memory slot 2",
          "kind": "Memory",
          "zones": [
            {
              "name": "Module",
              "kind": "Linear",
              "led_count": 8,
              "leds_min": 8,
              "leds_max": 8,
              "matrix": null
            }
          ],
          "led_count": 8,
          "colors": [
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            }
          ],
          "capabilities": {
            "direct_mode": 0,
            "active_mode": 1,
            "modes": [
              {
                "name": "Direct",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": true,
                "mode_colors": null,
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "None"
              },
              {
                "name": "Static",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": false,
                "mode_colors": {
                  "min": 1,
                  "max": 1
                },
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "ManualSave"
              },
              {
                "name": "Breathing",
                "speed": {
                  "min": 1,
                  "max": 5
                },
                "brightness": {
                  "min": 0,
                  "max": 100
                },
                "directions": [],
                "per_led_color": false,
                "mode_colors": {
                  "min": 1,
                  "max": 2
                },
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "ManualSave"
              }
            ],
            "resizable_zones": [],
            "saves_to_device_memory": true
          }
        },
        "available": true
      },
      {
        "identity": {
          "Matched": "demo-motherboard"
        },
        "descriptor": {
          "origin": "Simulated",
          "name": "Demo Motherboard",
          "vendor": "PrismForge Demo",
          "description": "Simulated device, no hardware",
          "version": "1.0",
          "serial": "SIMULATED-MB-01",
          "location": "Simulated system board",
          "kind": "Motherboard",
          "zones": [
            {
              "name": "Onboard",
              "kind": "Linear",
              "led_count": 6,
              "leds_min": 6,
              "leds_max": 6,
              "matrix": null
            },
            {
              "name": "Addressable Header 1",
              "kind": "Linear",
              "led_count": 30,
              "leds_min": 0,
              "leds_max": 120,
              "matrix": null
            },
            {
              "name": "Addressable Header 2",
              "kind": "Linear",
              "led_count": 16,
              "leds_min": 16,
              "leds_max": 16,
              "matrix": null
            }
          ],
          "led_count": 52,
          "colors": [
            {
              "r": 0,
              "g": 64,
              "b": 255
            },
            {
              "r": 5,
              "g": 64,
              "b": 250
            },
            {
              "r": 10,
              "g": 64,
              "b": 245
            },
            {
              "r": 15,
              "g": 64,
              "b": 240
            },
            {
              "r": 20,
              "g": 64,
              "b": 235
            },
            {
              "r": 25,
              "g": 64,
              "b": 230
            },
            {
              "r": 30,
              "g": 64,
              "b": 225
            },
            {
              "r": 35,
              "g": 64,
              "b": 220
            },
            {
              "r": 40,
              "g": 64,
              "b": 215
            },
            {
              "r": 45,
              "g": 64,
              "b": 210
            },
            {
              "r": 50,
              "g": 64,
              "b": 205
            },
            {
              "r": 55,
              "g": 64,
              "b": 200
            },
            {
              "r": 60,
              "g": 64,
              "b": 195
            },
            {
              "r": 65,
              "g": 64,
              "b": 190
            },
            {
              "r": 70,
              "g": 64,
              "b": 185
            },
            {
              "r": 75,
              "g": 64,
              "b": 180
            },
            {
              "r": 80,
              "g": 64,
              "b": 175
            },
            {
              "r": 85,
              "g": 64,
              "b": 170
            },
            {
              "r": 90,
              "g": 64,
              "b": 165
            },
            {
              "r": 95,
              "g": 64,
              "b": 160
            },
            {
              "r": 100,
              "g": 64,
              "b": 155
            },
            {
              "r": 105,
              "g": 64,
              "b": 150
            },
            {
              "r": 110,
              "g": 64,
              "b": 145
            },
            {
              "r": 115,
              "g": 64,
              "b": 140
            },
            {
              "r": 120,
              "g": 64,
              "b": 135
            },
            {
              "r": 125,
              "g": 64,
              "b": 130
            },
            {
              "r": 130,
              "g": 64,
              "b": 125
            },
            {
              "r": 135,
              "g": 64,
              "b": 120
            },
            {
              "r": 140,
              "g": 64,
              "b": 115
            },
            {
              "r": 145,
              "g": 64,
              "b": 110
            },
            {
              "r": 150,
              "g": 64,
              "b": 105
            },
            {
              "r": 155,
              "g": 64,
              "b": 100
            },
            {
              "r": 160,
              "g": 64,
              "b": 95
            },
            {
              "r": 165,
              "g": 64,
              "b": 90
            },
            {
              "r": 170,
              "g": 64,
              "b": 85
            },
            {
              "r": 175,
              "g": 64,
              "b": 80
            },
            {
              "r": 180,
              "g": 64,
              "b": 75
            },
            {
              "r": 185,
              "g": 64,
              "b": 70
            },
            {
              "r": 190,
              "g": 64,
              "b": 65
            },
            {
              "r": 195,
              "g": 64,
              "b": 60
            },
            {
              "r": 200,
              "g": 64,
              "b": 55
            },
            {
              "r": 205,
              "g": 64,
              "b": 50
            },
            {
              "r": 210,
              "g": 64,
              "b": 45
            },
            {
              "r": 215,
              "g": 64,
              "b": 40
            },
            {
              "r": 220,
              "g": 64,
              "b": 35
            },
            {
              "r": 225,
              "g": 64,
              "b": 30
            },
            {
              "r": 230,
              "g": 64,
              "b": 25
            },
            {
              "r": 235,
              "g": 64,
              "b": 20
            },
            {
              "r": 240,
              "g": 64,
              "b": 15
            },
            {
              "r": 245,
              "g": 64,
              "b": 10
            },
            {
              "r": 250,
              "g": 64,
              "b": 5
            },
            {
              "r": 255,
              "g": 64,
              "b": 0
            }
          ],
          "capabilities": {
            "direct_mode": 0,
            "active_mode": 0,
            "modes": [
              {
                "name": "Direct",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": true,
                "mode_colors": null,
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "None"
              },
              {
                "name": "Static",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": false,
                "mode_colors": {
                  "min": 1,
                  "max": 1
                },
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "ManualSave"
              },
              {
                "name": "Rainbow Wave",
                "speed": {
                  "min": 1,
                  "max": 5
                },
                "brightness": null,
                "directions": [
                  "LeftRight"
                ],
                "per_led_color": false,
                "mode_colors": null,
                "random_color": false,
                "applies_to_entire_device": true,
                "persistence": "None"
              }
            ],
            "resizable_zones": [
              {
                "zone": 1,
                "min": 0,
                "max": 120
              }
            ],
            "saves_to_device_memory": true
          }
        },
        "available": true
      },
      {
        "identity": {
          "Matched": "demo-graphics-card"
        },
        "descriptor": {
          "origin": "Simulated",
          "name": "Demo Graphics Card",
          "vendor": "PrismForge Demo",
          "description": "Simulated device, no hardware",
          "version": "1.0",
          "serial": "SIMULATED-GPU-01",
          "location": "Simulated expansion slot 1",
          "kind": "Gpu",
          "zones": [
            {
              "name": "Logo",
              "kind": "Linear",
              "led_count": 12,
              "leds_min": 12,
              "leds_max": 12,
              "matrix": null
            }
          ],
          "led_count": 12,
          "colors": [
            {
              "r": 255,
              "g": 0,
              "b": 0
            },
            {
              "r": 255,
              "g": 139,
              "b": 0
            },
            {
              "r": 232,
              "g": 255,
              "b": 0
            },
            {
              "r": 93,
              "g": 255,
              "b": 0
            },
            {
              "r": 0,
              "g": 255,
              "b": 46
            },
            {
              "r": 0,
              "g": 255,
              "b": 185
            },
            {
              "r": 0,
              "g": 185,
              "b": 255
            },
            {
              "r": 0,
              "g": 46,
              "b": 255
            },
            {
              "r": 93,
              "g": 0,
              "b": 255
            },
            {
              "r": 232,
              "g": 0,
              "b": 255
            },
            {
              "r": 255,
              "g": 0,
              "b": 139
            },
            {
              "r": 255,
              "g": 0,
              "b": 0
            }
          ],
          "capabilities": {
            "direct_mode": 0,
            "active_mode": 0,
            "modes": [
              {
                "name": "Direct",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": true,
                "mode_colors": null,
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "None"
              },
              {
                "name": "Static",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": false,
                "mode_colors": {
                  "min": 1,
                  "max": 1
                },
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "ManualSave"
              },
              {
                "name": "Breathing",
                "speed": {
                  "min": 1,
                  "max": 5
                },
                "brightness": {
                  "min": 0,
                  "max": 100
                },
                "directions": [],
                "per_led_color": false,
                "mode_colors": {
                  "min": 1,
                  "max": 2
                },
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "ManualSave"
              }
            ],
            "resizable_zones": [],
            "saves_to_device_memory": true
          }
        },
        "available": true
      },
      {
        "identity": {
          "Matched": "demo-light-bar"
        },
        "descriptor": {
          "origin": "Simulated",
          "name": "Demo Light Bar",
          "vendor": "PrismForge Demo",
          "description": "Simulated device, no hardware",
          "version": "1.0",
          "serial": "SIMULATED-BAR-01",
          "location": "Simulated USB port 1",
          "kind": "Light",
          "zones": [
            {
              "name": "Bar",
              "kind": "Linear",
              "led_count": 10,
              "leds_min": 10,
              "leds_max": 10,
              "matrix": null
            }
          ],
          "led_count": 10,
          "colors": [
            {
              "r": 255,
              "g": 255,
              "b": 255
            },
            {
              "r": 255,
              "g": 255,
              "b": 255
            },
            {
              "r": 255,
              "g": 255,
              "b": 255
            },
            {
              "r": 255,
              "g": 255,
              "b": 255
            },
            {
              "r": 255,
              "g": 255,
              "b": 255
            },
            {
              "r": 255,
              "g": 255,
              "b": 255
            },
            {
              "r": 255,
              "g": 255,
              "b": 255
            },
            {
              "r": 255,
              "g": 255,
              "b": 255
            },
            {
              "r": 255,
              "g": 255,
              "b": 255
            },
            {
              "r": 255,
              "g": 255,
              "b": 255
            }
          ],
          "capabilities": {
            "direct_mode": null,
            "active_mode": 1,
            "modes": [
              {
                "name": "Static",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": false,
                "mode_colors": {
                  "min": 1,
                  "max": 1
                },
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "ManualSave"
              },
              {
                "name": "Breathing",
                "speed": {
                  "min": 1,
                  "max": 5
                },
                "brightness": {
                  "min": 0,
                  "max": 100
                },
                "directions": [],
                "per_led_color": false,
                "mode_colors": {
                  "min": 1,
                  "max": 2
                },
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "ManualSave"
              }
            ],
            "resizable_zones": [],
            "saves_to_device_memory": true
          }
        },
        "available": true
      }
    ],
    "issues": [],
    "identify": null,
    "output": {
      "frame_rate": 45,
      "scene": {
        "assignments": [
          {
            "target": {
              "device": "demo-graphics-card",
              "zone": null
            },
            "effect": {
              "effect": "moving-highlight",
              "values": {
                "background": {
                  "Color": {
                    "r": 0,
                    "g": 0,
                    "b": 0
                  }
                },
                "brightness": {
                  "Number": 100
                },
                "color": {
                  "Color": {
                    "r": 255,
                    "g": 255,
                    "b": 255
                  }
                },
                "direction": {
                  "Direction": "Forward"
                },
                "speed": {
                  "Number": 30
                },
                "width": {
                  "Number": 20
                }
              }
            }
          },
          {
            "target": {
              "device": "demo-motherboard",
              "zone": 1
            },
            "effect": {
              "effect": "solid",
              "values": {
                "brightness": {
                  "Number": 100
                },
                "color": {
                  "Color": {
                    "r": 0,
                    "g": 200,
                    "b": 120
                  }
                }
              }
            }
          }
        ]
      },
      "devices": [
        {
          "device": "demo-graphics-card",
          "owner": "Effect",
          "condition": "Running"
        },
        {
          "device": "demo-light-bar",
          "owner": {
            "HardwareMode": {
              "mode": 1
            }
          },
          "condition": "Idle"
        },
        {
          "device": "demo-memory-2",
          "owner": {
            "External": {
              "label": "OpenRGB Effects plugin"
            }
          },
          "condition": "Idle"
        },
        {
          "device": "demo-motherboard",
          "owner": "Effect",
          "condition": "Running"
        }
      ]
    },
    "diagnostics": {
      "last_error": null,
      "counters": {
        "connect_attempts": 1,
        "connect_failures": 0,
        "inventory_refreshes": 3,
        "refresh_failures": 0,
        "commands_refused_busy": 0,
        "frames_written": 14,
        "ticks_skipped": 0,
        "output_writes_refused": 0,
        "output_writes_failed": 0,
        "output_wakeups": 13
      },
      "events": [
        {
          "sequence": 0,
          "at_ms": 0,
          "kind": "ConnectAttempt"
        },
        {
          "sequence": 1,
          "at_ms": 0,
          "kind": "Connected"
        },
        {
          "sequence": 2,
          "at_ms": 0,
          "kind": {
            "InventoryRefreshed": {
              "generation": 1,
              "devices": 5
            }
          }
        },
        {
          "sequence": 3,
          "at_ms": 300,
          "kind": {
            "InventoryRefreshed": {
              "generation": 2,
              "devices": 5
            }
          }
        },
        {
          "sequence": 4,
          "at_ms": 300,
          "kind": {
            "InventoryRefreshed": {
              "generation": 3,
              "devices": 5
            }
          }
        }
      ]
    }
  },
  "applying": {
    "id": "p-0000000000000001",
    "name": "Evening",
    "automatic": false
  },
  "connect_apply": null,
  "on_connect": {
    "Fired": {
      "ran": null,
      "next": null
    }
  }
};

export const viewConnectApplied: ShellView = {
  "revision": 38,
  "session": {
    "connection": "Connected",
    "backend": {
      "origin": "Simulated",
      "address": null,
      "protocol_version": null,
      "server_name": null
    },
    "devices": [
      {
        "identity": {
          "Matched": "demo-memory-1"
        },
        "descriptor": {
          "origin": "Simulated",
          "name": "Demo Memory Module",
          "vendor": "PrismForge Demo",
          "description": "Simulated device, no hardware",
          "version": "1.0",
          "serial": "",
          "location": "Simulated memory slot 1",
          "kind": "Memory",
          "zones": [
            {
              "name": "Module",
              "kind": "Linear",
              "led_count": 8,
              "leds_min": 8,
              "leds_max": 8,
              "matrix": null
            }
          ],
          "led_count": 8,
          "colors": [
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            }
          ],
          "capabilities": {
            "direct_mode": 0,
            "active_mode": 1,
            "modes": [
              {
                "name": "Direct",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": true,
                "mode_colors": null,
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "None"
              },
              {
                "name": "Static",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": false,
                "mode_colors": {
                  "min": 1,
                  "max": 1
                },
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "ManualSave"
              },
              {
                "name": "Breathing",
                "speed": {
                  "min": 1,
                  "max": 5
                },
                "brightness": {
                  "min": 0,
                  "max": 100
                },
                "directions": [],
                "per_led_color": false,
                "mode_colors": {
                  "min": 1,
                  "max": 2
                },
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "ManualSave"
              }
            ],
            "resizable_zones": [],
            "saves_to_device_memory": true
          }
        },
        "available": true
      },
      {
        "identity": {
          "Matched": "demo-memory-2"
        },
        "descriptor": {
          "origin": "Simulated",
          "name": "Demo Memory Module",
          "vendor": "PrismForge Demo",
          "description": "Simulated device, no hardware",
          "version": "1.0",
          "serial": "",
          "location": "Simulated memory slot 2",
          "kind": "Memory",
          "zones": [
            {
              "name": "Module",
              "kind": "Linear",
              "led_count": 8,
              "leds_min": 8,
              "leds_max": 8,
              "matrix": null
            }
          ],
          "led_count": 8,
          "colors": [
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            },
            {
              "r": 255,
              "g": 96,
              "b": 32
            }
          ],
          "capabilities": {
            "direct_mode": 0,
            "active_mode": 1,
            "modes": [
              {
                "name": "Direct",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": true,
                "mode_colors": null,
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "None"
              },
              {
                "name": "Static",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": false,
                "mode_colors": {
                  "min": 1,
                  "max": 1
                },
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "ManualSave"
              },
              {
                "name": "Breathing",
                "speed": {
                  "min": 1,
                  "max": 5
                },
                "brightness": {
                  "min": 0,
                  "max": 100
                },
                "directions": [],
                "per_led_color": false,
                "mode_colors": {
                  "min": 1,
                  "max": 2
                },
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "ManualSave"
              }
            ],
            "resizable_zones": [],
            "saves_to_device_memory": true
          }
        },
        "available": true
      },
      {
        "identity": {
          "Matched": "demo-motherboard"
        },
        "descriptor": {
          "origin": "Simulated",
          "name": "Demo Motherboard",
          "vendor": "PrismForge Demo",
          "description": "Simulated device, no hardware",
          "version": "1.0",
          "serial": "SIMULATED-MB-01",
          "location": "Simulated system board",
          "kind": "Motherboard",
          "zones": [
            {
              "name": "Onboard",
              "kind": "Linear",
              "led_count": 6,
              "leds_min": 6,
              "leds_max": 6,
              "matrix": null
            },
            {
              "name": "Addressable Header 1",
              "kind": "Linear",
              "led_count": 30,
              "leds_min": 0,
              "leds_max": 120,
              "matrix": null
            },
            {
              "name": "Addressable Header 2",
              "kind": "Linear",
              "led_count": 16,
              "leds_min": 16,
              "leds_max": 16,
              "matrix": null
            }
          ],
          "led_count": 52,
          "colors": [
            {
              "r": 0,
              "g": 64,
              "b": 255
            },
            {
              "r": 5,
              "g": 64,
              "b": 250
            },
            {
              "r": 10,
              "g": 64,
              "b": 245
            },
            {
              "r": 15,
              "g": 64,
              "b": 240
            },
            {
              "r": 20,
              "g": 64,
              "b": 235
            },
            {
              "r": 25,
              "g": 64,
              "b": 230
            },
            {
              "r": 30,
              "g": 64,
              "b": 225
            },
            {
              "r": 35,
              "g": 64,
              "b": 220
            },
            {
              "r": 40,
              "g": 64,
              "b": 215
            },
            {
              "r": 45,
              "g": 64,
              "b": 210
            },
            {
              "r": 50,
              "g": 64,
              "b": 205
            },
            {
              "r": 55,
              "g": 64,
              "b": 200
            },
            {
              "r": 60,
              "g": 64,
              "b": 195
            },
            {
              "r": 65,
              "g": 64,
              "b": 190
            },
            {
              "r": 70,
              "g": 64,
              "b": 185
            },
            {
              "r": 75,
              "g": 64,
              "b": 180
            },
            {
              "r": 80,
              "g": 64,
              "b": 175
            },
            {
              "r": 85,
              "g": 64,
              "b": 170
            },
            {
              "r": 90,
              "g": 64,
              "b": 165
            },
            {
              "r": 95,
              "g": 64,
              "b": 160
            },
            {
              "r": 100,
              "g": 64,
              "b": 155
            },
            {
              "r": 105,
              "g": 64,
              "b": 150
            },
            {
              "r": 110,
              "g": 64,
              "b": 145
            },
            {
              "r": 115,
              "g": 64,
              "b": 140
            },
            {
              "r": 120,
              "g": 64,
              "b": 135
            },
            {
              "r": 125,
              "g": 64,
              "b": 130
            },
            {
              "r": 130,
              "g": 64,
              "b": 125
            },
            {
              "r": 135,
              "g": 64,
              "b": 120
            },
            {
              "r": 140,
              "g": 64,
              "b": 115
            },
            {
              "r": 145,
              "g": 64,
              "b": 110
            },
            {
              "r": 150,
              "g": 64,
              "b": 105
            },
            {
              "r": 155,
              "g": 64,
              "b": 100
            },
            {
              "r": 160,
              "g": 64,
              "b": 95
            },
            {
              "r": 165,
              "g": 64,
              "b": 90
            },
            {
              "r": 170,
              "g": 64,
              "b": 85
            },
            {
              "r": 175,
              "g": 64,
              "b": 80
            },
            {
              "r": 180,
              "g": 64,
              "b": 75
            },
            {
              "r": 185,
              "g": 64,
              "b": 70
            },
            {
              "r": 190,
              "g": 64,
              "b": 65
            },
            {
              "r": 195,
              "g": 64,
              "b": 60
            },
            {
              "r": 200,
              "g": 64,
              "b": 55
            },
            {
              "r": 205,
              "g": 64,
              "b": 50
            },
            {
              "r": 210,
              "g": 64,
              "b": 45
            },
            {
              "r": 215,
              "g": 64,
              "b": 40
            },
            {
              "r": 220,
              "g": 64,
              "b": 35
            },
            {
              "r": 225,
              "g": 64,
              "b": 30
            },
            {
              "r": 230,
              "g": 64,
              "b": 25
            },
            {
              "r": 235,
              "g": 64,
              "b": 20
            },
            {
              "r": 240,
              "g": 64,
              "b": 15
            },
            {
              "r": 245,
              "g": 64,
              "b": 10
            },
            {
              "r": 250,
              "g": 64,
              "b": 5
            },
            {
              "r": 255,
              "g": 64,
              "b": 0
            }
          ],
          "capabilities": {
            "direct_mode": 0,
            "active_mode": 0,
            "modes": [
              {
                "name": "Direct",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": true,
                "mode_colors": null,
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "None"
              },
              {
                "name": "Static",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": false,
                "mode_colors": {
                  "min": 1,
                  "max": 1
                },
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "ManualSave"
              },
              {
                "name": "Rainbow Wave",
                "speed": {
                  "min": 1,
                  "max": 5
                },
                "brightness": null,
                "directions": [
                  "LeftRight"
                ],
                "per_led_color": false,
                "mode_colors": null,
                "random_color": false,
                "applies_to_entire_device": true,
                "persistence": "None"
              }
            ],
            "resizable_zones": [
              {
                "zone": 1,
                "min": 0,
                "max": 120
              }
            ],
            "saves_to_device_memory": true
          }
        },
        "available": true
      },
      {
        "identity": {
          "Matched": "demo-graphics-card"
        },
        "descriptor": {
          "origin": "Simulated",
          "name": "Demo Graphics Card",
          "vendor": "PrismForge Demo",
          "description": "Simulated device, no hardware",
          "version": "1.0",
          "serial": "SIMULATED-GPU-01",
          "location": "Simulated expansion slot 1",
          "kind": "Gpu",
          "zones": [
            {
              "name": "Logo",
              "kind": "Linear",
              "led_count": 12,
              "leds_min": 12,
              "leds_max": 12,
              "matrix": null
            }
          ],
          "led_count": 12,
          "colors": [
            {
              "r": 0,
              "g": 160,
              "b": 255
            },
            {
              "r": 0,
              "g": 160,
              "b": 255
            },
            {
              "r": 0,
              "g": 160,
              "b": 255
            },
            {
              "r": 0,
              "g": 160,
              "b": 255
            },
            {
              "r": 0,
              "g": 160,
              "b": 255
            },
            {
              "r": 0,
              "g": 160,
              "b": 255
            },
            {
              "r": 0,
              "g": 160,
              "b": 255
            },
            {
              "r": 0,
              "g": 160,
              "b": 255
            },
            {
              "r": 0,
              "g": 160,
              "b": 255
            },
            {
              "r": 0,
              "g": 160,
              "b": 255
            },
            {
              "r": 0,
              "g": 160,
              "b": 255
            },
            {
              "r": 0,
              "g": 160,
              "b": 255
            }
          ],
          "capabilities": {
            "direct_mode": 0,
            "active_mode": 2,
            "modes": [
              {
                "name": "Direct",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": true,
                "mode_colors": null,
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "None"
              },
              {
                "name": "Static",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": false,
                "mode_colors": {
                  "min": 1,
                  "max": 1
                },
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "ManualSave"
              },
              {
                "name": "Breathing",
                "speed": {
                  "min": 1,
                  "max": 5
                },
                "brightness": {
                  "min": 0,
                  "max": 100
                },
                "directions": [],
                "per_led_color": false,
                "mode_colors": {
                  "min": 1,
                  "max": 2
                },
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "ManualSave"
              }
            ],
            "resizable_zones": [],
            "saves_to_device_memory": true
          }
        },
        "available": true
      },
      {
        "identity": {
          "Matched": "demo-light-bar"
        },
        "descriptor": {
          "origin": "Simulated",
          "name": "Demo Light Bar",
          "vendor": "PrismForge Demo",
          "description": "Simulated device, no hardware",
          "version": "1.0",
          "serial": "SIMULATED-BAR-01",
          "location": "Simulated USB port 1",
          "kind": "Light",
          "zones": [
            {
              "name": "Bar",
              "kind": "Linear",
              "led_count": 10,
              "leds_min": 10,
              "leds_max": 10,
              "matrix": null
            }
          ],
          "led_count": 10,
          "colors": [
            {
              "r": 255,
              "g": 255,
              "b": 255
            },
            {
              "r": 255,
              "g": 255,
              "b": 255
            },
            {
              "r": 255,
              "g": 255,
              "b": 255
            },
            {
              "r": 255,
              "g": 255,
              "b": 255
            },
            {
              "r": 255,
              "g": 255,
              "b": 255
            },
            {
              "r": 255,
              "g": 255,
              "b": 255
            },
            {
              "r": 255,
              "g": 255,
              "b": 255
            },
            {
              "r": 255,
              "g": 255,
              "b": 255
            },
            {
              "r": 255,
              "g": 255,
              "b": 255
            },
            {
              "r": 255,
              "g": 255,
              "b": 255
            }
          ],
          "capabilities": {
            "direct_mode": null,
            "active_mode": 1,
            "modes": [
              {
                "name": "Static",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": false,
                "mode_colors": {
                  "min": 1,
                  "max": 1
                },
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "ManualSave"
              },
              {
                "name": "Breathing",
                "speed": {
                  "min": 1,
                  "max": 5
                },
                "brightness": {
                  "min": 0,
                  "max": 100
                },
                "directions": [],
                "per_led_color": false,
                "mode_colors": {
                  "min": 1,
                  "max": 2
                },
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "ManualSave"
              }
            ],
            "resizable_zones": [],
            "saves_to_device_memory": true
          }
        },
        "available": true
      }
    ],
    "issues": [],
    "identify": null,
    "output": {
      "frame_rate": 45,
      "scene": {
        "assignments": [
          {
            "target": {
              "device": "demo-graphics-card",
              "zone": null
            },
            "effect": {
              "effect": "moving-highlight",
              "values": {
                "background": {
                  "Color": {
                    "r": 0,
                    "g": 0,
                    "b": 0
                  }
                },
                "brightness": {
                  "Number": 100
                },
                "color": {
                  "Color": {
                    "r": 255,
                    "g": 255,
                    "b": 255
                  }
                },
                "direction": {
                  "Direction": "Forward"
                },
                "speed": {
                  "Number": 30
                },
                "width": {
                  "Number": 20
                }
              }
            }
          },
          {
            "target": {
              "device": "demo-motherboard",
              "zone": 1
            },
            "effect": {
              "effect": "solid",
              "values": {
                "brightness": {
                  "Number": 100
                },
                "color": {
                  "Color": {
                    "r": 0,
                    "g": 200,
                    "b": 120
                  }
                }
              }
            }
          }
        ]
      },
      "devices": [
        {
          "device": "demo-graphics-card",
          "owner": "Effect",
          "condition": "Running"
        },
        {
          "device": "demo-light-bar",
          "owner": {
            "HardwareMode": {
              "mode": 1
            }
          },
          "condition": "Idle"
        },
        {
          "device": "demo-memory-2",
          "owner": {
            "External": {
              "label": "OpenRGB Effects plugin"
            }
          },
          "condition": "Idle"
        },
        {
          "device": "demo-motherboard",
          "owner": "Effect",
          "condition": "Running"
        }
      ]
    },
    "diagnostics": {
      "last_error": null,
      "counters": {
        "connect_attempts": 1,
        "connect_failures": 0,
        "inventory_refreshes": 2,
        "refresh_failures": 0,
        "commands_refused_busy": 0,
        "frames_written": 13,
        "ticks_skipped": 0,
        "output_writes_refused": 0,
        "output_writes_failed": 0,
        "output_wakeups": 12
      },
      "events": [
        {
          "sequence": 0,
          "at_ms": 0,
          "kind": "ConnectAttempt"
        },
        {
          "sequence": 1,
          "at_ms": 0,
          "kind": "Connected"
        },
        {
          "sequence": 2,
          "at_ms": 0,
          "kind": {
            "InventoryRefreshed": {
              "generation": 1,
              "devices": 5
            }
          }
        },
        {
          "sequence": 3,
          "at_ms": 0,
          "kind": {
            "InventoryRefreshed": {
              "generation": 2,
              "devices": 5
            }
          }
        }
      ]
    }
  },
  "applying": null,
  "connect_apply": {
    "id": "p-0000000000000001",
    "name": "Evening",
    "outcome": {
      "Applied": {
        "report": {
          "frame_rate": {
            "Applied": {
              "in_use": 45
            }
          },
          "items": [
            {
              "device": "demo-memory-2",
              "label": "Demo Memory Module",
              "target": "demo-memory-2",
              "item": {
                "External": {
                  "label": "OpenRGB Effects plugin"
                }
              },
              "outcome": "Applied"
            },
            {
              "device": "demo-light-bar",
              "label": "Demo Light Bar",
              "target": "demo-light-bar",
              "item": {
                "HardwareMode": {
                  "index": 1,
                  "name": "Breathing"
                }
              },
              "outcome": "Applied"
            },
            {
              "device": "demo-graphics-card",
              "label": "Demo Graphics Card",
              "target": "demo-graphics-card",
              "item": {
                "Effect": {
                  "zone": null,
                  "effect": "moving-highlight"
                }
              },
              "outcome": "Applied"
            },
            {
              "device": "demo-motherboard",
              "label": "Demo Motherboard",
              "target": "demo-motherboard",
              "item": {
                "Effect": {
                  "zone": 1,
                  "effect": "solid"
                }
              },
              "outcome": "Applied"
            }
          ]
        }
      }
    }
  },
  "on_connect": {
    "Fired": {
      "ran": {
        "id": "p-0000000000000001",
        "name": "Evening",
        "outcome": {
          "Applied": {
            "report": {
              "frame_rate": {
                "Applied": {
                  "in_use": 45
                }
              },
              "items": [
                {
                  "device": "demo-memory-2",
                  "label": "Demo Memory Module",
                  "target": "demo-memory-2",
                  "item": {
                    "External": {
                      "label": "OpenRGB Effects plugin"
                    }
                  },
                  "outcome": "Applied"
                },
                {
                  "device": "demo-light-bar",
                  "label": "Demo Light Bar",
                  "target": "demo-light-bar",
                  "item": {
                    "HardwareMode": {
                      "index": 1,
                      "name": "Breathing"
                    }
                  },
                  "outcome": "Applied"
                },
                {
                  "device": "demo-graphics-card",
                  "label": "Demo Graphics Card",
                  "target": "demo-graphics-card",
                  "item": {
                    "Effect": {
                      "zone": null,
                      "effect": "moving-highlight"
                    }
                  },
                  "outcome": "Applied"
                },
                {
                  "device": "demo-motherboard",
                  "label": "Demo Motherboard",
                  "target": "demo-motherboard",
                  "item": {
                    "Effect": {
                      "zone": 1,
                      "effect": "solid"
                    }
                  },
                  "outcome": "Applied"
                }
              ]
            }
          }
        }
      },
      "next": {
        "Ready": {
          "name": "Evening"
        }
      }
    }
  }
};

export const viewArmedRetrying: ShellView = {
  "revision": 15,
  "session": {
    "connection": {
      "Retrying": {
        "attempt": 1,
        "reason": {
          "ConnectionLost": {
            "cause": "Timeout"
          }
        }
      }
    },
    "backend": {
      "origin": "OpenRgb",
      "address": null,
      "protocol_version": null,
      "server_name": null
    },
    "devices": [],
    "issues": [],
    "identify": null,
    "output": {
      "frame_rate": 30,
      "scene": {
        "assignments": []
      },
      "devices": []
    },
    "diagnostics": {
      "last_error": {
        "at_ms": 2000,
        "operation": "Refresh",
        "error": {
          "Timeout": {
            "operation": "controller data request",
            "timeout_ms": 2000
          }
        },
        "ended_connection": {
          "ConnectionLost": {
            "cause": "Timeout"
          }
        }
      },
      "counters": {
        "connect_attempts": 1,
        "connect_failures": 0,
        "inventory_refreshes": 0,
        "refresh_failures": 1,
        "commands_refused_busy": 0,
        "frames_written": 0,
        "ticks_skipped": 0,
        "output_writes_refused": 0,
        "output_writes_failed": 0,
        "output_wakeups": 0
      },
      "events": [
        {
          "sequence": 0,
          "at_ms": 0,
          "kind": "ConnectAttempt"
        },
        {
          "sequence": 1,
          "at_ms": 0,
          "kind": "Connected"
        },
        {
          "sequence": 2,
          "at_ms": 2000,
          "kind": {
            "RefreshFailed": {
              "error": {
                "Timeout": {
                  "operation": "controller data request",
                  "timeout_ms": 2000
                }
              }
            }
          }
        },
        {
          "sequence": 3,
          "at_ms": 2000,
          "kind": {
            "ConnectionLost": {
              "reason": {
                "ConnectionLost": {
                  "cause": "Timeout"
                }
              }
            }
          }
        },
        {
          "sequence": 4,
          "at_ms": 2000,
          "kind": {
            "RetryScheduled": {
              "attempt": 1,
              "delay_ms": 500
            }
          }
        }
      ]
    }
  },
  "applying": null,
  "connect_apply": null,
  "on_connect": {
    "Armed": {
      "id": "p-0000000000000003",
      "name": "Desk"
    }
  }
};

export const viewFiredRetrying: ShellView = {
  "revision": 22,
  "session": {
    "connection": {
      "Retrying": {
        "attempt": 1,
        "reason": {
          "ConnectionLost": {
            "cause": "ServerClosed"
          }
        }
      }
    },
    "backend": {
      "origin": "OpenRgb",
      "address": null,
      "protocol_version": null,
      "server_name": null
    },
    "devices": [
      {
        "identity": {
          "Matched": "dev-5ae0445e22cbba19"
        },
        "descriptor": {
          "origin": "OpenRgb",
          "name": "Fixture Strip",
          "vendor": "Fixture Vendor",
          "description": "Fixture device",
          "version": "1.0",
          "serial": "SN-desk",
          "location": "usb:desk",
          "kind": "LedStrip",
          "zones": [
            {
              "name": "Ring",
              "kind": "Linear",
              "led_count": 8,
              "leds_min": 8,
              "leds_max": 8,
              "matrix": null
            }
          ],
          "led_count": 8,
          "colors": [
            {
              "r": 0,
              "g": 0,
              "b": 0
            },
            {
              "r": 0,
              "g": 0,
              "b": 0
            },
            {
              "r": 0,
              "g": 0,
              "b": 0
            },
            {
              "r": 0,
              "g": 0,
              "b": 0
            },
            {
              "r": 0,
              "g": 0,
              "b": 0
            },
            {
              "r": 0,
              "g": 0,
              "b": 0
            },
            {
              "r": 0,
              "g": 0,
              "b": 0
            },
            {
              "r": 0,
              "g": 0,
              "b": 0
            }
          ],
          "capabilities": {
            "direct_mode": 0,
            "active_mode": 0,
            "modes": [
              {
                "name": "Direct",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": true,
                "mode_colors": null,
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "None"
              },
              {
                "name": "Static",
                "speed": null,
                "brightness": null,
                "directions": [],
                "per_led_color": false,
                "mode_colors": {
                  "min": 1,
                  "max": 1
                },
                "random_color": false,
                "applies_to_entire_device": false,
                "persistence": "ManualSave"
              }
            ],
            "resizable_zones": [],
            "saves_to_device_memory": true
          }
        },
        "available": false
      }
    ],
    "issues": [],
    "identify": null,
    "output": {
      "frame_rate": 30,
      "scene": {
        "assignments": [
          {
            "target": {
              "device": "dev-5ae0445e22cbba19",
              "zone": null
            },
            "effect": {
              "effect": "rainbow-wave",
              "values": {
                "brightness": {
                  "Number": 100
                },
                "direction": {
                  "Direction": "Forward"
                },
                "speed": {
                  "Number": 20
                },
                "waves": {
                  "Number": 1
                }
              }
            }
          }
        ]
      },
      "devices": [
        {
          "device": "dev-5ae0445e22cbba19",
          "owner": "Effect",
          "condition": {
            "Paused": {
              "reason": "NotConnected"
            }
          }
        }
      ]
    },
    "diagnostics": {
      "last_error": {
        "at_ms": 2000,
        "operation": "Refresh",
        "error": {
          "Timeout": {
            "operation": "controller data request",
            "timeout_ms": 2000
          }
        },
        "ended_connection": {
          "ConnectionLost": {
            "cause": "Timeout"
          }
        }
      },
      "counters": {
        "connect_attempts": 2,
        "connect_failures": 0,
        "inventory_refreshes": 1,
        "refresh_failures": 1,
        "commands_refused_busy": 0,
        "frames_written": 2,
        "ticks_skipped": 0,
        "output_writes_refused": 0,
        "output_writes_failed": 0,
        "output_wakeups": 2
      },
      "events": [
        {
          "sequence": 0,
          "at_ms": 0,
          "kind": "ConnectAttempt"
        },
        {
          "sequence": 1,
          "at_ms": 0,
          "kind": "Connected"
        },
        {
          "sequence": 2,
          "at_ms": 2000,
          "kind": {
            "RefreshFailed": {
              "error": {
                "Timeout": {
                  "operation": "controller data request",
                  "timeout_ms": 2000
                }
              }
            }
          }
        },
        {
          "sequence": 3,
          "at_ms": 2000,
          "kind": {
            "ConnectionLost": {
              "reason": {
                "ConnectionLost": {
                  "cause": "Timeout"
                }
              }
            }
          }
        },
        {
          "sequence": 4,
          "at_ms": 2000,
          "kind": {
            "RetryScheduled": {
              "attempt": 1,
              "delay_ms": 500
            }
          }
        },
        {
          "sequence": 5,
          "at_ms": 2500,
          "kind": "ConnectAttempt"
        },
        {
          "sequence": 6,
          "at_ms": 2500,
          "kind": "Connected"
        },
        {
          "sequence": 7,
          "at_ms": 2500,
          "kind": {
            "InventoryRefreshed": {
              "generation": 1,
              "devices": 1
            }
          }
        },
        {
          "sequence": 8,
          "at_ms": 2550,
          "kind": {
            "ConnectionLost": {
              "reason": {
                "ConnectionLost": {
                  "cause": "ServerClosed"
                }
              }
            }
          }
        },
        {
          "sequence": 9,
          "at_ms": 2550,
          "kind": {
            "RetryScheduled": {
              "attempt": 1,
              "delay_ms": 500
            }
          }
        }
      ]
    }
  },
  "applying": null,
  "connect_apply": {
    "id": "p-0000000000000003",
    "name": "Desk",
    "outcome": {
      "Applied": {
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
        }
      }
    }
  },
  "on_connect": {
    "Fired": {
      "ran": {
        "id": "p-0000000000000003",
        "name": "Desk",
        "outcome": {
          "Applied": {
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
            }
          }
        }
      },
      "next": {
        "Ready": {
          "name": "Desk"
        }
      }
    }
  }
};

export const viewStoppedArmed: ShellView = {
  "revision": 29,
  "session": {
    "connection": {
      "Disconnected": {
        "reason": {
          "UnsupportedProtocol": {
            "server_version": 5
          }
        }
      }
    },
    "backend": {
      "origin": "OpenRgb",
      "address": null,
      "protocol_version": null,
      "server_name": null
    },
    "devices": [],
    "issues": [],
    "identify": null,
    "output": {
      "frame_rate": 30,
      "scene": {
        "assignments": []
      },
      "devices": []
    },
    "diagnostics": {
      "last_error": {
        "at_ms": 0,
        "operation": "Connect",
        "error": {
          "UnsupportedProtocol": {
            "server_version": 5,
            "required": 6
          }
        },
        "ended_connection": null
      },
      "counters": {
        "connect_attempts": 1,
        "connect_failures": 1,
        "inventory_refreshes": 0,
        "refresh_failures": 0,
        "commands_refused_busy": 0,
        "frames_written": 0,
        "ticks_skipped": 0,
        "output_writes_refused": 0,
        "output_writes_failed": 0,
        "output_wakeups": 0
      },
      "events": [
        {
          "sequence": 0,
          "at_ms": 0,
          "kind": "ConnectAttempt"
        },
        {
          "sequence": 1,
          "at_ms": 0,
          "kind": {
            "ConnectFailed": {
              "error": {
                "UnsupportedProtocol": {
                  "server_version": 5,
                  "required": 6
                }
              }
            }
          }
        }
      ]
    }
  },
  "applying": null,
  "connect_apply": null,
  "on_connect": {
    "Armed": {
      "id": "p-0000000000000003",
      "name": "Desk"
    }
  }
};
