use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

use crate::dialog::{self, DialogLine, Language, Voice};

/// Audio encoding format for TTS output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Mp3,
    OggOpus,
}

impl AudioFormat {
    /// The encoding string expected by the Google TTS API.
    pub fn api_encoding(self) -> &'static str {
        match self {
            Self::Mp3 => "MP3",
            Self::OggOpus => "OGG_OPUS",
        }
    }

    /// File extension (without the leading dot).
    pub fn extension(self) -> &'static str {
        match self {
            Self::Mp3 => "mp3",
            Self::OggOpus => "ogg",
        }
    }

    /// Parse from a CLI string like "mp3" or "ogg".
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "mp3" => Some(Self::Mp3),
            "ogg" | "ogg_opus" | "opus" => Some(Self::OggOpus),
            _ => None,
        }
    }
}

const GOOGLE_TTS_URL: &str = "https://texttospeech.googleapis.com/v1/text:synthesize";

/// Milliseconds of silence inserted between dialog lines when combining.
const PAUSE_BETWEEN_LINES_MS: u32 = 750;

#[derive(Error, Debug)]
pub enum TtsError {
    #[error("GOOGLE_TTS_API_KEY environment variable not set")]
    MissingApiKey,
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("API returned error status {status}: {body}")]
    ApiError { status: u16, body: String },
    #[error("failed to decode audio content: {0}")]
    Decode(#[from] base64::DecodeError),
    #[error("file I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("no dialog lines found in input")]
    EmptyDialog,
}

#[derive(Serialize)]
struct SynthesizeRequest<'a> {
    input: SynthesisInput<'a>,
    voice: VoiceSelection<'a>,
    #[serde(rename = "audioConfig")]
    audio_config: AudioConfig,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SynthesisInput<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ssml: Option<&'a str>,
}

#[derive(Serialize)]
struct VoiceSelection<'a> {
    #[serde(rename = "languageCode")]
    language_code: &'a str,
    name: &'a str,
}

#[derive(Serialize)]
struct AudioConfig {
    #[serde(rename = "audioEncoding")]
    audio_encoding: &'static str,
}

#[derive(Deserialize)]
struct SynthesizeResponse {
    #[serde(rename = "audioContent")]
    audio_content: String,
}

/// Result of synthesizing an entire dialog.
pub struct DialogAudio {
    /// One audio file per dialog line, in order.
    pub lines: Vec<LineAudio>,
    /// All lines concatenated with silence between them (empty if not requested).
    pub combined: Vec<u8>,
}

pub struct LineAudio {
    pub index: usize,
    pub speaker: String,
    pub text: String,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct GoogleTts {
    client: Client,
    api_key: String,
}

impl GoogleTts {
    /// Creates a new client, reading `GOOGLE_TTS_API_KEY` from the environment.
    pub fn from_env() -> Result<Self, TtsError> {
        let api_key = std::env::var("GOOGLE_TTS_API_KEY").map_err(|_| TtsError::MissingApiKey)?;
        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }

    /// Synthesizes plain `text` using the given voice and returns raw audio bytes.
    ///
    /// Roman numerals in royal names and century notation are converted to
    /// spoken French before sending to the API (e.g. "Guillaume IX" →
    /// "Guillaume neuf", "XIIIe siècle" → "treizième siècle").
    pub async fn synthesize(
        &self,
        text: &str,
        voice: &Voice,
        format: AudioFormat,
    ) -> Result<Vec<u8>, TtsError> {
        let normalized = normalize_roman_numerals(text);
        self.synthesize_input(
            SynthesisInput {
                text: Some(&normalized),
                ssml: None,
            },
            voice,
            format,
        )
        .await
    }

    /// Synthesizes SSML content and returns raw audio bytes.
    pub async fn synthesize_ssml(
        &self,
        ssml: &str,
        voice: &Voice,
        format: AudioFormat,
    ) -> Result<Vec<u8>, TtsError> {
        self.synthesize_input(
            SynthesisInput {
                text: None,
                ssml: Some(ssml),
            },
            voice,
            format,
        )
        .await
    }

    async fn synthesize_input(
        &self,
        input: SynthesisInput<'_>,
        voice: &Voice,
        format: AudioFormat,
    ) -> Result<Vec<u8>, TtsError> {
        let body = SynthesizeRequest {
            input,
            voice: VoiceSelection {
                language_code: voice.language_code,
                name: voice.name,
            },
            audio_config: AudioConfig {
                audio_encoding: format.api_encoding(),
            },
        };

        let url = format!("{}?key={}", GOOGLE_TTS_URL, self.api_key);
        let resp = self.client.post(&url).json(&body).send().await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(TtsError::ApiError {
                status: status.as_u16(),
                body,
            });
        }

        let synth: SynthesizeResponse = resp.json().await?;
        let bytes = base64::engine::general_purpose::STANDARD.decode(&synth.audio_content)?;
        Ok(bytes)
    }

    /// Generate a short silence using SSML `<break>`.
    async fn synthesize_silence(
        &self,
        ms: u32,
        voice: &Voice,
        format: AudioFormat,
    ) -> Result<Vec<u8>, TtsError> {
        let ssml = format!(
            "<speak><break time=\"{}ms\"/></speak>",
            ms
        );
        self.synthesize_ssml(&ssml, voice, format).await
    }

    /// Synthesizes `text` and writes the audio to `output_path`.
    pub async fn synthesize_to_file(
        &self,
        text: &str,
        voice: &Voice,
        format: AudioFormat,
        output_path: &Path,
    ) -> Result<(), TtsError> {
        let bytes = self.synthesize(text, voice, format).await?;
        std::fs::write(output_path, &bytes)?;
        Ok(())
    }

    /// Synthesize an entire dialog file using a language-specific
    /// implementation for gender detection and voice selection.
    ///
    /// Returns per-line audio and, when `combined` is true, a single
    /// concatenated file with silence gaps between lines.
    pub async fn synthesize_dialog(
        &self,
        content: &str,
        format: AudioFormat,
        combined: bool,
        lang: &dyn Language,
    ) -> Result<DialogAudio, TtsError> {
        let parsed = dialog::parse_dialog(content);
        if parsed.is_empty() {
            return Err(TtsError::EmptyDialog);
        }

        let genders = dialog::parse_character_genders(content, lang);
        let voice_map = dialog::assign_voices(&parsed, &genders, lang);

        // Pre-generate the silence segment once (only needed for combined).
        let first_voice = &voice_map[&parsed[0].speaker];
        let silence = if combined {
            Some(
                self.synthesize_silence(PAUSE_BETWEEN_LINES_MS, first_voice, format)
                    .await?,
            )
        } else {
            None
        };

        let mut lines = Vec::with_capacity(parsed.len());
        let mut combined_data = Vec::new();

        for (i, DialogLine { speaker, text }) in parsed.into_iter().enumerate() {
            let voice = &voice_map[&speaker];
            let audio = self.synthesize(&text, voice, format).await?;

            if let Some(ref silence) = silence {
                if i > 0 {
                    combined_data.extend_from_slice(silence);
                }
                combined_data.extend_from_slice(&audio);
            }

            lines.push(LineAudio {
                index: i + 1,
                speaker,
                text,
                data: audio,
            });
        }

        Ok(DialogAudio { lines, combined: combined_data })
    }
}

/// Parse a Roman numeral string (I through XX) into its integer value.
fn parse_roman(s: &str) -> Option<u32> {
    match s {
        "I" => Some(1),
        "II" => Some(2),
        "III" => Some(3),
        "IV" => Some(4),
        "V" => Some(5),
        "VI" => Some(6),
        "VII" => Some(7),
        "VIII" => Some(8),
        "IX" => Some(9),
        "X" => Some(10),
        "XI" => Some(11),
        "XII" => Some(12),
        "XIII" => Some(13),
        "XIV" => Some(14),
        "XV" => Some(15),
        "XVI" => Some(16),
        "XVII" => Some(17),
        "XVIII" => Some(18),
        "XIX" => Some(19),
        "XX" => Some(20),
        _ => None,
    }
}

/// French cardinal number words (1–20).
const CARDINALS: &[&str] = &[
    "", "un", "deux", "trois", "quatre", "cinq", "six", "sept", "huit",
    "neuf", "dix", "onze", "douze", "treize", "quatorze", "quinze",
    "seize", "dix-sept", "dix-huit", "dix-neuf", "vingt",
];

/// French ordinal number words (1–20).
const ORDINALS: &[&str] = &[
    "", "premier", "deuxième", "troisième", "quatrième", "cinquième",
    "sixième", "septième", "huitième", "neuvième", "dixième", "onzième",
    "douzième", "treizième", "quatorzième", "quinzième", "seizième",
    "dix-septième", "dix-huitième", "dix-neuvième", "vingtième",
];

/// Returns true if `c` is a Roman numeral character.
fn is_roman(c: char) -> bool {
    matches!(c, 'I' | 'V' | 'X')
}

/// Normalize Roman numerals in French text for TTS.
///
/// Handles three patterns:
/// - "Ier" / "Ière" → "premier" / "première" (e.g. "François Ier")
/// - Century notation: "XIIe" / "XIIIe" → "douzième" / "treizième"
/// - Royal/papal names: "Guillaume IX" → "Guillaume neuf"
fn normalize_roman_numerals(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut result: Vec<String> = Vec::with_capacity(words.len());

    let mut i = 0;
    while i < words.len() {
        let word = words[i];

        // Pattern 1: "Ier" or "Ière" (standalone word) → premier/première
        if word == "Ier" {
            result.push("premier".into());
            i += 1;
            continue;
        }
        if word == "Ière" {
            result.push("première".into());
            i += 1;
            continue;
        }

        // Pattern 2: Century notation — "XIIe", "XIIIe", "XIe", etc.
        // Also handles "XIIe," (with trailing punctuation).
        if word.len() >= 3 {
            // Find where the Roman part ends.
            let roman_end = word.chars().take_while(|c| is_roman(*c)).count();
            if roman_end >= 1 {
                let roman_part = &word[..roman_end];
                let suffix = &word[roman_end..];
                // Check for ordinal suffix (e, è, ème) possibly followed by punctuation.
                let is_ordinal = suffix.starts_with("e ")
                    || suffix == "e"
                    || suffix.starts_with("e,")
                    || suffix.starts_with("e.")
                    || suffix.starts_with("e;")
                    || suffix.starts_with("e\u{00A0}")
                    || suffix.starts_with("è")
                    || suffix.starts_with("ème");

                if is_ordinal {
                    if let Some(val) = parse_roman(roman_part) {
                        if (val as usize) < ORDINALS.len() {
                            // Keep the punctuation after "e"/"è"/"ème".
                            let punct_start = if suffix.starts_with("ème") {
                                3
                            } else if suffix.starts_with("è") {
                                suffix.chars().next().map(|c| c.len_utf8()).unwrap_or(1)
                            } else {
                                1
                            };
                            let trailing = &suffix[punct_start..];
                            result.push(format!(
                                "{}{}",
                                ORDINALS[val as usize],
                                trailing,
                            ));
                            i += 1;
                            continue;
                        }
                    }
                }
            }
        }

        // Pattern 3: Roman numeral after a capitalized word (royal name).
        // e.g. "Guillaume IX", "Raimond V", "Innocent III"
        // Check if current word is all Roman numerals and previous word
        // starts with uppercase (a name).
        if !word.is_empty()
            && word.chars().all(is_roman)
            && i > 0
            && words[i - 1]
                .chars()
                .next()
                .is_some_and(|c| c.is_uppercase())
        {
            if let Some(val) = parse_roman(word) {
                if (val as usize) < CARDINALS.len() {
                    result.push(CARDINALS[val as usize].into());
                    i += 1;
                    continue;
                }
            }
        }

        // Default: keep the word as-is.
        result.push(word.to_string());
        i += 1;
    }

    result.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_voice() -> Voice {
        Voice {
            language_code: "fr-FR",
            name: "fr-FR-Studio-A",
        }
    }

    #[test]
    fn from_env_fails_without_key() {
        std::env::remove_var("GOOGLE_TTS_API_KEY");
        let result = GoogleTts::from_env();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TtsError::MissingApiKey));
    }

    #[test]
    fn request_body_serializes_text_input() {
        let voice = test_voice();
        let req = SynthesizeRequest {
            input: SynthesisInput {
                text: Some("Bonjour le monde"),
                ssml: None,
            },
            voice: VoiceSelection {
                language_code: voice.language_code,
                name: voice.name,
            },
            audio_config: AudioConfig {
                audio_encoding: AudioFormat::Mp3.api_encoding(),
            },
        };

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["input"]["text"], "Bonjour le monde");
        assert!(json["input"].get("ssml").is_none());
        assert_eq!(json["voice"]["languageCode"], "fr-FR");
        assert_eq!(json["voice"]["name"], "fr-FR-Studio-A");
        assert_eq!(json["audioConfig"]["audioEncoding"], "MP3");
    }

    #[test]
    fn request_body_serializes_ssml_input() {
        let ssml = "<speak><break time=\"750ms\"/></speak>";
        let voice = test_voice();
        let req = SynthesizeRequest {
            input: SynthesisInput {
                text: None,
                ssml: Some(ssml),
            },
            voice: VoiceSelection {
                language_code: voice.language_code,
                name: voice.name,
            },
            audio_config: AudioConfig {
                audio_encoding: AudioFormat::Mp3.api_encoding(),
            },
        };

        let json = serde_json::to_value(&req).unwrap();
        assert!(json["input"].get("text").is_none());
        assert_eq!(json["input"]["ssml"], ssml);
    }

    #[test]
    fn audio_format_from_str() {
        assert_eq!(AudioFormat::from_str("mp3"), Some(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_str("MP3"), Some(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_str("ogg"), Some(AudioFormat::OggOpus));
        assert_eq!(AudioFormat::from_str("OGG_OPUS"), Some(AudioFormat::OggOpus));
        assert_eq!(AudioFormat::from_str("opus"), Some(AudioFormat::OggOpus));
        assert_eq!(AudioFormat::from_str("wav"), None);
    }

    #[test]
    fn roman_royal_names() {
        assert_eq!(
            normalize_roman_numerals("Guillaume IX d'Aquitaine"),
            "Guillaume neuf d'Aquitaine"
        );
        assert_eq!(
            normalize_roman_numerals("le comte Raimond V tenait"),
            "le comte Raimond cinq tenait"
        );
        assert_eq!(
            normalize_roman_numerals("le pape Innocent III a lancé"),
            "le pape Innocent trois a lancé"
        );
        assert_eq!(
            normalize_roman_numerals("Louis XIV était roi"),
            "Louis quatorze était roi"
        );
    }

    #[test]
    fn roman_first_monarch() {
        assert_eq!(
            normalize_roman_numerals("François Ier a signé"),
            "François premier a signé"
        );
        assert_eq!(
            normalize_roman_numerals("Élisabeth Ière d'Angleterre"),
            "Élisabeth première d'Angleterre"
        );
    }

    #[test]
    fn roman_century_notation() {
        assert_eq!(
            normalize_roman_numerals("au XIIe siècle"),
            "au douzième siècle"
        );
        assert_eq!(
            normalize_roman_numerals("du XIIIe siècle"),
            "du treizième siècle"
        );
        assert_eq!(
            normalize_roman_numerals("le XIXe siècle"),
            "le dix-neuvième siècle"
        );
        assert_eq!(
            normalize_roman_numerals("XIIe et XIIIe siècle"),
            "douzième et treizième siècle"
        );
    }

    #[test]
    fn roman_century_with_punctuation() {
        assert_eq!(
            normalize_roman_numerals("au XIIIe, les Cathares"),
            "au treizième, les Cathares"
        );
    }

    #[test]
    fn roman_preserves_plain_text() {
        let plain = "Bonjour, comment allez-vous ?";
        assert_eq!(normalize_roman_numerals(plain), plain);
    }

    #[test]
    fn roman_does_not_convert_standalone_i() {
        // A standalone "I" not preceded by a capitalized name should not convert.
        // In French text this is unlikely, but ensure no false positive.
        let text = "et I et II sont des chiffres";
        assert_eq!(
            normalize_roman_numerals(text),
            "et I et II sont des chiffres"
        );
    }

    #[test]
    fn roman_multiple_in_one_sentence() {
        assert_eq!(
            normalize_roman_numerals("Henri IV et Louis XVI au XVIIe siècle"),
            "Henri quatre et Louis seize au dix-septième siècle"
        );
    }

    #[test]
    fn request_body_serializes_ogg_encoding() {
        let voice = test_voice();
        let req = SynthesizeRequest {
            input: SynthesisInput {
                text: Some("Bonjour"),
                ssml: None,
            },
            voice: VoiceSelection {
                language_code: voice.language_code,
                name: voice.name,
            },
            audio_config: AudioConfig {
                audio_encoding: AudioFormat::OggOpus.api_encoding(),
            },
        };

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["audioConfig"]["audioEncoding"], "OGG_OPUS");
    }
}
