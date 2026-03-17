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
    pub async fn synthesize(
        &self,
        text: &str,
        voice: &Voice,
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
