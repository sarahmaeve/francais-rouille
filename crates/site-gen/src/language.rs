use std::collections::HashMap;

use rand::seq::SliceRandom;

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

/// Build an ordered voice list: preferred voices first (in order), then
/// the remaining voices shuffled for variety.
fn build_voice_order(preferred: &[Voice], rest: &[Voice]) -> Vec<Voice> {
    let mut rng = rand::rng();
    let mut ordered: Vec<Voice> = preferred.to_vec();
    let mut fallback: Vec<Voice> = rest.to_vec();
    fallback.shuffle(&mut rng);
    ordered.extend(fallback);
    ordered
}

/// Assign distinct voices to speakers based on detected gender.
///
/// Preferred voices (e.g. Studio) are assigned first. When a dialog has
/// more speakers of one gender than there are preferred voices, additional
/// speakers receive randomly-selected voices from the rest of the pool.
/// Each character keeps the same voice throughout the dialog.
pub fn assign_voices(
    lines: &[DialogLine],
    genders: &HashMap<String, Gender>,
    lang: &dyn Language,
) -> HashMap<String, Voice> {
    let pool = lang.voice_pool();

    let female_voices = build_voice_order(pool.preferred_female, pool.female);
    let male_voices = build_voice_order(pool.preferred_male, pool.male);

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
        let voices = assign_voices(&lines, &genders, &TestLang);
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
        let voices = assign_voices(&lines, &genders, &TestLang);
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
        let voices = assign_voices(&lines, &genders, &TestLang);
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
        let voices = assign_voices(&lines, &genders, &TestLang);
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
        let voices = assign_voices(&lines, &genders, &TestLang);
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
        let voices = assign_voices(&lines, &genders, &TestLang);
        // First female speaker gets preferred.
        assert_eq!(voices["Ana"].name, "xx-XX-Studio-F");
        // Second female speaker gets a fallback voice.
        assert!(TEST_FEMALE.contains(&voices["Eva"]),
            "second speaker should get a fallback voice, got: {}", voices["Eva"].name);
    }
}
