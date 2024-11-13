use std::io::Cursor;

use cz::{common::CzVersion, DynamicCz};

const KODIM03: (u16, u16, &[u8]) = (128, 128, include_bytes!("test_images/kodim03.rgba"));
const KODIM23: (u16, u16, &[u8]) = (225, 225, include_bytes!("test_images/kodim23.rgba"));
const SQPTEXT: (u16, u16, &[u8]) = (2048, 810, include_bytes!("test_images/sqp_text.rgba"));
const DPFLOGO: (u16, u16, &[u8]) = (1123, 639, include_bytes!("test_images/dpf_logo.rgba"));

type TestImage = (u16, u16, &'static [u8]);
const TEST_IMAGES: &[TestImage] = &[KODIM03, KODIM23, SQPTEXT, DPFLOGO];

#[test]
fn cz0_round_trip() {
    for image in TEST_IMAGES {
        let original_cz = DynamicCz::from_raw(CzVersion::CZ0, image.0, image.1, image.2.to_vec());

        let mut cz_bytes = Vec::new();
        original_cz.encode(&mut cz_bytes).unwrap();

        let mut cz_bytes = Cursor::new(cz_bytes);
        let decoded_cz = DynamicCz::decode(&mut cz_bytes).unwrap();

        assert_eq!(original_cz.as_raw(), decoded_cz.as_raw());
    }
}

#[test]
fn cz1_round_trip() {
    for image in TEST_IMAGES {
        let original_cz = DynamicCz::from_raw(CzVersion::CZ1, image.0, image.1, image.2.to_vec());

        let mut cz_bytes = Vec::new();
        original_cz.encode(&mut cz_bytes).unwrap();

        let mut cz_bytes = Cursor::new(cz_bytes);
        let decoded_cz = DynamicCz::decode(&mut cz_bytes).unwrap();

        assert_eq!(original_cz.as_raw(), decoded_cz.as_raw());
    }
}

#[test]
fn cz2_round_trip() {
    let mut i = 0;
    for image in TEST_IMAGES {
        let original_cz = DynamicCz::from_raw(CzVersion::CZ2, image.0, image.1, image.2.to_vec());

        let mut cz_bytes = Vec::new();
        original_cz.encode(&mut cz_bytes).unwrap();

        let mut cz_bytes = Cursor::new(cz_bytes);
        let decoded_cz = DynamicCz::decode(&mut cz_bytes).unwrap();

        assert_eq!(original_cz.as_raw(), decoded_cz.as_raw());

        i += 1;
    }
}

#[test]
fn cz3_round_trip() {
    for image in TEST_IMAGES {
        let original_cz = DynamicCz::from_raw(CzVersion::CZ3, image.0, image.1, image.2.to_vec());

        let mut cz_bytes = Vec::new();
        original_cz.encode(&mut cz_bytes).unwrap();

        let mut cz_bytes = Cursor::new(cz_bytes);
        let decoded_cz = DynamicCz::decode(&mut cz_bytes).unwrap();

        assert_eq!(original_cz.as_raw(), decoded_cz.as_raw());
    }
}

#[test]
fn cz4_round_trip() {
    for image in TEST_IMAGES {
        let original_cz = DynamicCz::from_raw(CzVersion::CZ4, image.0, image.1, image.2.to_vec());

        let mut cz_bytes = Vec::new();
        original_cz.encode(&mut cz_bytes).unwrap();

        let mut cz_bytes = Cursor::new(cz_bytes);
        let decoded_cz = DynamicCz::decode(&mut cz_bytes).unwrap();

        assert_eq!(original_cz.as_raw(), decoded_cz.as_raw());
    }
}
