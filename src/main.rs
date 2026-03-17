mod build;
mod dialog;
mod tts;

use std::path::{Path, PathBuf};
use tts::{AudioFormat, GoogleTts};

use crate::dialog::{french::French, spanish::Spanish, slugify, Language, Voice};

fn print_usage(prog: &str) {
    eprintln!("Usage:");
    eprintln!("  {prog} file   <input.txt> <output>  [--format mp3|ogg] [--lang fr-FR|es-US]  Synthesize a text file");
    eprintln!("  {prog} dialog <input.txt> <output_dir> [--format mp3|ogg] [--lang fr-FR|es-US] [--combined]  Synthesize a dialog");
    eprintln!("  {prog} build  [<chapter>] [--output DIR] [--site-url URL]  Generate HTML + sitemap");
    eprintln!("  {prog} verify-language [<chapter>] [--lang fr-FR] [--fix] [--strict]  Check/fix typographic rules");
    eprintln!("  {prog} strip-metadata <path> [--output DIR] [--keep-icc]            Strip image EXIF/metadata");
    eprintln!("  {prog} check-csp     [--site DIR]                                  Verify HTML against CSP");
    eprintln!("  {prog} --help                             Show detailed help");
    eprintln!();
    eprintln!("Audio format defaults to mp3. Language defaults to fr-FR.");
}

fn print_help() {
    println!("francais-rouille — Text-to-speech and site generation for language learning");
    println!();
    println!("USAGE:");
    println!("  francais-rouille file   <input.txt> <output> [--format mp3|ogg] [--lang fr-FR|es-US]");
    println!("  francais-rouille dialog <input.txt> <output_dir> [--format mp3|ogg] [--lang fr-FR|es-US] [--combined]");
    println!();
    println!("COMMANDS:");
    println!("  file     Convert a plain text file to a single audio file.");
    println!("  dialog   Parse a dialog text file, assign a distinct voice to each");
    println!("           character based on gender, and produce per-line audio files in");
    println!("           <output_dir>/lines/.");
    println!("  build    Generate HTML pages and chapter indexes from content/.");
    println!("           Use --output (-o) to write to a different directory (default: site/).");
    println!("  verify-language");
    println!("           Check content files against typographic rules for a language.");
    println!("           Use --fix to auto-correct violations in place.");
    println!("           Use --strict to also enforce narrow no-break spaces (U+202F)");
    println!("           before high punctuation (; : ! ?).");
    println!("  strip-metadata <path> [--output DIR] [--keep-icc]");
    println!("           Strip privacy-sensitive metadata (EXIF, XMP, IPTC, comments)");
    println!("           from JPEG and PNG images. <path> can be a single file or a");
    println!("           directory (recursive). Without --output, overwrites in place.");
    println!("           Use --keep-icc to preserve ICC color profiles.");
    println!("  check-csp [--site DIR]");
    println!("           Scan HTML files for Content Security Policy violations.");
    println!("           Checks for inline scripts, inline styles, event handlers,");
    println!("           form elements, and external resource URLs.");
    println!("           Default site directory: site/");
    println!();
    println!("OPTIONS:");
    println!("  --format mp3|ogg     Audio encoding (default: mp3). Use \"ogg\" for OGG Opus.");
    println!("  --lang fr-FR|es-US   Language for voice selection and gender detection");
    println!("                       (default: fr-FR).");
    println!("  --combined           Also generate a single combined audio file with silence");
    println!("                       between lines. Off by default.");
    println!();
    println!("ENVIRONMENT:");
    println!("  GOOGLE_TTS_API_KEY   Required. Your Google Cloud API key with the");
    println!("                       Cloud Text-to-Speech API enabled.");
    println!();
    println!("VOICE ASSIGNMENT (dialog mode):");
    println!("  Character descriptions in the dialog file determine voice gender.");
    println!("  Gendered articles after the em-dash are used:");
    println!("    French:  - Claire — une cliente ...  (une/la = female, un/le = male)");
    println!("    Spanish: - María — una estudiante ... (una/la = female, un/el = male)");
    println!("  Voices are randomly selected from Premium Google Cloud voices.");
    println!("  Each character keeps the same voice throughout the dialog.");
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
        "verify-language" => run_verify_language(&args),
        "strip-metadata" => run_strip_metadata(&args),
        "check-csp" => run_check_csp(&args),
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

/// Parse `--lang fr-FR|es-US` from args, defaulting to French.
fn parse_language(args: &[String]) -> Result<Box<dyn Language>, Box<dyn std::error::Error>> {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--lang" {
            let value = args
                .get(i + 1)
                .ok_or("--lang requires a value (fr-FR or es-US)")?;
            return match value.as_str() {
                "fr-FR" => Ok(Box::new(French)),
                "es-US" => Ok(Box::new(Spanish)),
                other => Err(format!("unsupported language: {other}").into()),
            };
        }
    }
    Ok(Box::new(French))
}

/// Parse `--voice <name>` from args (e.g. `--voice fr-FR-Studio-D`).
fn parse_voice_override(args: &[String]) -> Option<String> {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--voice" {
            return args.get(i + 1).cloned();
        }
    }
    None
}

async fn run_file_mode(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 4 {
        eprintln!("Usage: {} file <input.txt> <output> [--format mp3|ogg] [--lang fr-FR|es-US] [--voice NAME]", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[2];
    let output_path = PathBuf::from(&args[3]);
    let format = parse_format(args)?;
    let lang = parse_language(args)?;

    let voice = if let Some(name) = parse_voice_override(args) {
        let code = lang.code();
        Voice {
            language_code: Box::leak(code.to_string().into_boxed_str()),
            name: Box::leak(name.into_boxed_str()),
        }
    } else {
        lang.voice_pool().female[0].clone()
    };

    let text = std::fs::read_to_string(input_path)?;
    let tts = GoogleTts::from_env()?;

    // Detect SSML input by checking if content starts with <speak>.
    let bytes = if text.trim_start().starts_with("<speak>") {
        tts.synthesize_ssml(&text, &voice, format).await?
    } else {
        tts.synthesize(&text, &voice, format).await?
    };

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output_path, &bytes)?;

    println!("Wrote {} audio to {} (voice: {})", format.extension(), output_path.display(), voice.name);
    Ok(())
}

async fn run_dialog_mode(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 4 {
        eprintln!("Usage: {} dialog <input.txt> <output_dir> [--format mp3|ogg] [--lang fr-FR|es-US] [--combined]", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[2];
    let output_dir = PathBuf::from(&args[3]);
    let format = parse_format(args)?;
    let lang = parse_language(args)?;
    let combined = args.iter().any(|a| a == "--combined");
    let ext = format.extension();
    let lines_dir = output_dir.join("lines");

    std::fs::create_dir_all(&lines_dir)?;

    let content = std::fs::read_to_string(input_path)?;
    let tts = GoogleTts::from_env()?;

    println!("Synthesizing dialog from {input_path} (lang: {}, format: {ext})...", lang.code());
    let result = tts.synthesize_dialog(&content, format, combined, lang.as_ref()).await?;

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

    // Parse optional flags and positional args.
    let mut site_url: Option<String> = None;
    let mut chapter_filter: Option<String> = None;
    let mut output_dir_override: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        if args[i] == "--site-url" {
            i += 1;
            site_url = Some(
                args.get(i)
                    .ok_or("--site-url requires a value")?
                    .clone(),
            );
        } else if args[i] == "--output" || args[i] == "-o" {
            i += 1;
            output_dir_override = Some(
                args.get(i)
                    .ok_or("--output requires a value")?
                    .clone(),
            );
        } else if !args[i].starts_with('-') && chapter_filter.is_none() {
            chapter_filter = Some(args[i].clone());
        }
        i += 1;
    }

    let site_dir = output_dir_override
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("site"));
    let output_root = site_dir.join("chapters");

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

fn run_verify_language(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let content_root = PathBuf::from("content");

    // Parse flags.
    let mut lang_code = "fr-FR";
    let mut fix = false;
    let mut strict = false;
    let mut chapter_filter: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--lang" => {
                i += 1;
                lang_code = args
                    .get(i)
                    .map(|s| s.as_str())
                    .ok_or("--lang requires a value")?;
            }
            "--fix" => {
                fix = true;
            }
            "--strict" => {
                strict = true;
            }
            other if !other.starts_with('-') && chapter_filter.is_none() => {
                chapter_filter = Some(other.to_string());
            }
            other => {
                return Err(format!("unknown flag: {other}").into());
            }
        }
        i += 1;
    }

    let rules = site_gen::typography::rules_for_language(lang_code, strict)
        .ok_or_else(|| format!("unsupported language for verification: {lang_code}"))?;

    // Discover chapters (same logic as build mode).
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

    if fix {
        let mut total = 0;
        for chapter in &chapters {
            let dir = content_root.join(chapter);
            let count = site_gen::typography::fix_files(&dir, rules.as_ref())?;
            if count > 0 {
                println!("{chapter}: fixed {count} file(s)");
            }
            total += count;
        }
        if total == 0 {
            println!("All files already conform to {} typography rules.", lang_code);
        } else {
            println!("\nFixed {total} file(s) total.");
        }
    } else {
        let mut total = 0;
        for chapter in &chapters {
            let dir = content_root.join(chapter);
            let violations = site_gen::typography::verify_files(&dir, rules.as_ref())?;
            for v in &violations {
                println!("{v}");
            }
            total += violations.len();
        }
        if total == 0 {
            println!("No violations found — all files conform to {} typography rules.", lang_code);
        } else {
            eprintln!("\nFound {total} violation(s). Run with --fix to auto-correct.");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn run_strip_metadata(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 3 {
        eprintln!(
            "Usage: {} strip-metadata <path> [--output DIR] [--keep-icc]",
            args[0]
        );
        std::process::exit(1);
    }

    let input_path = PathBuf::from(&args[2]);
    let mut output_dir: Option<PathBuf> = None;
    let mut keep_icc = false;

    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                i += 1;
                output_dir = Some(PathBuf::from(
                    args.get(i).ok_or("--output requires a value")?,
                ));
            }
            "--keep-icc" => {
                keep_icc = true;
            }
            other => {
                return Err(format!("unknown flag: {other}").into());
            }
        }
        i += 1;
    }

    let opts = image_strip::StripOptions { keep_icc };

    // Collect files to process.
    let files = if input_path.is_dir() {
        collect_images(&input_path)?
    } else if input_path.is_file() {
        vec![input_path.clone()]
    } else {
        return Err(format!("path does not exist: {}", input_path.display()).into());
    };

    if files.is_empty() {
        eprintln!("No JPEG or PNG files found in {}", input_path.display());
        std::process::exit(1);
    }

    let mut total_saved: u64 = 0;
    for file in &files {
        let out = match &output_dir {
            Some(dir) => {
                let name = file.file_name().unwrap();
                dir.join(name)
            }
            None => file.clone(), // overwrite in place
        };

        let report = image_strip::strip_metadata(file, &out, &opts)?;
        println!("{report}");
        total_saved += report.bytes_before.saturating_sub(report.bytes_after);
    }

    println!(
        "\nProcessed {} file(s), saved {} bytes total.",
        files.len(),
        total_saved
    );
    Ok(())
}

fn run_check_csp(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut site_dir = PathBuf::from("site");

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--site" => {
                i += 1;
                site_dir = PathBuf::from(
                    args.get(i).ok_or("--site requires a value")?,
                );
            }
            other => {
                return Err(format!("unknown flag: {other}").into());
            }
        }
        i += 1;
    }

    if !site_dir.is_dir() {
        eprintln!("Site directory not found: {}", site_dir.display());
        std::process::exit(1);
    }

    let violations = site_gen::build::check_csp(&site_dir)?;

    if violations.is_empty() {
        println!("No CSP violations found in {}", site_dir.display());
        Ok(())
    } else {
        for v in &violations {
            eprintln!("{v}");
        }
        eprintln!(
            "\nFound {} CSP violation(s) in {}",
            violations.len(),
            site_dir.display()
        );
        std::process::exit(1);
    }
}

fn collect_images(dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut results = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            results.extend(collect_images(&path)?);
        } else if image_strip::detect_format(&path).is_some() {
            results.push(path);
        }
    }
    results.sort();
    Ok(results)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max).collect();
        format!("{truncated}...")
    }
}
