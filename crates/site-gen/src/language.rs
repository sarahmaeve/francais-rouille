use std::collections::HashMap;

use crate::dialog::{DialogLine, Gender};

/// A TTS voice identified by its language code and Google Cloud voice name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Voice {
    /// BCP 47 language code, e.g. "fr-FR", "es-US".
    pub language_code: &'static str,
    /// Full Google Cloud voice name, e.g. "fr-FR-Studio-A".
    pub name: &'static str,
}

/// A pool of TTS voices partitioned by gender.
///
/// The first speaker of each gender is assigned a preferred voice (if any).
/// Additional speakers of the same gender draw from the remaining pool,
/// which is shuffled for variety.
pub struct VoicePool {
    /// Preferred female voices (e.g. Studio), assigned first in order.
    pub preferred_female: &'static [Voice],
    /// Additional female voices, shuffled for variety.
    pub female: &'static [Voice],
    /// Preferred male voices (e.g. Studio), assigned first in order.
    pub preferred_male: &'static [Voice],
    /// Additional male voices, shuffled for variety.
    pub male: &'static [Voice],
}

/// Language-specific behavior for gender detection and TTS voice selection.
pub trait Language {
    /// BCP 47 language code, e.g. "fr-FR", "es-US".
    fn code(&self) -> &'static str;

    /// Detect speaker gender from a character description line.
    ///
    /// The description is the text after the em-dash in a character line,
    /// e.g. for `- Claire — une cliente curieuse`, the description is
    /// `"une cliente curieuse"`.
    fn detect_gender(&self, description: &str) -> Option<Gender>;

    /// Return the available TTS voices, partitioned by gender.
    fn voice_pool(&self) -> VoicePool;
}

/// Parse character descriptions and detect gender using a language-specific
/// implementation.
pub fn parse_character_genders(content: &str, lang: &dyn Language) -> HashMap<String, Gender> {
    let mut genders = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if !line.starts_with('-') {
            continue;
        }
        let line = line.trim_start_matches('-').trim();

        let (name, description) = if let Some(pos) = line.find(" — ") {
            (&line[..pos], &line[pos + " — ".len()..])
        } else if let Some(pos) = line.find(" – ") {
            (&line[..pos], &line[pos + " – ".len()..])
        } else {
            continue;
        };

        let name = name.trim().to_string();
        let description = description.trim().to_lowercase();

        if let Some(gender) = lang.detect_gender(&description) {
            genders.insert(name, gender);
        }
    }

    genders
}

/// Build an ordered voice list: preferred voices first (in their
/// declared order, because Studio voices handle French TTS better than
/// the Chirp3-HD / Neural2 / Wavenet alternatives), then the remaining
/// voices permuted deterministically by `seed` for variety.
///
/// The ordering of the *fallback* tier is what changes per call: we
/// want the same dialog to produce the same voice assignments every
/// time (idempotent re-synth), but two different dialogs with the same
/// character count should not assign the same fallback voices to their
/// second-speaker-of-a-gender.
///
/// `seed` is typically the dialog slug.
fn build_voice_order(preferred: &[Voice], rest: &[Voice], seed: &str) -> Vec<Voice> {
    let mut ordered: Vec<Voice> = preferred.to_vec();
    let mut fallback: Vec<Voice> = rest.to_vec();
    // Sort fallbacks by hash(seed || voice.name). Same seed → same
    // order, always. Different seeds → different permutations, because
    // the hash is a strong function of both inputs.
    fallback.sort_by_cached_key(|v| {
        let mut hasher = blake3::Hasher::new();
        hasher.update(seed.as_bytes());
        hasher.update(b"\0");
        hasher.update(v.name.as_bytes());
        // 8 bytes of the digest is more than enough to order ~20 voices
        // without ties, and fits in a [u8; 8] sort key.
        let mut key = [0u8; 8];
        key.copy_from_slice(&hasher.finalize().as_bytes()[..8]);
        key
    });
    ordered.extend(fallback);
    ordered
}

/// Assign distinct voices to speakers based on detected gender.
///
/// Preferred voices (e.g. Studio) are assigned first — a dialog with a
/// single speaker per gender always uses the preferred voice, which is
/// the best-sounding Google voice for each language. When a dialog has
/// more speakers of one gender than there are preferred voices, the
/// additional speakers receive voices from the fallback pool, in an
/// order that is deterministic per `dialog_slug` (so re-runs reproduce
/// the same assignments).
///
/// Each character keeps the same voice throughout the dialog.
pub fn assign_voices(
    lines: &[DialogLine],
    genders: &HashMap<String, Gender>,
    lang: &dyn Language,
    dialog_slug: &str,
) -> HashMap<String, Voice> {
    let pool = lang.voice_pool();

    let female_voices = build_voice_order(pool.preferred_female, pool.female, dialog_slug);
    let male_voices = build_voice_order(pool.preferred_male, pool.male, dialog_slug);

    let mut map = HashMap::new();
    let mut female_idx: usize = 0;
    let mut male_idx: usize = 0;

    for line in lines {
        if map.contains_key(&line.speaker) {
            continue;
        }

        let gender = genders
            .get(&line.speaker)
            .copied()
            .unwrap_or_else(|| {
                if (female_idx + male_idx).is_multiple_of(2) {
                    Gender::Female
                } else {
                    Gender::Male
                }
            });

        let voice = match gender {
            Gender::Female => {
                let v = female_voices[female_idx % female_voices.len()].clone();
                female_idx += 1;
                v
            }
            Gender::Male => {
                let v = male_voices[male_idx % male_voices.len()].clone();
                male_idx += 1;
                v
            }
        };

        map.insert(line.speaker.clone(), voice);
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialog::parse_dialog;

    /// Minimal test language for unit tests.
    struct TestLang;

    static TEST_PREF_FEMALE: &[Voice] = &[
        Voice { language_code: "xx-XX", name: "xx-XX-Studio-F" },
    ];

    static TEST_FEMALE: &[Voice] = &[
        Voice { language_code: "xx-XX", name: "xx-XX-FemaleA" },
        Voice { language_code: "xx-XX", name: "xx-XX-FemaleB" },
    ];

    static TEST_PREF_MALE: &[Voice] = &[
        Voice { language_code: "xx-XX", name: "xx-XX-Studio-M" },
    ];

    static TEST_MALE: &[Voice] = &[
        Voice { language_code: "xx-XX", name: "xx-XX-MaleA" },
        Voice { language_code: "xx-XX", name: "xx-XX-MaleB" },
    ];

    /// All female voices (preferred + rest) for assertions.
    fn all_female() -> Vec<Voice> {
        let mut v = TEST_PREF_FEMALE.to_vec();
        v.extend_from_slice(TEST_FEMALE);
        v
    }

    /// All male voices (preferred + rest) for assertions.
    fn all_male() -> Vec<Voice> {
        let mut v = TEST_PREF_MALE.to_vec();
        v.extend_from_slice(TEST_MALE);
        v
    }

    impl Language for TestLang {
        fn code(&self) -> &'static str { "xx-XX" }

        fn detect_gender(&self, description: &str) -> Option<Gender> {
            let first = description.split_whitespace().next()?;
            match first {
                "una" | "la" => Some(Gender::Female),
                "un" | "el" => Some(Gender::Male),
                _ => None,
            }
        }

        fn voice_pool(&self) -> VoicePool {
            VoicePool {
                preferred_female: TEST_PREF_FEMALE,
                female: TEST_FEMALE,
                preferred_male: TEST_PREF_MALE,
                male: TEST_MALE,
            }
        }
    }

    #[test]
    fn parse_character_genders_uses_language() {
        let content = "\
- María — una estudiante
- Carlos — un profesor";
        let genders = parse_character_genders(content, &TestLang);
        assert_eq!(genders["María"], Gender::Female);
        assert_eq!(genders["Carlos"], Gender::Male);
    }

    #[test]
    fn assign_voices_respects_gender() {
        let content = "\
- María — una estudiante
- Carlos — un profesor

María : Hola.

Carlos : Buenos días.
";
        let lines = parse_dialog(content);
        let genders = parse_character_genders(content, &TestLang);
        let voices = assign_voices(&lines, &genders, &TestLang, "test_slug");
        assert_eq!(voices.len(), 2);
        assert!(all_female().contains(&voices["María"]));
        assert!(all_male().contains(&voices["Carlos"]));
    }

    #[test]
    fn assign_voices_distinct_same_gender() {
        let content = "\
- Ana — una doctora
- Eva — una abogada

Ana : Hola.

Eva : Hola.
";
        let lines = parse_dialog(content);
        let genders = parse_character_genders(content, &TestLang);
        let voices = assign_voices(&lines, &genders, &TestLang, "test_slug");
        assert!(all_female().contains(&voices["Ana"]));
        assert!(all_female().contains(&voices["Eva"]));
        assert_ne!(voices["Ana"], voices["Eva"]);
    }

    #[test]
    fn assign_voices_falls_back_without_gender() {
        let lines = vec![
            DialogLine { speaker: "Unknown".into(), text: "Hi".into() },
            DialogLine { speaker: "Stranger".into(), text: "Hey".into() },
        ];
        let genders = HashMap::new();
        let voices = assign_voices(&lines, &genders, &TestLang, "test_slug");
        assert!(all_female().contains(&voices["Unknown"]));
        assert!(all_male().contains(&voices["Stranger"]));
    }

    #[test]
    fn single_speaker_gets_preferred_voice() {
        // A monologue should always use the preferred (Studio) voice.
        let content = "\
- Isabelle — la guide

Isabelle : Bienvenue.
";
        let lines = parse_dialog(content);
        let genders = parse_character_genders(content, &TestLang);
        let voices = assign_voices(&lines, &genders, &TestLang, "test_slug");
        assert_eq!(voices["Isabelle"].name, "xx-XX-Studio-F");
    }

    #[test]
    fn first_speaker_per_gender_gets_preferred_voice() {
        let content = "\
- María — una estudiante
- Carlos — un profesor

María : Hola.

Carlos : Buenos días.
";
        let lines = parse_dialog(content);
        let genders = parse_character_genders(content, &TestLang);
        let voices = assign_voices(&lines, &genders, &TestLang, "test_slug");
        assert_eq!(voices["María"].name, "xx-XX-Studio-F");
        assert_eq!(voices["Carlos"].name, "xx-XX-Studio-M");
    }

    #[test]
    fn second_speaker_same_gender_gets_fallback_voice() {
        let content = "\
- Ana — una doctora
- Eva — una abogada

Ana : Hola.

Eva : Hola.
";
        let lines = parse_dialog(content);
        let genders = parse_character_genders(content, &TestLang);
        let voices = assign_voices(&lines, &genders, &TestLang, "test_slug");
        // First female speaker gets preferred.
        assert_eq!(voices["Ana"].name, "xx-XX-Studio-F");
        // Second female speaker gets a fallback voice.
        assert!(TEST_FEMALE.contains(&voices["Eva"]),
            "second speaker should get a fallback voice, got: {}", voices["Eva"].name);
    }

    /// Same dialog slug → same voice assignments, every time.
    ///
    /// This is the contract that makes cached re-synth possible: if the
    /// dialog hasn't changed, running `assign_voices` again must yield
    /// the same voice per speaker — otherwise the `audio_hash` for each
    /// line would shift and every line would become a cache miss.
    #[test]
    fn same_slug_yields_same_voices() {
        let content = "\
- Ana — una doctora
- Eva — una abogada

Ana : Hola.

Eva : Hola.
";
        let lines = parse_dialog(content);
        let genders = parse_character_genders(content, &TestLang);
        let first  = assign_voices(&lines, &genders, &TestLang, "chapter_05");
        let second = assign_voices(&lines, &genders, &TestLang, "chapter_05");
        assert_eq!(first["Ana"], second["Ana"]);
        assert_eq!(first["Eva"], second["Eva"]);
    }

    /// Different dialog slugs permute the fallback pool — so two
    /// different dialogs with two female speakers don't both put the
    /// same fallback voice on their second-speaker. Provides variety
    /// across the repo without sacrificing reproducibility within a
    /// dialog.
    #[test]
    fn different_slugs_can_produce_different_fallback_voices() {
        let content = "\
- Ana — una doctora
- Eva — una abogada

Ana : Hola.

Eva : Hola.
";
        let lines = parse_dialog(content);
        let genders = parse_character_genders(content, &TestLang);

        // Collect Eva's voice across several distinct slugs. With a
        // 2-element fallback pool we expect to see both voices appear.
        let eva_voices: std::collections::HashSet<String> = ["a", "b", "c", "d", "e", "f", "g", "h"]
            .iter()
            .map(|slug| {
                assign_voices(&lines, &genders, &TestLang, slug)["Eva"].name.to_string()
            })
            .collect();

        // In a 2-voice fallback pool, 8 distinct slugs should cover both
        // voices with probability ~1 − 2 × 0.5⁸ ≈ 0.992. If this test
        // flakes, either the hash has stopped acting as a good
        // permutation or we got extraordinarily unlucky.
        assert_eq!(
            eva_voices.len(),
            2,
            "expected the second-speaker voice to vary across different \
             dialog slugs, but it was always: {eva_voices:?}",
        );
    }
}
