mod tts;

use std::path::PathBuf;
use tts::{FrenchVoice, GoogleTts};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input.txt> <output.mp3>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = PathBuf::from(&args[2]);

    let text = std::fs::read_to_string(input_path)?;
    let tts = GoogleTts::from_env()?;
    tts.synthesize_to_file(&text, FrenchVoice::WavenetA, &output_path)
        .await?;

    println!("Wrote audio to {}", output_path.display());
    Ok(())
}
