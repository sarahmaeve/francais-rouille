use site_gen::dialog::Gender;
use site_gen::language::{Language, Voice, VoicePool};

/// Spanish (es-US) language support.
pub struct Spanish;

// ── Female voices (es-US Premium) ──────────────────────────────────────

static PREFERRED_FEMALE: &[Voice] = &[];

static FEMALE: &[Voice] = &[
    Voice { language_code: "es-US", name: "es-US-Neural2-A" },
    Voice { language_code: "es-US", name: "es-US-Wavenet-A" },
    Voice { language_code: "es-US", name: "es-US-Chirp-HD-F" },
    Voice { language_code: "es-US", name: "es-US-Chirp-HD-O" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Achernar" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Aoede" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Autonoe" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Callirrhoe" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Despina" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Erinome" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Gacrux" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Kore" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Laomedeia" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Leda" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Pulcherrima" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Sulafat" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Vindemiatrix" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Zephyr" },
];

// ── Male voices (es-US Premium) ────────────────────────────────────────

static PREFERRED_MALE: &[Voice] = &[
    Voice { language_code: "es-US", name: "es-US-Studio-B" },
];

static MALE: &[Voice] = &[
    Voice { language_code: "es-US", name: "es-US-Neural2-B" },
    Voice { language_code: "es-US", name: "es-US-Neural2-C" },
    Voice { language_code: "es-US", name: "es-US-Wavenet-B" },
    Voice { language_code: "es-US", name: "es-US-Wavenet-C" },
    Voice { language_code: "es-US", name: "es-US-Polyglot-1" },
    Voice { language_code: "es-US", name: "es-US-Chirp-HD-D" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Achird" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Algenib" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Algieba" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Alnilam" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Charon" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Enceladus" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Fenrir" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Iapetus" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Orus" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Puck" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Rasalgethi" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Sadachbia" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Sadaltager" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Schedar" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Umbriel" },
    Voice { language_code: "es-US", name: "es-US-Chirp3-HD-Zubenelgenubi" },
];

impl Language for Spanish {
    fn code(&self) -> &'static str {
        "es-US"
    }

    fn detect_gender(&self, description: &str) -> Option<Gender> {
        let first_word = description.split_whitespace().next()?;

        match first_word {
            "una" | "la" => Some(Gender::Female),
            "un" | "el" => Some(Gender::Male),
            _ => None,
        }
    }

    fn voice_pool(&self) -> VoicePool {
        VoicePool {
            preferred_female: PREFERRED_FEMALE,
            female: FEMALE,
            preferred_male: PREFERRED_MALE,
            male: MALE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use site_gen::language::parse_character_genders;

    #[test]
    fn detects_all_spanish_articles() {
        let content = "\
- Ana — una doctora
- Pedro — un profesor
- Rosa — la directora
- Luis — el ingeniero";
        let genders = parse_character_genders(content, &Spanish);
        assert_eq!(genders["Ana"], Gender::Female);
        assert_eq!(genders["Pedro"], Gender::Male);
        assert_eq!(genders["Rosa"], Gender::Female);
        assert_eq!(genders["Luis"], Gender::Male);
    }

    #[test]
    fn returns_none_for_unrecognized_descriptions() {
        assert_eq!(Spanish.detect_gender("estudiante de medicina"), None);
        assert_eq!(Spanish.detect_gender("joven abogado"), None);
        assert_eq!(Spanish.detect_gender(""), None);
    }

    #[test]
    fn voice_names_match_language_code() {
        let pool = Spanish.voice_pool();
        for v in pool.female.iter().chain(pool.male.iter()) {
            assert!(
                v.name.starts_with("es-US-"),
                "voice name {} doesn't match language code {}",
                v.name,
                v.language_code
            );
            assert_eq!(v.language_code, "es-US");
        }
    }
}
