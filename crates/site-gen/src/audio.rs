//! Content addressing for generated dialog audio.
//!
//! Each spoken line in a dialog is synthesized into a per-line MP3 file.
//! To make re-generation idempotent (edit one line → one API call, not
//! forty), we associate each MP3 with a BLAKE3 hash of the inputs that
//! produced it. A sidecar [`Manifest`] file in the dialog's audio
//! directory records the mapping, and a re-synth that sees a matching
//! hash skips the TTS call.
//!
//! The hash is *scoped to one dialog*: we never dedup across dialogs,
//! because a dialog is the pedagogical unit. Re-using a phrase in
//! another dialog produces its own MP3, with that dialog's voice,
//! context, and pacing.
//!
//! The four fields in the hash each play a specific role:
//! - `dialog_slug` — scopes the address to one dialog, so the same text
//!   said in two dialogs produces two distinct MP3s.
//! - `speaker` — distinguishes identical lines spoken by different
//!   characters in the same dialog.
//! - `canonical_text` — the actual text that gets sent to TTS, with
//!   whitespace normalized so trivial formatting edits don't invalidate
//!   the cache.
//! - `voice_name` — serves as the cache-invalidation key when a voice
//!   changes: swapping Maeve's voice from `fr-FR-Studio-A` to another
//!   voice changes every Maeve MP3's hash, forcing a re-synth.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Current on-disk manifest schema version. Bump when the file format
/// changes in a way that makes older manifests unreadable; older
/// versions are then treated as cache-invalid (everything re-synths).
pub const MANIFEST_VERSION: u32 = 1;

/// Default filename for the per-dialog manifest sidecar.
///
/// The leading dot keeps it out of the way of line MP3s when the audio
/// directory is listed.
pub const MANIFEST_FILENAME: &str = ".manifest.json";

/// Record of which content-addressed audio files are present in a
/// dialog's output directory, and what `audio_hash` produced each one.
///
/// A cache lookup is: for a line's expected filename, find the manifest
/// entry; if its hash matches the current `audio_hash`, the file on
/// disk is valid and can be reused.
///
/// `BTreeMap` gives deterministic JSON output, which keeps git diffs
/// minimal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    /// Schema version. See [`MANIFEST_VERSION`].
    pub version: u32,
    /// The dialog slug this manifest belongs to. Not used in lookup;
    /// present for human readability and debug cross-checks.
    pub dialog_slug: String,
    /// Map from per-line filename (e.g. `01_antoine.mp3`) to the
    /// [`audio_hash`] that produced it.
    pub entries: BTreeMap<String, String>,
}

impl Manifest {
    /// Create an empty manifest for `dialog_slug`.
    pub fn new(dialog_slug: &str) -> Self {
        Self {
            version: MANIFEST_VERSION,
            dialog_slug: dialog_slug.to_string(),
            entries: BTreeMap::new(),
        }
    }

    /// Load an existing manifest from `path`.
    ///
    /// Returns [`None`] if:
    /// - the file doesn't exist (first-ever build of this dialog)
    /// - the file is unreadable (will be recreated next write)
    /// - the JSON is unparseable (likely a format change, force re-synth)
    /// - the schema version is unknown
    ///
    /// All of these cases are safe: the caller proceeds as if every
    /// line were a cache miss, which re-synthesizes and writes a fresh
    /// manifest.
    pub fn load(path: &Path) -> Option<Self> {
        let bytes = std::fs::read(path).ok()?;
        let manifest: Self = serde_json::from_slice(&bytes).ok()?;
        if manifest.version != MANIFEST_VERSION {
            return None;
        }
        Some(manifest)
    }

    /// Load an existing manifest for `dialog_slug` from `dir`, or
    /// return a fresh empty one.
    pub fn load_or_new(dir: &Path, dialog_slug: &str) -> Self {
        Self::load(&dir.join(MANIFEST_FILENAME)).unwrap_or_else(|| Self::new(dialog_slug))
    }

    /// Write the manifest into `dir` as `.manifest.json`, pretty-
    /// printed for readability and stable diffs.
    pub fn save(&self, dir: &Path) -> std::io::Result<()> {
        let pretty = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(dir.join(MANIFEST_FILENAME), pretty)
    }

    /// Return true if `filename` is recorded with hash `hash`.
    pub fn is_cached(&self, filename: &str, hash: &str) -> bool {
        self.entries.get(filename).map(|h| h == hash).unwrap_or(false)
    }

    /// Record that `filename` was written with content hash `hash`.
    pub fn insert(&mut self, filename: String, hash: String) {
        self.entries.insert(filename, hash);
    }
}

/// Compute a stable 16-hex-char content address for one spoken dialog
/// line.
///
/// The prefix length (16 hex = 64 bits) is the same as intreccio's drill
/// hashes. At the scale of this project (~1000 lines across all
/// chapters) collision risk is negligible.
///
/// This function is a pinned schema: changing any of its inputs,
/// separators, or the prefix length silently invalidates every
/// committed manifest entry in the repo. The companion test
/// [`tests::pinned_values`] guards against that.
pub fn audio_hash(
    dialog_slug: &str,
    speaker: &str,
    canonical_text: &str,
    voice_name: &str,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(dialog_slug.as_bytes());
    hasher.update(b"\0");
    hasher.update(speaker.as_bytes());
    hasher.update(b"\0");
    hasher.update(canonical_text.as_bytes());
    hasher.update(b"\0");
    hasher.update(voice_name.as_bytes());
    hasher.finalize().to_hex()[..16].to_string()
}

/// Canonicalize a raw dialog line for hashing.
///
/// Collapses any run of whitespace (spaces, tabs, newlines, narrow
/// no-break spaces, etc.) to a single ASCII space and trims the ends.
/// Punctuation is preserved because it affects TTS prosody — a period
/// vs. a comma is not the same speech.
///
/// This keeps trivial whitespace edits from invalidating the cache
/// while still catching any real content change.
pub fn canonical_text(line: &str) -> String {
    line.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_stable_and_sixteen_hex() {
        let h = audio_hash("02_viennoiserie", "Antoine", "Bonjour madame !", "fr-FR-Studio-D");
        assert_eq!(h.len(), 16);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hash_differs_across_dialogs() {
        let a = audio_hash("02_viennoiserie", "Antoine", "Bonjour madame !", "fr-FR-Studio-D");
        let b = audio_hash("07_boulangerie", "Antoine", "Bonjour madame !", "fr-FR-Studio-D");
        assert_ne!(a, b, "dialog_slug must contribute to the hash");
    }

    #[test]
    fn hash_differs_across_speakers() {
        let a = audio_hash("02_viennoiserie", "Antoine", "Bonjour madame !", "fr-FR-Studio-D");
        let b = audio_hash("02_viennoiserie", "Sophie",  "Bonjour madame !", "fr-FR-Studio-D");
        assert_ne!(a, b, "speaker must contribute to the hash");
    }

    #[test]
    fn hash_differs_across_text() {
        let a = audio_hash("02_viennoiserie", "Antoine", "Bonjour madame !", "fr-FR-Studio-D");
        let b = audio_hash("02_viennoiserie", "Antoine", "Bonjour monsieur !", "fr-FR-Studio-D");
        assert_ne!(a, b, "text must contribute to the hash");
    }

    #[test]
    fn hash_differs_across_voices() {
        // A voice change must invalidate the cache — this is the field
        // that makes "I updated Maeve's voice assignment" into a
        // deliberate, observable cache miss rather than a silent
        // stale-file.
        let a = audio_hash("02_viennoiserie", "Antoine", "Bonjour madame !", "fr-FR-Studio-D");
        let b = audio_hash("02_viennoiserie", "Antoine", "Bonjour madame !", "fr-FR-Neural2-G");
        assert_ne!(a, b, "voice_name must contribute to the hash");
    }

    #[test]
    fn canonical_text_collapses_whitespace() {
        assert_eq!(canonical_text("  bonjour   monde  "), "bonjour monde");
        assert_eq!(canonical_text("a\tb\nc"), "a b c");
        // Narrow no-break space (U+202F), used in French typography
        // before high punctuation.
        assert_eq!(canonical_text("Bonjour\u{202F}!"), "Bonjour !");
    }

    #[test]
    fn canonical_text_preserves_punctuation() {
        // Punctuation alters TTS prosody, so it must NOT be stripped.
        assert_ne!(
            canonical_text("Bonjour, madame."),
            canonical_text("Bonjour madame")
        );
    }

    /// Pin a handful of (inputs → hash) tuples for regression detection.
    ///
    /// If this test fails, someone has changed the hash schema — the
    /// field separator, the prefix length, the input order, or
    /// switched away from BLAKE3 — and every committed
    /// `.manifest.json` entry in the repo is now stale. Either revert
    /// the change or migrate manifests.
    ///
    /// These values were computed by running `audio_hash` once and
    /// copying the output. They have no external meaning beyond
    /// guarding the schema.
    #[test]
    fn pinned_values() {
        let cases: &[(&str, &str, &str, &str, &str)] = &[
            (
                "02_viennoiserie",
                "Antoine",
                "Bonjour madame !",
                "fr-FR-Studio-D",
                "3f8e87dbb4d27f1b",
            ),
            (
                "05_chat_perdu",
                "Maeve",
                "On est vraiment inquiètes.",
                "fr-FR-Studio-A",
                "fb1498e47a5a4c71",
            ),
            (
                "01_paris_metro",
                "Léa",
                "Excusez-moi, je cherche la ligne 1.",
                "fr-FR-Studio-A",
                "94b7218483520c0d",
            ),
        ];

        for (slug, speaker, text, voice, expected) in cases {
            assert_eq!(
                audio_hash(slug, speaker, text, voice),
                *expected,
                "hash for ({slug}, {speaker}, {text:?}, {voice}) changed — \
                 every committed manifest is now stale",
            );
        }
    }

    // ── Manifest ────────────────────────────────────────────────────

    #[test]
    fn manifest_new_is_empty() {
        let m = Manifest::new("02_viennoiserie");
        assert_eq!(m.version, MANIFEST_VERSION);
        assert_eq!(m.dialog_slug, "02_viennoiserie");
        assert!(m.entries.is_empty());
    }

    #[test]
    fn manifest_cache_hit_and_miss() {
        let mut m = Manifest::new("02_viennoiserie");
        m.insert("01_antoine.mp3".to_string(), "abc123".to_string());

        assert!(m.is_cached("01_antoine.mp3", "abc123"));
        assert!(!m.is_cached("01_antoine.mp3", "def456"), "different hash → miss");
        assert!(!m.is_cached("02_sophie.mp3", "abc123"), "different filename → miss");
    }

    #[test]
    fn manifest_roundtrip_through_disk() {
        let dir = tempfile::tempdir().unwrap();
        let mut m = Manifest::new("02_viennoiserie");
        m.insert("01_antoine.mp3".to_string(), "3f8e87dbb4d27f1b".to_string());
        m.insert("02_sophie.mp3".to_string(), "a1b2c3d4e5f67890".to_string());

        m.save(dir.path()).unwrap();
        let loaded = Manifest::load_or_new(dir.path(), "02_viennoiserie");

        assert_eq!(loaded, m, "manifest should roundtrip unchanged");
    }

    #[test]
    fn manifest_load_or_new_returns_empty_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let loaded = Manifest::load_or_new(dir.path(), "fresh_dialog");
        assert_eq!(loaded.dialog_slug, "fresh_dialog");
        assert!(loaded.entries.is_empty());
    }

    #[test]
    fn manifest_load_returns_none_on_corrupt_json() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(MANIFEST_FILENAME),
            "{ this is not valid json",
        ).unwrap();
        // Corrupt → load returns None → load_or_new returns a fresh
        // empty manifest, triggering full re-synth.
        let loaded = Manifest::load_or_new(dir.path(), "recovering");
        assert!(loaded.entries.is_empty());
    }

    #[test]
    fn manifest_load_rejects_unknown_schema_version() {
        let dir = tempfile::tempdir().unwrap();
        let bogus = serde_json::json!({
            "version": 9999,
            "dialog_slug": "future",
            "entries": { "01_foo.mp3": "abc" }
        });
        std::fs::write(
            dir.path().join(MANIFEST_FILENAME),
            serde_json::to_string(&bogus).unwrap(),
        ).unwrap();

        // Unknown future version → safe to ignore; treat as cache-miss.
        let loaded = Manifest::load_or_new(dir.path(), "future");
        assert!(loaded.entries.is_empty());
    }
}
