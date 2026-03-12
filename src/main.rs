mod dialog;
mod tts;

use std::path::PathBuf;
use tts::{FrenchVoice, GoogleTts};

use crate::dialog::slugify;

fn print_usage(prog: &str) {
    eprintln!("Usage:");
    eprintln!("  {prog} file   <input.txt> <output.mp3>    Synthesize a text file");
    eprintln!("  {prog} dialog <input.txt> <output_dir>    Synthesize a dialog file");
    eprintln!("  {prog} --help                             Show detailed help");
    eprintln!();
    eprintln!("Dialog mode produces one MP3 per line in <output_dir>/lines/");
    eprintln!("and a combined <output_dir>/combined.mp3 with pauses between lines.");
}

fn print_help() {
    println!("francais-rouille — French text-to-speech using Google Cloud TTS");
    println!();
    println!("USAGE:");
    println!("  francais-rouille file   <input.txt> <output.mp3>");
    println!("  francais-rouille dialog <input.txt> <output_dir>");
    println!();
    println!("COMMANDS:");
    println!("  file     Convert a plain text file to a single MP3 using a default");
    println!("           French female voice.");
    println!("  dialog   Parse a dialog text file, assign a distinct voice to each");
    println!("           character based on gender, and produce per-line MP3s in");
    println!("           <output_dir>/lines/ plus a combined <output_dir>/combined.mp3.");
    println!();
    println!("ENVIRONMENT:");
    println!("  GOOGLE_TTS_API_KEY   Required. Your Google Cloud API key with the");
    println!("                       Cloud Text-to-Speech API enabled.");
    println!();
    println!("  Export it before running:");
    println!("    export GOOGLE_TTS_API_KEY=\"your-api-key-here\"");
    println!();
    println!("VOICE ASSIGNMENT (dialog mode):");
    println!("  Character descriptions in the dialog file determine voice gender.");
    println!("  French gendered articles after the em-dash are used:");
    println!("    - Claire — une cliente ...   → female voice");
    println!("    - M. Duval — le patron ...   → male voice");
    println!("  Voices are randomly selected from Premium fr-FR Google Cloud voices.");
    println!("  Each character keeps the same voice throughout the dialog.");
    println!();
    println!("DIALOG FILE FORMAT:");
    println!("  Title of the Dialog");
    println!();
    println!("  Personnages :");
    println!("  - Speaker Name — une/un description");
    println!("  - Speaker Name — le/la description");
    println!();
    println!("  Speaker Name : First line of dialog.");
    println!("  Speaker Name : Second line of dialog.");
    println!();
    println!("See docs/TTS.md for full documentation.");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "--help" | "-h" => {
            print_help();
            Ok(())
        }
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
    tts.synthesize_to_file(&text, FrenchVoice::FEMALE[0], &output_path)
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
