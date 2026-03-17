use image_strip::{
    detect_format, exif, strip_metadata, strip_metadata_bytes, ImageFormat, StripOptions,
};
use img_parts::ImageEXIF;
use std::path::Path;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(FIXTURES).join(name)
}

/// Return true if `needle` appears anywhere in `haystack`.
fn bytes_contain(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

// ── JPEG tests ──────────────────────────────────────────────────────

#[test]
fn jpeg_strips_exif_and_iptc_and_comment() {
    let data = std::fs::read(fixture("with_exif.jpg")).unwrap();
    let opts = StripOptions::default();

    // Verify the sensitive data is present in the original.
    assert!(
        bytes_contain(&data, b"FakeGPSData"),
        "fixture should contain EXIF GPS data"
    );
    assert!(
        bytes_contain(&data, b"FakeIPTC"),
        "fixture should contain IPTC data"
    );
    assert!(
        bytes_contain(&data, b"Secret comment"),
        "fixture should contain comment"
    );

    let (cleaned, removed) = strip_metadata_bytes(&data, ImageFormat::Jpeg, &opts).unwrap();

    // Verify the sensitive data is gone from the output.
    assert!(
        !bytes_contain(&cleaned, b"FakeGPSData"),
        "EXIF GPS data must be stripped from output"
    );
    assert!(
        !bytes_contain(&cleaned, b"FakeIPTC"),
        "IPTC data must be stripped from output"
    );
    assert!(
        !bytes_contain(&cleaned, b"Secret comment"),
        "comment must be stripped from output"
    );

    // Verify the report is accurate.
    assert!(
        removed.iter().any(|s| s.contains("EXIF")),
        "report should mention EXIF removal: {removed:?}"
    );
    assert!(
        removed.iter().any(|s| s.contains("IPTC")),
        "report should mention IPTC removal: {removed:?}"
    );
    assert!(
        removed.iter().any(|s| s.contains("Comment")),
        "report should mention COM removal: {removed:?}"
    );

    // Output must be parseable JPEG.
    assert_eq!(&cleaned[..2], b"\xFF\xD8", "output must start with SOI");
    img_parts::jpeg::Jpeg::from_bytes(cleaned.into()).expect("output must parse as valid JPEG");
}

#[test]
fn jpeg_keeps_image_data_segments() {
    let data = std::fs::read(fixture("with_exif.jpg")).unwrap();
    let opts = StripOptions::default();
    let (cleaned, _) = strip_metadata_bytes(&data, ImageFormat::Jpeg, &opts).unwrap();

    let jpeg = img_parts::jpeg::Jpeg::from_bytes(cleaned.into()).unwrap();
    let markers: Vec<u8> = jpeg.segments().iter().map(|s| s.marker()).collect();

    assert!(markers.contains(&0xC0), "must keep SOF0");
    assert!(markers.contains(&0xDB), "must keep DQT");
    assert!(markers.contains(&0xC4), "must keep DHT");
    assert!(markers.contains(&0xDA), "must keep SOS");

    // Verify no metadata markers leaked through.
    assert!(!markers.contains(&0xED), "APP13/IPTC must not survive");
    assert!(!markers.contains(&0xFE), "COM must not survive");
}

#[test]
fn jpeg_strips_icc_by_default() {
    let data = std::fs::read(fixture("with_icc.jpg")).unwrap();
    let opts = StripOptions::default();
    let (cleaned, removed) = strip_metadata_bytes(&data, ImageFormat::Jpeg, &opts).unwrap();

    assert!(
        removed.iter().any(|s| s.contains("ICC")),
        "ICC should be removed by default: {removed:?}"
    );

    // Verify no APP2 segments remain.
    let jpeg = img_parts::jpeg::Jpeg::from_bytes(cleaned.into()).unwrap();
    let has_app2 = jpeg.segments().iter().any(|s| s.marker() == 0xE2);
    assert!(!has_app2, "APP2/ICC segment must not survive default strip");
}

#[test]
fn jpeg_keeps_icc_with_flag() {
    let data = std::fs::read(fixture("with_icc.jpg")).unwrap();
    let opts = StripOptions { keep_icc: true };
    let (cleaned, removed) = strip_metadata_bytes(&data, ImageFormat::Jpeg, &opts).unwrap();

    assert!(
        !removed.iter().any(|s| s.contains("ICC")),
        "ICC should not be in removed list: {removed:?}"
    );

    // Verify APP2 actually survived in the output.
    let jpeg = img_parts::jpeg::Jpeg::from_bytes(cleaned.into()).unwrap();
    let has_app2 = jpeg.segments().iter().any(|s| s.marker() == 0xE2);
    assert!(has_app2, "APP2/ICC segment must survive with keep_icc");
}

// ── PNG tests ───────────────────────────────────────────────────────

#[test]
fn png_strips_text_and_exif_and_time() {
    let data = std::fs::read(fixture("with_text.png")).unwrap();
    let opts = StripOptions::default();

    // Verify the metadata content is in the original.
    assert!(
        bytes_contain(&data, b"Secret Person"),
        "fixture should contain tEXt author"
    );

    let (cleaned, removed) = strip_metadata_bytes(&data, ImageFormat::Png, &opts).unwrap();

    // Verify metadata content is gone.
    assert!(
        !bytes_contain(&cleaned, b"Secret Person"),
        "tEXt author must be stripped from output"
    );

    assert!(removed.contains(&"tEXt".to_string()), "report: {removed:?}");
    assert!(removed.contains(&"eXIf".to_string()), "report: {removed:?}");
    assert!(removed.contains(&"tIME".to_string()), "report: {removed:?}");

    // Output must parse as valid PNG.
    assert_eq!(&cleaned[..8], b"\x89PNG\r\n\x1a\n");
    img_parts::png::Png::from_bytes(cleaned.into()).expect("output must parse as valid PNG");
}

#[test]
fn png_keeps_critical_chunks() {
    let data = std::fs::read(fixture("with_text.png")).unwrap();
    let opts = StripOptions::default();
    let (cleaned, _) = strip_metadata_bytes(&data, ImageFormat::Png, &opts).unwrap();

    let png = img_parts::png::Png::from_bytes(cleaned.into()).unwrap();
    let kinds: Vec<String> = png
        .chunks()
        .iter()
        .map(|c| std::str::from_utf8(c.kind().as_ref()).unwrap().to_string())
        .collect();

    assert!(kinds.contains(&"IHDR".to_string()), "must keep IHDR");
    assert!(kinds.contains(&"IDAT".to_string()), "must keep IDAT");
    assert!(kinds.contains(&"IEND".to_string()), "must keep IEND");

    // Verify metadata chunks are gone.
    assert!(!kinds.contains(&"tEXt".to_string()), "tEXt must not survive");
    assert!(!kinds.contains(&"eXIf".to_string()), "eXIf must not survive");
    assert!(!kinds.contains(&"tIME".to_string()), "tIME must not survive");
}

#[test]
fn png_strips_iccp_by_default() {
    let data = std::fs::read(fixture("with_iccp.png")).unwrap();
    let opts = StripOptions::default();
    let (cleaned, removed) = strip_metadata_bytes(&data, ImageFormat::Png, &opts).unwrap();

    assert!(removed.contains(&"iCCP".to_string()), "{removed:?}");

    // Verify iCCP chunk is actually absent.
    let png = img_parts::png::Png::from_bytes(cleaned.into()).unwrap();
    let has_iccp = png
        .chunks()
        .iter()
        .any(|c| c.kind().as_ref() == b"iCCP");
    assert!(!has_iccp, "iCCP chunk must not survive default strip");
}

#[test]
fn png_keeps_iccp_with_flag() {
    let data = std::fs::read(fixture("with_iccp.png")).unwrap();
    let opts = StripOptions { keep_icc: true };
    let (cleaned, removed) = strip_metadata_bytes(&data, ImageFormat::Png, &opts).unwrap();

    assert!(!removed.contains(&"iCCP".to_string()), "{removed:?}");

    // Verify iCCP chunk actually survived.
    let png = img_parts::png::Png::from_bytes(cleaned.into()).unwrap();
    let has_iccp = png
        .chunks()
        .iter()
        .any(|c| c.kind().as_ref() == b"iCCP");
    assert!(has_iccp, "iCCP chunk must survive with keep_icc");
}

// ── File-level API tests ────────────────────────────────────────────

#[test]
fn strip_metadata_file_api() {
    let tmp = tempfile::tempdir().unwrap();
    let input = fixture("with_exif.jpg");
    let output = tmp.path().join("out.jpg");

    let report = strip_metadata(&input, &output, &StripOptions::default()).unwrap();

    assert_eq!(report.format, ImageFormat::Jpeg);
    assert!(!report.segments_removed.is_empty());
    assert!(report.bytes_after < report.bytes_before);
    assert!(output.exists());

    // Verify the output file is a valid parseable JPEG.
    let out_data = std::fs::read(&output).unwrap();
    img_parts::jpeg::Jpeg::from_bytes(out_data.into()).expect("output file must parse as JPEG");
}

// ── Orientation preservation ────────────────────────────────────────

#[test]
fn jpeg_preserves_orientation() {
    // Build a JPEG with a full EXIF blob (containing GPS etc.) AND orientation=6.
    // We do this by taking the fixture's existing EXIF and prepending an
    // orientation tag via a full EXIF blob that img-parts will recognise.
    let base_data = std::fs::read(fixture("with_exif.jpg")).unwrap();
    let base_jpeg = img_parts::jpeg::Jpeg::from_bytes(base_data.clone().into()).unwrap();

    // The fixture has EXIF with fake GPS but no orientation. Build a new
    // EXIF that has orientation=6 and is larger than our minimal blob
    // (proving the stripping actually reduces it).
    let mut big_exif = exif::build_orientation_exif(6);
    // Append junk to simulate a real EXIF with extra tags.
    big_exif.extend_from_slice(b"FAKE_GPS_DATA_AND_DEVICE_INFO_HERE_PADDING");

    let mut jpeg = base_jpeg;
    jpeg.set_exif(Some(big_exif.clone().into()));
    let mut input_bytes = Vec::new();
    jpeg.encoder().write_to(&mut input_bytes).unwrap();

    // Verify the fixture has orientation 6 before stripping.
    let pre = img_parts::jpeg::Jpeg::from_bytes(input_bytes.clone().into()).unwrap();
    let pre_orient = pre.exif().as_deref().and_then(exif::read_orientation);
    assert_eq!(pre_orient, Some(6), "precondition: orientation must be 6");

    // The EXIF should be bigger than our minimal orientation-only blob.
    let pre_exif_len = pre.exif().as_ref().map(|b| b.len()).unwrap_or(0);
    let minimal_len = exif::build_orientation_exif(6).len();
    assert!(
        pre_exif_len > minimal_len,
        "precondition: EXIF ({pre_exif_len} bytes) should be larger than minimal ({minimal_len})"
    );

    // Strip metadata.
    let opts = StripOptions::default();
    let (cleaned, removed) = strip_metadata_bytes(&input_bytes, ImageFormat::Jpeg, &opts).unwrap();

    // EXIF should be reported as removed (the bloated one was replaced).
    assert!(
        removed.iter().any(|s| s.contains("EXIF")),
        "should report EXIF removal: {removed:?}"
    );

    // But orientation must survive in the output.
    let out_jpeg = img_parts::jpeg::Jpeg::from_bytes(cleaned.into()).unwrap();
    let out_orient = out_jpeg
        .exif()
        .as_deref()
        .and_then(exif::read_orientation);
    assert_eq!(
        out_orient,
        Some(6),
        "orientation 6 must be preserved after stripping"
    );

    // The output EXIF should be exactly our minimal blob.
    let out_exif = out_jpeg.exif().unwrap();
    assert_eq!(
        out_exif.len(),
        minimal_len,
        "output EXIF should be minimal orientation-only blob"
    );
}

#[test]
fn jpeg_no_orientation_exif_when_default() {
    // A JPEG with orientation=1 (or no orientation) should NOT get a
    // minimal EXIF written back — it's unnecessary.
    let data = std::fs::read(fixture("with_exif.jpg")).unwrap();
    let opts = StripOptions::default();
    let (cleaned, _) = strip_metadata_bytes(&data, ImageFormat::Jpeg, &opts).unwrap();

    let out_jpeg = img_parts::jpeg::Jpeg::from_bytes(cleaned.into()).unwrap();
    assert!(
        out_jpeg.exif().is_none(),
        "JPEG with no meaningful orientation should have no EXIF after strip"
    );
}

// ── Idempotency ─────────────────────────────────────────────────────

#[test]
fn jpeg_idempotent() {
    let data = std::fs::read(fixture("with_exif.jpg")).unwrap();
    let opts = StripOptions::default();
    let (first, first_removed) = strip_metadata_bytes(&data, ImageFormat::Jpeg, &opts).unwrap();
    assert!(
        !first_removed.is_empty(),
        "precondition: first strip should remove something"
    );
    let (second, removed) = strip_metadata_bytes(&first, ImageFormat::Jpeg, &opts).unwrap();

    assert!(
        removed.is_empty(),
        "second strip should find nothing to remove, got: {removed:?}"
    );
    assert_eq!(first, second, "output should be byte-identical");
}

#[test]
fn jpeg_idempotent_with_orientation() {
    // Build a JPEG with orientation=6, strip it, then strip again.
    let base_data = std::fs::read(fixture("with_exif.jpg")).unwrap();
    let mut jpeg = img_parts::jpeg::Jpeg::from_bytes(base_data.into()).unwrap();
    jpeg.set_exif(Some(exif::build_orientation_exif(6).into()));
    let mut input_bytes = Vec::new();
    jpeg.encoder().write_to(&mut input_bytes).unwrap();

    let opts = StripOptions::default();
    let (first, _) = strip_metadata_bytes(&input_bytes, ImageFormat::Jpeg, &opts).unwrap();
    let (second, removed) = strip_metadata_bytes(&first, ImageFormat::Jpeg, &opts).unwrap();

    assert!(
        removed.is_empty(),
        "second strip should find nothing to remove, got: {removed:?}"
    );
    assert_eq!(first, second, "output should be byte-identical");

    // Orientation must still be 6 after both passes.
    let out = img_parts::jpeg::Jpeg::from_bytes(second.into()).unwrap();
    assert_eq!(out.exif().as_deref().and_then(exif::read_orientation), Some(6));
}

#[test]
fn png_idempotent() {
    let data = std::fs::read(fixture("with_text.png")).unwrap();
    let opts = StripOptions::default();
    let (first, first_removed) = strip_metadata_bytes(&data, ImageFormat::Png, &opts).unwrap();
    assert!(
        !first_removed.is_empty(),
        "precondition: first strip should remove something"
    );
    let (second, removed) = strip_metadata_bytes(&first, ImageFormat::Png, &opts).unwrap();

    assert!(
        removed.is_empty(),
        "second strip should find nothing to remove, got: {removed:?}"
    );
    assert_eq!(first, second, "output should be byte-identical");
}

// ── Format detection ────────────────────────────────────────────────

#[test]
fn detect_format_works() {
    assert_eq!(
        detect_format(Path::new("photo.jpg")),
        Some(ImageFormat::Jpeg)
    );
    assert_eq!(
        detect_format(Path::new("photo.JPEG")),
        Some(ImageFormat::Jpeg)
    );
    assert_eq!(
        detect_format(Path::new("image.png")),
        Some(ImageFormat::Png)
    );
    assert_eq!(
        detect_format(Path::new("image.PNG")),
        Some(ImageFormat::Png)
    );
    assert_eq!(detect_format(Path::new("file.gif")), None);
    assert_eq!(detect_format(Path::new("noext")), None);
}
