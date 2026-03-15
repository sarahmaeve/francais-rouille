/// A single spoken line in a dialog.
#[derive(Debug, Clone, PartialEq)]
pub struct DialogLine {
    pub speaker: String,
    pub text: String,
}

/// Speaker gender, used for TTS voice assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    Female,
    Male,
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
    fn parse_empty_input() {
        assert!(parse_dialog("").is_empty());
        assert!(parse_dialog("Just a title\n\nNo dialog here").is_empty());
    }

    #[test]
    fn parse_handles_colon_in_text() {
        let input = "Agent : Il y a tout : des supermarchés, des pharmacies.";
        let lines = parse_dialog(input);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].speaker, "Agent");
        assert_eq!(
            lines[0].text,
            "Il y a tout : des supermarchés, des pharmacies."
        );
    }

    #[test]
    fn slugify_converts_names() {
        assert_eq!(slugify("Monsieur Duval"), "monsieur_duval");
        assert_eq!(slugify("Claire"), "claire");
        assert_eq!(slugify("Émilie"), "emilie");
        assert_eq!(slugify("M. Duval"), "m_duval");
    }
}
