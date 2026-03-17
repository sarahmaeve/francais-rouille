use std::fmt;
use std::path::{Path, PathBuf};

/// A single typographic rule violation in a content file.
#[derive(Debug, Clone, PartialEq)]
pub struct Violation {
    pub file: PathBuf,
    pub line: usize,
    pub col: usize,
    pub rule: &'static str,
    pub found: String,
    pub expected: String,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}: [{}] found {}, expected {}",
            self.file.display(),
            self.line,
            self.col,
            self.rule,
            self.found,
            self.expected,
        )
    }
}

/// Language-specific typographic rules for content verification.
pub trait TypographyRules {
    /// BCP 47 language code this rule set applies to.
    fn language_code(&self) -> &'static str;

    /// Enable strict mode, which activates additional rules that may be noisy.
    fn set_strict(&mut self, strict: bool);

    /// Check a single line of text and return any violations found.
    fn check_line(&self, line: &str, line_number: usize) -> Vec<Violation>;

    /// Apply fixes to a single line of text, returning the corrected version.
    fn fix_line(&self, line: &str) -> String;
}

// ── French (fr-FR) ──────────────────────────────────────────────────────

/// French typographic rules for fr-FR content.
///
/// Default rule:
/// 1. Ellipsis character (U+2026) instead of three ASCII dots.
///
/// Strict-only rule:
/// 2. Narrow no-break space (U+202F) before high punctuation (; : ! ?).
pub struct FrenchTypography {
    strict: bool,
}

/// High punctuation marks that require a preceding space in French.
const HIGH_PUNCTUATION: &[char] = &[';', ':', '!', '?'];

impl TypographyRules for FrenchTypography {
    fn language_code(&self) -> &'static str {
        "fr-FR"
    }

    fn set_strict(&mut self, strict: bool) {
        self.strict = strict;
    }

    fn check_line(&self, line: &str, line_number: usize) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Rule 1: Three consecutive dots → ellipsis character
        if let Some(byte_pos) = line.find("...") {
            // Avoid matching four or more dots (not an ellipsis).
            let before_ok = byte_pos == 0
                || line.as_bytes()[byte_pos - 1] != b'.';
            let after_ok = byte_pos + 3 >= line.len()
                || line.as_bytes()[byte_pos + 3] != b'.';
            if before_ok && after_ok {
                let col = line[..byte_pos].chars().count() + 1;
                violations.push(Violation {
                    file: PathBuf::new(),
                    line: line_number,
                    col,
                    rule: "ellipsis",
                    found: "... (three dots)".into(),
                    expected: "\u{2026} (U+2026)".into(),
                });
            }
        }

        // Rule 3 (strict only): Narrow no-break space before high punctuation
        if self.strict {
            for &punct in HIGH_PUNCTUATION {
                for (byte_pos, _) in line.match_indices(punct) {
                    if byte_pos == 0 {
                        continue;
                    }

                    let before = &line[..byte_pos];
                    let prev_char = before.chars().next_back();

                    match prev_char {
                        Some('\u{202F}') => {}
                        Some(' ') | Some('\u{00A0}') => {
                            let col = before.chars().count();
                            violations.push(Violation {
                                file: PathBuf::new(),
                                line: line_number,
                                col,
                                rule: "punctuation-space",
                                found: format!(
                                    "regular space before '{}'",
                                    punct
                                ),
                                expected: format!(
                                    "narrow no-break space (U+202F) before '{}'",
                                    punct
                                ),
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        violations
    }

    fn fix_line(&self, line: &str) -> String {
        let mut result = String::with_capacity(line.len());
        let bytes = line.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            // Rule 1: Three-dot ellipsis.
            if i + 2 < bytes.len()
                && bytes[i] == b'.'
                && bytes[i + 1] == b'.'
                && bytes[i + 2] == b'.'
            {
                let before_ok = i == 0 || bytes[i - 1] != b'.';
                let after_ok = i + 3 >= bytes.len() || bytes[i + 3] != b'.';
                if before_ok && after_ok {
                    result.push('\u{2026}');
                    i += 3;
                    continue;
                }
            }

            // Rule 2 (strict only): Replace regular/non-breaking space before
            // high punctuation with narrow no-break space.
            if self.strict
                && (bytes[i] == b' ' || line[i..].starts_with('\u{00A0}'))
                && i > 0
            {
                let skip = if bytes[i] == b' ' { 1 } else { 2 }; // NBSP is 2 bytes in UTF-8
                if i + skip < line.len() {
                    let next_char = line[i + skip..].chars().next();
                    if let Some(ch) = next_char {
                        if HIGH_PUNCTUATION.contains(&ch) {
                            result.push('\u{202F}');
                            i += skip;
                            continue;
                        }
                    }
                }
            }

            // Default: copy the character as-is.
            let ch = line[i..].chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
        }

        result
    }
}

/// Return the typography rules for a given language code, if supported.
pub fn rules_for_language(code: &str, strict: bool) -> Option<Box<dyn TypographyRules>> {
    match code {
        "fr-FR" => Some(Box::new(FrenchTypography { strict })),
        _ => None,
    }
}

// ── File scanning ───────────────────────────────────────────────────────

/// Check all content files in `content_dir` for typography violations.
///
/// Scans `.txt` and `.md` files, skipping `_en.md` (English translations),
/// `.toml`, and `.html` files.
pub fn verify_files(
    content_dir: &Path,
    rules: &dyn TypographyRules,
) -> Result<Vec<Violation>, std::io::Error> {
    let mut all_violations = Vec::new();

    let mut files: Vec<PathBuf> = Vec::new();
    collect_content_files(content_dir, &mut files)?;
    files.sort();

    for file in &files {
        let text = std::fs::read_to_string(file)?;
        for (i, line) in text.lines().enumerate() {
            let mut line_violations = rules.check_line(line, i + 1);
            for v in &mut line_violations {
                v.file.clone_from(file);
            }
            all_violations.extend(line_violations);
        }
    }

    Ok(all_violations)
}

/// Fix all content files in `content_dir` in place, returning the count of
/// files modified.
pub fn fix_files(
    content_dir: &Path,
    rules: &dyn TypographyRules,
) -> Result<usize, std::io::Error> {
    let mut files: Vec<PathBuf> = Vec::new();
    collect_content_files(content_dir, &mut files)?;
    files.sort();

    let mut modified_count = 0;

    for file in &files {
        let original = std::fs::read_to_string(file)?;
        let fixed: String = original
            .lines()
            .map(|line| rules.fix_line(line))
            .collect::<Vec<_>>()
            .join("\n");

        // Preserve trailing newline if the original had one.
        let fixed = if original.ends_with('\n') && !fixed.ends_with('\n') {
            fixed + "\n"
        } else {
            fixed
        };

        if fixed != original {
            std::fs::write(file, &fixed)?;
            modified_count += 1;
        }
    }

    Ok(modified_count)
}

/// Recursively collect `.txt` and `.md` content files, excluding English
/// translations (`_en.md`), `.toml`, and `.html`.
fn collect_content_files(
    dir: &Path,
    out: &mut Vec<PathBuf>,
) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_content_files(&path, out)?;
            continue;
        }

        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        // Skip English translations — they follow English typography.
        if name.ends_with("_en.md") {
            continue;
        }

        if name.ends_with(".txt") || name.ends_with(".md") {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn french() -> FrenchTypography {
        FrenchTypography { strict: false }
    }

    fn french_strict() -> FrenchTypography {
        FrenchTypography { strict: true }
    }

    // ── Ellipsis rule ───────────────────────────────────────────────

    #[test]
    fn detects_three_dot_ellipsis() {
        let v = french().check_line("Bon...", 1);
        assert_eq!(v.iter().filter(|v| v.rule == "ellipsis").count(), 1);
    }

    #[test]
    fn accepts_ellipsis_character() {
        let v = french().check_line("Bon\u{2026}", 1);
        assert!(v.iter().all(|v| v.rule != "ellipsis"));
    }

    #[test]
    fn ignores_four_dots() {
        let v = french().check_line("Hmm....", 1);
        assert!(v.iter().all(|v| v.rule != "ellipsis"));
    }

    // ── Punctuation space rule ──────────────────────────────────────

    #[test]
    fn detects_regular_space_before_exclamation_in_strict() {
        let v = french_strict().check_line("Bonjour !", 1);
        assert_eq!(v.iter().filter(|v| v.rule == "punctuation-space").count(), 1);
    }

    #[test]
    fn default_mode_skips_punctuation_space() {
        let v = french().check_line("Bonjour !", 1);
        assert!(v.iter().all(|v| v.rule != "punctuation-space"));
    }

    #[test]
    fn detects_regular_space_before_all_high_punct_in_strict() {
        for punct in [';', ':', '!', '?'] {
            let line = format!("mot {}", punct);
            let v = french_strict().check_line(&line, 1);
            assert!(
                v.iter().any(|v| v.rule == "punctuation-space"),
                "expected violation for '{}'",
                punct,
            );
        }
    }

    #[test]
    fn accepts_nnbsp_before_punctuation() {
        let v = french_strict().check_line("Bonjour\u{202F}!", 1);
        assert!(v.iter().all(|v| v.rule != "punctuation-space"));
    }

    #[test]
    fn no_false_positive_for_no_space() {
        // URLs, times, etc. — no space before colon is not flagged.
        let v = french_strict().check_line("http://example.com", 1);
        assert!(v.iter().all(|v| v.rule != "punctuation-space"));
    }

    // ── Fix line ────────────────────────────────────────────────────

    #[test]
    fn fix_replaces_three_dots() {
        assert_eq!(
            french().fix_line("Bon..."),
            "Bon\u{2026}",
        );
    }

    #[test]
    fn fix_replaces_space_before_punctuation_in_strict() {
        assert_eq!(
            french_strict().fix_line("Bonjour !"),
            "Bonjour\u{202F}!",
        );
    }

    #[test]
    fn fix_preserves_space_before_punctuation_in_default() {
        assert_eq!(
            french().fix_line("Bonjour !"),
            "Bonjour !",
        );
    }

    #[test]
    fn fix_handles_multiple_rules_at_once() {
        assert_eq!(
            french_strict().fix_line("j'ai dit : Bon..."),
            "j'ai dit\u{202F}: Bon\u{2026}",
        );
    }

    #[test]
    fn fix_idempotent() {
        let line = "l'homme dit\u{202F}: Bon\u{2026}";
        assert_eq!(french_strict().fix_line(line), line);
    }

    // ── File collection ─────────────────────────────────────────────

    #[test]
    fn collect_content_files_skips_english() {
        let dir = std::env::temp_dir().join("typo_test_collect");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(dir.join("dialog.txt"), "test").unwrap();
        std::fs::write(dir.join("dialog.md"), "test").unwrap();
        std::fs::write(dir.join("dialog_en.md"), "test").unwrap();
        std::fs::write(dir.join("chapter.toml"), "test").unwrap();
        std::fs::write(dir.join("map.html"), "test").unwrap();

        let mut files = Vec::new();
        collect_content_files(&dir, &mut files).unwrap();
        files.sort();

        let names: Vec<&str> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, vec!["dialog.md", "dialog.txt"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── Verify and fix integration ──────────────────────────────────

    #[test]
    fn verify_finds_violations_in_default_mode() {
        let dir = std::env::temp_dir().join("typo_test_verify");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(dir.join("test.txt"), "l'homme dit : Bon...\n").unwrap();

        let rules = FrenchTypography { strict: false };
        let violations = verify_files(&dir, &rules).unwrap();
        assert_eq!(violations.len(), 1); // ellipsis

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_finds_all_violations_in_strict_mode() {
        let dir = std::env::temp_dir().join("typo_test_verify_strict");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(dir.join("test.txt"), "l'homme dit : Bon...\n").unwrap();

        let rules = FrenchTypography { strict: true };
        let violations = verify_files(&dir, &rules).unwrap();
        assert_eq!(violations.len(), 2); // ellipsis + space

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn fix_files_default_mode() {
        let dir = std::env::temp_dir().join("typo_test_fix_default");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(dir.join("test.txt"), "l'homme dit : Bon...\n").unwrap();

        let rules = FrenchTypography { strict: false };
        let count = fix_files(&dir, &rules).unwrap();
        assert_eq!(count, 1);

        let fixed = std::fs::read_to_string(dir.join("test.txt")).unwrap();
        assert_eq!(fixed, "l'homme dit : Bon\u{2026}\n");

        let count = fix_files(&dir, &rules).unwrap();
        assert_eq!(count, 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn fix_files_strict_mode() {
        let dir = std::env::temp_dir().join("typo_test_fix_strict");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(dir.join("test.txt"), "l'homme dit : Bon...\n").unwrap();

        let rules = FrenchTypography { strict: true };
        let count = fix_files(&dir, &rules).unwrap();
        assert_eq!(count, 1);

        let fixed = std::fs::read_to_string(dir.join("test.txt")).unwrap();
        assert_eq!(fixed, "l'homme dit\u{202F}: Bon\u{2026}\n");

        let count = fix_files(&dir, &rules).unwrap();
        assert_eq!(count, 0);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
