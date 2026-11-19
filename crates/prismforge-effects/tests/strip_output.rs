mod support;

use std::time::Duration;

use prismforge_effects::{ControlValue, Direction, Effect, Rgb};

use support::{BLUE, GREEN, RED, WHITE, color, effect, number, palette};

const LED_COUNTS: [usize; 6] = [1, 2, 3, 7, 36, 120];
const TIMES_MS: [u64; 6] = [0, 33, 250, 1_000, 7_919, 3_600_000];

fn effects() -> Vec<Effect> {
    let backward = ("direction", ControlValue::Direction(Direction::Backward));
    vec![
        effect("solid", &[("color", color(Rgb::new(200, 100, 50)))]),
        effect(
            "breathing",
            &[
                ("color", color(Rgb::new(255, 120, 0))),
                ("speed", number(37)),
            ],
        ),
        effect("gradient", &[("palette", palette(&[RED, GREEN, BLUE]))]),
        effect(
            "gradient",
            &[("palette", palette(&[RED, WHITE])), backward.clone()],
        ),
        effect(
            "rainbow-wave",
            &[("speed", number(45)), ("waves", number(3))],
        ),
        effect("rainbow-wave", &[("speed", number(200)), backward.clone()]),
        effect(
            "color-cycle",
            &[
                ("palette", palette(&[RED, GREEN, BLUE, WHITE])),
                ("speed", number(90)),
            ],
        ),
        effect(
            "moving-highlight",
            &[("speed", number(77)), ("width", number(35))],
        ),
        effect(
            "moving-highlight",
            &[
                ("speed", number(240)),
                ("width", number(5)),
                ("brightness", number(61)),
                backward,
            ],
        ),
    ]
}

fn fnv(hash: u64, byte: u8) -> u64 {
    (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
}

#[test]
fn each_device_output_is_unchanged() {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    for effect in effects() {
        for leds in LED_COUNTS {
            for millis in TIMES_MS {
                for led in effect.render_strip(Duration::from_millis(millis), leds) {
                    hash = [led.r, led.g, led.b].into_iter().fold(hash, fnv);
                }
            }
        }
    }

    assert_eq!(hash, 6_624_374_272_489_201_060);
}
