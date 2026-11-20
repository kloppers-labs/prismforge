// Generated from real sessions by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.
import type { SetupStoreView } from "../../lib/api/bindings/SetupStoreView";

export const setupDemoEmpty: SetupStoreView = {
  "kind": "Demo",
  "setup": {
    "components": [],
    "outputs": [],
    "views": []
  },
  "outputs": [],
  "limits": {
    "components": 512,
    "outputs": 256,
    "views": 16,
    "branches": 16,
    "component_leds": 4096,
    "line_points": 256,
    "name": 60,
    "frame_min": 10,
    "frame_max": 1000,
    "outline_size": 4000
  }
};

export const setupDemoTower: SetupStoreView = {
  "kind": "Demo",
  "setup": {
    "components": [
      {
        "id": "c-front-fan-1",
        "name": "Front fan 1",
        "geometry": {
          "Ring": {
            "leds": 8,
            "first_angle": 0,
            "turn": "Clockwise"
          }
        }
      },
      {
        "id": "c-front-fan-2",
        "name": "Front fan 2",
        "geometry": {
          "Ring": {
            "leds": 8,
            "first_angle": 0,
            "turn": "Clockwise"
          }
        }
      },
      {
        "id": "c-front-fan-3",
        "name": "Front fan 3",
        "geometry": {
          "Ring": {
            "leds": 8,
            "first_angle": 0,
            "turn": "Clockwise"
          }
        }
      },
      {
        "id": "c-rear-fan",
        "name": "Rear fan",
        "geometry": {
          "Ring": {
            "leds": 8,
            "first_angle": 0,
            "turn": "Clockwise"
          }
        }
      },
      {
        "id": "c-cooler-left",
        "name": "Cooler left strip",
        "geometry": {
          "Strip": {
            "leds": 8,
            "first": "AtStart"
          }
        }
      },
      {
        "id": "c-cooler-right",
        "name": "Cooler right strip",
        "geometry": {
          "Strip": {
            "leds": 8,
            "first": "AtStart"
          }
        }
      },
      {
        "id": "c-memory-1",
        "name": "Memory module 1",
        "geometry": {
          "Strip": {
            "leds": 8,
            "first": "AtStart"
          }
        }
      },
      {
        "id": "c-memory-2",
        "name": "Memory module 2",
        "geometry": {
          "Strip": {
            "leds": 8,
            "first": "AtStart"
          }
        }
      },
      {
        "id": "c-graphics",
        "name": "Graphics card",
        "geometry": {
          "Strip": {
            "leds": 12,
            "first": "AtStart"
          }
        }
      },
      {
        "id": "c-board-edge",
        "name": "Board edge",
        "geometry": {
          "Strip": {
            "leds": 6,
            "first": "AtStart"
          }
        }
      },
      {
        "id": "c-desk-bar",
        "name": "Desk light bar",
        "geometry": {
          "Strip": {
            "leds": 10,
            "first": "AtStart"
          }
        }
      }
    ],
    "outputs": [
      {
        "id": "o-fan-hub",
        "device": "demo-motherboard",
        "zone": 1,
        "label": "Demo Motherboard",
        "zone_name": "Addressable Header 1",
        "first_led": 0,
        "branches": [
          [
            "c-front-fan-1"
          ],
          [
            "c-front-fan-2"
          ],
          [
            "c-front-fan-3"
          ],
          [
            "c-rear-fan"
          ]
        ]
      },
      {
        "id": "o-cooler",
        "device": "demo-motherboard",
        "zone": 2,
        "label": "Demo Motherboard",
        "zone_name": "Addressable Header 2",
        "first_led": 0,
        "branches": [
          [
            "c-cooler-left",
            "c-cooler-right"
          ]
        ]
      },
      {
        "id": "o-board-edge",
        "device": "demo-motherboard",
        "zone": 0,
        "label": "Demo Motherboard",
        "zone_name": "Onboard",
        "first_led": 0,
        "branches": [
          [
            "c-board-edge"
          ]
        ]
      },
      {
        "id": "o-memory-1",
        "device": "demo-memory-1",
        "zone": null,
        "label": "Demo Memory Module",
        "zone_name": null,
        "first_led": 0,
        "branches": [
          [
            "c-memory-1"
          ]
        ]
      },
      {
        "id": "o-memory-2",
        "device": "demo-memory-2",
        "zone": null,
        "label": "Demo Memory Module",
        "zone_name": null,
        "first_led": 0,
        "branches": [
          [
            "c-memory-2"
          ]
        ]
      },
      {
        "id": "o-graphics",
        "device": "demo-graphics-card",
        "zone": null,
        "label": "Demo Graphics Card",
        "zone_name": null,
        "first_led": 0,
        "branches": [
          [
            "c-graphics"
          ]
        ]
      },
      {
        "id": "o-desk-bar",
        "device": "demo-light-bar",
        "zone": null,
        "label": "Demo Light Bar",
        "zone_name": null,
        "first_led": 0,
        "branches": [
          [
            "c-desk-bar"
          ]
        ]
      }
    ],
    "views": [
      {
        "id": "v-pc",
        "name": "PC",
        "placements": [
          {
            "component": "c-front-fan-1",
            "x": 160,
            "y": 130,
            "size": 100,
            "rotation": 0,
            "flipped": false
          },
          {
            "component": "c-front-fan-2",
            "x": 160,
            "y": 300,
            "size": 100,
            "rotation": 0,
            "flipped": false
          },
          {
            "component": "c-front-fan-3",
            "x": 160,
            "y": 470,
            "size": 100,
            "rotation": 0,
            "flipped": false
          },
          {
            "component": "c-rear-fan",
            "x": 910,
            "y": 130,
            "size": 100,
            "rotation": 0,
            "flipped": false
          },
          {
            "component": "c-cooler-left",
            "x": 440,
            "y": 270,
            "size": 120,
            "rotation": 90,
            "flipped": false
          },
          {
            "component": "c-cooler-right",
            "x": 560,
            "y": 270,
            "size": 120,
            "rotation": 90,
            "flipped": false
          },
          {
            "component": "c-memory-1",
            "x": 690,
            "y": 270,
            "size": 130,
            "rotation": 90,
            "flipped": false
          },
          {
            "component": "c-memory-2",
            "x": 800,
            "y": 270,
            "size": 130,
            "rotation": 90,
            "flipped": false
          },
          {
            "component": "c-graphics",
            "x": 560,
            "y": 470,
            "size": 320,
            "rotation": 0,
            "flipped": false
          },
          {
            "component": "c-board-edge",
            "x": 530,
            "y": 610,
            "size": 200,
            "rotation": 0,
            "flipped": false
          }
        ],
        "outlines": []
      },
      {
        "id": "v-desk",
        "name": "Desk",
        "placements": [
          {
            "component": "c-desk-bar",
            "x": 500,
            "y": 120,
            "size": 520,
            "rotation": 0,
            "flipped": false
          },
          {
            "component": "c-graphics",
            "x": 820,
            "y": 470,
            "size": 160,
            "rotation": 0,
            "flipped": false
          }
        ],
        "outlines": []
      }
    ]
  },
  "outputs": [
    {
      "output": "o-fan-hub",
      "target": {
        "device": "demo-motherboard",
        "zone": 1
      },
      "label": "Demo Motherboard",
      "zone_name": "Addressable Header 1",
      "state": "Resolved",
      "problem": null
    },
    {
      "output": "o-cooler",
      "target": {
        "device": "demo-motherboard",
        "zone": 2
      },
      "label": "Demo Motherboard",
      "zone_name": "Addressable Header 2",
      "state": "Resolved",
      "problem": null
    },
    {
      "output": "o-board-edge",
      "target": {
        "device": "demo-motherboard",
        "zone": 0
      },
      "label": "Demo Motherboard",
      "zone_name": "Onboard",
      "state": "Resolved",
      "problem": null
    },
    {
      "output": "o-memory-1",
      "target": {
        "device": "demo-memory-1",
        "zone": null
      },
      "label": "Demo Memory Module",
      "zone_name": null,
      "state": "Resolved",
      "problem": null
    },
    {
      "output": "o-memory-2",
      "target": {
        "device": "demo-memory-2",
        "zone": null
      },
      "label": "Demo Memory Module",
      "zone_name": null,
      "state": "Resolved",
      "problem": null
    },
    {
      "output": "o-graphics",
      "target": {
        "device": "demo-graphics-card",
        "zone": null
      },
      "label": "Demo Graphics Card",
      "zone_name": null,
      "state": "Resolved",
      "problem": null
    },
    {
      "output": "o-desk-bar",
      "target": {
        "device": "demo-light-bar",
        "zone": null
      },
      "label": "Demo Light Bar",
      "zone_name": null,
      "state": "Resolved",
      "problem": null
    }
  ],
  "limits": {
    "components": 512,
    "outputs": 256,
    "views": 16,
    "branches": 16,
    "component_leds": 4096,
    "line_points": 256,
    "name": 60,
    "frame_min": 10,
    "frame_max": 1000,
    "outline_size": 4000
  }
};

export const setupDemoZoneChanged: SetupStoreView = {
  "kind": "Demo",
  "setup": {
    "components": [
      {
        "id": "c-front-fan-1",
        "name": "Front fan 1",
        "geometry": {
          "Ring": {
            "leds": 8,
            "first_angle": 0,
            "turn": "Clockwise"
          }
        }
      },
      {
        "id": "c-front-fan-2",
        "name": "Front fan 2",
        "geometry": {
          "Ring": {
            "leds": 8,
            "first_angle": 0,
            "turn": "Clockwise"
          }
        }
      },
      {
        "id": "c-front-fan-3",
        "name": "Front fan 3",
        "geometry": {
          "Ring": {
            "leds": 8,
            "first_angle": 0,
            "turn": "Clockwise"
          }
        }
      },
      {
        "id": "c-rear-fan",
        "name": "Rear fan",
        "geometry": {
          "Ring": {
            "leds": 8,
            "first_angle": 0,
            "turn": "Clockwise"
          }
        }
      },
      {
        "id": "c-cooler-left",
        "name": "Cooler left strip",
        "geometry": {
          "Strip": {
            "leds": 8,
            "first": "AtStart"
          }
        }
      },
      {
        "id": "c-cooler-right",
        "name": "Cooler right strip",
        "geometry": {
          "Strip": {
            "leds": 8,
            "first": "AtStart"
          }
        }
      },
      {
        "id": "c-memory-1",
        "name": "Memory module 1",
        "geometry": {
          "Strip": {
            "leds": 8,
            "first": "AtStart"
          }
        }
      },
      {
        "id": "c-memory-2",
        "name": "Memory module 2",
        "geometry": {
          "Strip": {
            "leds": 8,
            "first": "AtStart"
          }
        }
      },
      {
        "id": "c-graphics",
        "name": "Graphics card",
        "geometry": {
          "Strip": {
            "leds": 12,
            "first": "AtStart"
          }
        }
      },
      {
        "id": "c-board-edge",
        "name": "Board edge",
        "geometry": {
          "Strip": {
            "leds": 6,
            "first": "AtStart"
          }
        }
      },
      {
        "id": "c-desk-bar",
        "name": "Desk light bar",
        "geometry": {
          "Strip": {
            "leds": 10,
            "first": "AtStart"
          }
        }
      }
    ],
    "outputs": [
      {
        "id": "o-fan-hub",
        "device": "demo-motherboard",
        "zone": 1,
        "label": "Demo Motherboard",
        "zone_name": "Addressable Header 1",
        "first_led": 0,
        "branches": [
          [
            "c-front-fan-1"
          ],
          [
            "c-front-fan-2"
          ],
          [
            "c-front-fan-3"
          ],
          [
            "c-rear-fan"
          ]
        ]
      },
      {
        "id": "o-cooler",
        "device": "demo-motherboard",
        "zone": 2,
        "label": "Demo Motherboard",
        "zone_name": "Addressable Header 2",
        "first_led": 0,
        "branches": [
          [
            "c-cooler-left",
            "c-cooler-right"
          ]
        ]
      },
      {
        "id": "o-board-edge",
        "device": "demo-motherboard",
        "zone": 0,
        "label": "Demo Motherboard",
        "zone_name": "Board logo",
        "first_led": 0,
        "branches": [
          [
            "c-board-edge"
          ]
        ]
      },
      {
        "id": "o-memory-1",
        "device": "demo-memory-1",
        "zone": null,
        "label": "Demo Memory Module",
        "zone_name": null,
        "first_led": 0,
        "branches": [
          [
            "c-memory-1"
          ]
        ]
      },
      {
        "id": "o-memory-2",
        "device": "demo-memory-2",
        "zone": null,
        "label": "Demo Memory Module",
        "zone_name": null,
        "first_led": 0,
        "branches": [
          [
            "c-memory-2"
          ]
        ]
      },
      {
        "id": "o-graphics",
        "device": "demo-graphics-card",
        "zone": null,
        "label": "Demo Graphics Card",
        "zone_name": null,
        "first_led": 0,
        "branches": [
          [
            "c-graphics"
          ]
        ]
      },
      {
        "id": "o-desk-bar",
        "device": "demo-light-bar",
        "zone": null,
        "label": "Demo Light Bar",
        "zone_name": null,
        "first_led": 0,
        "branches": [
          [
            "c-desk-bar"
          ]
        ]
      }
    ],
    "views": [
      {
        "id": "v-pc",
        "name": "PC",
        "placements": [
          {
            "component": "c-front-fan-1",
            "x": 160,
            "y": 130,
            "size": 100,
            "rotation": 0,
            "flipped": false
          },
          {
            "component": "c-front-fan-2",
            "x": 160,
            "y": 300,
            "size": 100,
            "rotation": 0,
            "flipped": false
          },
          {
            "component": "c-front-fan-3",
            "x": 160,
            "y": 470,
            "size": 100,
            "rotation": 0,
            "flipped": false
          },
          {
            "component": "c-rear-fan",
            "x": 910,
            "y": 130,
            "size": 100,
            "rotation": 0,
            "flipped": false
          },
          {
            "component": "c-cooler-left",
            "x": 440,
            "y": 270,
            "size": 120,
            "rotation": 90,
            "flipped": false
          },
          {
            "component": "c-cooler-right",
            "x": 560,
            "y": 270,
            "size": 120,
            "rotation": 90,
            "flipped": false
          },
          {
            "component": "c-memory-1",
            "x": 690,
            "y": 270,
            "size": 130,
            "rotation": 90,
            "flipped": false
          },
          {
            "component": "c-memory-2",
            "x": 800,
            "y": 270,
            "size": 130,
            "rotation": 90,
            "flipped": false
          },
          {
            "component": "c-graphics",
            "x": 560,
            "y": 470,
            "size": 320,
            "rotation": 0,
            "flipped": false
          },
          {
            "component": "c-board-edge",
            "x": 530,
            "y": 610,
            "size": 200,
            "rotation": 0,
            "flipped": false
          },
          {
            "component": "c-desk-bar",
            "x": 860,
            "y": 600,
            "size": 200,
            "rotation": 0,
            "flipped": false
          }
        ],
        "outlines": []
      },
      {
        "id": "v-desk",
        "name": "Desk",
        "placements": [
          {
            "component": "c-desk-bar",
            "x": 500,
            "y": 120,
            "size": 520,
            "rotation": 0,
            "flipped": false
          },
          {
            "component": "c-graphics",
            "x": 820,
            "y": 470,
            "size": 160,
            "rotation": 0,
            "flipped": false
          }
        ],
        "outlines": []
      }
    ]
  },
  "outputs": [
    {
      "output": "o-fan-hub",
      "target": {
        "device": "demo-motherboard",
        "zone": 1
      },
      "label": "Demo Motherboard",
      "zone_name": "Addressable Header 1",
      "state": "Resolved",
      "problem": null
    },
    {
      "output": "o-cooler",
      "target": {
        "device": "demo-motherboard",
        "zone": 2
      },
      "label": "Demo Motherboard",
      "zone_name": "Addressable Header 2",
      "state": "Resolved",
      "problem": null
    },
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
    },
    {
      "output": "o-memory-1",
      "target": {
        "device": "demo-memory-1",
        "zone": null
      },
      "label": "Demo Memory Module",
      "zone_name": null,
      "state": "Resolved",
      "problem": null
    },
    {
      "output": "o-memory-2",
      "target": {
        "device": "demo-memory-2",
        "zone": null
      },
      "label": "Demo Memory Module",
      "zone_name": null,
      "state": "Resolved",
      "problem": null
    },
    {
      "output": "o-graphics",
      "target": {
        "device": "demo-graphics-card",
        "zone": null
      },
      "label": "Demo Graphics Card",
      "zone_name": null,
      "state": "Resolved",
      "problem": null
    },
    {
      "output": "o-desk-bar",
      "target": {
        "device": "demo-light-bar",
        "zone": null
      },
      "label": "Demo Light Bar",
      "zone_name": null,
      "state": "Resolved",
      "problem": null
    }
  ],
  "limits": {
    "components": 512,
    "outputs": 256,
    "views": 16,
    "branches": 16,
    "component_leds": 4096,
    "line_points": 256,
    "name": 60,
    "frame_min": 10,
    "frame_max": 1000,
    "outline_size": 4000
  }
};

export const setupRealEmpty: SetupStoreView = {
  "kind": "Real",
  "setup": {
    "components": [],
    "outputs": [],
    "views": []
  },
  "outputs": [],
  "limits": {
    "components": 512,
    "outputs": 256,
    "views": 16,
    "branches": 16,
    "component_leds": 4096,
    "line_points": 256,
    "name": 60,
    "frame_min": 10,
    "frame_max": 1000,
    "outline_size": 4000
  }
};

export const setupRealTower: SetupStoreView = {
  "kind": "Real",
  "setup": {
    "components": [
      {
        "id": "c-front-fan-1",
        "name": "Front fan 1",
        "geometry": {
          "Ring": {
            "leds": 8,
            "first_angle": 0,
            "turn": "Clockwise"
          }
        }
      },
      {
        "id": "c-front-fan-2",
        "name": "Front fan 2",
        "geometry": {
          "Ring": {
            "leds": 8,
            "first_angle": 0,
            "turn": "Clockwise"
          }
        }
      },
      {
        "id": "c-front-fan-3",
        "name": "Front fan 3",
        "geometry": {
          "Ring": {
            "leds": 8,
            "first_angle": 0,
            "turn": "Clockwise"
          }
        }
      },
      {
        "id": "c-rear-fan",
        "name": "Rear fan",
        "geometry": {
          "Ring": {
            "leds": 8,
            "first_angle": 0,
            "turn": "Clockwise"
          }
        }
      },
      {
        "id": "c-cooler-left",
        "name": "Cooler left strip",
        "geometry": {
          "Strip": {
            "leds": 8,
            "first": "AtStart"
          }
        }
      },
      {
        "id": "c-cooler-right",
        "name": "Cooler right strip",
        "geometry": {
          "Strip": {
            "leds": 8,
            "first": "AtStart"
          }
        }
      },
      {
        "id": "c-graphics",
        "name": "Graphics card",
        "geometry": {
          "Strip": {
            "leds": 12,
            "first": "AtStart"
          }
        }
      },
      {
        "id": "c-keyboard",
        "name": "Keyboard",
        "geometry": {
          "Matrix": {
            "columns": 12,
            "rows": 4,
            "first": "TopLeft",
            "runs": "AlongRows",
            "serpentine": false
          }
        }
      },
      {
        "id": "c-desk-edge",
        "name": "Desk edge strip",
        "geometry": {
          "Strip": {
            "leds": 12,
            "first": "AtStart"
          }
        }
      }
    ],
    "outputs": [
      {
        "id": "o-fan-hub",
        "device": "dev-6f970707d354ee85",
        "zone": 0,
        "label": "Fixture Motherboard",
        "zone_name": "ARGB Header 1",
        "first_led": 0,
        "branches": [
          [
            "c-front-fan-1"
          ],
          [
            "c-front-fan-2"
          ],
          [
            "c-front-fan-3"
          ],
          [
            "c-rear-fan"
          ]
        ]
      },
      {
        "id": "o-cooler",
        "device": "dev-6f970707d354ee85",
        "zone": 1,
        "label": "Fixture Motherboard",
        "zone_name": "ARGB Header 2",
        "first_led": 0,
        "branches": [
          [
            "c-cooler-left",
            "c-cooler-right"
          ]
        ]
      },
      {
        "id": "o-graphics",
        "device": "dev-420d41cc43011cb2",
        "zone": null,
        "label": "Fixture Graphics Card",
        "zone_name": null,
        "first_led": 0,
        "branches": [
          [
            "c-graphics"
          ]
        ]
      },
      {
        "id": "o-keyboard",
        "device": "dev-8760c9a322bc0ba5",
        "zone": null,
        "label": "Fixture Keyboard",
        "zone_name": null,
        "first_led": 0,
        "branches": [
          [
            "c-keyboard"
          ]
        ]
      },
      {
        "id": "o-desk-edge",
        "device": "dev-2bb511c8407d4ad7",
        "zone": null,
        "label": "Fixture Desk Strip",
        "zone_name": null,
        "first_led": 0,
        "branches": [
          [
            "c-desk-edge"
          ]
        ]
      }
    ],
    "views": [
      {
        "id": "v-pc",
        "name": "PC",
        "placements": [
          {
            "component": "c-front-fan-1",
            "x": 160,
            "y": 130,
            "size": 100,
            "rotation": 0,
            "flipped": false
          },
          {
            "component": "c-front-fan-2",
            "x": 160,
            "y": 300,
            "size": 100,
            "rotation": 0,
            "flipped": false
          },
          {
            "component": "c-front-fan-3",
            "x": 160,
            "y": 470,
            "size": 100,
            "rotation": 0,
            "flipped": false
          },
          {
            "component": "c-rear-fan",
            "x": 910,
            "y": 130,
            "size": 100,
            "rotation": 0,
            "flipped": false
          },
          {
            "component": "c-cooler-left",
            "x": 440,
            "y": 270,
            "size": 120,
            "rotation": 90,
            "flipped": false
          },
          {
            "component": "c-cooler-right",
            "x": 560,
            "y": 270,
            "size": 120,
            "rotation": 90,
            "flipped": false
          },
          {
            "component": "c-graphics",
            "x": 560,
            "y": 470,
            "size": 320,
            "rotation": 0,
            "flipped": false
          }
        ],
        "outlines": []
      },
      {
        "id": "v-desk",
        "name": "Desk",
        "placements": [
          {
            "component": "c-keyboard",
            "x": 430,
            "y": 470,
            "size": 360,
            "rotation": 0,
            "flipped": false
          },
          {
            "component": "c-desk-edge",
            "x": 500,
            "y": 640,
            "size": 760,
            "rotation": 0,
            "flipped": false
          },
          {
            "component": "c-graphics",
            "x": 840,
            "y": 300,
            "size": 160,
            "rotation": 90,
            "flipped": false
          }
        ],
        "outlines": []
      }
    ]
  },
  "outputs": [
    {
      "output": "o-fan-hub",
      "target": {
        "device": "dev-6f970707d354ee85",
        "zone": 0
      },
      "label": "Fixture Motherboard",
      "zone_name": "ARGB Header 1",
      "state": "Resolved",
      "problem": null
    },
    {
      "output": "o-cooler",
      "target": {
        "device": "dev-6f970707d354ee85",
        "zone": 1
      },
      "label": "Fixture Motherboard",
      "zone_name": "ARGB Header 2",
      "state": "Resolved",
      "problem": null
    },
    {
      "output": "o-graphics",
      "target": {
        "device": "dev-420d41cc43011cb2",
        "zone": null
      },
      "label": "Fixture Graphics Card",
      "zone_name": null,
      "state": "Resolved",
      "problem": null
    },
    {
      "output": "o-keyboard",
      "target": {
        "device": "dev-8760c9a322bc0ba5",
        "zone": null
      },
      "label": "Fixture Keyboard",
      "zone_name": null,
      "state": "Resolved",
      "problem": null
    },
    {
      "output": "o-desk-edge",
      "target": {
        "device": "dev-2bb511c8407d4ad7",
        "zone": null
      },
      "label": "Fixture Desk Strip",
      "zone_name": null,
      "state": "Resolved",
      "problem": null
    }
  ],
  "limits": {
    "components": 512,
    "outputs": 256,
    "views": 16,
    "branches": 16,
    "component_leds": 4096,
    "line_points": 256,
    "name": 60,
    "frame_min": 10,
    "frame_max": 1000,
    "outline_size": 4000
  }
};
