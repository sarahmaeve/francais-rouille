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
pub struct VoicePool {
    pub female: &'static [Voice],
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

/// Assign randomly-selected distinct voices to speakers based on detected
/// gender. The voice pools are shuffled once per call, so each program run
/// produces a different combination, but every line for a given character
/// uses the same voice throughout the dialog.
pub fn assign_voices(
    lines: &[DialogLine],
    genders: &HashMap<String, Gender>,
    lang: &dyn Language,
) -> HashMap<String, Voice> {
    let mut rng = rand::rng();
    let pool = lang.voice_pool();

    let mut female_pool: Vec<Voice> = pool.female.to_vec();
    let mut male_pool: Vec<Voice> = pool.male.to_vec();
    female_pool.shuffle(&mut rng);
    male_pool.shuffle(&mut rng);

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
                if (female_idx + male_idx) % 2 == 0 {
                    Gender::Female
                } else {
                    Gender::Male
                }
            });

        let voice = match gender {
            Gender::Female => {
                let v = female_pool[female_idx % female_pool.len()].clone();
                female_idx += 1;
                v
            }
            Gender::Male => {
                let v = male_pool[male_idx % male_pool.len()].clone();
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

    static TEST_FEMALE: &[Voice] = &[
        Voice { language_code: "xx-XX", name: "xx-XX-FemaleA" },
        Voice { language_code: "xx-XX", name: "xx-XX-FemaleB" },
    ];

    static TEST_MALE: &[Voice] = &[
        Voice { language_code: "xx-XX", name: "xx-XX-MaleA" },
        Voice { language_code: "xx-XX", name: "xx-XX-MaleB" },
    ];

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
            VoicePool { female: TEST_FEMALE, male: TEST_MALE }
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
        assert!(TEST_FEMALE.contains(&voices["María"]));
        assert!(TEST_MALE.contains(&voices["Carlos"]));
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
        assert!(TEST_FEMALE.contains(&voices["Ana"]));
        assert!(TEST_FEMALE.contains(&voices["Eva"]));
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
        assert!(TEST_FEMALE.contains(&voices["Unknown"]));
        assert!(TEST_MALE.contains(&voices["Stranger"]));
    }
}
