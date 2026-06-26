//! Reading-comprehension exercise data: schema + structural validator.
//!
//! A `reading-data.json` file holds one or more *passages*. Each passage
//! bundles its source text(s) with a set of auto-scorable *items* (questions)
//! and their answer keys. The same file is what the browser fetches at
//! runtime, so validating it here guarantees the answer keys the learner is
//! graded against actually reference real options, statements, and sentences.
//!
//! This module is deliberately validation-only. The deterministic *scoring*
//! formulas (partial credit, span overlap, …) live in the JS quiz engine and
//! are specified in `docs/READING.md`; nothing here computes a score.
//!
//! See `docs/READING.md` for the authoring guide and the canonical example in
//! `examples/reading/neo-ruraux.json`.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::Deserialize;

// ── Schema ──────────────────────────────────────────────────────────────

/// Top-level `reading-data.json` document.
#[derive(Debug, Deserialize)]
pub struct ReadingFile {
    /// Schema marker, e.g. `"reading/v1"`. Optional but recommended.
    #[serde(default)]
    pub schema: Option<String>,
    pub passages: Vec<Passage>,
}

/// A single reading passage and the items asked about it.
///
/// `sources` holds one text for a standard passage, or two-or-more for an
/// AP-style *paired sources* set with cross-source synthesis items.
#[derive(Debug, Deserialize)]
pub struct Passage {
    pub id: String,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    pub sources: Vec<Source>,
    pub items: Vec<Item>,
}

/// One source text, split into individually selectable sentences.
///
/// Sentences are the unit of evidence selection: a `highlight_span` answer
/// references sentences by their zero-based index in this array.
#[derive(Debug, Deserialize)]
pub struct Source {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub attribution: Option<String>,
    pub sentences: Vec<String>,
}

/// A labelled option, statement, matching entry, or word-bank token.
#[derive(Debug, Deserialize)]
pub struct Choice {
    pub id: String,
    pub text: String,
}

/// One question. Common metadata lives here; `kind` carries the
/// type-specific payload and answer key, discriminated by the `type` field.
#[derive(Debug, Deserialize)]
pub struct Item {
    pub id: String,
    /// Maximum points. Must be > 0. Fractional weights are allowed.
    pub points: f64,
    /// Pedagogical skill tag (e.g. `inference`), for assembling balanced forms.
    #[serde(default)]
    pub skill: Option<String>,
    /// Difficulty 1–5, for assembling balanced forms.
    #[serde(default)]
    pub difficulty: Option<u8>,
    /// Which source this item refers to. Optional when there is exactly one.
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(flatten)]
    pub kind: ItemKind,
}

/// Type-specific payload for an [`Item`]. The JSON `type` field selects the
/// variant; remaining fields are flattened alongside the shared [`Item`] keys.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ItemKind {
    /// Single-answer multiple choice.
    McSingle {
        options: Vec<Choice>,
        answer: String,
    },
    /// "Choose all that apply" — partial credit by default.
    MultiSelect {
        options: Vec<Choice>,
        answer: Vec<String>,
        /// `"partial"` (default) or `"all_or_nothing"`.
        #[serde(default)]
        scoring: Option<String>,
    },
    /// IELTS-style Vrai / Faux / Non précisé over several sub-statements.
    /// Answer values are `"V"`, `"F"`, or `"NP"`.
    TrueFalseNotgiven {
        statements: Vec<Choice>,
        answer: BTreeMap<String, String>,
    },
    /// Evidence selection: click the sentence(s) supporting a claim.
    /// Each accepted answer is a set of sentence indices into the source.
    HighlightSpan {
        #[serde(default)]
        min_overlap: Option<f64>,
        accepted: Vec<Vec<usize>>,
    },
    /// Match each left entry to a right entry (right may carry distractors).
    Matching {
        left: Vec<Choice>,
        right: Vec<Choice>,
        answer: BTreeMap<String, String>,
    },
    /// Reconstruct the correct order of the elements.
    Ordering {
        elements: Vec<Choice>,
        answer: Vec<String>,
    },
    /// Fill gaps (`___`) from a shared word bank with extra distractors.
    BankedCloze {
        text: String,
        bank: Vec<Choice>,
        answer: Vec<String>,
    },
}

const TFNG_VALUES: &[&str] = &["V", "F", "NP"];
const GAP_MARKER: &str = "___";

// ── Errors ──────────────────────────────────────────────────────────────

/// A single structural problem found while validating a reading file.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadingError {
    pub file: PathBuf,
    /// Human-readable path to the offending node, e.g.
    /// `passage 'neo_ruraux_b2_01' › item 'q5'`.
    pub location: String,
    pub message: String,
}

impl fmt::Display for ReadingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}: {}",
            self.file.display(),
            self.location,
            self.message,
        )
    }
}

// ── Validation ──────────────────────────────────────────────────────────

/// Parse and validate a `reading-data.json` file on disk.
///
/// Returns every problem found. A JSON parse error is reported as a single
/// error against the file. An empty vector means the file is valid.
pub fn validate_file(path: &Path) -> Vec<ReadingError> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            return vec![ReadingError {
                file: path.to_path_buf(),
                location: "(file)".into(),
                message: format!("could not read file: {e}"),
            }]
        }
    };
    validate_str(path, &text)
}

/// Parse and validate reading JSON already held in memory.
pub fn validate_str(file: &Path, json: &str) -> Vec<ReadingError> {
    match serde_json::from_str::<ReadingFile>(json) {
        Ok(doc) => validate_doc(file, &doc),
        Err(e) => vec![ReadingError {
            file: file.to_path_buf(),
            location: "(parse)".into(),
            message: e.to_string(),
        }],
    }
}

/// Validate an already-parsed document. Useful for tests and tooling.
pub fn validate_doc(file: &Path, doc: &ReadingFile) -> Vec<ReadingError> {
    let mut errors = Vec::new();

    if let Some(schema) = &doc.schema {
        if schema != "reading/v1" {
            errors.push(err(
                file,
                "(file)",
                format!("unknown schema '{schema}', expected 'reading/v1'"),
            ));
        }
    }

    let mut seen_passages = BTreeMap::new();
    for passage in &doc.passages {
        let count = seen_passages.entry(passage.id.clone()).or_insert(0);
        *count += 1;
        if *count == 2 {
            errors.push(err(
                file,
                &format!("passage '{}'", passage.id),
                "duplicate passage id".into(),
            ));
        }
        validate_passage(file, passage, &mut errors);
    }

    errors
}

fn validate_passage(file: &Path, passage: &Passage, errors: &mut Vec<ReadingError>) {
    let loc = format!("passage '{}'", passage.id);

    // Sources.
    if passage.sources.is_empty() {
        errors.push(err(file, &loc, "passage has no sources".into()));
    }
    let mut source_ids: BTreeMap<&str, usize> = BTreeMap::new();
    for source in &passage.sources {
        *source_ids.entry(source.id.as_str()).or_insert(0) += 1;
        if source.sentences.is_empty() {
            errors.push(err(
                file,
                &format!("{loc} › source '{}'", source.id),
                "source has no sentences".into(),
            ));
        }
    }
    for (sid, n) in &source_ids {
        if *n > 1 {
            errors.push(err(
                file,
                &loc,
                format!("duplicate source id '{sid}'"),
            ));
        }
    }

    // Items.
    let mut item_ids: BTreeMap<&str, usize> = BTreeMap::new();
    for item in &passage.items {
        *item_ids.entry(item.id.as_str()).or_insert(0) += 1;
        validate_item(file, passage, item, &source_ids, errors);
    }
    for (iid, n) in &item_ids {
        if *n > 1 {
            errors.push(err(file, &loc, format!("duplicate item id '{iid}'")));
        }
    }
}

fn validate_item(
    file: &Path,
    passage: &Passage,
    item: &Item,
    source_ids: &BTreeMap<&str, usize>,
    errors: &mut Vec<ReadingError>,
) {
    let loc = format!("passage '{}' › item '{}'", passage.id, item.id);
    let push = |errors: &mut Vec<ReadingError>, msg: String| errors.push(err(file, &loc, msg));

    if item.points <= 0.0 || item.points.is_nan() {
        push(errors, format!("points must be > 0 (got {})", item.points));
    }
    if let Some(d) = item.difficulty {
        if !(1..=5).contains(&d) {
            push(errors, format!("difficulty must be 1–5 (got {d})"));
        }
    }
    if let Some(src) = &item.source {
        if !source_ids.contains_key(src.as_str()) {
            push(errors, format!("references unknown source '{src}'"));
        }
    }

    match &item.kind {
        ItemKind::McSingle { options, answer } => {
            let ids = check_choices(options, "options", &push, errors);
            if !ids.contains(answer.as_str()) {
                push(errors, format!("answer '{answer}' is not one of the options"));
            }
        }
        ItemKind::MultiSelect {
            options,
            answer,
            scoring,
        } => {
            let ids = check_choices(options, "options", &push, errors);
            if answer.is_empty() {
                push(errors, "multi_select answer is empty".into());
            }
            check_unique(answer, "answer", &push, errors);
            for a in answer {
                if !ids.contains(a.as_str()) {
                    push(errors, format!("answer '{a}' is not one of the options"));
                }
            }
            if let Some(s) = scoring {
                if s != "partial" && s != "all_or_nothing" {
                    push(
                        errors,
                        format!("scoring must be 'partial' or 'all_or_nothing' (got '{s}')"),
                    );
                }
            }
        }
        ItemKind::TrueFalseNotgiven { statements, answer } => {
            let ids = check_choices(statements, "statements", &push, errors);
            // Answer keys must match the statement ids exactly.
            for sid in &ids {
                if !answer.contains_key(*sid) {
                    push(errors, format!("statement '{sid}' has no answer"));
                }
            }
            for (k, v) in answer {
                if !ids.contains(k.as_str()) {
                    push(errors, format!("answer key '{k}' has no matching statement"));
                }
                if !TFNG_VALUES.contains(&v.as_str()) {
                    push(
                        errors,
                        format!("answer for '{k}' must be V, F, or NP (got '{v}')"),
                    );
                }
            }
        }
        ItemKind::HighlightSpan {
            min_overlap,
            accepted,
        } => {
            if let Some(m) = min_overlap {
                if !(0.0 < *m && *m <= 1.0) {
                    push(errors, format!("min_overlap must be in (0, 1] (got {m})"));
                }
            }
            if accepted.is_empty() {
                push(errors, "highlight_span has no accepted answers".into());
            }
            // Resolve the source the spans index into.
            match resolve_source(passage, item) {
                Ok(source) => {
                    let n = source.sentences.len();
                    for (i, span) in accepted.iter().enumerate() {
                        if span.is_empty() {
                            push(errors, format!("accepted[{i}] is an empty span"));
                        }
                        for idx in span {
                            if *idx >= n {
                                push(
                                    errors,
                                    format!(
                                        "accepted[{i}] sentence index {idx} out of range \
                                         (source '{}' has {n} sentences)",
                                        source.id
                                    ),
                                );
                            }
                        }
                    }
                }
                Err(msg) => push(errors, msg),
            }
        }
        ItemKind::Matching { left, right, answer } => {
            let left_ids = check_choices(left, "left", &push, errors);
            let right_ids = check_choices(right, "right", &push, errors);
            for lid in &left_ids {
                if !answer.contains_key(*lid) {
                    push(errors, format!("left entry '{lid}' has no answer"));
                }
            }
            for (k, v) in answer {
                if !left_ids.contains(k.as_str()) {
                    push(errors, format!("answer key '{k}' has no matching left entry"));
                }
                if !right_ids.contains(v.as_str()) {
                    push(errors, format!("answer '{k}' → '{v}' has no matching right entry"));
                }
            }
        }
        ItemKind::Ordering { elements, answer } => {
            let ids = check_choices(elements, "elements", &push, errors);
            check_unique(answer, "answer", &push, errors);
            if answer.len() != ids.len() {
                push(
                    errors,
                    format!(
                        "answer must order all {} elements (got {})",
                        ids.len(),
                        answer.len()
                    ),
                );
            }
            for a in answer {
                if !ids.contains(a.as_str()) {
                    push(errors, format!("answer '{a}' is not one of the elements"));
                }
            }
        }
        ItemKind::BankedCloze { text, bank, answer } => {
            let ids = check_choices(bank, "bank", &push, errors);
            let gaps = text.matches(GAP_MARKER).count();
            if gaps == 0 {
                push(errors, format!("cloze text has no gaps ('{GAP_MARKER}')"));
            }
            if answer.len() != gaps {
                push(
                    errors,
                    format!("answer has {} entries but text has {gaps} gap(s)", answer.len()),
                );
            }
            for a in answer {
                if !ids.contains(a.as_str()) {
                    push(errors, format!("answer '{a}' is not in the word bank"));
                }
            }
        }
    }
}

/// Resolve which source an item refers to: its explicit `source`, or the lone
/// source when there is exactly one. Ambiguity is an error.
fn resolve_source<'a>(passage: &'a Passage, item: &Item) -> Result<&'a Source, String> {
    if let Some(src) = &item.source {
        return passage
            .sources
            .iter()
            .find(|s| &s.id == src)
            .ok_or_else(|| format!("references unknown source '{src}'"));
    }
    match passage.sources.as_slice() {
        [only] => Ok(only),
        [] => Err("no source to reference".into()),
        _ => Err("must name a 'source' when the passage has several".into()),
    }
}

/// Check that a choice list is non-empty and has unique ids; return the id set.
fn check_choices<'a>(
    choices: &'a [Choice],
    field: &str,
    push: &impl Fn(&mut Vec<ReadingError>, String),
    errors: &mut Vec<ReadingError>,
) -> std::collections::BTreeSet<&'a str> {
    if choices.is_empty() {
        push(errors, format!("{field} is empty"));
    }
    let mut ids = std::collections::BTreeSet::new();
    for c in choices {
        if !ids.insert(c.id.as_str()) {
            push(errors, format!("duplicate {field} id '{}'", c.id));
        }
    }
    ids
}

/// Flag duplicate entries in a list of ids.
fn check_unique(
    values: &[String],
    field: &str,
    push: &impl Fn(&mut Vec<ReadingError>, String),
    errors: &mut Vec<ReadingError>,
) {
    let mut seen = std::collections::BTreeSet::new();
    for v in values {
        if !seen.insert(v.as_str()) {
            push(errors, format!("duplicate {field} entry '{v}'"));
        }
    }
}

fn err(file: &Path, location: &str, message: String) -> ReadingError {
    ReadingError {
        file: file.to_path_buf(),
        location: location.to_string(),
        message,
    }
}

// ── File discovery ──────────────────────────────────────────────────────

/// Recursively collect every file named `reading-data.json` under `dir`.
pub fn collect_reading_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_reading_files(&path, out)?;
        } else if path.file_name().and_then(|n| n.to_str()) == Some("reading-data.json") {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOLDEN: &str = include_str!("../../../examples/reading/neo-ruraux.json");
    const PAIRED: &str = include_str!("../../../examples/reading/semaine-quatre-jours.json");

    fn p() -> PathBuf {
        PathBuf::from("test.json")
    }

    fn validate(json: &str) -> Vec<ReadingError> {
        validate_str(&p(), json)
    }

    fn messages(errors: &[ReadingError]) -> String {
        errors.iter().map(|e| e.message.clone()).collect::<Vec<_>>().join(" | ")
    }

    #[test]
    fn golden_file_is_valid() {
        let errors = validate(GOLDEN);
        assert!(errors.is_empty(), "expected no errors, got: {}", messages(&errors));
    }

    #[test]
    fn golden_file_parses_all_item_kinds() {
        let doc: ReadingFile = serde_json::from_str(GOLDEN).unwrap();
        let kinds: Vec<&str> = doc.passages[0]
            .items
            .iter()
            .map(|i| match i.kind {
                ItemKind::McSingle { .. } => "mc_single",
                ItemKind::MultiSelect { .. } => "multi_select",
                ItemKind::TrueFalseNotgiven { .. } => "true_false_notgiven",
                ItemKind::HighlightSpan { .. } => "highlight_span",
                ItemKind::Matching { .. } => "matching",
                ItemKind::Ordering { .. } => "ordering",
                ItemKind::BankedCloze { .. } => "banked_cloze",
            })
            .collect();
        assert_eq!(
            kinds,
            vec!["mc_single", "mc_single", "mc_single", "true_false_notgiven", "multi_select", "highlight_span"]
        );
    }

    #[test]
    fn paired_source_file_is_valid() {
        let errors = validate(PAIRED);
        assert!(errors.is_empty(), "expected no errors, got: {}", messages(&errors));
    }

    #[test]
    fn paired_source_has_two_sources_and_cross_source_items() {
        let doc: ReadingFile = serde_json::from_str(PAIRED).unwrap();
        let passage = &doc.passages[0];
        assert_eq!(passage.sources.len(), 2, "paired set must have two sources");

        // Cross-source synthesis items omit `source` (they span both texts);
        // the validator must accept that, while single-source items name one.
        let cross: Vec<&Item> = passage
            .items
            .iter()
            .filter(|i| i.skill.as_deref() == Some("cross_source_synthesis"))
            .collect();
        assert!(cross.len() >= 2, "expected at least two cross-source items");
        assert!(
            cross.iter().all(|i| i.source.is_none()),
            "cross-source items should not be pinned to a single source"
        );
    }

    /// Build a minimal one-passage document around a single item's JSON.
    fn wrap(item: &str) -> String {
        format!(
            r#"{{ "passages": [ {{ "id": "p1",
                "sources": [ {{ "id": "A", "sentences": ["s0.", "s1.", "s2."] }} ],
                "items": [ {item} ] }} ] }}"#
        )
    }

    #[test]
    fn parse_error_is_reported() {
        let errors = validate("{ not valid json ");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].location, "(parse)");
    }

    #[test]
    fn unknown_schema_flagged() {
        let json = r#"{ "schema": "reading/v9", "passages": [] }"#;
        let errors = validate(json);
        assert!(messages(&errors).contains("unknown schema"));
    }

    #[test]
    fn duplicate_passage_id_flagged() {
        let json = r#"{ "passages": [
            { "id": "dup", "sources": [{"id":"A","sentences":["x."]}], "items": [] },
            { "id": "dup", "sources": [{"id":"A","sentences":["x."]}], "items": [] }
        ] }"#;
        assert!(messages(&validate(json)).contains("duplicate passage id"));
    }

    #[test]
    fn mc_single_answer_must_be_an_option() {
        let item = r#"{ "type": "mc_single", "id": "q", "points": 1,
            "options": [{"id":"A","text":"a"},{"id":"B","text":"b"}], "answer": "C" }"#;
        assert!(messages(&validate(&wrap(item))).contains("answer 'C' is not one of the options"));
    }

    #[test]
    fn multi_select_unknown_and_duplicate_answers() {
        let item = r#"{ "type": "multi_select", "id": "q", "points": 2,
            "options": [{"id":"A","text":"a"},{"id":"B","text":"b"}],
            "answer": ["A", "A", "Z"] }"#;
        let m = messages(&validate(&wrap(item)));
        assert!(m.contains("duplicate answer entry 'A'"), "{m}");
        assert!(m.contains("answer 'Z' is not one of the options"), "{m}");
    }

    #[test]
    fn multi_select_bad_scoring_flagged() {
        let item = r#"{ "type": "multi_select", "id": "q", "points": 2,
            "options": [{"id":"A","text":"a"},{"id":"B","text":"b"}],
            "answer": ["A"], "scoring": "magic" }"#;
        assert!(messages(&validate(&wrap(item))).contains("scoring must be"));
    }

    #[test]
    fn tfng_bad_value_and_key_mismatch() {
        let item = r#"{ "type": "true_false_notgiven", "id": "q", "points": 2,
            "statements": [{"id":"a","text":"x"},{"id":"b","text":"y"}],
            "answer": {"a": "MAYBE", "c": "V"} }"#;
        let m = messages(&validate(&wrap(item)));
        assert!(m.contains("must be V, F, or NP"), "{m}");
        assert!(m.contains("statement 'b' has no answer"), "{m}");
        assert!(m.contains("answer key 'c' has no matching statement"), "{m}");
    }

    #[test]
    fn highlight_index_out_of_range() {
        // Source A has 3 sentences (indices 0–2); 5 is out of range.
        let item = r#"{ "type": "highlight_span", "id": "q", "points": 1,
            "source": "A", "accepted": [[5]] }"#;
        assert!(messages(&validate(&wrap(item))).contains("out of range"));
    }

    #[test]
    fn highlight_ambiguous_source() {
        let json = r#"{ "passages": [ { "id": "p1",
            "sources": [
                {"id":"A","sentences":["a."]},
                {"id":"B","sentences":["b."]}
            ],
            "items": [ { "type": "highlight_span", "id": "q", "points": 1,
                "accepted": [[0]] } ] } ] }"#;
        assert!(messages(&validate(json)).contains("must name a 'source'"));
    }

    #[test]
    fn matching_value_not_in_right() {
        let item = r#"{ "type": "matching", "id": "q", "points": 2,
            "left": [{"id":"1","text":"x"}],
            "right": [{"id":"a","text":"y"}],
            "answer": {"1": "z"} }"#;
        assert!(messages(&validate(&wrap(item))).contains("no matching right entry"));
    }

    #[test]
    fn ordering_must_be_a_permutation() {
        let item = r#"{ "type": "ordering", "id": "q", "points": 1,
            "elements": [{"id":"1","text":"x"},{"id":"2","text":"y"},{"id":"3","text":"z"}],
            "answer": ["1", "2"] }"#;
        assert!(messages(&validate(&wrap(item))).contains("must order all 3 elements"));
    }

    #[test]
    fn banked_cloze_gap_count_mismatch() {
        let item = r#"{ "type": "banked_cloze", "id": "q", "points": 2,
            "text": "Le ___ mange la ___ rouge.",
            "bank": [{"id":"w1","text":"chat"},{"id":"w2","text":"pomme"},{"id":"w3","text":"chien"}],
            "answer": ["w1"] }"#;
        assert!(messages(&validate(&wrap(item))).contains("but text has 2 gap(s)"));
    }

    #[test]
    fn points_must_be_positive() {
        let item = r#"{ "type": "mc_single", "id": "q", "points": 0,
            "options": [{"id":"A","text":"a"},{"id":"B","text":"b"}], "answer": "A" }"#;
        assert!(messages(&validate(&wrap(item))).contains("points must be > 0"));
    }

    #[test]
    fn difficulty_out_of_range() {
        let item = r#"{ "type": "mc_single", "id": "q", "points": 1, "difficulty": 9,
            "options": [{"id":"A","text":"a"},{"id":"B","text":"b"}], "answer": "A" }"#;
        assert!(messages(&validate(&wrap(item))).contains("difficulty must be 1–5"));
    }

    #[test]
    fn duplicate_item_id_flagged() {
        let json = r#"{ "passages": [ { "id": "p1",
            "sources": [{"id":"A","sentences":["x."]}],
            "items": [
                { "type": "mc_single", "id": "dup", "points": 1,
                  "options":[{"id":"A","text":"a"},{"id":"B","text":"b"}], "answer":"A" },
                { "type": "mc_single", "id": "dup", "points": 1,
                  "options":[{"id":"A","text":"a"},{"id":"B","text":"b"}], "answer":"A" }
            ] } ] }"#;
        assert!(messages(&validate(json)).contains("duplicate item id 'dup'"));
    }

    #[test]
    fn unknown_source_reference_flagged() {
        let item = r#"{ "type": "mc_single", "id": "q", "points": 1, "source": "Z",
            "options": [{"id":"A","text":"a"},{"id":"B","text":"b"}], "answer": "A" }"#;
        assert!(messages(&validate(&wrap(item))).contains("references unknown source 'Z'"));
    }
}
