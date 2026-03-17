use img_parts::jpeg::{markers, Jpeg};

/// Segments we always keep — these contain the actual image data and JFIF header.
fn is_allowed(marker: u8, keep_icc: bool) -> bool {
    matches!(
        marker,
        markers::SOI
            | markers::SOF0
            | markers::SOF1
            | markers::SOF2
            | markers::SOF3
            | markers::DHT
            | markers::DQT
            | markers::DRI
            | markers::SOS
            | markers::APP0 // JFIF header (pixel density)
            | markers::EOI
    ) || is_rst(marker)
        || (keep_icc && marker == markers::APP2)
}

fn is_rst(marker: u8) -> bool {
    (0xD0..=0xD7).contains(&marker)
}

/// Returns `(stripped_jpeg_bytes, list_of_removed_segment_names)`.
pub fn strip(jpeg: &Jpeg, keep_icc: bool) -> (Vec<u8>, Vec<String>) {
    let mut out = jpeg.clone();
    let mut removed = Vec::new();

    out.segments_mut().retain(|seg| {
        let marker = seg.marker();
        if is_allowed(marker, keep_icc) {
            true
        } else {
            removed.push(segment_name(marker));
            false
        }
    });

    let mut buf = Vec::new();
    out.encoder()
        .write_to(&mut buf)
        .expect("encoding to Vec<u8> should not fail");
    (buf, removed)
}

fn segment_name(marker: u8) -> String {
    match marker {
        markers::APP0 => "APP0/JFIF".into(),
        markers::APP1 => "APP1/EXIF+XMP".into(),
        markers::APP2 => "APP2/ICC".into(),
        0xE3 => "APP3".into(),
        0xE4 => "APP4".into(),
        0xE5 => "APP5".into(),
        0xE6 => "APP6".into(),
        0xE7 => "APP7".into(),
        0xE8 => "APP8".into(),
        0xE9 => "APP9".into(),
        0xEA => "APP10".into(),
        0xEB => "APP11".into(),
        0xEC => "APP12".into(),
        0xED => "APP13/IPTC".into(),
        0xEE => "APP14".into(),
        0xEF => "APP15".into(),
        markers::COM => "COM/Comment".into(),
        other => format!("0x{other:02X}"),
    }
}
