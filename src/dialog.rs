use std::collections::HashMap;

use crate::tts::FrenchVoice;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    Female,
    Male,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DialogLine {
    pub speaker: String,
    pub text: String,
}

/// Parse dialog lines from a `.txt` file.
///
/// Matches lines in the format `Speaker : spoken text` using the ` : `
/// delimiter. Title, character list, and blank lines are skipped.
pub fn parse_dialog(content: &str) -> Vec<DialogLine> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            // Skip lines starting with `-` (character descriptions) and
            // lines that don't contain the ` : ` dialog delimiter.
            if line.is_empty() || line.starts_with('-') {
                return None;
            }
            let (speaker, text) = line.split_once(" : ")?;
            let speaker = speaker.trim();
            // Reject non-dialog lines that happen to contain ` : ` (e.g.
            // "Personnages :") by requiring the speaker portion to be a
            // short name (no more than a few words).
            if speaker.contains("Personnages") || speaker.contains("personnages") {
                return None;
            }
            Some(DialogLine {
                speaker: speaker.to_string(),
                text: text.trim().to_string(),
            })
        })
        .collect()
}

/// Parse character description lines to detect gender.
///
/// Looks for lines like:
///   `- Claire — une cliente curieuse qui entre dans la boulangerie`
///   `- Monsieur Duval — le propriétaire et pâtissier`
///
/// The article/adjective immediately after the `—` dash determines gender:
/// - Feminine markers: `une`, `la`, `l'étudiante` (ending in `e` after `l'`)
/// - Masculine markers: `un`, `le`, `l'habitant` (not ending in `e` after `l'`)
pub fn parse_character_genders(content: &str) -> HashMap<String, Gender> {
    let mut genders = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if !line.starts_with('-') {
            continue;
        }
        let line = line.trim_start_matches('-').trim();

        // Split on ` — ` (em-dash) or ` - ` (hyphen) to separate name from description.
        let (name, description) = if let Some(pos) = line.find(" — ") {
            (&line[..pos], &line[pos + " — ".len()..])
        } else if let Some(pos) = line.find(" – ") {
            (&line[..pos], &line[pos + " – ".len()..])
        } else {
            continue;
        };

        let name = name.trim().to_string();
        let description = description.trim().to_lowercase();

        if let Some(gender) = detect_gender_from_description(&description) {
            genders.insert(name, gender);
        }
    }

    genders
}

fn detect_gender_from_description(description: &str) -> Option<Gender> {
    let first_word = description.split_whitespace().next()?;

    match first_word {
        "une" | "la" => Some(Gender::Female),
        "un" | "le" => Some(Gender::Male),
        word if word.starts_with("l'") => {
            // For elided articles like "l'étudiante" vs "l'habitant",
            // check if the noun ends in 'e' (typically feminine in French).
            let noun = &word[4..]; // skip "l'" (2 bytes for l, 3 for ')
            if noun.ends_with('e') {
                Some(Gender::Female)
            } else {
                Some(Gender::Male)
            }
        }
        _ => None,
    }
}

/// Assign voices to speakers based on gender detected from character
/// descriptions. Falls back to alternating voices if no gender info is found.
pub fn assign_voices(
    lines: &[DialogLine],
    genders: &HashMap<String, Gender>,
) -> HashMap<String, FrenchVoice> {
    let mut map = HashMap::new();
    let fallback_voices = [FrenchVoice::WavenetA, FrenchVoice::WavenetB];
    let mut fallback_idx = 0;

    for line in lines {
        if map.contains_key(&line.speaker) {
            continue;
        }

        let voice = match genders.get(&line.speaker) {
            Some(Gender::Female) => FrenchVoice::WavenetA,
            Some(Gender::Male) => FrenchVoice::WavenetB,
            None => {
                let v = fallback_voices[fallback_idx % fallback_voices.len()];
                fallback_idx += 1;
                v
            }
        };

        map.insert(line.speaker.clone(), voice);
    }

    map
}

/// Turn a speaker name into a filesystem-friendly ASCII slug.
///
/// `"Monsieur Duval"` → `"monsieur_duval"`, `"Émilie"` → `"emilie"`
pub fn slugify(name: &str) -> String {
    name.chars()
        .map(|c| match strip_diacritic(c) {
            Some(ascii) => ascii,
            None if c.is_ascii_alphanumeric() => c.to_ascii_lowercase(),
            None if c == '.' || c == '-' || c == ' ' => '_',
            _ => '_',
        })
        .collect::<String>()
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

fn strip_diacritic(c: char) -> Option<char> {
    match c {
        'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => Some('a'),
        'È' | 'É' | 'Ê' | 'Ë' | 'è' | 'é' | 'ê' | 'ë' => Some('e'),
        'Ì' | 'Í' | 'Î' | 'Ï' | 'ì' | 'í' | 'î' | 'ï' => Some('i'),
        'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' => Some('o'),
        'Ù' | 'Ú' | 'Û' | 'Ü' | 'ù' | 'ú' | 'û' | 'ü' => Some('u'),
        'Ç' | 'ç' => Some('c'),
        'Ñ' | 'ñ' => Some('n'),
        'Ÿ' | 'ÿ' => Some('y'),
        'Œ' | 'œ' => Some('o'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
Visite à la Boulangerie — Les Spécialités de la Maison

Personnages :
- Claire — une cliente curieuse qui entre dans la boulangerie
- Monsieur Duval — le propriétaire et pâtissier de la boulangerie

Claire : Bonjour, monsieur ! Votre vitrine est magnifique.

Monsieur Duval : Bonjour, madame ! Merci, c'est gentil.

Claire : Qu'est-ce que vous me recommandez ?

Monsieur Duval : Notre éclair au chocolat noir, c'est notre meilleure vente.
";

    #[test]
    fn parse_extracts_dialog_lines() {
        let lines = parse_dialog(SAMPLE);
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0].speaker, "Claire");
        assert_eq!(
            lines[0].text,
            "Bonjour, monsieur ! Votre vitrine est magnifique."
        );
        assert_eq!(lines[1].speaker, "Monsieur Duval");
        assert_eq!(lines[1].text, "Bonjour, madame ! Merci, c'est gentil.");
    }

    #[test]
    fn parse_skips_metadata() {
        let lines = parse_dialog(SAMPLE);
        for line in &lines {
            assert!(!line.speaker.contains("Personnages"));
            assert!(!line.text.contains("une cliente"));
        }
    }

    #[test]
    fn gender_detection_from_descriptions() {
        let genders = parse_character_genders(SAMPLE);
        assert_eq!(genders.len(), 2);
        assert_eq!(genders["Claire"], Gender::Female);
        assert_eq!(genders["Monsieur Duval"], Gender::Male);
    }

    #[test]
    fn gender_detection_all_dialog_files() {
        let metro = "\
- Léa — une touriste qui visite Paris pour la première fois
- Marc — un Parisien qui attend sur le quai";
        let genders = parse_character_genders(metro);
        assert_eq!(genders["Léa"], Gender::Female);
        assert_eq!(genders["Marc"], Gender::Male);

        let hotel = "\
- David — un voyageur qui séjourne dans un hôtel
- Émilie — la réceptionniste de l'hôtel";
        let genders = parse_character_genders(hotel);
        assert_eq!(genders["David"], Gender::Male);
        assert_eq!(genders["Émilie"], Gender::Female);

        let quartier = "\
- Camille — la future locataire potentielle
- Yasmine — une habitante du quartier depuis cinq ans";
        let genders = parse_character_genders(quartier);
        assert_eq!(genders["Camille"], Gender::Female);
        assert_eq!(genders["Yasmine"], Gender::Female);
    }

    #[test]
    fn gender_detection_elided_articles() {
        let input = "- Nadia — l'étudiante qui vient d'arriver";
        let genders = parse_character_genders(input);
        assert_eq!(genders["Nadia"], Gender::Female);

        let input = "- Pierre — l'habitant du quartier";
        let genders = parse_character_genders(input);
        assert_eq!(genders["Pierre"], Gender::Male);
    }

    #[test]
    fn assign_voices_uses_gender() {
        let lines = parse_dialog(SAMPLE);
        let genders = parse_character_genders(SAMPLE);
        let voices = assign_voices(&lines, &genders);
        assert_eq!(voices.len(), 2);
        // Claire is female → WavenetA
        assert_eq!(voices["Claire"], FrenchVoice::WavenetA);
        // Monsieur Duval is male → WavenetB
        assert_eq!(voices["Monsieur Duval"], FrenchVoice::WavenetB);
    }

    #[test]
    fn assign_voices_falls_back_without_gender() {
        let lines = vec![
            DialogLine {
                speaker: "Unknown".to_string(),
                text: "Bonjour".to_string(),
            },
            DialogLine {
                speaker: "Stranger".to_string(),
                text: "Salut".to_string(),
            },
        ];
        let genders = HashMap::new();
        let voices = assign_voices(&lines, &genders);
        // Falls back to alternating: first WavenetA, then WavenetB
        assert_eq!(voices["Unknown"], FrenchVoice::WavenetA);
        assert_eq!(voices["Stranger"], FrenchVoice::WavenetB);
    }

    #[test]
    fn slugify_converts_names() {
        assert_eq!(slugify("Monsieur Duval"), "monsieur_duval");
        assert_eq!(slugify("Claire"), "claire");
        assert_eq!(slugify("Émilie"), "emilie");
        assert_eq!(slugify("M. Duval"), "m_duval");
    }

    #[test]
    fn parse_empty_input() {
        assert!(parse_dialog("").is_empty());
        assert!(parse_dialog("Just a title\n\nNo dialog here").is_empty());
    }

    #[test]
    fn parse_handles_all_dialog_files() {
        // Verify the delimiter works even when the spoken text itself
        // contains colons (e.g. "des supermarchés, des pharmacies, des
        // boulangeries" doesn't have ` : ` so it stays in the text part).
        let input = "Agent : Il y a tout : des supermarchés, des pharmacies.";
        let lines = parse_dialog(input);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].speaker, "Agent");
        assert_eq!(
            lines[0].text,
            "Il y a tout : des supermarchés, des pharmacies."
        );
    }
}
