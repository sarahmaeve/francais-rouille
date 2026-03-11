use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

const GOOGLE_TTS_URL: &str = "https://texttospeech.googleapis.com/v1/text:synthesize";

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
}

#[derive(Debug, Clone, Copy)]
pub enum FrenchVoice {
    /// Standard female voice (fr-FR)
    StandardA,
    /// WaveNet female voice (fr-FR, higher quality)
    WavenetA,
    /// WaveNet male voice (fr-FR, higher quality)
    WavenetB,
}

impl FrenchVoice {
    fn name(self) -> &'static str {
        match self {
            Self::StandardA => "fr-FR-Standard-A",
            Self::WavenetA => "fr-FR-Wavenet-A",
            Self::WavenetB => "fr-FR-Wavenet-B",
        }
    }
}

#[derive(Serialize)]
struct SynthesizeRequest<'a> {
    input: SynthesisInput<'a>,
    voice: VoiceSelection,
    #[serde(rename = "audioConfig")]
    audio_config: AudioConfig,
}

#[derive(Serialize)]
struct SynthesisInput<'a> {
    text: &'a str,
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

    /// Synthesizes `text` as French speech and returns raw MP3 bytes.
    pub async fn synthesize(
        &self,
        text: &str,
        voice: FrenchVoice,
    ) -> Result<Vec<u8>, TtsError> {
        let body = SynthesizeRequest {
            input: SynthesisInput { text },
            voice: VoiceSelection {
                language_code: "fr-FR",
                name: voice.name(),
            },
            audio_config: AudioConfig {
                audio_encoding: "MP3",
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

    /// Synthesizes `text` and writes the MP3 to `output_path`.
    pub async fn synthesize_to_file(
        &self,
        text: &str,
        voice: FrenchVoice,
        output_path: &Path,
    ) -> Result<(), TtsError> {
        let bytes = self.synthesize(text, voice).await?;
        std::fs::write(output_path, &bytes)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_names_are_valid_google_ids() {
        assert_eq!(FrenchVoice::StandardA.name(), "fr-FR-Standard-A");
        assert_eq!(FrenchVoice::WavenetA.name(), "fr-FR-Wavenet-A");
        assert_eq!(FrenchVoice::WavenetB.name(), "fr-FR-Wavenet-B");
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
    fn request_body_serializes_correctly() {
        let req = SynthesizeRequest {
            input: SynthesisInput {
                text: "Bonjour le monde",
            },
            voice: VoiceSelection {
                language_code: "fr-FR",
                name: FrenchVoice::WavenetA.name(),
            },
            audio_config: AudioConfig {
                audio_encoding: "MP3",
            },
        };

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["input"]["text"], "Bonjour le monde");
        assert_eq!(json["voice"]["languageCode"], "fr-FR");
        assert_eq!(json["voice"]["name"], "fr-FR-Wavenet-A");
        assert_eq!(json["audioConfig"]["audioEncoding"], "MP3");
    }
}
