mod build;
mod dialog;
mod tts;

use std::path::PathBuf;
use tts::{AudioFormat, FrenchVoice, GoogleTts};

use crate::dialog::slugify;

fn print_usage(prog: &str) {
    eprintln!("Usage:");
    eprintln!("  {prog} file   <input.txt> <output>  [--format mp3|ogg]  Synthesize a text file");
    eprintln!("  {prog} dialog <input.txt> <output_dir> [--format mp3|ogg] [--combined]  Synthesize a dialog");
    eprintln!("  {prog} build  [<chapter>] [--site-url URL]  Generate HTML + sitemap");
    eprintln!("  {prog} --help                             Show detailed help");
    eprintln!();
    eprintln!("Audio format defaults to mp3. Use --format ogg for OGG Opus output.");
}

fn print_help() {
    println!("francais-rouille — French text-to-speech using Google Cloud TTS");
    println!();
    println!("USAGE:");
    println!("  francais-rouille file   <input.txt> <output> [--format mp3|ogg]");
    println!("  francais-rouille dialog <input.txt> <output_dir> [--format mp3|ogg] [--combined]");
    println!();
    println!("COMMANDS:");
    println!("  file     Convert a plain text file to a single audio file using a default");
    println!("           French female voice.");
    println!("  dialog   Parse a dialog text file, assign a distinct voice to each");
    println!("           character based on gender, and produce per-line audio files in");
    println!("           <output_dir>/lines/ plus a combined file with pauses between lines.");
    println!();
    println!("OPTIONS:");
    println!("  --format mp3|ogg   Audio encoding (default: mp3). Use \"ogg\" for OGG Opus,");
    println!("                     which produces smaller files at comparable quality.");
    println!("  --combined         Also generate a single combined audio file with silence");
    println!("                     between lines. Off by default.");
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
        "build" => run_build_mode(&args),
        _ => {
            print_usage(&args[0]);
            std::process::exit(1);
        }
    }
}

/// Parse `--format mp3|ogg` from args, defaulting to MP3.
fn parse_format(args: &[String]) -> Result<AudioFormat, Box<dyn std::error::Error>> {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--format" {
            let value = args
                .get(i + 1)
                .ok_or("--format requires a value (mp3 or ogg)")?;
            return AudioFormat::from_str(value)
                .ok_or_else(|| format!("unknown audio format: {value}").into());
        }
    }
    Ok(AudioFormat::Mp3)
}

async fn run_file_mode(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 4 {
        eprintln!("Usage: {} file <input.txt> <output> [--format mp3|ogg]", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[2];
    let output_path = PathBuf::from(&args[3]);
    let format = parse_format(args)?;

    let text = std::fs::read_to_string(input_path)?;
    let tts = GoogleTts::from_env()?;
    tts.synthesize_to_file(&text, FrenchVoice::FEMALE[0], format, &output_path)
        .await?;

    println!("Wrote {} audio to {}", format.extension(), output_path.display());
    Ok(())
}

async fn run_dialog_mode(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 4 {
        eprintln!("Usage: {} dialog <input.txt> <output_dir> [--format mp3|ogg] [--combined]", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[2];
    let output_dir = PathBuf::from(&args[3]);
    let format = parse_format(args)?;
    let combined = args.iter().any(|a| a == "--combined");
    let ext = format.extension();
    let lines_dir = output_dir.join("lines");

    std::fs::create_dir_all(&lines_dir)?;

    let content = std::fs::read_to_string(input_path)?;
    let tts = GoogleTts::from_env()?;

    println!("Synthesizing dialog from {input_path} (format: {ext})...");
    let result = tts.synthesize_dialog(&content, format, combined).await?;

    for line in &result.lines {
        let filename = format!(
            "{:02}_{}.{ext}",
            line.index,
            slugify(&line.speaker),
        );
        let path = lines_dir.join(&filename);
        std::fs::write(&path, &line.data)?;
        println!("  {} — {}: {}...", filename, line.speaker, truncate(&line.text, 50));
    }

    println!();
    println!("Wrote {} individual {ext} files to {}", result.lines.len(), lines_dir.display());

    if combined {
        let combined_path = output_dir.join(format!("combined.{ext}"));
        std::fs::write(&combined_path, &result.combined)?;
        println!("Wrote combined audio to {}", combined_path.display());
    }

    Ok(())
}

fn run_build_mode(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let templates_dir = PathBuf::from("templates");
    let content_root = PathBuf::from("content");
    let output_root = PathBuf::from("site/chapters");
    let site_dir = PathBuf::from("site");

    // Parse optional flags and positional args.
    let mut site_url: Option<String> = None;
    let mut chapter_filter: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        if args[i] == "--site-url" {
            i += 1;
            site_url = Some(
                args.get(i)
                    .ok_or("--site-url requires a value")?
                    .clone(),
            );
        } else if !args[i].starts_with('-') && chapter_filter.is_none() {
            chapter_filter = Some(args[i].clone());
        }
        i += 1;
    }

    // Discover chapters.
    let chapters: Vec<String> = if let Some(name) = chapter_filter {
        vec![name]
    } else {
        let mut names = Vec::new();
        for entry in std::fs::read_dir(&content_root)? {
            let entry = entry?;
            if entry.path().join("chapter.toml").exists() {
                if let Some(name) = entry.file_name().to_str() {
                    names.push(name.to_string());
                }
            }
        }
        names.sort();
        names
    };

    if chapters.is_empty() {
        eprintln!("No chapters found in {}", content_root.display());
        std::process::exit(1);
    }

    for chapter in &chapters {
        let content_dir = content_root.join(chapter);
        let output_dir = output_root.join(chapter);
        println!("Building chapter: {chapter}");
        build::build_chapter(&content_dir, &output_dir, &templates_dir, site_url.as_deref())?;
    }

    // Generate sitemap if a site URL is provided.
    if let Some(url) = &site_url {
        build::generate_sitemap(&site_dir, url)?;
    }

    println!("\nDone.");
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
