//! Translation-practice data: generator + alignment validator + schema.
//!
//! A `translation-data.json` file pairs each spoken French line of a dialog
//! with its reference English translation. The learner translates one line at
//! a time and self-grades against the reference (no auto-grading of free
//! English prose — see `docs/TRANSLATION.md`).
//!
//! The pairing comes from the two parallel content files every dialog already
//! ships: `content/<ch>/<slug>.txt` (French) and `<slug>_en.txt` (English),
//! which mirror each other line-for-line in the same `Speaker : text` format.
//!
//! The quiet win here is [`check_alignment`]: it **errors** (not warns) when
//! the two files have drifted — different line counts, or a different speaker
//! at the same index. Nothing else in the pipeline catches a translation that
//! has fallen out of sync with its source, so this guards every parallel pair
//! in the repo, not just this feature.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::build::ChapterConfig;
use crate::dialog::{self, line_audio_filename, slugify};

/// Schema marker written into every generated file.
pub const SCHEMA: &str = "translation/v1";

// ── Schema ──────────────────────────────────────────────────────────────

/// Top-level `translation-data.json` document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslationFile {
    pub schema: String,
    pub exercises: Vec<TranslationExercise>,
}

/// One dialog paired with its translation, line by line.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslationExercise {
    /// Unique within the file — the dialog page slug.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Chapter-relative directory holding the line MP3s, with trailing slash
    /// (e.g. `audio/01_paris_metro/lines/`). Present when any line has audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_base: Option<String>,
    pub lines: Vec<TranslationLine>,
}

/// A French line and its English reference translation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslationLine {
    /// 1-based line number, dense and ordered within the exercise.
    pub n: usize,
    pub speaker: String,
    /// French — graded/self-checked source. French typography applies here.
    pub fr: String,
    /// English reference translation. **Not** subject to French typography.
    pub en: String,
    /// Line MP3 filename relative to the exercise `audio_base`
    /// (e.g. `01_lea.mp3`), present only when the file exists on disk.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<String>,
}

// ── Alignment (the content-hardening check) ─────────────────────────────

/// A drift between a French dialog and its English translation.
#[derive(Debug, Clone, PartialEq)]
pub struct AlignmentError {
    pub message: String,
}

impl fmt::Display for AlignmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

/// Check that a French dialog and its English translation line up.
///
/// Two structural signals are used, both robust to the fact that translations
/// legitimately **rename** role-based speakers (*Le guichetier* → *The
/// clerk*, *Vendeuse* → *Saleswoman*) while keeping proper names (*Léa*):
///
/// 1. **Line count** — the files must have the same number of spoken lines.
/// 2. **Turn-taking shape** — the sequence of *distinct-speaker indices*
///    (each speaker mapped to the order it first appears within its own file)
///    must match position for position. This catches a dropped, added,
///    merged, or split turn that shifts the alternation, without ever
///    comparing the speaker *names* across languages.
///
/// A pure adjacent swap in a strictly alternating two-speaker dialog is not
/// structurally detectable once names are translated; the line-count check
/// remains the strong guard against the common drift (a line added or lost).
pub fn check_alignment(fr_content: &str, en_content: &str) -> Vec<AlignmentError> {
    let fr = dialog::parse_dialog(fr_content);
    let en = dialog::parse_dialog(en_content);
    let mut errors = Vec::new();

    if fr.len() != en.len() {
        errors.push(AlignmentError {
            message: format!(
                "line count mismatch: French has {} line(s), English has {}",
                fr.len(),
                en.len(),
            ),
        });
    }

    let fr_shape = turn_shape(&fr);
    let en_shape = turn_shape(&en);
    for (i, (f, e)) in fr_shape.iter().zip(en_shape.iter()).enumerate() {
        if f != e {
            errors.push(AlignmentError {
                message: format!(
                    "turn-taking mismatch at line {}: the speaker alternation \
                     diverges from the French here (a line may be added, dropped, \
                     merged, or split in the translation)",
                    i + 1,
                ),
            });
            break; // One report per file — the rest cascade from the same slip.
        }
    }

    errors
}

/// Map each line to the order-of-first-appearance index of its speaker within
/// the dialog, e.g. speakers `[Léa, Marc, Léa]` → `[0, 1, 0]`. Speakers are
/// keyed by [`slugify`] so trivial casing/accent variants collapse together.
fn turn_shape(lines: &[dialog::DialogLine]) -> Vec<usize> {
    let mut order: Vec<String> = Vec::new();
    lines
        .iter()
        .map(|l| {
            let key = slugify(&l.speaker);
            match order.iter().position(|s| *s == key) {
                Some(idx) => idx,
                None => {
                    order.push(key);
                    order.len() - 1
                }
            }
        })
        .collect()
}

// ── Generation ──────────────────────────────────────────────────────────

/// Build the in-memory translation document for a chapter.
///
/// For every `type = "dialog"` page that also has a `_en.txt`, it aligns the
/// two files and zips them by index, attaching each line's audio filename
/// when the MP3 exists under `output_dir`. Alignment errors are returned
/// separately (keyed by page) and, when present, the caller must treat them
/// as a hard failure rather than emit misaligned data.
pub fn build_chapter(
    content_dir: &Path,
    output_dir: &Path,
    config: &ChapterConfig,
) -> Result<(Option<TranslationFile>, Vec<AlignmentError>), std::io::Error> {
    let mut exercises = Vec::new();
    let mut alignment = Vec::new();

    for page in config.sections.iter().flat_map(|s| &s.pages) {
        if page.page_type != "dialog" {
            continue;
        }
        let fr_path = content_dir.join(format!("{}.txt", page.slug));
        let en_path = content_dir.join(format!("{}_en.txt", page.slug));
        let (Ok(fr_content), Ok(en_content)) = (
            std::fs::read_to_string(&fr_path),
            std::fs::read_to_string(&en_path),
        ) else {
            continue; // Needs both a French source and an English translation.
        };

        // Alignment first: a drift here poisons every subsequent pairing.
        let page_errors = check_alignment(&fr_content, &en_content);
        if !page_errors.is_empty() {
            for e in page_errors {
                alignment.push(AlignmentError {
                    message: format!("{}: {}", page.slug, e.message),
                });
            }
            continue;
        }

        let audio_dir = page.audio_dir.clone().unwrap_or_else(|| page.slug.clone());
        let audio_base = format!("audio/{audio_dir}/lines/");
        let fr = dialog::parse_dialog(&fr_content);
        let en = dialog::parse_dialog(&en_content);

        let mut any_audio = false;
        let mut lines = Vec::new();
        for (i, (f, e)) in fr.into_iter().zip(en).enumerate() {
            let n = i + 1;
            let filename = line_audio_filename(n, &f.speaker, "mp3");
            let audio = if output_dir.join(format!("{audio_base}{filename}")).is_file() {
                any_audio = true;
                Some(filename)
            } else {
                None
            };
            lines.push(TranslationLine {
                n,
                speaker: f.speaker,
                fr: f.text,
                en: e.text,
                audio,
            });
        }

        if lines.is_empty() {
            continue;
        }
        exercises.push(TranslationExercise {
            id: page.slug.clone(),
            title: Some(page.title.clone()),
            audio_base: any_audio.then_some(audio_base),
            lines,
        });
    }

    let doc = if exercises.is_empty() {
        None
    } else {
        Some(TranslationFile {
            schema: SCHEMA.to_string(),
            exercises,
        })
    };
    Ok((doc, alignment))
}

/// Generate `translation-data.json` for a chapter when opted in via
/// `[chapter] translation = true`. Returns an error if any dialog's
/// translation has drifted out of alignment with its source.
pub fn generate_chapter(
    content_dir: &Path,
    output_dir: &Path,
    config: &ChapterConfig,
) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    if !config.chapter.translation {
        return Ok(None);
    }
    let (doc, alignment) = build_chapter(content_dir, output_dir, config)?;
    if !alignment.is_empty() {
        for e in &alignment {
            eprintln!("  translation alignment error: {e}");
        }
        return Err(format!(
            "{} translation alignment error(s); fix the parallel files before building",
            alignment.len(),
        )
        .into());
    }
    let Some(doc) = doc else {
        return Ok(None);
    };
    let out_path = output_dir.join("translation-data.json");
    let json = serde_json::to_string_pretty(&doc)?;
    std::fs::create_dir_all(output_dir)?;
    std::fs::write(&out_path, format!("{json}\n"))?;
    let lines: usize = doc.exercises.iter().map(|e| e.lines.len()).sum();
    println!(
        "  wrote translation-data.json ({} exercise(s), {lines} line(s))",
        doc.exercises.len(),
    );
    Ok(Some(out_path))
}

// ── Errors ──────────────────────────────────────────────────────────────

/// A single structural problem found while validating a translation file.
#[derive(Debug, Clone, PartialEq)]
pub struct TranslationError {
    pub file: PathBuf,
    pub location: String,
    pub message: String,
}

impl fmt::Display for TranslationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}: {}", self.file.display(), self.location, self.message)
    }
}

// ── Validation ──────────────────────────────────────────────────────────

/// Parse and validate a `translation-data.json` file on disk.
pub fn validate_file(path: &Path) -> Vec<TranslationError> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            return vec![TranslationError {
                file: path.to_path_buf(),
                location: "(file)".into(),
                message: format!("could not read file: {e}"),
            }]
        }
    };
    validate_str(path, &text)
}

/// Parse and validate translation JSON held in memory.
pub fn validate_str(file: &Path, json: &str) -> Vec<TranslationError> {
    match serde_json::from_str::<TranslationFile>(json) {
        Ok(doc) => validate_doc(file, &doc),
        Err(e) => vec![TranslationError {
            file: file.to_path_buf(),
            location: "(parse)".into(),
            message: e.to_string(),
        }],
    }
}

/// Validate an already-parsed document.
pub fn validate_doc(file: &Path, doc: &TranslationFile) -> Vec<TranslationError> {
    let mut errors = Vec::new();
    let push = |errors: &mut Vec<TranslationError>, loc: &str, msg: String| {
        errors.push(TranslationError {
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
            if line.n != i + 1 {
                push(
                    &mut errors,
                    &loc,
                    format!("line numbers must be dense and 1-based (got {} at position {})", line.n, i + 1),
                );
            }
            if line.fr.trim().is_empty() {
                push(&mut errors, &lloc, "fr text is empty".into());
            }
            if line.en.trim().is_empty() {
                push(&mut errors, &lloc, "en text is empty".into());
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

/// Collect every French (`fr`) string in a translation file, for applying
/// French typography checks to the French side **only** (the English `en`
/// side must not be held to French typographic rules). Each string is paired
/// with a semantic locator (`exercise 'id' › line n`) — positions inside the
/// generated JSON are ephemeral, and the exercise id is the source dialog
/// slug, so this points at the `.txt` line to actually edit. Returns an empty
/// vector if the JSON does not parse (structural validation reports that).
pub fn french_texts_located(json: &str) -> Vec<(String, String)> {
    match serde_json::from_str::<TranslationFile>(json) {
        Ok(doc) => doc
            .exercises
            .into_iter()
            .flat_map(|e| {
                let id = e.id;
                e.lines
                    .into_iter()
                    .map(move |l| (format!("exercise '{id}' › line {}", l.n), l.fr))
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

// ── File discovery ──────────────────────────────────────────────────────

/// Recursively collect every file named `translation-data.json` under `dir`.
pub fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out)?;
        } else if path.file_name().and_then(|n| n.to_str()) == Some("translation-data.json") {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> PathBuf {
        PathBuf::from("translation-data.json")
    }

    fn messages<E: fmt::Display>(errors: &[E]) -> String {
        errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(" | ")
    }

    // ── Alignment ───────────────────────────────────────────────────

    const FR: &str = "Titre\n\nPersonnages :\n- Léa — une passante\n\nLéa : Bonjour.\nMarc : Salut.\n";
    const EN: &str = "Title\n\nCharacters:\n- Lea — a passerby\n\nLea: Hello.\nMarc: Hi.\n";

    #[test]
    fn aligned_files_pass() {
        assert!(check_alignment(FR, EN).is_empty());
    }

    #[test]
    fn translated_role_labels_pass() {
        // Real content translates role labels: the alternation shape matches
        // even though the names differ, so alignment must be clean.
        let fr = "Titre\n\nCliente : Bonjour.\nLe guichetier : Oui ?\nCliente : Un timbre.\n";
        let en = "Title\n\nCustomer: Hello.\nThe clerk: Yes?\nCustomer: A stamp.\n";
        assert!(check_alignment(fr, en).is_empty(), "{}", messages(&check_alignment(fr, en)));
    }

    #[test]
    fn extra_english_line_flagged() {
        let en = "Title\n\nLea: Hello.\nMarc: Hi.\nLea: Bye.\n";
        assert!(messages(&check_alignment(FR, en)).contains("line count mismatch"));
    }

    #[test]
    fn turn_shape_divergence_flagged() {
        // French shape [0,1,0] (A,B,A) vs English [0,0,1] (A,A,B): a turn was
        // merged/split in translation even though the count matches.
        let fr = "Titre\n\nLéa : Un.\nMarc : Deux.\nLéa : Trois.\n";
        let en = "Title\n\nLea: One.\nLea: Two.\nMarc: Three.\n";
        assert!(messages(&check_alignment(fr, en)).contains("turn-taking mismatch at line 2"));
    }

    // ── Validation ──────────────────────────────────────────────────

    const GOOD: &str = r#"{
        "schema": "translation/v1",
        "exercises": [
            { "id": "01_demo", "title": "Démo", "audio_base": "audio/01_demo/lines/",
              "lines": [
                { "n": 1, "speaker": "Léa", "fr": "Bonjour.", "en": "Hello.", "audio": "01_lea.mp3" },
                { "n": 2, "speaker": "Marc", "fr": "Salut.", "en": "Hi." }
              ] }
        ]
    }"#;

    #[test]
    fn good_file_is_valid() {
        assert!(validate_str(&p(), GOOD).is_empty());
    }

    #[test]
    fn unknown_schema_flagged() {
        let json = GOOD.replace("translation/v1", "translation/v9");
        assert!(messages(&validate_str(&p(), &json)).contains("unknown schema"));
    }

    #[test]
    fn empty_en_flagged() {
        let json = r#"{ "schema": "translation/v1", "exercises": [
            { "id": "e", "lines": [ {"n":1,"speaker":"A","fr":"Bonjour","en":"  "} ] }
        ] }"#;
        assert!(messages(&validate_str(&p(), json)).contains("en text is empty"));
    }

    #[test]
    fn duplicate_exercise_id_flagged() {
        let json = r#"{ "schema": "translation/v1", "exercises": [
            { "id": "d", "lines": [ {"n":1,"speaker":"A","fr":"x","en":"y"} ] },
            { "id": "d", "lines": [ {"n":1,"speaker":"A","fr":"x","en":"y"} ] }
        ] }"#;
        assert!(messages(&validate_str(&p(), json)).contains("duplicate exercise id 'd'"));
    }

    #[test]
    fn french_texts_located_extracts_fr_only() {
        let fr = french_texts_located(GOOD);
        assert_eq!(
            fr,
            vec![
                ("exercise '01_demo' › line 1".to_string(), "Bonjour.".to_string()),
                ("exercise '01_demo' › line 2".to_string(), "Salut.".to_string()),
            ]
        );
    }

    #[test]
    fn french_texts_located_empty_on_bad_json() {
        assert!(french_texts_located("{ not json").is_empty());
    }

    // ── Generation ──────────────────────────────────────────────────

    fn chapter(translation: bool) -> ChapterConfig {
        let toml = format!(
            r#"
            [chapter]
            title = "T"
            subtitle = "S"
            vocab_page = "vocabulaire"
            footer_text = "F"
            footer_suffix = "FS"
            translation = {translation}

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

    fn write_parallel(content_dir: &Path, en: &str) {
        std::fs::create_dir_all(content_dir).unwrap();
        std::fs::write(content_dir.join("01_demo.txt"), FR).unwrap();
        std::fs::write(content_dir.join("01_demo_en.txt"), en).unwrap();
    }

    #[test]
    fn generate_zips_and_attaches_audio() {
        let tmp = tempfile::tempdir().unwrap();
        let content = tmp.path().join("content");
        let output = tmp.path().join("out");
        write_parallel(&content, EN);
        let lines_dir = output.join("audio/01_demo/lines");
        std::fs::create_dir_all(&lines_dir).unwrap();
        std::fs::write(lines_dir.join("01_lea.mp3"), b"x").unwrap();

        let path = generate_chapter(&content, &output, &chapter(true)).unwrap().unwrap();
        assert!(validate_file(&path).is_empty());
        let doc: TranslationFile = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let ex = &doc.exercises[0];
        assert_eq!(ex.audio_base.as_deref(), Some("audio/01_demo/lines/"));
        assert_eq!(ex.lines[0].fr, "Bonjour.");
        assert_eq!(ex.lines[0].en, "Hello.");
        assert_eq!(ex.lines[0].audio.as_deref(), Some("01_lea.mp3"));
        assert_eq!(ex.lines[1].audio, None, "line without an MP3 has no audio");
    }

    #[test]
    fn generate_fails_on_misalignment() {
        let tmp = tempfile::tempdir().unwrap();
        let content = tmp.path().join("content");
        let output = tmp.path().join("out");
        // English has an extra line → alignment error → generation fails.
        write_parallel(&content, "Title\n\nLea: Hello.\nMarc: Hi.\nLea: Bye.\n");
        let err = generate_chapter(&content, &output, &chapter(true)).unwrap_err();
        assert!(err.to_string().contains("alignment error"));
        assert!(!output.join("translation-data.json").exists());
    }

    #[test]
    fn generate_respects_opt_out() {
        let tmp = tempfile::tempdir().unwrap();
        let content = tmp.path().join("content");
        let output = tmp.path().join("out");
        write_parallel(&content, EN);
        assert!(generate_chapter(&content, &output, &chapter(false)).unwrap().is_none());
    }
}
