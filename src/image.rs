use std::path::{Path, PathBuf};
use std::process::Command;

/// Recursively collect JPEG/PNG files from a directory.
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

pub fn run_strip_metadata(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
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

/// Image role determines output naming and sizing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImageRole {
    Hero,
    Thumbnail,
    Page,
}

pub fn run_prepare_image(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 4 {
        eprintln!(
            "Usage: {} prepare-image <path> --chapter <chapter> [--role hero|thumbnail|page] [--slug NAME] [--widths W,W] [--quality N]",
            args[0]
        );
        std::process::exit(1);
    }

    let input_path = PathBuf::from(&args[2]);
    let mut chapter: Option<String> = None;
    let mut role = ImageRole::Page;
    let mut slug: Option<String> = None;
    let mut widths: Vec<u32> = vec![800, 400];
    let mut quality: u32 = 80;

    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--chapter" | "-c" => {
                i += 1;
                chapter = Some(
                    args.get(i)
                        .ok_or("--chapter requires a value")?
                        .clone(),
                );
            }
            "--role" | "-r" => {
                i += 1;
                let value = args.get(i).ok_or("--role requires a value")?;
                role = match value.as_str() {
                    "hero" => ImageRole::Hero,
                    "thumbnail" | "thumb" => ImageRole::Thumbnail,
                    "page" => ImageRole::Page,
                    other => return Err(format!("unknown role: {other} (expected hero, thumbnail, or page)").into()),
                };
            }
            "--slug" | "-s" => {
                i += 1;
                slug = Some(
                    args.get(i)
                        .ok_or("--slug requires a value")?
                        .clone(),
                );
            }
            "--widths" | "-w" => {
                i += 1;
                let value = args.get(i).ok_or("--widths requires a value")?;
                widths = value
                    .split(',')
                    .map(|s| s.trim().parse::<u32>())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| format!("invalid width: {e}"))?;
                if widths.is_empty() {
                    return Err("--widths requires at least one value".into());
                }
            }
            "--quality" | "-q" => {
                i += 1;
                quality = args
                    .get(i)
                    .ok_or("--quality requires a value")?
                    .parse::<u32>()
                    .map_err(|e| format!("invalid quality: {e}"))?;
                if quality > 100 {
                    return Err("--quality must be 0–100".into());
                }
            }
            other => {
                return Err(format!("unknown flag: {other}").into());
            }
        }
        i += 1;
    }

    let chapter = chapter.ok_or("--chapter is required")?;

    if role == ImageRole::Page && slug.is_none() {
        return Err("--slug is required when --role is page".into());
    }

    if !input_path.is_file() {
        return Err(format!("file not found: {}", input_path.display()).into());
    }

    // Validate external tools are available.
    require_command("magick")?;
    require_command("cwebp")?;

    // Determine output directory.
    let output_dir = if chapter == "landing" {
        PathBuf::from("site/shared/images")
    } else {
        PathBuf::from(format!("site/chapters/{chapter}/images"))
    };
    std::fs::create_dir_all(&output_dir)?;

    // Step 1: Auto-orient (bake EXIF rotation into pixels), then strip metadata.
    let temp_dir = tempfile::tempdir()?;
    let oriented_path = temp_dir.path().join("oriented.jpg");
    run_magick(&input_path, &oriented_path, &["-auto-orient"])?;

    let stripped_path = temp_dir.path().join("stripped.jpg");
    let strip_report =
        image_strip::strip_metadata(&oriented_path, &stripped_path, &image_strip::StripOptions::default())?;
    println!("Stripped metadata: {strip_report}");

    // Step 2: Generate outputs based on role.
    let input_bytes = std::fs::metadata(&input_path)?.len();
    let mut outputs: Vec<(PathBuf, u64)> = Vec::new();

    match role {
        ImageRole::Thumbnail => {
            let out_path = output_dir.join("thumb.webp");
            let resized = temp_dir.path().join("thumb-resized.png");

            // Square center-crop resize.
            run_magick(&stripped_path, &resized, &[
                "-resize", "160x160^",
                "-gravity", "center",
                "-extent", "160x160",
            ])?;
            run_cwebp(&resized, &out_path, quality)?;

            let size = std::fs::metadata(&out_path)?.len();
            outputs.push((out_path, size));
        }
        ImageRole::Hero | ImageRole::Page => {
            let base_name = match role {
                ImageRole::Hero => "hero".to_string(),
                ImageRole::Page => slug.clone().unwrap(),
                ImageRole::Thumbnail => unreachable!(),
            };

            for &width in &widths {
                let out_path = output_dir.join(format!("{base_name}-{width}w.webp"));
                let resized = temp_dir.path().join(format!("resized-{width}.png"));

                run_magick(&stripped_path, &resized, &[
                    "-resize", &format!("{width}x"),
                ])?;
                run_cwebp(&resized, &out_path, quality)?;

                let size = std::fs::metadata(&out_path)?.len();
                outputs.push((out_path, size));
            }
        }
    }

    // Step 3: Summary.
    println!();
    println!(
        "Prepared {} from {} ({} bytes):",
        role_label(role),
        input_path.display(),
        input_bytes,
    );
    for (path, size) in &outputs {
        println!("  {} ({} bytes)", path.display(), size);
    }

    Ok(())
}

fn role_label(role: ImageRole) -> &'static str {
    match role {
        ImageRole::Hero => "hero image",
        ImageRole::Thumbnail => "thumbnail",
        ImageRole::Page => "page image",
    }
}

fn require_command(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    Command::new(name)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|_| format!("{name} not found on PATH — please install it"))?;
    Ok(())
}

fn run_magick(input: &Path, output: &Path, extra_args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("magick")
        .arg(input)
        .args(extra_args)
        .arg(output)
        .status()?;
    if !status.success() {
        return Err(format!("magick failed with {status}").into());
    }
    Ok(())
}

fn run_cwebp(input: &Path, output: &Path, quality: u32) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("cwebp")
        .args(["-q", &quality.to_string()])
        .arg(input)
        .args(["-o"])
        .arg(output)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;
    if !status.success() {
        return Err(format!("cwebp failed with {status}").into());
    }
    Ok(())
}
