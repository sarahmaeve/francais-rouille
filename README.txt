  src/tts.rs — The TTS module:
  - GoogleTts::from_env() — reads GOOGLE_TTS_API_KEY from the environment
  - synthesize(text, voice) — returns raw MP3 bytes
  - synthesize_to_file(text, voice, path) — writes MP3 directly to disk
  - FrenchVoice enum with StandardA, WavenetA, and WavenetB variants
  - TtsError for structured error handling
  - 3 unit tests covering voice names, missing key, and request serialization

  src/main.rs — CLI entry point:
  Usage: francais-rouille <input.txt> <output.mp3>

  To use it:
  export GOOGLE_TTS_API_KEY="your-key-here"
  cargo run -- passage.txt passage.mp3
