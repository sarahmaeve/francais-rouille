//! Dictée exercise data: generator + schema + structural validator.
//!
//! A `dictee-data.json` file holds one *exercise* per dialog page in a
//! chapter. Each exercise lists the dialog's spoken lines with the audio
//! file the learner listens to and the transcript that is the answer key.
//! The learner hears a line, types what they heard, and the JS engine grades
//! a deterministic word-level diff against the transcript (scoring lives in
//! `site/shared/quiz.js`; see `docs/DICTEE.md`).
//!
//! The file is *generated*, not authored: [`generate_chapter`] parses each
//! dialog `.txt`, reconstructs the per-line audio filenames with the shared
//! [`crate::dialog::line_audio_filename`] convention (so the path can never
//! drift from what the TTS writer produced), and includes a line only when
//! its MP3 exists on disk. The same file the browser fetches is the file the
//! validator checks.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::build::ChapterConfig;
use crate::dialog::{self, line_audio_filename};

/// Schema marker written into every generated file.
pub const SCHEMA: &str = "dictee/v1";

// ── Schema ──────────────────────────────────────────────────────────────

/// Top-level `dictee-data.json` document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DicteeFile {
    /// Schema marker, e.g. `"dictee/v1"`.
    pub schema: String,
    pub exercises: Vec<DicteeExercise>,
}

/// One dialog turned into a dictée: an ordered list of spoken lines.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DicteeExercise {
    /// Unique within the file — the dialog page slug.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub lines: Vec<DicteeLine>,
}

/// A single spoken line: its audio and its transcript (the answer key).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DicteeLine {
    /// 1-based line number, dense and ordered within the exercise.
    pub n: usize,
    pub speaker: String,
    /// Chapter-relative path to the line's MP3, e.g.
    /// `audio/01_paris_metro/lines/01_lea.mp3`.
    pub audio: String,
    /// The transcript the learner is graded against.
    pub text: String,
}

// ── Generation ──────────────────────────────────────────────────────────

/// Build the in-memory dictée document for a chapter.
///
/// For every `type = "dialog"` page it parses `content/<ch>/<slug>.txt`,
/// reconstructs each line's audio filename, and keeps the line only if its
/// MP3 already exists under `output_dir` (a missing MP3 is skipped with a
/// warning rather than pointing the learner at a 404). Exercises with no
/// surviving lines are dropped. Returns [`None`] when nothing produced any
/// lines, so the caller can avoid writing an empty file.
pub fn build_chapter(
    content_dir: &Path,
    output_dir: &Path,
    config: &ChapterConfig,
) -> Result<Option<DicteeFile>, std::io::Error> {
    let mut exercises = Vec::new();

    for page in config.sections.iter().flat_map(|s| &s.pages) {
        if page.page_type != "dialog" {
            continue;
        }
        let txt_path = content_dir.join(format!("{}.txt", page.slug));
        let content = match std::fs::read_to_string(&txt_path) {
            Ok(c) => c,
            Err(_) => continue, // No source .txt — nothing to build.
        };
        let audio_dir = page.audio_dir.clone().unwrap_or_else(|| page.slug.clone());

        let mut lines = Vec::new();
        for (i, line) in dialog::parse_dialog(&content).into_iter().enumerate() {
            let n = i + 1;
            let filename = line_audio_filename(n, &line.speaker, "mp3");
            let audio_rel = format!("audio/{audio_dir}/lines/{filename}");
            if !output_dir.join(&audio_rel).is_file() {
                eprintln!(
                    "  dictee: skipping {}#{n} ({}) — audio not found: {audio_rel}",
                    page.slug, line.speaker,
                );
                continue;
            }
            lines.push(DicteeLine {
                n,
                speaker: line.speaker,
                audio: audio_rel,
                text: line.text,
            });
        }

        if lines.is_empty() {
            continue;
        }
        exercises.push(DicteeExercise {
            id: page.slug.clone(),
            title: Some(page.title.clone()),
            lines,
        });
    }

    if exercises.is_empty() {
        return Ok(None);
    }
    Ok(Some(DicteeFile {
        schema: SCHEMA.to_string(),
        exercises,
    }))
}

/// Generate `dictee-data.json` for a chapter when opted in via
/// `[chapter] dictee = true`. Writes the file under `output_dir` and returns
/// its path, or [`None`] when the chapter is not opted in or produced no
/// lines.
pub fn generate_chapter(
    content_dir: &Path,
    output_dir: &Path,
    config: &ChapterConfig,
) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    if !config.chapter.dictee {
        return Ok(None);
    }
    let Some(doc) = build_chapter(content_dir, output_dir, config)? else {
        return Ok(None);
    };
    let out_path = output_dir.join("dictee-data.json");
    let json = serde_json::to_string_pretty(&doc)?;
    std::fs::create_dir_all(output_dir)?;
    std::fs::write(&out_path, format!("{json}\n"))?;
    let lines: usize = doc.exercises.iter().map(|e| e.lines.len()).sum();
    println!(
        "  wrote dictee-data.json ({} exercise(s), {lines} line(s))",
        doc.exercises.len(),
    );
    Ok(Some(out_path))
}

// ── Errors ──────────────────────────────────────────────────────────────

/// A single structural problem found while validating a dictée file.
#[derive(Debug, Clone, PartialEq)]
pub struct DicteeError {
    pub file: PathBuf,
    pub location: String,
    pub message: String,
}

impl fmt::Display for DicteeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}: {}", self.file.display(), self.location, self.message)
    }
}

// ── Validation ──────────────────────────────────────────────────────────

/// Parse and validate a `dictee-data.json` file on disk.
///
/// Audio paths are resolved relative to the file's own directory, so this
/// also confirms every referenced MP3 exists.
pub fn validate_file(path: &Path) -> Vec<DicteeError> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            return vec![DicteeError {
                file: path.to_path_buf(),
                location: "(file)".into(),
                message: format!("could not read file: {e}"),
            }]
        }
    };
    validate_str(path, &text, path.parent())
}

/// Parse and validate dictée JSON held in memory. When `base` is `Some`,
/// every `audio` path is checked for existence relative to it.
pub fn validate_str(file: &Path, json: &str, base: Option<&Path>) -> Vec<DicteeError> {
    match serde_json::from_str::<DicteeFile>(json) {
        Ok(doc) => validate_doc(file, &doc, base),
        Err(e) => vec![DicteeError {
            file: file.to_path_buf(),
            location: "(parse)".into(),
            message: e.to_string(),
        }],
    }
}

/// Validate an already-parsed document.
pub fn validate_doc(file: &Path, doc: &DicteeFile, base: Option<&Path>) -> Vec<DicteeError> {
    let mut errors = Vec::new();
    let push = |errors: &mut Vec<DicteeError>, loc: &str, msg: String| {
        errors.push(DicteeError {
            file: file.to_path_buf(),
            location: loc.to_string(),
            message: msg,
        });
    };

    if doc.schema != SCHEMA {
        push(
            &mut errors,
            "(file)",
            format!("unknown schema '{}', expected '{SCHEMA}'", doc.schema),
        );
    }

    let mut seen = BTreeMap::new();
    for ex in &doc.exercises {
        *seen.entry(ex.id.as_str()).or_insert(0) += 1;
        let loc = format!("exercise '{}'", ex.id);
        if ex.lines.is_empty() {
            push(&mut errors, &loc, "exercise has no lines".into());
        }
        for (i, line) in ex.lines.iter().enumerate() {
            let lloc = format!("{loc} › line {}", line.n);
            // Line numbers must be dense and ordered: 1, 2, 3, …
            if line.n != i + 1 {
                push(
                    &mut errors,
                    &loc,
                    format!("line numbers must be dense and 1-based (got {} at position {})", line.n, i + 1),
                );
            }
            if line.text.trim().is_empty() {
                push(&mut errors, &lloc, "text is empty".into());
            }
            if line.audio.trim().is_empty() {
                push(&mut errors, &lloc, "audio path is empty".into());
            } else if let Some(base) = base {
                if !base.join(&line.audio).is_file() {
                    push(&mut errors, &lloc, format!("audio file not found: {}", line.audio));
                }
            }
        }
    }
    for (id, n) in &seen {
        if *n > 1 {
            push(&mut errors, "(file)", format!("duplicate exercise id '{id}'"));
        }
    }

    errors
}

// ── File discovery ──────────────────────────────────────────────────────

/// Recursively collect every file named `dictee-data.json` under `dir`.
pub fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out)?;
        } else if path.file_name().and_then(|n| n.to_str()) == Some("dictee-data.json") {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> PathBuf {
        PathBuf::from("dictee-data.json")
    }

    fn messages(errors: &[DicteeError]) -> String {
        errors.iter().map(|e| e.message.clone()).collect::<Vec<_>>().join(" | ")
    }

    // ── Validation ──────────────────────────────────────────────────

    const GOOD: &str = r#"{
        "schema": "dictee/v1",
        "exercises": [
            { "id": "01_paris_metro", "title": "Le métro",
              "lines": [
                { "n": 1, "speaker": "Léa", "audio": "a.mp3", "text": "Excusez-moi." },
                { "n": 2, "speaker": "Marc", "audio": "b.mp3", "text": "Oui ?" }
              ] }
        ]
    }"#;

    #[test]
    fn good_file_is_valid_without_audio_check() {
        // base = None skips the on-disk audio existence check.
        assert!(validate_str(&p(), GOOD, None).is_empty());
    }

    #[test]
    fn unknown_schema_flagged() {
        let json = GOOD.replace("dictee/v1", "dictee/v9");
        assert!(messages(&validate_str(&p(), &json, None)).contains("unknown schema"));
    }

    #[test]
    fn duplicate_exercise_id_flagged() {
        let json = r#"{ "schema": "dictee/v1", "exercises": [
            { "id": "dup", "lines": [ {"n":1,"speaker":"A","audio":"a.mp3","text":"x"} ] },
            { "id": "dup", "lines": [ {"n":1,"speaker":"A","audio":"a.mp3","text":"x"} ] }
        ] }"#;
        assert!(messages(&validate_str(&p(), json, None)).contains("duplicate exercise id 'dup'"));
    }

    #[test]
    fn non_dense_line_numbers_flagged() {
        let json = r#"{ "schema": "dictee/v1", "exercises": [
            { "id": "e", "lines": [
                {"n":1,"speaker":"A","audio":"a.mp3","text":"x"},
                {"n":3,"speaker":"A","audio":"b.mp3","text":"y"}
            ] }
        ] }"#;
        assert!(messages(&validate_str(&p(), json, None)).contains("dense and 1-based"));
    }

    #[test]
    fn empty_text_flagged() {
        let json = r#"{ "schema": "dictee/v1", "exercises": [
            { "id": "e", "lines": [ {"n":1,"speaker":"A","audio":"a.mp3","text":"   "} ] }
        ] }"#;
        assert!(messages(&validate_str(&p(), json, None)).contains("text is empty"));
    }

    #[test]
    fn missing_audio_flagged_when_base_given() {
        let dir = tempfile::tempdir().unwrap();
        // Only a.mp3 exists on disk; b.mp3 is referenced but absent.
        std::fs::write(dir.path().join("a.mp3"), b"x").unwrap();
        let errors = validate_str(&p(), GOOD, Some(dir.path()));
        assert!(messages(&errors).contains("audio file not found: b.mp3"), "{}", messages(&errors));
        assert!(!messages(&errors).contains("a.mp3"), "existing audio must not be flagged");
    }

    #[test]
    fn parse_error_reported() {
        let errors = validate_str(&p(), "{ not json", None);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].location, "(parse)");
    }

    #[test]
    fn roundtrips_through_json() {
        let doc: DicteeFile = serde_json::from_str(GOOD).unwrap();
        let json = serde_json::to_string(&doc).unwrap();
        let back: DicteeFile = serde_json::from_str(&json).unwrap();
        assert_eq!(doc, back);
    }

    // ── Generation ──────────────────────────────────────────────────

    /// A minimal chapter config with one dialog page.
    fn chapter_with_dialog(dictee: bool) -> ChapterConfig {
        let toml = format!(
            r#"
            [chapter]
            title = "T"
            subtitle = "S"
            vocab_page = "vocabulaire"
            footer_text = "F"
            footer_suffix = "FS"
            dictee = {dictee}

            [[sections]]
            heading = "H"
            [[sections.pages]]
            slug = "01_demo"
            title = "Démo"
            description = "d"
            type = "dialog"
            "#
        );
        toml::from_str(&toml).unwrap()
    }

    fn write_dialog(content_dir: &Path) {
        std::fs::create_dir_all(content_dir).unwrap();
        std::fs::write(
            content_dir.join("01_demo.txt"),
            "Titre\n\nPersonnages :\n- Léa — une passante\n\nLéa : Bonjour.\nMarc : Salut.\n",
        )
        .unwrap();
    }

    #[test]
    fn build_includes_only_lines_with_audio() {
        let tmp = tempfile::tempdir().unwrap();
        let content = tmp.path().join("content");
        let output = tmp.path().join("out");
        write_dialog(&content);
        // Provide audio for line 1 (Léa) only; line 2 (Marc) must be skipped.
        let lines_dir = output.join("audio/01_demo/lines");
        std::fs::create_dir_all(&lines_dir).unwrap();
        std::fs::write(lines_dir.join("01_lea.mp3"), b"x").unwrap();

        let config = chapter_with_dialog(true);
        let doc = build_chapter(&content, &output, &config).unwrap().unwrap();
        assert_eq!(doc.exercises.len(), 1);
        assert_eq!(doc.exercises[0].lines.len(), 1, "line without audio must be skipped");
        let line = &doc.exercises[0].lines[0];
        assert_eq!(line.n, 1);
        assert_eq!(line.speaker, "Léa");
        assert_eq!(line.audio, "audio/01_demo/lines/01_lea.mp3");
        assert_eq!(line.text, "Bonjour.");
    }

    #[test]
    fn build_returns_none_when_no_audio() {
        let tmp = tempfile::tempdir().unwrap();
        let content = tmp.path().join("content");
        let output = tmp.path().join("out");
        write_dialog(&content);
        // No audio at all → no lines → no document.
        let config = chapter_with_dialog(true);
        assert!(build_chapter(&content, &output, &config).unwrap().is_none());
    }

    #[test]
    fn generate_respects_opt_in() {
        let tmp = tempfile::tempdir().unwrap();
        let content = tmp.path().join("content");
        let output = tmp.path().join("out");
        write_dialog(&content);
        let lines_dir = output.join("audio/01_demo/lines");
        std::fs::create_dir_all(&lines_dir).unwrap();
        std::fs::write(lines_dir.join("01_lea.mp3"), b"x").unwrap();

        // Opted out → no file written.
        let off = chapter_with_dialog(false);
        assert!(generate_chapter(&content, &output, &off).unwrap().is_none());
        assert!(!output.join("dictee-data.json").exists());

        // Opted in → file written and it validates (including audio check).
        let on = chapter_with_dialog(true);
        let path = generate_chapter(&content, &output, &on).unwrap().unwrap();
        assert!(path.is_file());
        assert!(validate_file(&path).is_empty(), "generated file must validate");
    }
}
