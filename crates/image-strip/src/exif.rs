/// Extract the EXIF orientation value (1–8) from raw EXIF bytes.
///
/// The input should be the EXIF payload as returned by `img_parts::ImageEXIF::exif()`,
/// which starts with the TIFF header (byte order marker), not the `Exif\0\0` prefix.
pub fn read_orientation(exif_bytes: &[u8]) -> Option<u16> {
    if exif_bytes.len() < 8 {
        return None;
    }

    let big_endian = match &exif_bytes[..2] {
        b"MM" => true,
        b"II" => false,
        _ => return None,
    };

    let read_u16 = |off: usize| -> Option<u16> {
        let bytes: [u8; 2] = exif_bytes.get(off..off + 2)?.try_into().ok()?;
        Some(if big_endian {
            u16::from_be_bytes(bytes)
        } else {
            u16::from_le_bytes(bytes)
        })
    };

    let read_u32 = |off: usize| -> Option<u32> {
        let bytes: [u8; 4] = exif_bytes.get(off..off + 4)?.try_into().ok()?;
        Some(if big_endian {
            u32::from_be_bytes(bytes)
        } else {
            u32::from_le_bytes(bytes)
        })
    };

    // Magic number check.
    if read_u16(2)? != 42 {
        return None;
    }

    let ifd_offset = read_u32(4)? as usize;
    let entry_count = read_u16(ifd_offset)? as usize;

    const ORIENTATION_TAG: u16 = 0x0112;

    for i in 0..entry_count {
        let entry_off = ifd_offset + 2 + i * 12;
        let tag = read_u16(entry_off)?;
        if tag == ORIENTATION_TAG {
            // Type should be SHORT (3). Value is in bytes 8–9 of the entry.
            let val = read_u16(entry_off + 8)?;
            if (1..=8).contains(&val) {
                return Some(val);
            }
        }
    }

    None
}

/// Build a minimal EXIF (TIFF) blob containing only the orientation tag.
///
/// Returns bytes suitable for `img_parts::ImageEXIF::set_exif(Some(...))`.
/// Uses big-endian byte order for simplicity.
pub fn build_orientation_exif(orientation: u16) -> Vec<u8> {
    let mut buf = Vec::with_capacity(26);

    // TIFF header: big-endian, magic 42, IFD0 offset = 8
    buf.extend_from_slice(b"MM");
    buf.extend_from_slice(&42u16.to_be_bytes());
    buf.extend_from_slice(&8u32.to_be_bytes());

    // IFD0: 1 entry
    buf.extend_from_slice(&1u16.to_be_bytes());

    // Entry: tag=0x0112 (Orientation), type=3 (SHORT), count=1, value
    buf.extend_from_slice(&0x0112u16.to_be_bytes()); // tag
    buf.extend_from_slice(&3u16.to_be_bytes()); // type = SHORT
    buf.extend_from_slice(&1u32.to_be_bytes()); // count
    buf.extend_from_slice(&orientation.to_be_bytes()); // value (2 bytes)
    buf.extend_from_slice(&[0, 0]); // padding to fill 4-byte value field

    // Next IFD offset = 0 (no more IFDs)
    buf.extend_from_slice(&0u32.to_be_bytes());

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_orientation() {
        for orientation in 1..=8 {
            let exif = build_orientation_exif(orientation);
            let read_back = read_orientation(&exif);
            assert_eq!(read_back, Some(orientation), "orientation {orientation}");
        }
    }

    #[test]
    fn read_orientation_from_empty() {
        assert_eq!(read_orientation(b""), None);
        assert_eq!(read_orientation(b"MM"), None);
    }
}
