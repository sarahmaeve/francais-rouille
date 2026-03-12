mod dialog;
mod tts;

use std::path::PathBuf;
use tts::{FrenchVoice, GoogleTts};

use crate::dialog::slugify;

fn print_usage(prog: &str) {
    eprintln!("Usage:");
    eprintln!("  {prog} file   <input.txt> <output.mp3>         Synthesize a text file");
    eprintln!("  {prog} dialog <input.txt> <output_dir>          Synthesize a dialog file");
    eprintln!();
    eprintln!("Dialog mode produces one MP3 per line in <output_dir>/lines/");
    eprintln!("and a combined <output_dir>/combined.mp3 with pauses between lines.");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "file" => run_file_mode(&args).await,
        "dialog" => run_dialog_mode(&args).await,
        _ => {
            print_usage(&args[0]);
            std::process::exit(1);
        }
    }
}

async fn run_file_mode(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 4 {
        eprintln!("Usage: {} file <input.txt> <output.mp3>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[2];
    let output_path = PathBuf::from(&args[3]);

    let text = std::fs::read_to_string(input_path)?;
    let tts = GoogleTts::from_env()?;
    tts.synthesize_to_file(&text, FrenchVoice::WavenetA, &output_path)
        .await?;

    println!("Wrote audio to {}", output_path.display());
    Ok(())
}

async fn run_dialog_mode(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 4 {
        eprintln!("Usage: {} dialog <input.txt> <output_dir>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[2];
    let output_dir = PathBuf::from(&args[3]);
    let lines_dir = output_dir.join("lines");

    std::fs::create_dir_all(&lines_dir)?;

    let content = std::fs::read_to_string(input_path)?;
    let tts = GoogleTts::from_env()?;

    println!("Synthesizing dialog from {input_path}...");
    let result = tts.synthesize_dialog(&content).await?;

    for line in &result.lines {
        let filename = format!(
            "{:02}_{}.mp3",
            line.index,
            slugify(&line.speaker)
        );
        let path = lines_dir.join(&filename);
        std::fs::write(&path, &line.mp3)?;
        println!("  {} — {}: {}...", filename, line.speaker, truncate(&line.text, 50));
    }

    let combined_path = output_dir.join("combined.mp3");
    std::fs::write(&combined_path, &result.combined)?;

    println!();
    println!("Wrote {} individual files to {}", result.lines.len(), lines_dir.display());
    println!("Wrote combined audio to {}", combined_path.display());
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max).collect();
        format!("{truncated}...")
    }
}
