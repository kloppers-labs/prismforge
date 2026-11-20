// Generated from real sessions by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.
import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";

export const demoConnected: SessionSnapshot = {
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
        "New": "demo-memory-1"
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
        "New": "demo-memory-2"
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
        "New": "demo-motherboard"
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
        "New": "demo-graphics-card"
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
        "New": "demo-light-bar"
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
          "active_mode": 0,
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
    "frame_rate": 30,
    "scene": {
      "assignments": []
    },
    "devices": []
  },
  "diagnostics": {
    "last_error": null,
    "counters": {
      "connect_attempts": 1,
      "connect_failures": 0,
      "inventory_refreshes": 1,
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
      }
    ]
  }
};

export const demoIdentifying: SessionSnapshot = {
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
        "New": "demo-memory-1"
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
        "New": "demo-memory-2"
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
        "New": "demo-motherboard"
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
        "New": "demo-graphics-card"
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
        "New": "demo-light-bar"
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
          "active_mode": 0,
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
  "identify": {
    "device": "demo-motherboard",
    "zone": 1
  },
  "output": {
    "frame_rate": 30,
    "scene": {
      "assignments": []
    },
    "devices": []
  },
  "diagnostics": {
    "last_error": null,
    "counters": {
      "connect_attempts": 1,
      "connect_failures": 0,
      "inventory_refreshes": 1,
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
          "IdentifyStarted": {
            "device": "demo-motherboard",
            "zone": 1
          }
        }
      }
    ]
  }
};

export const openRgbConnected: SessionSnapshot = {
  "connection": "Connected",
  "backend": {
    "origin": "OpenRgb",
    "address": "127.0.0.1:6742",
    "protocol_version": 6,
    "server_name": "PrismForge fake OpenRGB"
  },
  "devices": [
    {
      "identity": {
        "New": "dev-1052f35afd339289"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Strip",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-alpha",
        "location": "usb:alpha",
        "kind": "LedStrip",
        "zones": [
          {
            "name": "Header",
            "kind": "Linear",
            "led_count": 12,
            "leds_min": 0,
            "leds_max": 60,
            "matrix": null
          }
        ],
        "led_count": 12,
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
              "directions": [
                "LeftRight"
              ],
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
          "resizable_zones": [
            {
              "zone": 0,
              "min": 0,
              "max": 60
            }
          ],
          "saves_to_device_memory": true
        }
      },
      "available": true
    },
    {
      "identity": {
        "New": "dev-3aa84c4a5a06ce30"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Graphics Card",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-card",
        "location": "pci:1",
        "kind": "Gpu",
        "zones": [
          {
            "name": "Logo",
            "kind": "Linear",
            "led_count": 4,
            "leds_min": 4,
            "leds_max": 4,
            "matrix": null
          }
        ],
        "led_count": 4,
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
          }
        ],
        "capabilities": {
          "direct_mode": null,
          "active_mode": 0,
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
              "directions": [
                "LeftRight"
              ],
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
        "New": "dev-d59a35050e3ef684"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Light Bar",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-gamma",
        "location": "usb:gamma",
        "kind": "LedStrip",
        "zones": [
          {
            "name": "Header",
            "kind": "Linear",
            "led_count": 12,
            "leds_min": 0,
            "leds_max": 60,
            "matrix": null
          }
        ],
        "led_count": 12,
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
              "directions": [
                "LeftRight"
              ],
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
          "resizable_zones": [
            {
              "zone": 0,
              "min": 0,
              "max": 60
            }
          ],
          "saves_to_device_memory": true
        }
      },
      "available": true
    }
  ],
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
    "last_error": null,
    "counters": {
      "connect_attempts": 1,
      "connect_failures": 0,
      "inventory_refreshes": 1,
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
        "kind": "Connected"
      },
      {
        "sequence": 2,
        "at_ms": 0,
        "kind": {
          "InventoryRefreshed": {
            "generation": 1,
            "devices": 3
          }
        }
      }
    ]
  }
};

export const openRgbDegraded: SessionSnapshot = {
  "connection": {
    "Degraded": {
      "reasons": [
        {
          "ControllerDataRejected": {
            "controllers": 1
          }
        },
        {
          "AmbiguousDevices": {
            "devices": 2
          }
        }
      ]
    }
  },
  "backend": {
    "origin": "OpenRgb",
    "address": "127.0.0.1:6742",
    "protocol_version": 6,
    "server_name": "PrismForge fake OpenRGB"
  },
  "devices": [
    {
      "identity": {
        "Matched": "dev-1052f35afd339289"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Strip",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-alpha",
        "location": "usb:alpha",
        "kind": "LedStrip",
        "zones": [
          {
            "name": "Header",
            "kind": "Linear",
            "led_count": 12,
            "leds_min": 0,
            "leds_max": 60,
            "matrix": null
          }
        ],
        "led_count": 12,
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
              "directions": [
                "LeftRight"
              ],
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
          "resizable_zones": [
            {
              "zone": 0,
              "min": 0,
              "max": 60
            }
          ],
          "saves_to_device_memory": true
        }
      },
      "available": true
    },
    {
      "identity": {
        "Matched": "dev-3aa84c4a5a06ce30"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Graphics Card",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-card",
        "location": "pci:1",
        "kind": "Gpu",
        "zones": [
          {
            "name": "Logo",
            "kind": "Linear",
            "led_count": 4,
            "leds_min": 4,
            "leds_max": 4,
            "matrix": null
          }
        ],
        "led_count": 4,
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
          }
        ],
        "capabilities": {
          "direct_mode": null,
          "active_mode": 0,
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
              "directions": [
                "LeftRight"
              ],
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
        "New": "dev-db43b492ddc029f7"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Keyboard",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-keys",
        "location": "usb:keys",
        "kind": "Keyboard",
        "zones": [
          {
            "name": "Keys",
            "kind": "Matrix",
            "led_count": 8,
            "leds_min": 8,
            "leds_max": 8,
            "matrix": {
              "width": 4,
              "height": 2
            }
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
      "available": true
    },
    {
      "identity": {
        "Ambiguous": {
          "candidates": []
        }
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Fan",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "",
        "location": "",
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
      "available": true
    },
    {
      "identity": {
        "Ambiguous": {
          "candidates": []
        }
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Fan",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "",
        "location": "",
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
      "available": true
    },
    {
      "identity": {
        "New": "dev-d59a35050e3ef684"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Light Bar",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-gamma",
        "location": "usb:gamma",
        "kind": "LedStrip",
        "zones": [
          {
            "name": "Header",
            "kind": "Linear",
            "led_count": 12,
            "leds_min": 0,
            "leds_max": 60,
            "matrix": null
          }
        ],
        "led_count": 12,
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
              "directions": [
                "LeftRight"
              ],
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
          "resizable_zones": [
            {
              "zone": 0,
              "min": 0,
              "max": 60
            }
          ],
          "saves_to_device_memory": true
        }
      },
      "available": false
    }
  ],
  "issues": [
    {
      "runtime_id": 5,
      "reason": "InconsistentData",
      "detail": "zones hold 12 LEDs but the LED list has 3"
    }
  ],
  "identify": null,
  "output": {
    "frame_rate": 30,
    "scene": {
      "assignments": []
    },
    "devices": []
  },
  "diagnostics": {
    "last_error": null,
    "counters": {
      "connect_attempts": 2,
      "connect_failures": 0,
      "inventory_refreshes": 2,
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
        "kind": "Connected"
      },
      {
        "sequence": 2,
        "at_ms": 0,
        "kind": {
          "InventoryRefreshed": {
            "generation": 1,
            "devices": 3
          }
        }
      },
      {
        "sequence": 3,
        "at_ms": 0,
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
        "sequence": 4,
        "at_ms": 0,
        "kind": {
          "RetryScheduled": {
            "attempt": 1,
            "delay_ms": 500
          }
        }
      },
      {
        "sequence": 5,
        "at_ms": 500,
        "kind": "ConnectAttempt"
      },
      {
        "sequence": 6,
        "at_ms": 500,
        "kind": "Connected"
      },
      {
        "sequence": 7,
        "at_ms": 500,
        "kind": {
          "InventoryRefreshed": {
            "generation": 2,
            "devices": 5
          }
        }
      }
    ]
  }
};

export const openRgbLostWhileConnected: SessionSnapshot = {
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
    "address": "127.0.0.1:6742",
    "protocol_version": null,
    "server_name": null
  },
  "devices": [
    {
      "identity": {
        "New": "dev-1052f35afd339289"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Strip",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-alpha",
        "location": "usb:alpha",
        "kind": "LedStrip",
        "zones": [
          {
            "name": "Header",
            "kind": "Linear",
            "led_count": 12,
            "leds_min": 0,
            "leds_max": 60,
            "matrix": null
          }
        ],
        "led_count": 12,
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
              "directions": [
                "LeftRight"
              ],
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
          "resizable_zones": [
            {
              "zone": 0,
              "min": 0,
              "max": 60
            }
          ],
          "saves_to_device_memory": true
        }
      },
      "available": false
    },
    {
      "identity": {
        "New": "dev-3aa84c4a5a06ce30"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Graphics Card",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-card",
        "location": "pci:1",
        "kind": "Gpu",
        "zones": [
          {
            "name": "Logo",
            "kind": "Linear",
            "led_count": 4,
            "leds_min": 4,
            "leds_max": 4,
            "matrix": null
          }
        ],
        "led_count": 4,
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
          }
        ],
        "capabilities": {
          "direct_mode": null,
          "active_mode": 0,
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
              "directions": [
                "LeftRight"
              ],
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
      "available": false
    }
  ],
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
    "last_error": null,
    "counters": {
      "connect_attempts": 1,
      "connect_failures": 0,
      "inventory_refreshes": 1,
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
        "kind": "Connected"
      },
      {
        "sequence": 2,
        "at_ms": 0,
        "kind": {
          "InventoryRefreshed": {
            "generation": 1,
            "devices": 2
          }
        }
      },
      {
        "sequence": 3,
        "at_ms": 0,
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
        "sequence": 4,
        "at_ms": 0,
        "kind": {
          "RetryScheduled": {
            "attempt": 1,
            "delay_ms": 500
          }
        }
      }
    ]
  }
};

export const openRgbLostRetryFailed: SessionSnapshot = {
  "connection": {
    "Retrying": {
      "attempt": 2,
      "reason": {
        "UnsupportedProtocol": {
          "server_version": null
        }
      }
    }
  },
  "backend": {
    "origin": "OpenRgb",
    "address": "127.0.0.1:6742",
    "protocol_version": null,
    "server_name": null
  },
  "devices": [
    {
      "identity": {
        "New": "dev-1052f35afd339289"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Strip",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-alpha",
        "location": "usb:alpha",
        "kind": "LedStrip",
        "zones": [
          {
            "name": "Header",
            "kind": "Linear",
            "led_count": 12,
            "leds_min": 0,
            "leds_max": 60,
            "matrix": null
          }
        ],
        "led_count": 12,
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
              "directions": [
                "LeftRight"
              ],
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
          "resizable_zones": [
            {
              "zone": 0,
              "min": 0,
              "max": 60
            }
          ],
          "saves_to_device_memory": true
        }
      },
      "available": false
    },
    {
      "identity": {
        "New": "dev-3aa84c4a5a06ce30"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Graphics Card",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-card",
        "location": "pci:1",
        "kind": "Gpu",
        "zones": [
          {
            "name": "Logo",
            "kind": "Linear",
            "led_count": 4,
            "leds_min": 4,
            "leds_max": 4,
            "matrix": null
          }
        ],
        "led_count": 4,
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
          }
        ],
        "capabilities": {
          "direct_mode": null,
          "active_mode": 0,
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
              "directions": [
                "LeftRight"
              ],
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
      "available": false
    }
  ],
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
      "at_ms": 2500,
      "operation": "Connect",
      "error": {
        "UnsupportedProtocol": {
          "server_version": null,
          "required": 6
        }
      },
      "ended_connection": null
    },
    "counters": {
      "connect_attempts": 2,
      "connect_failures": 1,
      "inventory_refreshes": 1,
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
        "kind": "Connected"
      },
      {
        "sequence": 2,
        "at_ms": 0,
        "kind": {
          "InventoryRefreshed": {
            "generation": 1,
            "devices": 2
          }
        }
      },
      {
        "sequence": 3,
        "at_ms": 0,
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
        "sequence": 4,
        "at_ms": 0,
        "kind": {
          "RetryScheduled": {
            "attempt": 1,
            "delay_ms": 500
          }
        }
      },
      {
        "sequence": 5,
        "at_ms": 500,
        "kind": "ConnectAttempt"
      },
      {
        "sequence": 6,
        "at_ms": 2500,
        "kind": {
          "ConnectFailed": {
            "error": {
              "UnsupportedProtocol": {
                "server_version": null,
                "required": 6
              }
            }
          }
        }
      },
      {
        "sequence": 7,
        "at_ms": 2500,
        "kind": {
          "RetryScheduled": {
            "attempt": 2,
            "delay_ms": 1000
          }
        }
      }
    ]
  }
};

export const openRgbAllRejected: SessionSnapshot = {
  "connection": {
    "Degraded": {
      "reasons": [
        {
          "ControllerDataRejected": {
            "controllers": 1
          }
        }
      ]
    }
  },
  "backend": {
    "origin": "OpenRgb",
    "address": "127.0.0.1:6742",
    "protocol_version": 6,
    "server_name": "PrismForge fake OpenRGB"
  },
  "devices": [],
  "issues": [
    {
      "runtime_id": 0,
      "reason": "InconsistentData",
      "detail": "zones hold 12 LEDs but the LED list has 3"
    }
  ],
  "identify": null,
  "output": {
    "frame_rate": 30,
    "scene": {
      "assignments": []
    },
    "devices": []
  },
  "diagnostics": {
    "last_error": null,
    "counters": {
      "connect_attempts": 1,
      "connect_failures": 0,
      "inventory_refreshes": 1,
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
        "kind": "Connected"
      },
      {
        "sequence": 2,
        "at_ms": 0,
        "kind": {
          "InventoryRefreshed": {
            "generation": 1,
            "devices": 0
          }
        }
      }
    ]
  }
};

export const openRgbRefused: SessionSnapshot = {
  "connection": {
    "Retrying": {
      "attempt": 3,
      "reason": "ServerUnreachable"
    }
  },
  "backend": {
    "origin": "OpenRgb",
    "address": "127.0.0.1:6742",
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
      "at_ms": 1500,
      "operation": "Connect",
      "error": {
        "Unreachable": {
          "detail": "connection refused"
        }
      },
      "ended_connection": null
    },
    "counters": {
      "connect_attempts": 3,
      "connect_failures": 3,
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
              "Unreachable": {
                "detail": "connection refused"
              }
            }
          }
        }
      },
      {
        "sequence": 2,
        "at_ms": 0,
        "kind": {
          "RetryScheduled": {
            "attempt": 1,
            "delay_ms": 500
          }
        }
      },
      {
        "sequence": 3,
        "at_ms": 500,
        "kind": "ConnectAttempt"
      },
      {
        "sequence": 4,
        "at_ms": 500,
        "kind": {
          "ConnectFailed": {
            "error": {
              "Unreachable": {
                "detail": "connection refused"
              }
            }
          }
        }
      },
      {
        "sequence": 5,
        "at_ms": 500,
        "kind": {
          "RetryScheduled": {
            "attempt": 2,
            "delay_ms": 1000
          }
        }
      },
      {
        "sequence": 6,
        "at_ms": 1500,
        "kind": "ConnectAttempt"
      },
      {
        "sequence": 7,
        "at_ms": 1500,
        "kind": {
          "ConnectFailed": {
            "error": {
              "Unreachable": {
                "detail": "connection refused"
              }
            }
          }
        }
      },
      {
        "sequence": 8,
        "at_ms": 1500,
        "kind": {
          "RetryScheduled": {
            "attempt": 3,
            "delay_ms": 2000
          }
        }
      }
    ]
  }
};

export const openRgbConnecting: SessionSnapshot = {
  "connection": "Connecting",
  "backend": {
    "origin": "OpenRgb",
    "address": "127.0.0.1:6742",
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
    "last_error": null,
    "counters": {
      "connect_attempts": 0,
      "connect_failures": 0,
      "inventory_refreshes": 0,
      "refresh_failures": 0,
      "commands_refused_busy": 0,
      "frames_written": 0,
      "ticks_skipped": 0,
      "output_writes_refused": 0,
      "output_writes_failed": 0,
      "output_wakeups": 0
    },
    "events": []
  }
};

export const openRgbNoVersionAnswer: SessionSnapshot = {
  "connection": {
    "Retrying": {
      "attempt": 1,
      "reason": {
        "UnsupportedProtocol": {
          "server_version": null
        }
      }
    }
  },
  "backend": {
    "origin": "OpenRgb",
    "address": "127.0.0.1:6742",
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
      "operation": "Connect",
      "error": {
        "UnsupportedProtocol": {
          "server_version": null,
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
        "at_ms": 2000,
        "kind": {
          "ConnectFailed": {
            "error": {
              "UnsupportedProtocol": {
                "server_version": null,
                "required": 6
              }
            }
          }
        }
      },
      {
        "sequence": 2,
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
};

export const openRgbOldServer: SessionSnapshot = {
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
    "address": "127.0.0.1:6742",
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
};

export const demoEffectsRunning: SessionSnapshot = {
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
    "frame_rate": 30,
    "scene": {
      "assignments": [
        {
          "target": {
            "device": "demo-graphics-card",
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
          },
          "mode": "EachDevice"
        },
        {
          "target": {
            "device": "demo-memory-1",
            "zone": null
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
                  "g": 128,
                  "b": 255
                }
              }
            }
          },
          "mode": "EachDevice"
        },
        {
          "target": {
            "device": "demo-motherboard",
            "zone": 1
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
                  "g": 64,
                  "b": 0
                }
              },
              "direction": {
                "Direction": "Backward"
              },
              "speed": {
                "Number": 30
              },
              "width": {
                "Number": 20
              }
            }
          },
          "mode": "EachDevice"
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
        "device": "demo-memory-1",
        "owner": "Effect",
        "condition": {
          "Refused": {
            "error": {
              "Rejected": {
                "operation": "simulated colour write",
                "status": 1
              }
            },
            "attempts": 1
          }
        }
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
    "last_error": {
      "at_ms": 34,
      "operation": "Output",
      "error": {
        "Rejected": {
          "operation": "simulated colour write",
          "status": 1
        }
      },
      "ended_connection": null
    },
    "counters": {
      "connect_attempts": 1,
      "connect_failures": 0,
      "inventory_refreshes": 2,
      "refresh_failures": 0,
      "commands_refused_busy": 0,
      "frames_written": 3,
      "ticks_skipped": 0,
      "output_writes_refused": 1,
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
};

export const demoEffectIdentifying: SessionSnapshot = {
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
  "identify": {
    "device": "demo-graphics-card",
    "zone": null
  },
  "output": {
    "frame_rate": 30,
    "scene": {
      "assignments": [
        {
          "target": {
            "device": "demo-graphics-card",
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
          },
          "mode": "EachDevice"
        },
        {
          "target": {
            "device": "demo-memory-1",
            "zone": null
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
                  "g": 128,
                  "b": 255
                }
              }
            }
          },
          "mode": "EachDevice"
        },
        {
          "target": {
            "device": "demo-motherboard",
            "zone": 1
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
                  "g": 64,
                  "b": 0
                }
              },
              "direction": {
                "Direction": "Backward"
              },
              "speed": {
                "Number": 30
              },
              "width": {
                "Number": 20
              }
            }
          },
          "mode": "EachDevice"
        }
      ]
    },
    "devices": [
      {
        "device": "demo-graphics-card",
        "owner": "Effect",
        "condition": {
          "Paused": {
            "reason": "Identify"
          }
        }
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
        "device": "demo-memory-1",
        "owner": "Effect",
        "condition": {
          "Refused": {
            "error": {
              "Rejected": {
                "operation": "simulated colour write",
                "status": 1
              }
            },
            "attempts": 1
          }
        }
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
    "last_error": {
      "at_ms": 34,
      "operation": "Output",
      "error": {
        "Rejected": {
          "operation": "simulated colour write",
          "status": 1
        }
      },
      "ended_connection": null
    },
    "counters": {
      "connect_attempts": 1,
      "connect_failures": 0,
      "inventory_refreshes": 2,
      "refresh_failures": 0,
      "commands_refused_busy": 0,
      "frames_written": 3,
      "ticks_skipped": 0,
      "output_writes_refused": 1,
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
      },
      {
        "sequence": 4,
        "at_ms": 34,
        "kind": {
          "IdentifyStarted": {
            "device": "demo-graphics-card",
            "zone": null
          }
        }
      }
    ]
  }
};

export const demoEffectsNotConnected: SessionSnapshot = {
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
      "available": false
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
      "available": false
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
            "r": 85,
            "g": 21,
            "b": 0
          },
          {
            "r": 170,
            "g": 43,
            "b": 0
          },
          {
            "r": 255,
            "g": 64,
            "b": 0
          },
          {
            "r": 170,
            "g": 43,
            "b": 0
          },
          {
            "r": 85,
            "g": 21,
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
      "available": false
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
            "b": 17
          },
          {
            "r": 255,
            "g": 122,
            "b": 0
          },
          {
            "r": 249,
            "g": 255,
            "b": 0
          },
          {
            "r": 110,
            "g": 255,
            "b": 0
          },
          {
            "r": 0,
            "g": 255,
            "b": 29
          },
          {
            "r": 0,
            "g": 255,
            "b": 168
          },
          {
            "r": 0,
            "g": 203,
            "b": 255
          },
          {
            "r": 0,
            "g": 64,
            "b": 255
          },
          {
            "r": 75,
            "g": 0,
            "b": 255
          },
          {
            "r": 214,
            "g": 0,
            "b": 255
          },
          {
            "r": 255,
            "g": 0,
            "b": 156
          },
          {
            "r": 255,
            "g": 0,
            "b": 17
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
      "available": false
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
            "device": "demo-graphics-card",
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
          },
          "mode": "EachDevice"
        },
        {
          "target": {
            "device": "demo-memory-1",
            "zone": null
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
                  "g": 128,
                  "b": 255
                }
              }
            }
          },
          "mode": "EachDevice"
        },
        {
          "target": {
            "device": "demo-motherboard",
            "zone": 1
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
                  "g": 64,
                  "b": 0
                }
              },
              "direction": {
                "Direction": "Backward"
              },
              "speed": {
                "Number": 30
              },
              "width": {
                "Number": 20
              }
            }
          },
          "mode": "EachDevice"
        }
      ]
    },
    "devices": [
      {
        "device": "demo-graphics-card",
        "owner": "Effect",
        "condition": {
          "Paused": {
            "reason": "NotConnected"
          }
        }
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
        "device": "demo-memory-1",
        "owner": "Effect",
        "condition": {
          "Paused": {
            "reason": "NotConnected"
          }
        }
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
      "at_ms": 3034,
      "operation": "Output",
      "error": {
        "Rejected": {
          "operation": "simulated colour write",
          "status": 1
        }
      },
      "ended_connection": null
    },
    "counters": {
      "connect_attempts": 1,
      "connect_failures": 0,
      "inventory_refreshes": 3,
      "refresh_failures": 0,
      "commands_refused_busy": 0,
      "frames_written": 94,
      "ticks_skipped": 0,
      "output_writes_refused": 3,
      "output_writes_failed": 0,
      "output_wakeups": 92
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
      },
      {
        "sequence": 4,
        "at_ms": 34,
        "kind": {
          "IdentifyStarted": {
            "device": "demo-graphics-card",
            "zone": null
          }
        }
      },
      {
        "sequence": 5,
        "at_ms": 3034,
        "kind": {
          "IdentifyFinished": {
            "outcome": {
              "device": "demo-graphics-card",
              "zone": null,
              "light_failure": null,
              "restore": "Restored"
            }
          }
        }
      },
      {
        "sequence": 6,
        "at_ms": 3034,
        "kind": {
          "InventoryRefreshed": {
            "generation": 3,
            "devices": 5
          }
        }
      },
      {
        "sequence": 7,
        "at_ms": 3034,
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
        "sequence": 8,
        "at_ms": 3034,
        "kind": {
          "RetryScheduled": {
            "attempt": 1,
            "delay_ms": 500
          }
        }
      }
    ]
  }
};

export const demoBatchRunning: SessionSnapshot = {
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
        "New": "demo-memory-1"
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
        "New": "demo-memory-2"
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
        "New": "demo-motherboard"
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
        "New": "demo-graphics-card"
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
        "New": "demo-light-bar"
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
          "active_mode": 0,
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
    "frame_rate": 30,
    "scene": {
      "assignments": [
        {
          "target": {
            "device": "demo-graphics-card",
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
          },
          "mode": "EachDevice"
        },
        {
          "target": {
            "device": "demo-memory-2",
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
          },
          "mode": "EachDevice"
        },
        {
          "target": {
            "device": "demo-motherboard",
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
          },
          "mode": "EachDevice"
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
        "device": "demo-memory-1",
        "owner": {
          "External": {
            "label": "OpenRGB Effects plugin"
          }
        },
        "condition": "Idle"
      },
      {
        "device": "demo-memory-2",
        "owner": "Effect",
        "condition": "Running"
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
      "inventory_refreshes": 1,
      "refresh_failures": 0,
      "commands_refused_busy": 0,
      "frames_written": 3,
      "ticks_skipped": 0,
      "output_writes_refused": 0,
      "output_writes_failed": 0,
      "output_wakeups": 1
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
      }
    ]
  }
};

export const demoIdentifyingGpu: SessionSnapshot = {
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
        "New": "demo-memory-1"
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
        "New": "demo-memory-2"
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
        "New": "demo-motherboard"
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
        "New": "demo-graphics-card"
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
        "New": "demo-light-bar"
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
          "active_mode": 0,
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
  "identify": {
    "device": "demo-graphics-card",
    "zone": null
  },
  "output": {
    "frame_rate": 30,
    "scene": {
      "assignments": []
    },
    "devices": []
  },
  "diagnostics": {
    "last_error": null,
    "counters": {
      "connect_attempts": 1,
      "connect_failures": 0,
      "inventory_refreshes": 1,
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
          "IdentifyStarted": {
            "device": "demo-graphics-card",
            "zone": null
          }
        }
      }
    ]
  }
};

export const demoBatchDuringIdentify: SessionSnapshot = {
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
        "New": "demo-memory-1"
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
        "New": "demo-memory-2"
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
        "New": "demo-motherboard"
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
        "New": "demo-graphics-card"
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
        "New": "demo-light-bar"
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
          "active_mode": 0,
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
  "identify": {
    "device": "demo-graphics-card",
    "zone": null
  },
  "output": {
    "frame_rate": 30,
    "scene": {
      "assignments": [
        {
          "target": {
            "device": "demo-memory-1",
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
          },
          "mode": "EachDevice"
        },
        {
          "target": {
            "device": "demo-memory-2",
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
          },
          "mode": "EachDevice"
        },
        {
          "target": {
            "device": "demo-motherboard",
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
          },
          "mode": "EachDevice"
        }
      ]
    },
    "devices": [
      {
        "device": "demo-memory-1",
        "owner": "Effect",
        "condition": "Running"
      },
      {
        "device": "demo-memory-2",
        "owner": "Effect",
        "condition": "Running"
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
      "inventory_refreshes": 1,
      "refresh_failures": 0,
      "commands_refused_busy": 0,
      "frames_written": 3,
      "ticks_skipped": 0,
      "output_writes_refused": 0,
      "output_writes_failed": 0,
      "output_wakeups": 1
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
          "IdentifyStarted": {
            "device": "demo-graphics-card",
            "zone": null
          }
        }
      }
    ]
  }
};

export const openRgbShelfGone: SessionSnapshot = {
  "connection": "Connected",
  "backend": {
    "origin": "OpenRgb",
    "address": null,
    "protocol_version": 6,
    "server_name": "PrismForge fake OpenRGB"
  },
  "devices": [
    {
      "identity": {
        "Matched": "dev-4947ff71bba71ad6"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Desk Strip",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-desk",
        "location": "usb:desk",
        "kind": "LedStrip",
        "zones": [
          {
            "name": "Strip",
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
      "available": true
    }
  ],
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
    "last_error": null,
    "counters": {
      "connect_attempts": 1,
      "connect_failures": 0,
      "inventory_refreshes": 1,
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
        "kind": "Connected"
      },
      {
        "sequence": 2,
        "at_ms": 0,
        "kind": {
          "InventoryRefreshed": {
            "generation": 1,
            "devices": 1
          }
        }
      }
    ]
  }
};

export const openRgbKeyboardRunning: SessionSnapshot = {
  "connection": "Connected",
  "backend": {
    "origin": "OpenRgb",
    "address": "127.0.0.1:6742",
    "protocol_version": 6,
    "server_name": "PrismForge fake OpenRGB"
  },
  "devices": [
    {
      "identity": {
        "New": "dev-bb7e37a791b1baed"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Keyboard",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-keys",
        "location": "usb:keys",
        "kind": "LedStrip",
        "zones": [
          {
            "name": "Keys",
            "kind": "Matrix",
            "led_count": 48,
            "leds_min": 48,
            "leds_max": 48,
            "matrix": {
              "width": 12,
              "height": 4
            }
          }
        ],
        "led_count": 48,
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
      "available": true
    },
    {
      "identity": {
        "New": "dev-2bb511c8407d4ad7"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Desk Strip",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-desk",
        "location": "usb:desk",
        "kind": "LedStrip",
        "zones": [
          {
            "name": "Header",
            "kind": "Linear",
            "led_count": 12,
            "leds_min": 0,
            "leds_max": 60,
            "matrix": null
          }
        ],
        "led_count": 12,
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
              "directions": [
                "LeftRight"
              ],
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
          "resizable_zones": [
            {
              "zone": 0,
              "min": 0,
              "max": 60
            }
          ],
          "saves_to_device_memory": true
        }
      },
      "available": true
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
            "device": "dev-2bb511c8407d4ad7",
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
          },
          "mode": "EachDevice"
        },
        {
          "target": {
            "device": "dev-bb7e37a791b1baed",
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
          },
          "mode": "EachDevice"
        }
      ]
    },
    "devices": [
      {
        "device": "dev-2bb511c8407d4ad7",
        "owner": "Effect",
        "condition": "Running"
      },
      {
        "device": "dev-bb7e37a791b1baed",
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
      "inventory_refreshes": 1,
      "refresh_failures": 0,
      "commands_refused_busy": 0,
      "frames_written": 32,
      "ticks_skipped": 0,
      "output_writes_refused": 0,
      "output_writes_failed": 0,
      "output_wakeups": 16
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
            "devices": 2
          }
        }
      }
    ]
  }
};

export const demoAcrossRunning: SessionSnapshot = {
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
        "New": "demo-memory-1"
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
        "New": "demo-memory-2"
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
        "New": "demo-motherboard"
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
        "New": "demo-graphics-card"
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
        "New": "demo-light-bar"
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
          "active_mode": 0,
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
    "frame_rate": 30,
    "scene": {
      "assignments": [
        {
          "target": {
            "device": "demo-graphics-card",
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
          },
          "mode": {
            "AcrossSetup": {
              "view": "v-pc",
              "flow": {
                "Along": {
                  "angle": 0
                }
              }
            }
          }
        },
        {
          "target": {
            "device": "demo-memory-1",
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
          },
          "mode": {
            "AcrossSetup": {
              "view": "v-pc",
              "flow": {
                "Along": {
                  "angle": 0
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
          },
          "mode": {
            "AcrossSetup": {
              "view": "v-pc",
              "flow": {
                "Along": {
                  "angle": 0
                }
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
          },
          "mode": {
            "AcrossSetup": {
              "view": "v-pc",
              "flow": {
                "Along": {
                  "angle": 0
                }
              }
            }
          }
        },
        {
          "target": {
            "device": "demo-motherboard",
            "zone": 2
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
          },
          "mode": {
            "AcrossSetup": {
              "view": "v-pc",
              "flow": {
                "Along": {
                  "angle": 0
                }
              }
            }
          }
        },
        {
          "target": {
            "device": "demo-motherboard",
            "zone": 0
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
          },
          "mode": {
            "AcrossSetup": {
              "view": "v-pc",
              "flow": {
                "Along": {
                  "angle": 0
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
        "device": "demo-memory-1",
        "owner": "Effect",
        "condition": "Running"
      },
      {
        "device": "demo-memory-2",
        "owner": "Effect",
        "condition": "Running"
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
      "inventory_refreshes": 1,
      "refresh_failures": 0,
      "commands_refused_busy": 0,
      "frames_written": 90,
      "ticks_skipped": 0,
      "output_writes_refused": 0,
      "output_writes_failed": 0,
      "output_wakeups": 15
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
      }
    ]
  }
};

export const demoAcrossBesideEachDevice: SessionSnapshot = {
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
        "New": "demo-memory-1"
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
        "New": "demo-memory-2"
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
        "New": "demo-motherboard"
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
        "New": "demo-graphics-card"
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
        "New": "demo-light-bar"
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
          "active_mode": 0,
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
    "frame_rate": 30,
    "scene": {
      "assignments": [
        {
          "target": {
            "device": "demo-graphics-card",
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
          },
          "mode": {
            "AcrossSetup": {
              "view": "v-pc",
              "flow": {
                "Along": {
                  "angle": 0
                }
              }
            }
          }
        },
        {
          "target": {
            "device": "demo-memory-1",
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
          },
          "mode": {
            "AcrossSetup": {
              "view": "v-pc",
              "flow": {
                "Along": {
                  "angle": 0
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
          },
          "mode": "EachDevice"
        },
        {
          "target": {
            "device": "demo-motherboard",
            "zone": 1
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
          },
          "mode": {
            "AcrossSetup": {
              "view": "v-pc",
              "flow": {
                "Along": {
                  "angle": 0
                }
              }
            }
          }
        },
        {
          "target": {
            "device": "demo-motherboard",
            "zone": 2
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
          },
          "mode": {
            "AcrossSetup": {
              "view": "v-pc",
              "flow": {
                "Along": {
                  "angle": 0
                }
              }
            }
          }
        },
        {
          "target": {
            "device": "demo-motherboard",
            "zone": 0
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
          },
          "mode": {
            "AcrossSetup": {
              "view": "v-pc",
              "flow": {
                "Along": {
                  "angle": 0
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
        "device": "demo-memory-1",
        "owner": "Effect",
        "condition": "Running"
      },
      {
        "device": "demo-memory-2",
        "owner": "Effect",
        "condition": "Running"
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
      "inventory_refreshes": 1,
      "refresh_failures": 0,
      "commands_refused_busy": 0,
      "frames_written": 180,
      "ticks_skipped": 0,
      "output_writes_refused": 0,
      "output_writes_failed": 0,
      "output_wakeups": 30
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
      }
    ]
  }
};

export const demoAcrossSkipped: SessionSnapshot = {
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
        "New": "demo-memory-1"
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
        "New": "demo-memory-2"
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
        "New": "demo-motherboard"
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
        "New": "demo-graphics-card"
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
        "New": "demo-light-bar"
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
          "active_mode": 0,
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
  "identify": {
    "device": "demo-graphics-card",
    "zone": null
  },
  "output": {
    "frame_rate": 30,
    "scene": {
      "assignments": [
        {
          "target": {
            "device": "demo-memory-1",
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
          },
          "mode": {
            "AcrossSetup": {
              "view": "v-pc",
              "flow": {
                "Along": {
                  "angle": 0
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
          },
          "mode": {
            "AcrossSetup": {
              "view": "v-pc",
              "flow": {
                "Along": {
                  "angle": 0
                }
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
          },
          "mode": {
            "AcrossSetup": {
              "view": "v-pc",
              "flow": {
                "Along": {
                  "angle": 0
                }
              }
            }
          }
        },
        {
          "target": {
            "device": "demo-motherboard",
            "zone": 2
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
          },
          "mode": {
            "AcrossSetup": {
              "view": "v-pc",
              "flow": {
                "Along": {
                  "angle": 0
                }
              }
            }
          }
        }
      ]
    },
    "devices": [
      {
        "device": "demo-memory-1",
        "owner": "Effect",
        "condition": "Running"
      },
      {
        "device": "demo-memory-2",
        "owner": "Effect",
        "condition": "Running"
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
      "inventory_refreshes": 1,
      "refresh_failures": 0,
      "commands_refused_busy": 0,
      "frames_written": 4,
      "ticks_skipped": 0,
      "output_writes_refused": 0,
      "output_writes_failed": 0,
      "output_wakeups": 1
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
          "IdentifyStarted": {
            "device": "demo-graphics-card",
            "zone": null
          }
        }
      }
    ]
  }
};

export const openRgbSetupDevices: SessionSnapshot = {
  "connection": "Connected",
  "backend": {
    "origin": "OpenRgb",
    "address": null,
    "protocol_version": 6,
    "server_name": "PrismForge fake OpenRGB"
  },
  "devices": [
    {
      "identity": {
        "New": "dev-6f970707d354ee85"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Motherboard",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-board",
        "location": "pci:board",
        "kind": "LedStrip",
        "zones": [
          {
            "name": "ARGB Header 1",
            "kind": "Linear",
            "led_count": 36,
            "leds_min": 0,
            "leds_max": 1024,
            "matrix": null
          },
          {
            "name": "ARGB Header 2",
            "kind": "Linear",
            "led_count": 16,
            "leds_min": 0,
            "leds_max": 1024,
            "matrix": null
          },
          {
            "name": "Onboard",
            "kind": "Linear",
            "led_count": 6,
            "leds_min": 6,
            "leds_max": 6,
            "matrix": null
          }
        ],
        "led_count": 58,
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
          "resizable_zones": [
            {
              "zone": 0,
              "min": 0,
              "max": 1024
            },
            {
              "zone": 1,
              "min": 0,
              "max": 1024
            }
          ],
          "saves_to_device_memory": true
        }
      },
      "available": true
    },
    {
      "identity": {
        "New": "dev-420d41cc43011cb2"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Graphics Card",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-card",
        "location": "pci:1",
        "kind": "Gpu",
        "zones": [
          {
            "name": "Light bar",
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
      "available": true
    },
    {
      "identity": {
        "New": "dev-8760c9a322bc0ba5"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Keyboard",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-keys",
        "location": "usb:keys",
        "kind": "Keyboard",
        "zones": [
          {
            "name": "Keys",
            "kind": "Matrix",
            "led_count": 48,
            "leds_min": 48,
            "leds_max": 48,
            "matrix": {
              "width": 12,
              "height": 4
            }
          }
        ],
        "led_count": 48,
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
      "available": true
    },
    {
      "identity": {
        "New": "dev-2bb511c8407d4ad7"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Desk Strip",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-desk",
        "location": "usb:desk",
        "kind": "LedStrip",
        "zones": [
          {
            "name": "Header",
            "kind": "Linear",
            "led_count": 12,
            "leds_min": 0,
            "leds_max": 60,
            "matrix": null
          }
        ],
        "led_count": 12,
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
              "directions": [
                "LeftRight"
              ],
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
          "resizable_zones": [
            {
              "zone": 0,
              "min": 0,
              "max": 60
            }
          ],
          "saves_to_device_memory": true
        }
      },
      "available": true
    }
  ],
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
    "last_error": null,
    "counters": {
      "connect_attempts": 1,
      "connect_failures": 0,
      "inventory_refreshes": 1,
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
        "kind": "Connected"
      },
      {
        "sequence": 2,
        "at_ms": 0,
        "kind": {
          "InventoryRefreshed": {
            "generation": 1,
            "devices": 4
          }
        }
      }
    ]
  }
};

export const openRgbAcrossRunning: SessionSnapshot = {
  "connection": "Connected",
  "backend": {
    "origin": "OpenRgb",
    "address": null,
    "protocol_version": 6,
    "server_name": "PrismForge fake OpenRGB"
  },
  "devices": [
    {
      "identity": {
        "New": "dev-6f970707d354ee85"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Motherboard",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-board",
        "location": "pci:board",
        "kind": "LedStrip",
        "zones": [
          {
            "name": "ARGB Header 1",
            "kind": "Linear",
            "led_count": 36,
            "leds_min": 0,
            "leds_max": 1024,
            "matrix": null
          },
          {
            "name": "ARGB Header 2",
            "kind": "Linear",
            "led_count": 16,
            "leds_min": 0,
            "leds_max": 1024,
            "matrix": null
          },
          {
            "name": "Onboard",
            "kind": "Linear",
            "led_count": 6,
            "leds_min": 6,
            "leds_max": 6,
            "matrix": null
          }
        ],
        "led_count": 58,
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
          "resizable_zones": [
            {
              "zone": 0,
              "min": 0,
              "max": 1024
            },
            {
              "zone": 1,
              "min": 0,
              "max": 1024
            }
          ],
          "saves_to_device_memory": true
        }
      },
      "available": true
    },
    {
      "identity": {
        "New": "dev-420d41cc43011cb2"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Graphics Card",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-card",
        "location": "pci:1",
        "kind": "Gpu",
        "zones": [
          {
            "name": "Light bar",
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
      "available": true
    },
    {
      "identity": {
        "New": "dev-8760c9a322bc0ba5"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Keyboard",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-keys",
        "location": "usb:keys",
        "kind": "Keyboard",
        "zones": [
          {
            "name": "Keys",
            "kind": "Matrix",
            "led_count": 48,
            "leds_min": 48,
            "leds_max": 48,
            "matrix": {
              "width": 12,
              "height": 4
            }
          }
        ],
        "led_count": 48,
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
      "available": true
    },
    {
      "identity": {
        "New": "dev-2bb511c8407d4ad7"
      },
      "descriptor": {
        "origin": "OpenRgb",
        "name": "Fixture Desk Strip",
        "vendor": "Fixture Vendor",
        "description": "Fixture device",
        "version": "1.0",
        "serial": "SN-desk",
        "location": "usb:desk",
        "kind": "LedStrip",
        "zones": [
          {
            "name": "Header",
            "kind": "Linear",
            "led_count": 12,
            "leds_min": 0,
            "leds_max": 60,
            "matrix": null
          }
        ],
        "led_count": 12,
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
              "directions": [
                "LeftRight"
              ],
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
          "resizable_zones": [
            {
              "zone": 0,
              "min": 0,
              "max": 60
            }
          ],
          "saves_to_device_memory": true
        }
      },
      "available": true
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
            "device": "dev-420d41cc43011cb2",
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
          },
          "mode": {
            "AcrossSetup": {
              "view": "v-pc",
              "flow": {
                "Along": {
                  "angle": 0
                }
              }
            }
          }
        },
        {
          "target": {
            "device": "dev-6f970707d354ee85",
            "zone": 0
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
          },
          "mode": {
            "AcrossSetup": {
              "view": "v-pc",
              "flow": {
                "Along": {
                  "angle": 0
                }
              }
            }
          }
        },
        {
          "target": {
            "device": "dev-6f970707d354ee85",
            "zone": 1
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
          },
          "mode": {
            "AcrossSetup": {
              "view": "v-pc",
              "flow": {
                "Along": {
                  "angle": 0
                }
              }
            }
          }
        }
      ]
    },
    "devices": [
      {
        "device": "dev-420d41cc43011cb2",
        "owner": "Effect",
        "condition": "Running"
      },
      {
        "device": "dev-6f970707d354ee85",
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
      "inventory_refreshes": 1,
      "refresh_failures": 0,
      "commands_refused_busy": 0,
      "frames_written": 45,
      "ticks_skipped": 0,
      "output_writes_refused": 0,
      "output_writes_failed": 0,
      "output_wakeups": 15
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
            "devices": 4
          }
        }
      }
    ]
  }
};
