use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

use crate::dialog::{self, DialogLine};

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

/// Premium fr-FR voices from Google Cloud Text-to-Speech, segmented by gender.
///
/// See: https://docs.cloud.google.com/text-to-speech/docs/list-voices-and-types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrenchVoice {
    // ── Female voices ────────────────────────────────────────────
    // Studio
    StudioA,
    // Neural2
    Neural2F,
    // Wavenet
    WavenetF,
    // Chirp HD
    ChirpHdF,
    ChirpHdO,
    // Chirp3 HD
    Chirp3HdAchernar,
    Chirp3HdAoede,
    Chirp3HdAutonoe,
    Chirp3HdCallirrhoe,
    Chirp3HdDespina,
    Chirp3HdErinome,
    Chirp3HdGacrux,
    Chirp3HdKore,
    Chirp3HdLaomedeia,
    Chirp3HdLeda,
    Chirp3HdPulcherrima,
    Chirp3HdSulafat,
    Chirp3HdVindemiatrix,
    Chirp3HdZephyr,

    // ── Male voices ──────────────────────────────────────────────
    // Studio
    StudioD,
    // Neural2
    Neural2G,
    // Wavenet
    WavenetG,
    // Polyglot
    Polyglot1,
    // Chirp HD
    ChirpHdD,
    // Chirp3 HD
    Chirp3HdAchird,
    Chirp3HdAlgenib,
    Chirp3HdAlgieba,
    Chirp3HdAlnilam,
    Chirp3HdCharon,
    Chirp3HdEnceladus,
    Chirp3HdFenrir,
    Chirp3HdIapetus,
    Chirp3HdOrus,
    Chirp3HdPuck,
    Chirp3HdRasalgethi,
    Chirp3HdSadachbia,
    Chirp3HdSadaltager,
    Chirp3HdSchedar,
    Chirp3HdUmbriel,
    Chirp3HdZubenelgenubi,
}

impl FrenchVoice {
    fn name(self) -> &'static str {
        match self {
            // Female
            Self::StudioA => "fr-FR-Studio-A",
            Self::Neural2F => "fr-FR-Neural2-F",
            Self::WavenetF => "fr-FR-Wavenet-F",
            Self::ChirpHdF => "fr-FR-Chirp-HD-F",
            Self::ChirpHdO => "fr-FR-Chirp-HD-O",
            Self::Chirp3HdAchernar => "fr-FR-Chirp3-HD-Achernar",
            Self::Chirp3HdAoede => "fr-FR-Chirp3-HD-Aoede",
            Self::Chirp3HdAutonoe => "fr-FR-Chirp3-HD-Autonoe",
            Self::Chirp3HdCallirrhoe => "fr-FR-Chirp3-HD-Callirrhoe",
            Self::Chirp3HdDespina => "fr-FR-Chirp3-HD-Despina",
            Self::Chirp3HdErinome => "fr-FR-Chirp3-HD-Erinome",
            Self::Chirp3HdGacrux => "fr-FR-Chirp3-HD-Gacrux",
            Self::Chirp3HdKore => "fr-FR-Chirp3-HD-Kore",
            Self::Chirp3HdLaomedeia => "fr-FR-Chirp3-HD-Laomedeia",
            Self::Chirp3HdLeda => "fr-FR-Chirp3-HD-Leda",
            Self::Chirp3HdPulcherrima => "fr-FR-Chirp3-HD-Pulcherrima",
            Self::Chirp3HdSulafat => "fr-FR-Chirp3-HD-Sulafat",
            Self::Chirp3HdVindemiatrix => "fr-FR-Chirp3-HD-Vindemiatrix",
            Self::Chirp3HdZephyr => "fr-FR-Chirp3-HD-Zephyr",
            // Male
            Self::StudioD => "fr-FR-Studio-D",
            Self::Neural2G => "fr-FR-Neural2-G",
            Self::WavenetG => "fr-FR-Wavenet-G",
            Self::Polyglot1 => "fr-FR-Polyglot-1",
            Self::ChirpHdD => "fr-FR-Chirp-HD-D",
            Self::Chirp3HdAchird => "fr-FR-Chirp3-HD-Achird",
            Self::Chirp3HdAlgenib => "fr-FR-Chirp3-HD-Algenib",
            Self::Chirp3HdAlgieba => "fr-FR-Chirp3-HD-Algieba",
            Self::Chirp3HdAlnilam => "fr-FR-Chirp3-HD-Alnilam",
            Self::Chirp3HdCharon => "fr-FR-Chirp3-HD-Charon",
            Self::Chirp3HdEnceladus => "fr-FR-Chirp3-HD-Enceladus",
            Self::Chirp3HdFenrir => "fr-FR-Chirp3-HD-Fenrir",
            Self::Chirp3HdIapetus => "fr-FR-Chirp3-HD-Iapetus",
            Self::Chirp3HdOrus => "fr-FR-Chirp3-HD-Orus",
            Self::Chirp3HdPuck => "fr-FR-Chirp3-HD-Puck",
            Self::Chirp3HdRasalgethi => "fr-FR-Chirp3-HD-Rasalgethi",
            Self::Chirp3HdSadachbia => "fr-FR-Chirp3-HD-Sadachbia",
            Self::Chirp3HdSadaltager => "fr-FR-Chirp3-HD-Sadaltager",
            Self::Chirp3HdSchedar => "fr-FR-Chirp3-HD-Schedar",
            Self::Chirp3HdUmbriel => "fr-FR-Chirp3-HD-Umbriel",
            Self::Chirp3HdZubenelgenubi => "fr-FR-Chirp3-HD-Zubenelgenubi",
        }
    }

    /// Default female voices, ordered by preference (highest quality first).
    pub const FEMALE: &[FrenchVoice] = &[
        Self::StudioA,
        Self::Neural2F,
        Self::WavenetF,
        Self::ChirpHdF,
        Self::ChirpHdO,
        Self::Chirp3HdAchernar,
        Self::Chirp3HdAoede,
        Self::Chirp3HdAutonoe,
        Self::Chirp3HdCallirrhoe,
        Self::Chirp3HdDespina,
        Self::Chirp3HdErinome,
        Self::Chirp3HdGacrux,
        Self::Chirp3HdKore,
        Self::Chirp3HdLaomedeia,
        Self::Chirp3HdLeda,
        Self::Chirp3HdPulcherrima,
        Self::Chirp3HdSulafat,
        Self::Chirp3HdVindemiatrix,
        Self::Chirp3HdZephyr,
    ];

    /// Default male voices, ordered by preference (highest quality first).
    pub const MALE: &[FrenchVoice] = &[
        Self::StudioD,
        Self::Neural2G,
        Self::WavenetG,
        Self::Polyglot1,
        Self::ChirpHdD,
        Self::Chirp3HdAchird,
        Self::Chirp3HdAlgenib,
        Self::Chirp3HdAlgieba,
        Self::Chirp3HdAlnilam,
        Self::Chirp3HdCharon,
        Self::Chirp3HdEnceladus,
        Self::Chirp3HdFenrir,
        Self::Chirp3HdIapetus,
        Self::Chirp3HdOrus,
        Self::Chirp3HdPuck,
        Self::Chirp3HdRasalgethi,
        Self::Chirp3HdSadachbia,
        Self::Chirp3HdSadaltager,
        Self::Chirp3HdSchedar,
        Self::Chirp3HdUmbriel,
        Self::Chirp3HdZubenelgenubi,
    ];
}

#[derive(Serialize)]
struct SynthesizeRequest<'a> {
    input: SynthesisInput<'a>,
    voice: VoiceSelection,
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
struct VoiceSelection {
    #[serde(rename = "languageCode")]
    language_code: &'static str,
    name: &'static str,
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
    /// All lines concatenated with silence between them.
    pub combined: Vec<u8>,
    /// The format of the audio data.
    pub format: AudioFormat,
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

    /// Synthesizes plain `text` as French speech and returns raw audio bytes.
    pub async fn synthesize(
        &self,
        text: &str,
        voice: FrenchVoice,
        format: AudioFormat,
    ) -> Result<Vec<u8>, TtsError> {
        self.synthesize_input(
            SynthesisInput {
                text: Some(text),
                ssml: None,
            },
            voice,
            format,
        )
        .await
    }

    /// Synthesizes SSML content and returns raw audio bytes.
    async fn synthesize_ssml(
        &self,
        ssml: &str,
        voice: FrenchVoice,
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
        voice: FrenchVoice,
        format: AudioFormat,
    ) -> Result<Vec<u8>, TtsError> {
        let body = SynthesizeRequest {
            input,
            voice: VoiceSelection {
                language_code: "fr-FR",
                name: voice.name(),
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
        voice: FrenchVoice,
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
        voice: FrenchVoice,
        format: AudioFormat,
        output_path: &Path,
    ) -> Result<(), TtsError> {
        let bytes = self.synthesize(text, voice, format).await?;
        std::fs::write(output_path, &bytes)?;
        Ok(())
    }

    /// Synthesize an entire dialog file.
    ///
    /// Returns per-line audio and a single combined file with silence
    /// gaps between lines. For MP3, individual frames are independently
    /// decodable so concatenation produces a valid stream. For OGG_OPUS,
    /// the combined file is a simple concatenation (each segment is a
    /// standalone Ogg stream).
    pub async fn synthesize_dialog(
        &self,
        content: &str,
        format: AudioFormat,
    ) -> Result<DialogAudio, TtsError> {
        let parsed = dialog::parse_dialog(content);
        if parsed.is_empty() {
            return Err(TtsError::EmptyDialog);
        }

        let genders = dialog::parse_character_genders(content);
        let voice_map: HashMap<String, FrenchVoice> =
            dialog::assign_voices(&parsed, &genders);

        // Pre-generate the silence segment once (using the first voice).
        let silence = self
            .synthesize_silence(PAUSE_BETWEEN_LINES_MS, FrenchVoice::FEMALE[0], format)
            .await?;

        let mut lines = Vec::with_capacity(parsed.len());
        let mut combined = Vec::new();

        for (i, DialogLine { speaker, text }) in parsed.into_iter().enumerate() {
            let voice = voice_map[&speaker];
            let audio = self.synthesize(&text, voice, format).await?;

            // Append to combined stream.
            if i > 0 {
                combined.extend_from_slice(&silence);
            }
            combined.extend_from_slice(&audio);

            lines.push(LineAudio {
                index: i + 1,
                speaker,
                text,
                data: audio,
            });
        }

        Ok(DialogAudio { lines, combined, format })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_names_are_valid_google_ids() {
        // Spot-check representative voices from each tier.
        assert_eq!(FrenchVoice::StudioA.name(), "fr-FR-Studio-A");
        assert_eq!(FrenchVoice::StudioD.name(), "fr-FR-Studio-D");
        assert_eq!(FrenchVoice::Neural2F.name(), "fr-FR-Neural2-F");
        assert_eq!(FrenchVoice::Neural2G.name(), "fr-FR-Neural2-G");
        assert_eq!(FrenchVoice::WavenetF.name(), "fr-FR-Wavenet-F");
        assert_eq!(FrenchVoice::WavenetG.name(), "fr-FR-Wavenet-G");
        assert_eq!(FrenchVoice::Polyglot1.name(), "fr-FR-Polyglot-1");
        assert_eq!(FrenchVoice::ChirpHdD.name(), "fr-FR-Chirp-HD-D");
        assert_eq!(FrenchVoice::ChirpHdF.name(), "fr-FR-Chirp-HD-F");
        assert_eq!(
            FrenchVoice::Chirp3HdAchernar.name(),
            "fr-FR-Chirp3-HD-Achernar"
        );
        assert_eq!(
            FrenchVoice::Chirp3HdZubenelgenubi.name(),
            "fr-FR-Chirp3-HD-Zubenelgenubi"
        );
    }

    #[test]
    fn female_and_male_pools_are_nonempty() {
        assert!(!FrenchVoice::FEMALE.is_empty());
        assert!(!FrenchVoice::MALE.is_empty());
    }

    #[test]
    fn from_env_fails_without_key() {
        // Ensure the variable is not set for this test.
        std::env::remove_var("GOOGLE_TTS_API_KEY");
        let result = GoogleTts::from_env();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TtsError::MissingApiKey));
    }

    #[test]
    fn request_body_serializes_text_input() {
        let req = SynthesizeRequest {
            input: SynthesisInput {
                text: Some("Bonjour le monde"),
                ssml: None,
            },
            voice: VoiceSelection {
                language_code: "fr-FR",
                name: FrenchVoice::StudioA.name(),
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
        let req = SynthesizeRequest {
            input: SynthesisInput {
                text: None,
                ssml: Some(ssml),
            },
            voice: VoiceSelection {
                language_code: "fr-FR",
                name: FrenchVoice::StudioA.name(),
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
    fn audio_format_api_encoding() {
        assert_eq!(AudioFormat::Mp3.api_encoding(), "MP3");
        assert_eq!(AudioFormat::OggOpus.api_encoding(), "OGG_OPUS");
    }

    #[test]
    fn audio_format_extension() {
        assert_eq!(AudioFormat::Mp3.extension(), "mp3");
        assert_eq!(AudioFormat::OggOpus.extension(), "ogg");
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
    fn request_body_serializes_ogg_encoding() {
        let req = SynthesizeRequest {
            input: SynthesisInput {
                text: Some("Bonjour"),
                ssml: None,
            },
            voice: VoiceSelection {
                language_code: "fr-FR",
                name: FrenchVoice::StudioA.name(),
            },
            audio_config: AudioConfig {
                audio_encoding: AudioFormat::OggOpus.api_encoding(),
            },
        };

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["audioConfig"]["audioEncoding"], "OGG_OPUS");
    }
}
