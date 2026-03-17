use site_gen::dialog::Gender;
use site_gen::language::{Language, Voice, VoicePool};

/// French (fr-FR) language support.
pub struct French;

// ── Female voices ──────────────────────────────────────────────────────

static FEMALE: &[Voice] = &[
    Voice { language_code: "fr-FR", name: "fr-FR-Studio-A" },
    Voice { language_code: "fr-FR", name: "fr-FR-Neural2-F" },
    Voice { language_code: "fr-FR", name: "fr-FR-Wavenet-F" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp-HD-F" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp-HD-O" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Achernar" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Aoede" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Autonoe" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Callirrhoe" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Despina" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Erinome" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Gacrux" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Kore" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Laomedeia" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Leda" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Pulcherrima" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Sulafat" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Vindemiatrix" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Zephyr" },
];

// ── Male voices ────────────────────────────────────────────────────────

static MALE: &[Voice] = &[
    Voice { language_code: "fr-FR", name: "fr-FR-Studio-D" },
    Voice { language_code: "fr-FR", name: "fr-FR-Neural2-G" },
    Voice { language_code: "fr-FR", name: "fr-FR-Wavenet-G" },
    Voice { language_code: "fr-FR", name: "fr-FR-Polyglot-1" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp-HD-D" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Achird" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Algenib" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Algieba" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Alnilam" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Charon" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Enceladus" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Fenrir" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Iapetus" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Orus" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Puck" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Rasalgethi" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Sadachbia" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Sadaltager" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Schedar" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Umbriel" },
    Voice { language_code: "fr-FR", name: "fr-FR-Chirp3-HD-Zubenelgenubi" },
];

impl Language for French {
    fn code(&self) -> &'static str {
        "fr-FR"
    }

    fn detect_gender(&self, description: &str) -> Option<Gender> {
        let first_word = description.split_whitespace().next()?;

        match first_word {
            "une" | "la" => Some(Gender::Female),
            "un" | "le" => Some(Gender::Male),
            word if word.starts_with("l\u{2019}") || word.starts_with("l'") => {
                // For elided articles like "l'étudiante" vs "l'habitant",
                // check if the noun ends in 'e' (typically feminine in French).
                // Handle both typographic (') and ASCII (') apostrophes.
                let noun = if let Some(rest) = word.strip_prefix("l\u{2019}") {
                    rest
                } else {
                    &word[2..] // "l'"
                };
                if noun.ends_with('e') {
                    Some(Gender::Female)
                } else {
                    Some(Gender::Male)
                }
            }
            _ => None,
        }
    }

    fn voice_pool(&self) -> VoicePool {
        VoicePool { female: FEMALE, male: MALE }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use site_gen::language::parse_character_genders;

    #[test]
    fn detects_all_french_articles() {
        let content = "\
- Léa — une touriste
- Marc — un Parisien
- Émilie — la réceptionniste
- David — le voyageur";
        let genders = parse_character_genders(content, &French);
        assert_eq!(genders["Léa"], Gender::Female);
        assert_eq!(genders["Marc"], Gender::Male);
        assert_eq!(genders["Émilie"], Gender::Female);
        assert_eq!(genders["David"], Gender::Male);
    }

    #[test]
    fn detects_gender_from_elided_articles() {
        // Typographic apostrophe (U+2019)
        let input = "- Nadia — l\u{2019}étudiante qui vient d\u{2019}arriver";
        let genders = parse_character_genders(input, &French);
        assert_eq!(genders["Nadia"], Gender::Female);

        // ASCII apostrophe
        let input = "- Pierre — l'habitant du quartier";
        let genders = parse_character_genders(input, &French);
        assert_eq!(genders["Pierre"], Gender::Male);
    }

    #[test]
    fn returns_none_for_unrecognized_descriptions() {
        // Descriptions that don't start with a gendered article.
        assert_eq!(French.detect_gender("professeur de musique"), None);
        assert_eq!(French.detect_gender("jeune étudiant"), None);
        assert_eq!(French.detect_gender(""), None);
    }

    #[test]
    fn voice_names_match_language_code() {
        let pool = French.voice_pool();
        for v in pool.female.iter().chain(pool.male.iter()) {
            assert!(
                v.name.starts_with("fr-FR-"),
                "voice name {} doesn't match language code {}",
                v.name,
                v.language_code
            );
            assert_eq!(v.language_code, "fr-FR");
        }
    }
}
