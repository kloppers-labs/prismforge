use prismforge_core::Rgb;
use prismforge_openrgb::protocol::{
    ControllerDataError, FrameError, MAX_PACKET_SIZE, PacketHeader, StringProblem,
    decode_controller_ids, encode_update_leds, encode_update_zone_leds, parse_controller_data,
};
use prismforge_testkit::{FixtureDevice, FixtureMode};

use crate::controller_data::{
    name_length_beyond_packet, truncated_block, unterminated_name, with_declared_size,
};
use crate::support::strip;

fn keyboard() -> FixtureDevice {
    FixtureDevice::new("kbd", "Keyboard K1")
        .serial("SN-K1")
        .location("hid:1")
        .matrix_zone("Keys", 2, 3)
        .resizable_zone("Header", 0, 60, 4)
        .modes(vec![
            FixtureMode::direct(),
            FixtureMode::static_color(),
            FixtureMode::breathing(),
        ])
}

#[test]
fn parses_fixture_controller_block() {
    let data = parse_controller_data(&keyboard().encode()).expect("valid block");

    assert_eq!(data.name, "Keyboard K1");
    assert_eq!(data.vendor, "Fixture Vendor");
    assert_eq!(data.serial, "SN-K1");
    assert_eq!(data.location, "hid:1");
    assert_eq!(data.active_mode, 0);
    let mode_names: Vec<&str> = data.modes.iter().map(|mode| mode.name.as_str()).collect();
    assert_eq!(mode_names, ["Direct", "Static", "Breathing"]);
    assert_eq!(data.modes[2].speed_max, 5);
    assert_eq!(data.modes[1].colors, [Rgb::new(255, 0, 0)]);
    assert_eq!(data.zones.len(), 2);
    let matrix = data.zones[0].matrix.as_ref().expect("matrix map");
    assert_eq!((matrix.height, matrix.width), (2, 3));
    assert_eq!(matrix.cells, [0, 1, 2, 3, 4, 5]);
    assert_eq!(data.zones[0].active_mode, -1);
    assert_eq!(data.zones[1].leds_max, 60);
    assert_eq!(data.led_names.len(), 10);
    assert_eq!(data.colors.len(), 10);
    assert_eq!(data.display_name, "Keyboard K1");
}

#[test]
fn every_truncation_is_rejected_without_panicking() {
    let full = keyboard().encode();

    for len in 4..full.len() {
        let result = parse_controller_data(&with_declared_size(full[..len].to_vec()));
        assert!(result.is_err(), "prefix of {len} bytes was accepted");
    }
}

#[test]
fn rejects_declared_size_that_disagrees_with_packet() {
    let mut bytes = keyboard().encode();
    bytes.push(0);

    assert_eq!(
        parse_controller_data(&bytes),
        Err(ControllerDataError::SizeMismatch {
            declared: bytes.len() as u32 - 1,
            actual: bytes.len()
        })
    );
}

#[test]
fn rejects_lengths_beyond_packet() {
    assert_eq!(
        parse_controller_data(&name_length_beyond_packet()),
        Err(ControllerDataError::Truncated { field: "name" })
    );
    assert!(matches!(
        parse_controller_data(&truncated_block()),
        Err(ControllerDataError::Truncated { .. })
    ));
}

#[test]
fn rejects_huge_counts_without_matching_bytes() {
    let mut bytes = strip("tiny", 1).encode();
    let tail = 2 + 4 + (2 + "Strip tiny".len() + 1) + (4 + 1);
    let colors_count_at = bytes.len() - tail - 4 - 2;
    bytes[colors_count_at..colors_count_at + 2].copy_from_slice(&u16::MAX.to_le_bytes());

    assert_eq!(
        parse_controller_data(&bytes),
        Err(ControllerDataError::Truncated { field: "colors" })
    );
}

#[test]
fn rejects_invalid_strings() {
    assert_eq!(
        parse_controller_data(&unterminated_name()),
        Err(ControllerDataError::InvalidString {
            field: "name",
            problem: StringProblem::Unterminated
        })
    );

    let mut bytes = strip("utf", 1).encode();
    bytes[10] = 0xFF;
    assert_eq!(
        parse_controller_data(&bytes),
        Err(ControllerDataError::InvalidString {
            field: "name",
            problem: StringProblem::InvalidUtf8
        })
    );

    let mut bytes = strip("nul", 1).encode();
    bytes[10] = 0;
    assert_eq!(
        parse_controller_data(&bytes),
        Err(ControllerDataError::InvalidString {
            field: "name",
            problem: StringProblem::InteriorNul
        })
    );
}

#[test]
fn rejects_matrix_length_that_disagrees_with_dimensions() {
    let mut device = keyboard();
    if let Some((_, _, cells)) = device.zones[0].matrix.as_mut() {
        cells.pop();
    }

    assert_eq!(
        parse_controller_data(&device.encode()),
        Err(ControllerDataError::MatrixSizeMismatch {
            declared: 28,
            height: 2,
            width: 3
        })
    );
}

#[test]
fn rejects_inconsistent_controllers() {
    let mut uneven = strip("uneven", 4);
    uneven.led_count_override = Some(3);
    assert_eq!(
        parse_controller_data(&uneven.encode()),
        Err(ControllerDataError::ZoneLedTotalMismatch {
            zone_leds: 4,
            leds: 3
        })
    );

    let inverted = strip("inverted", 0).resizable_zone("Header", 10, 5, 7);
    assert_eq!(
        parse_controller_data(&inverted.encode()),
        Err(ControllerDataError::ZoneBoundsInverted {
            zone: 1,
            min: 10,
            max: 5
        })
    );

    let mut bad_mode = strip("bad-mode", 4);
    bad_mode.active_mode = 2;
    assert_eq!(
        parse_controller_data(&bad_mode.encode()),
        Err(ControllerDataError::ActiveModeOutOfRange {
            active: 2,
            modes: 2
        })
    );
}

#[test]
fn header_round_trips_and_rejects_bad_frames() {
    let header = PacketHeader {
        device_id: 7,
        packet_id: 1050,
        size: 22,
    };
    let bytes = header.encode();
    assert_eq!(&bytes[..4], b"ORGB");
    assert_eq!(PacketHeader::decode(&bytes, MAX_PACKET_SIZE), Ok(header));

    let mut bad_magic = bytes;
    bad_magic[0] = b'X';
    assert_eq!(
        PacketHeader::decode(&bad_magic, MAX_PACKET_SIZE),
        Err(FrameError::BadMagic(*b"XRGB"))
    );

    let oversized = PacketHeader {
        size: MAX_PACKET_SIZE + 1,
        ..header
    };
    assert_eq!(
        PacketHeader::decode(&oversized.encode(), MAX_PACKET_SIZE),
        Err(FrameError::TooLarge {
            size: MAX_PACKET_SIZE + 1,
            limit: MAX_PACKET_SIZE
        })
    );
}

#[test]
fn encodes_colour_payloads_with_inclusive_size() {
    let colors = [Rgb::new(1, 2, 3), Rgb::new(4, 5, 6)];

    assert_eq!(
        encode_update_leds(&colors).unwrap(),
        [14, 0, 0, 0, 2, 0, 1, 2, 3, 0, 4, 5, 6, 0]
    );
    assert_eq!(
        encode_update_zone_leds(3, &colors).unwrap(),
        [18, 0, 0, 0, 3, 0, 0, 0, 2, 0, 1, 2, 3, 0, 4, 5, 6, 0]
    );
}

#[test]
fn controller_id_list_must_match_its_count() {
    assert_eq!(
        decode_controller_ids(&[2, 0, 0, 0, 5, 0, 0, 0, 9, 0, 0, 0]),
        Ok(vec![5, 9])
    );
    assert!(decode_controller_ids(&[3, 0, 0, 0, 5, 0, 0, 0]).is_err());
    assert!(decode_controller_ids(&[255, 255, 255, 255]).is_err());
}
