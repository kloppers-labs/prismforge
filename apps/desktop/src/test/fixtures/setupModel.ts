// Generated from the Rust setup model by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.
import type { ComponentId } from "../../lib/api/bindings/ComponentId";
import type { Geometry } from "../../lib/api/bindings/Geometry";
import type { OutputTarget } from "../../lib/api/bindings/OutputTarget";
import type { Placement } from "../../lib/api/bindings/Placement";
import type { Setup } from "../../lib/api/bindings/Setup";

export const ledSamples: { geometry: Geometry; placement: Placement; leds: { x: number; y: number }[] }[] = [
  {
    "geometry": {
      "Strip": {
        "leds": 5,
        "first": "AtStart"
      }
    },
    "placement": {
      "component": "c-sample",
      "x": 200,
      "y": 150,
      "size": 100,
      "rotation": 0,
      "flipped": false
    },
    "leds": [
      {
        "x": 160.0,
        "y": 150.0
      },
      {
        "x": 180.0,
        "y": 150.0
      },
      {
        "x": 200.0,
        "y": 150.0
      },
      {
        "x": 220.0,
        "y": 150.0
      },
      {
        "x": 240.0,
        "y": 150.0
      }
    ]
  },
  {
    "geometry": {
      "Strip": {
        "leds": 5,
        "first": "AtEnd"
      }
    },
    "placement": {
      "component": "c-sample",
      "x": 200,
      "y": 150,
      "size": 100,
      "rotation": 90,
      "flipped": true
    },
    "leds": [
      {
        "x": 200.0,
        "y": 110.0
      },
      {
        "x": 200.0,
        "y": 130.0
      },
      {
        "x": 200.0,
        "y": 150.0
      },
      {
        "x": 200.0,
        "y": 170.0
      },
      {
        "x": 200.0,
        "y": 190.0
      }
    ]
  },
  {
    "geometry": {
      "Ring": {
        "leds": 8,
        "first_angle": 45,
        "turn": "CounterClockwise"
      }
    },
    "placement": {
      "component": "c-sample",
      "x": 200,
      "y": 150,
      "size": 100,
      "rotation": 30,
      "flipped": false
    },
    "leds": [
      {
        "x": 248.2962913144534,
        "y": 137.05904774487396
      },
      {
        "x": 225.0,
        "y": 106.69872981077806
      },
      {
        "x": 187.05904774487396,
        "y": 101.70370868554659
      },
      {
        "x": 156.69872981077805,
        "y": 125.0
      },
      {
        "x": 151.7037086855466,
        "y": 162.94095225512604
      },
      {
        "x": 175.0,
        "y": 193.30127018922195
      },
      {
        "x": 212.94095225512604,
        "y": 198.2962913144534
      },
      {
        "x": 243.30127018922195,
        "y": 175.0
      }
    ]
  },
  {
    "geometry": {
      "Matrix": {
        "columns": 4,
        "rows": 3,
        "first": "BottomRight",
        "runs": "AlongColumns",
        "serpentine": true
      }
    },
    "placement": {
      "component": "c-sample",
      "x": 200,
      "y": 150,
      "size": 100,
      "rotation": 15,
      "flipped": true
    },
    "leds": [
      {
        "x": 157.3073053865969,
        "y": 164.44243146588218
      },
      {
        "x": 163.77778151415993,
        "y": 140.29428580865547
      },
      {
        "x": 170.24825764172294,
        "y": 116.14614015142877
      },
      {
        "x": 194.39640329894968,
        "y": 122.61661627899178
      },
      {
        "x": 187.92592717138666,
        "y": 146.7647619362185
      },
      {
        "x": 181.45545104382364,
        "y": 170.9129075934452
      },
      {
        "x": 205.60359670105032,
        "y": 177.38338372100822
      },
      {
        "x": 212.07407282861334,
        "y": 153.2352380637815
      },
      {
        "x": 218.54454895617636,
        "y": 129.0870924065548
      },
      {
        "x": 242.6926946134031,
        "y": 135.55756853411782
      },
      {
        "x": 236.22221848584007,
        "y": 159.70571419134453
      },
      {
        "x": 229.75174235827706,
        "y": 183.85385984857123
      }
    ]
  },
  {
    "geometry": {
      "Matrix": {
        "columns": 5,
        "rows": 1,
        "first": "TopLeft",
        "runs": "AlongRows",
        "serpentine": false
      }
    },
    "placement": {
      "component": "c-sample",
      "x": 200,
      "y": 150,
      "size": 100,
      "rotation": 0,
      "flipped": false
    },
    "leds": [
      {
        "x": 160.0,
        "y": 150.0
      },
      {
        "x": 180.0,
        "y": 150.0
      },
      {
        "x": 200.0,
        "y": 150.0
      },
      {
        "x": 220.0,
        "y": 150.0
      },
      {
        "x": 240.0,
        "y": 150.0
      }
    ]
  },
  {
    "geometry": {
      "Perimeter": {
        "top": 4,
        "right": 2,
        "bottom": 4,
        "left": 2,
        "width": 16,
        "height": 9,
        "first": "TopRight",
        "turn": "CounterClockwise"
      }
    },
    "placement": {
      "component": "c-sample",
      "x": 200,
      "y": 150,
      "size": 100,
      "rotation": 345,
      "flipped": false
    },
    "leds": [
      {
        "x": 228.9429328423317,
        "y": 113.12762194427543
      },
      {
        "x": 204.79478718510495,
        "y": 119.59809807183845
      },
      {
        "x": 180.64664152787827,
        "y": 126.06857419940147
      },
      {
        "x": 156.49849587065154,
        "y": 132.53905032696449
      },
      {
        "x": 148.0640658637924,
        "y": 149.35762032293601
      },
      {
        "x": 155.34335150730078,
        "y": 176.52428418731606
      },
      {
        "x": 171.0570671576683,
        "y": 186.87237805572457
      },
      {
        "x": 195.20521281489505,
        "y": 180.40190192816155
      },
      {
        "x": 219.35335847212173,
        "y": 173.93142580059853
      },
      {
        "x": 243.50150412934846,
        "y": 167.46094967303551
      },
      {
        "x": 251.9359341362076,
        "y": 150.64237967706399
      },
      {
        "x": 244.65664849269922,
        "y": 123.47571581268394
      }
    ]
  },
  {
    "geometry": {
      "Point": {}
    },
    "placement": {
      "component": "c-sample",
      "x": 200,
      "y": 150,
      "size": 100,
      "rotation": 0,
      "flipped": false
    },
    "leds": [
      {
        "x": 200.0,
        "y": 150.0
      }
    ]
  },
  {
    "geometry": {
      "Line": {
        "leds": 6,
        "points": [
          {
            "x": 0,
            "y": 0
          },
          {
            "x": 10,
            "y": 0
          },
          {
            "x": 10,
            "y": 5
          }
        ],
        "first": "AtEnd"
      }
    },
    "placement": {
      "component": "c-sample",
      "x": 200,
      "y": 150,
      "size": 100,
      "rotation": 180,
      "flipped": false
    },
    "leds": [
      {
        "x": 150.0,
        "y": 137.5
      },
      {
        "x": 150.0,
        "y": 162.5
      },
      {
        "x": 162.5,
        "y": 175.0
      },
      {
        "x": 187.5,
        "y": 175.0
      },
      {
        "x": 212.5,
        "y": 175.0
      },
      {
        "x": 237.5,
        "y": 175.0
      }
    ]
  },
  {
    "geometry": {
      "Ring": {
        "leds": 8,
        "first_angle": 400,
        "turn": "Clockwise"
      }
    },
    "placement": {
      "component": "c-sample",
      "x": 200,
      "y": 150,
      "size": 100,
      "rotation": 0,
      "flipped": false
    },
    "leds": []
  }
];

export const demoTower: Setup = {
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
      ]
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
      ]
    }
  ]
};

export const demoTowerParts: { component: ComponentId; branch: number; start: number; end: number }[] = [
  {
    "component": "c-front-fan-1",
    "branch": 0,
    "start": 0,
    "end": 8
  },
  {
    "component": "c-front-fan-2",
    "branch": 1,
    "start": 0,
    "end": 8
  },
  {
    "component": "c-front-fan-3",
    "branch": 2,
    "start": 0,
    "end": 8
  },
  {
    "component": "c-rear-fan",
    "branch": 3,
    "start": 0,
    "end": 8
  },
  {
    "component": "c-cooler-left",
    "branch": 0,
    "start": 0,
    "end": 8
  },
  {
    "component": "c-cooler-right",
    "branch": 0,
    "start": 8,
    "end": 16
  },
  {
    "component": "c-board-edge",
    "branch": 0,
    "start": 0,
    "end": 6
  },
  {
    "component": "c-memory-1",
    "branch": 0,
    "start": 0,
    "end": 8
  },
  {
    "component": "c-memory-2",
    "branch": 0,
    "start": 0,
    "end": 8
  },
  {
    "component": "c-graphics",
    "branch": 0,
    "start": 0,
    "end": 12
  },
  {
    "component": "c-desk-bar",
    "branch": 0,
    "start": 0,
    "end": 10
  }
];

export const demoTowerShared: { component: ComponentId; shared_with: ComponentId[] }[] = [
  {
    "component": "c-front-fan-1",
    "shared_with": [
      "c-front-fan-2",
      "c-front-fan-3",
      "c-rear-fan"
    ]
  },
  {
    "component": "c-front-fan-2",
    "shared_with": [
      "c-front-fan-1",
      "c-front-fan-3",
      "c-rear-fan"
    ]
  },
  {
    "component": "c-front-fan-3",
    "shared_with": [
      "c-front-fan-1",
      "c-front-fan-2",
      "c-rear-fan"
    ]
  },
  {
    "component": "c-rear-fan",
    "shared_with": [
      "c-front-fan-1",
      "c-front-fan-2",
      "c-front-fan-3"
    ]
  },
  {
    "component": "c-cooler-left",
    "shared_with": []
  },
  {
    "component": "c-cooler-right",
    "shared_with": []
  },
  {
    "component": "c-memory-1",
    "shared_with": []
  },
  {
    "component": "c-memory-2",
    "shared_with": []
  },
  {
    "component": "c-graphics",
    "shared_with": []
  },
  {
    "component": "c-board-edge",
    "shared_with": []
  },
  {
    "component": "c-desk-bar",
    "shared_with": []
  }
];

export const demoTowerPlanned: { target: OutputTarget; leds: number }[] = [
  {
    "target": {
      "device": "demo-motherboard",
      "zone": 1
    },
    "leds": 8
  },
  {
    "target": {
      "device": "demo-motherboard",
      "zone": 2
    },
    "leds": 16
  },
  {
    "target": {
      "device": "demo-motherboard",
      "zone": 0
    },
    "leds": 6
  },
  {
    "target": {
      "device": "demo-memory-1",
      "zone": null
    },
    "leds": 8
  },
  {
    "target": {
      "device": "demo-memory-2",
      "zone": null
    },
    "leds": 8
  },
  {
    "target": {
      "device": "demo-graphics-card",
      "zone": null
    },
    "leds": 12
  }
];
