use img_parts::png::Png;

/// Critical + safe ancillary chunks we keep.
const ALLOWED_CHUNKS: &[&str] = &[
    "IHDR", "PLTE", "IDAT", "IEND", // critical
    "tRNS", "gAMA", "cHRM", "sRGB", "sBIT", "bKGD", "pHYs", // safe ancillary
];

const ICC_CHUNK: &str = "iCCP";

fn is_allowed(kind: &str, keep_icc: bool) -> bool {
    ALLOWED_CHUNKS.contains(&kind) || (keep_icc && kind == ICC_CHUNK)
}

/// Returns `(stripped_png_bytes, list_of_removed_chunk_names)`.
pub fn strip(png: &Png, keep_icc: bool) -> (Vec<u8>, Vec<String>) {
    let mut out = png.clone();
    let mut removed = Vec::new();

    out.chunks_mut().retain(|chunk| {
        let kind = chunk.kind();
        let kind_str = std::str::from_utf8(kind.as_ref()).unwrap_or("????");
        if is_allowed(kind_str, keep_icc) {
            true
        } else {
            removed.push(kind_str.to_string());
            false
        }
    });

    let mut buf = Vec::new();
    out.encoder()
        .write_to(&mut buf)
        .expect("encoding to Vec<u8> should not fail");
    (buf, removed)
}
