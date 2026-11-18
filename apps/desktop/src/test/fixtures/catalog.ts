// Generated from real sessions by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.
import type { EffectCatalog } from "../../lib/api/bindings/EffectCatalog";

export const catalog: EffectCatalog = {
  "effects": [
    {
      "id": "solid",
      "label": "Solid colour",
      "description": "Every LED shows one colour.",
      "controls": [
        {
          "id": "color",
          "label": "Colour",
          "description": "The colour of every LED.",
          "kind": {
            "Color": {
              "default": null
            }
          }
        },
        {
          "id": "brightness",
          "label": "Brightness",
          "description": "Percent of the effect's colours. 0 is off.",
          "kind": {
            "Brightness": {
              "range": {
                "min": 0,
                "max": 100
              },
              "default": 100
            }
          }
        }
      ]
    },
    {
      "id": "breathing",
      "label": "Breathing",
      "description": "One colour fades out and back in.",
      "controls": [
        {
          "id": "color",
          "label": "Colour",
          "description": "The colour at full strength.",
          "kind": {
            "Color": {
              "default": {
                "r": 255,
                "g": 96,
                "b": 0
              }
            }
          }
        },
        {
          "id": "speed",
          "label": "Speed",
          "description": "Breaths per minute. 0 holds the full colour.",
          "kind": {
            "Speed": {
              "range": {
                "min": 0,
                "max": 120
              },
              "default": 15,
              "unit": "CyclesPerMinute"
            }
          }
        },
        {
          "id": "brightness",
          "label": "Brightness",
          "description": "Percent of the effect's colours. 0 is off.",
          "kind": {
            "Brightness": {
              "range": {
                "min": 0,
                "max": 100
              },
              "default": 100
            }
          }
        }
      ]
    },
    {
      "id": "gradient",
      "label": "Gradient",
      "description": "Colours blend from the first LED to the last.",
      "controls": [
        {
          "id": "palette",
          "label": "Colours",
          "description": "The first colour is at the start, the last at the end.",
          "kind": {
            "Palette": {
              "min_colors": 2,
              "max_colors": 8,
              "default": [
                {
                  "r": 255,
                  "g": 0,
                  "b": 64
                },
                {
                  "r": 0,
                  "g": 96,
                  "b": 255
                }
              ]
            }
          }
        },
        {
          "id": "direction",
          "label": "Direction",
          "description": "Forward starts at the first LED.",
          "kind": {
            "Direction": {
              "default": "Forward"
            }
          }
        },
        {
          "id": "brightness",
          "label": "Brightness",
          "description": "Percent of the effect's colours. 0 is off.",
          "kind": {
            "Brightness": {
              "range": {
                "min": 0,
                "max": 100
              },
              "default": 100
            }
          }
        }
      ]
    },
    {
      "id": "rainbow-wave",
      "label": "Rainbow wave",
      "description": "The colour wheel travels along the LEDs.",
      "controls": [
        {
          "id": "speed",
          "label": "Speed",
          "description": "How often each LED passes through every colour per minute. 0 stops the wave.",
          "kind": {
            "Speed": {
              "range": {
                "min": 0,
                "max": 240
              },
              "default": 20,
              "unit": "CyclesPerMinute"
            }
          }
        },
        {
          "id": "direction",
          "label": "Direction",
          "description": "Forward moves from the first LED towards the last.",
          "kind": {
            "Direction": {
              "default": "Forward"
            }
          }
        },
        {
          "id": "waves",
          "label": "Rainbows",
          "description": "How many full rainbows fit along the LEDs.",
          "kind": {
            "Number": {
              "range": {
                "min": 1,
                "max": 8
              },
              "step": 1,
              "default": 1,
              "unit": "Count"
            }
          }
        },
        {
          "id": "brightness",
          "label": "Brightness",
          "description": "Percent of the effect's colours. 0 is off.",
          "kind": {
            "Brightness": {
              "range": {
                "min": 0,
                "max": 100
              },
              "default": 100
            }
          }
        }
      ]
    },
    {
      "id": "color-cycle",
      "label": "Colour cycle",
      "description": "All LEDs blend through the colours in turn, then start again.",
      "controls": [
        {
          "id": "palette",
          "label": "Colours",
          "description": "The colours in the order they appear.",
          "kind": {
            "Palette": {
              "min_colors": 2,
              "max_colors": 8,
              "default": [
                {
                  "r": 255,
                  "g": 0,
                  "b": 0
                },
                {
                  "r": 0,
                  "g": 255,
                  "b": 0
                },
                {
                  "r": 0,
                  "g": 0,
                  "b": 255
                }
              ]
            }
          }
        },
        {
          "id": "speed",
          "label": "Speed",
          "description": "Full rounds through all colours per minute. 0 holds the first colour.",
          "kind": {
            "Speed": {
              "range": {
                "min": 0,
                "max": 120
              },
              "default": 6,
              "unit": "CyclesPerMinute"
            }
          }
        },
        {
          "id": "brightness",
          "label": "Brightness",
          "description": "Percent of the effect's colours. 0 is off.",
          "kind": {
            "Brightness": {
              "range": {
                "min": 0,
                "max": 100
              },
              "default": 100
            }
          }
        }
      ]
    },
    {
      "id": "moving-highlight",
      "label": "Moving highlight",
      "description": "A bright band runs along the LEDs over a background colour.",
      "controls": [
        {
          "id": "color",
          "label": "Highlight",
          "description": "The colour at the centre of the band.",
          "kind": {
            "Color": {
              "default": {
                "r": 255,
                "g": 255,
                "b": 255
              }
            }
          }
        },
        {
          "id": "background",
          "label": "Background",
          "description": "The colour outside the band.",
          "kind": {
            "Color": {
              "default": {
                "r": 0,
                "g": 0,
                "b": 0
              }
            }
          }
        },
        {
          "id": "speed",
          "label": "Speed",
          "description": "Rounds along the LEDs per minute. 0 holds the band still on the first LED.",
          "kind": {
            "Speed": {
              "range": {
                "min": 0,
                "max": 240
              },
              "default": 30,
              "unit": "CyclesPerMinute"
            }
          }
        },
        {
          "id": "direction",
          "label": "Direction",
          "description": "Forward runs from the first LED towards the last.",
          "kind": {
            "Direction": {
              "default": "Forward"
            }
          }
        },
        {
          "id": "width",
          "label": "Width",
          "description": "Width of the band as a percent of the LEDs.",
          "kind": {
            "Number": {
              "range": {
                "min": 5,
                "max": 100
              },
              "step": 5,
              "default": 20,
              "unit": "Percent"
            }
          }
        },
        {
          "id": "brightness",
          "label": "Brightness",
          "description": "Percent of the effect's colours. 0 is off.",
          "kind": {
            "Brightness": {
              "range": {
                "min": 0,
                "max": 100
              },
              "default": 100
            }
          }
        }
      ]
    }
  ],
  "frame_rate": {
    "min": 1,
    "max": 60
  },
  "preview": {
    "leds": {
      "min": 1,
      "max": 60
    },
    "frames": {
      "min": 1,
      "max": 90
    },
    "interval_ms": {
      "min": 20,
      "max": 1000
    }
  }
};
