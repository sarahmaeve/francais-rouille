//! `voice-report` CLI: inspect voice assignments across dialog files.
//!
//! Reuses the same `parse_dialog`, `parse_character_genders`, and
//! `assign_voices` used by the `dialog` command — so what it prints is
//! exactly the voice map that synthesis would use, with no text
//! parsing or guessing.
//!
//! Two views:
//!
//! - **Per-file view (default).** For each `.txt` dialog under
//!   `content/<chapter>/`, list each distinct speaker, whether their
//!   gender was detected from the character block (vs. fallen back to
//!   alternating assignment), and the voice they were assigned.
//! - **Cross-file consistency (`--consistency`).** After scanning, list
//!   every speaker name that received more than one distinct voice
//!   across the repo. Useful for spotting drift of recurring
//!   characters (e.g. Maeve getting Studio-A in one dialog and a
//!   fallback voice in another).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::dialog::{assign_voices, parse_character_genders, parse_dialog, Language, Voice};
use crate::dialog::{french::French, spanish::Spanish};

/// CLI entry.
pub fn run_voice_report(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let chapter_filter: Option<String> = args
        .iter()
        .skip(2)
        .find(|s| !s.starts_with("--"))
        .cloned();
    let consistency = args.iter().any(|a| a == "--consistency");
    let lang = parse_language(args)?;

    let content_root = PathBuf::from("content");
    if !content_root.is_dir() {
        return Err(format!("no content/ directory found at {}", content_root.display()).into());
    }

    let chapters = if let Some(name) = &chapter_filter {
        let path = content_root.join(name);
        if !path.join("chapter.toml").exists() {
            return Err(format!("chapter not found: {}", path.display()).into());
        }
        vec![(name.clone(), path)]
    } else {
        discover_chapters(&content_root)?
    };

    // (speaker, voice) → list of (chapter, slug) where that pairing occurred.
    let mut by_speaker: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();

    for (chapter, chapter_dir) in &chapters {
        for dialog_path in discover_dialog_files(chapter_dir)? {
            let report = report_file(&dialog_path, lang.as_ref())?;
            let Some(report) = report else { continue };

            let where_tag = format!("{}/{}", chapter, report.slug);
            println!("── {} ──", where_tag);
            for row in &report.rows {
                let gender_tag = match row.gender {
                    Some(g) => format!("{g:?}"),
                    None => "fallback".to_string(),
                };
                println!(
                    "  {:<24}  [{gender_tag:>8}]  → {}",
                    row.speaker, row.voice_name,
                );

                by_speaker
                    .entry(row.speaker.clone())
                    .or_default()
                    .entry(row.voice_name.clone())
                    .or_default()
                    .push(where_tag.clone());
            }
            println!();
        }
    }

    if consistency {
        print_consistency(&by_speaker);
    }

    Ok(())
}

/// The voice assignment for one dialog file.
struct FileReport {
    slug: String,
    rows: Vec<Row>,
}

struct Row {
    speaker: String,
    /// `None` if the speaker's gender could not be matched from the
    /// character block (i.e. `assign_voices` used the alternating
    /// fallback). The voice may still be correct; this flag just says
    /// "the gender wasn't looked up".
    gender: Option<crate::dialog::Gender>,
    voice_name: String,
}

/// Run the real pipeline on one file and return its assignments.
///
/// Returns `Ok(None)` if the file isn't actually a dialog — either
/// because it has no parseable dialog lines, or because it's a static
/// page (email, notice, classifieds, form) that happens to have `Field
/// : Value` lines which `parse_dialog` would otherwise interpret as
/// speakers. The heuristic: a real dialog has at least one character
/// block entry, *and* every "speaker" in the dialog body corresponds
/// to something in that block. Static pages like email headers
/// ("Objet : ...") have no character block at all, and are skipped.
fn report_file(
    path: &Path,
    lang: &dyn Language,
) -> Result<Option<FileReport>, Box<dyn std::error::Error>> {
    let slug = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("unreadable file stem")?
        .to_string();

    let content = std::fs::read_to_string(path)?;
    let lines = parse_dialog(&content);
    if lines.is_empty() {
        return Ok(None);
    }

    // Require at least one character-block entry. parse_character_genders
    // only returns entries whose description matched an article, so
    // static files with no `- Name — une/un/le/la ...` block map to an
    // empty HashMap, which we treat as "not a dialog file".
    let genders = parse_character_genders(&content, lang);
    if genders.is_empty() {
        return Ok(None);
    }

    let voice_map = assign_voices(&lines, &genders, lang, &slug);

    // Emit one row per distinct speaker, in first-speaking order —
    // that's what the reader will hear, and it matches how voices are
    // actually assigned.
    let mut seen = std::collections::HashSet::new();
    let mut rows = Vec::new();
    for line in &lines {
        if !seen.insert(line.speaker.clone()) {
            continue;
        }
        let voice: &Voice = voice_map
            .get(&line.speaker)
            .ok_or("speaker missing from voice map")?;
        rows.push(Row {
            speaker: line.speaker.clone(),
            gender: genders.get(&line.speaker).copied(),
            voice_name: voice.name.to_string(),
        });
    }

    Ok(Some(FileReport { slug, rows }))
}

fn print_consistency(by_speaker: &BTreeMap<String, BTreeMap<String, Vec<String>>>) {
    // BTreeMap iteration is already alphabetical by key.
    let drifted: Vec<_> = by_speaker
        .iter()
        .filter(|(_, voices)| voices.len() > 1)
        .collect();

    println!("═══ Cross-file consistency ═══");
    if drifted.is_empty() {
        println!("  All recurring speakers have a single voice across the repo.");
        return;
    }

    for (speaker, voices) in drifted {
        println!("  ⚠ {} has {} distinct voices across the repo:", speaker, voices.len());
        for (voice, wheres) in voices {
            println!("      {:<28}  in: {}", voice, wheres.join(", "));
        }
    }
}

/// Collect `(chapter_slug, chapter_dir)` pairs for every directory
/// under `content_root` that has a `chapter.toml`.
fn discover_chapters(
    content_root: &Path,
) -> Result<Vec<(String, PathBuf)>, Box<dyn std::error::Error>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(content_root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join("chapter.toml").exists() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                out.push((name.to_string(), path.clone()));
            }
        }
    }
    out.sort_by_key(|(name, _)| name.clone());
    Ok(out)
}

/// All `*.txt` files in `chapter_dir` that aren't English translations.
fn discover_dialog_files(chapter_dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(chapter_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.ends_with(".txt") || name.ends_with("_en.txt") {
            continue;
        }
        out.push(path);
    }
    out.sort();
    Ok(out)
}

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
