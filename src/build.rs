use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::dialog::{self, slugify};

/// Chapter configuration loaded from `chapter.toml`.
#[derive(Debug, Deserialize)]
pub struct ChapterConfig {
    pub chapter: ChapterMeta,
    pub sections: Vec<SectionConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChapterMeta {
    pub title: String,
    pub subtitle: String,
    #[serde(default = "default_level")]
    pub level: String,
    pub vocab_page: String,
    pub footer_text: String,
    pub footer_suffix: String,
}

fn default_level() -> String {
    "B1".to_string()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SectionConfig {
    pub heading: String,
    pub pages: Vec<PageConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PageConfig {
    pub slug: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub page_type: String,
    pub subtitle: Option<String>,
    pub audio_dir: Option<String>,
}

/// Section data for the chapter index template, enriched with audio info.
#[derive(Debug, Serialize)]
struct IndexSectionData {
    heading: String,
    pages: Vec<IndexPageData>,
}

/// Per-page data for the chapter index template.
#[derive(Debug, Serialize)]
struct IndexPageData {
    slug: String,
    title: String,
    description: String,
    has_audio: bool,
}

/// Data passed to dialog templates for each spoken line.
#[derive(Debug, Serialize)]
struct DialogLineData {
    index: String,
    speaker: String,
    speaker_class: String,
    text: String,
    audio_file: String,
}

/// Character name and description for the personnages box.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Character {
    pub name: String,
    pub description: String,
}

/// Parse character names and full descriptions from a dialog `.txt` file.
///
/// Looks for lines starting with `-` that contain an em-dash or en-dash
/// separator, e.g. `- Claire — une cliente curieuse`.
pub fn parse_characters(content: &str) -> Vec<Character> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if !line.starts_with('-') {
                return None;
            }
            let line = line.trim_start_matches('-').trim();

            let (name, description) = if let Some(pos) = line.find(" — ") {
                (&line[..pos], &line[pos + " — ".len()..])
            } else if let Some(pos) = line.find(" – ") {
                (&line[..pos], &line[pos + " – ".len()..])
            } else {
                return None;
            };

            Some(Character {
                name: name.trim().to_string(),
                description: description.trim().to_string(),
            })
        })
        .collect()
}

/// Build all HTML pages for a single chapter.
///
/// Reads `chapter.toml` from `content_dir`, renders templates from
/// `templates_dir`, and writes generated HTML into `output_dir`.
/// Pages marked as `"static"` are skipped (they already exist in the
/// output directory).
pub fn build_chapter(
    content_dir: &Path,
    output_dir: &Path,
    templates_dir: &Path,
    site_url: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let template_glob = format!("{}/**/*.html", templates_dir.display());
    let tera = Tera::new(&template_glob)?;

    let config_str = std::fs::read_to_string(content_dir.join("chapter.toml"))?;
    let config: ChapterConfig = toml::from_str(&config_str)?;

    // Derive chapter name from the content directory for canonical URLs.
    let chapter_name = content_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let base_url = site_url.map(|u| {
        format!(
            "{}/chapters/{}",
            u.trim_end_matches('/'),
            chapter_name
        )
    });

    std::fs::create_dir_all(output_dir)?;

    for section in &config.sections {
        for page in &section.pages {
            match page.page_type.as_str() {
                "dialog" => {
                    build_dialog_page(
                        &tera, &config, page, content_dir, output_dir, base_url.as_deref(),
                    )?;
                }
                "fragment" => {
                    build_fragment_page(
                        &tera, &config, page, content_dir, output_dir, base_url.as_deref(),
                    )?;
                }
                "static" => {
                    println!("  skip (static): {}.html", page.slug);
                }
                other => {
                    eprintln!("  warning: unknown page type '{other}' for {}", page.slug);
                }
            }
        }
    }

    build_chapter_index(&tera, &config, output_dir, base_url.as_deref())?;
    Ok(())
}

fn build_dialog_page(
    tera: &Tera,
    config: &ChapterConfig,
    page: &PageConfig,
    content_dir: &Path,
    output_dir: &Path,
    base_url: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let txt_path = content_dir.join(format!("{}.txt", page.slug));
    let content = std::fs::read_to_string(&txt_path)?;

    let characters = parse_characters(&content);
    let dialog_lines = dialog::parse_dialog(&content);

    // Assign speaker classes in order of first appearance.
    let classes = ["speaker-a", "speaker-b", "speaker-c", "speaker-d"];
    let mut speaker_classes: HashMap<String, String> = HashMap::new();
    let mut class_idx = 0;
    for line in &dialog_lines {
        if !speaker_classes.contains_key(&line.speaker) {
            speaker_classes.insert(
                line.speaker.clone(),
                classes[class_idx % classes.len()].to_string(),
            );
            class_idx += 1;
        }
    }

    let audio_dir = page
        .audio_dir
        .clone()
        .unwrap_or_else(|| page.slug.clone());

    let has_audio = output_dir
        .join("audio")
        .join(&audio_dir)
        .join("lines")
        .is_dir();

    let lines_data: Vec<DialogLineData> = dialog_lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let index = format!("{:02}", i + 1);
            let audio_file = format!("{}_{}.mp3", index, slugify(&line.speaker));
            DialogLineData {
                index,
                speaker: line.speaker.clone(),
                speaker_class: speaker_classes[&line.speaker].clone(),
                text: line.text.clone(),
                audio_file,
            }
        })
        .collect();

    let mut ctx = Context::new();
    ctx.insert("chapter", &config.chapter);
    ctx.insert("title", &page.title);
    ctx.insert("subtitle", &page.subtitle);
    ctx.insert("description", &page.description);
    ctx.insert("slug", &page.slug);
    ctx.insert("vocab_page", &config.chapter.vocab_page);
    ctx.insert("has_audio", &has_audio);
    ctx.insert("audio_dir", &audio_dir);
    ctx.insert("personnages", &characters);
    ctx.insert("lines", &lines_data);
    if let Some(base) = base_url {
        ctx.insert("canonical_url", &format!("{}/{}.html", base, page.slug));
    }

    let html = tera.render("dialog.html", &ctx)?;
    let out_path = output_dir.join(format!("{}.html", page.slug));
    std::fs::write(&out_path, html)?;

    println!(
        "  wrote {}.html ({} lines, {} characters)",
        page.slug,
        dialog_lines.len(),
        characters.len()
    );
    Ok(())
}

fn build_fragment_page(
    tera: &Tera,
    config: &ChapterConfig,
    page: &PageConfig,
    content_dir: &Path,
    output_dir: &Path,
    base_url: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let fragment_path = content_dir.join(format!("{}.html", page.slug));
    let fragment = std::fs::read_to_string(&fragment_path)?;

    // Optional extra CSS from a companion .css file.
    let css_path = content_dir.join(format!("{}.css", page.slug));
    let extra_css = std::fs::read_to_string(&css_path).ok();

    let mut ctx = Context::new();
    ctx.insert("chapter", &config.chapter);
    ctx.insert("title", &page.title);
    ctx.insert("subtitle", &page.subtitle);
    ctx.insert("description", &page.description);
    ctx.insert("slug", &page.slug);
    ctx.insert("vocab_page", &config.chapter.vocab_page);
    ctx.insert("has_audio", &false);
    ctx.insert("content", &fragment);
    ctx.insert("extra_css", &extra_css);
    if let Some(base) = base_url {
        ctx.insert("canonical_url", &format!("{}/{}.html", base, page.slug));
    }

    let html = tera.render("fragment.html", &ctx)?;
    let out_path = output_dir.join(format!("{}.html", page.slug));
    std::fs::write(&out_path, html)?;

    println!("  wrote {}.html (fragment)", page.slug);
    Ok(())
}

fn build_chapter_index(
    tera: &Tera,
    config: &ChapterConfig,
    output_dir: &Path,
    base_url: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let sections: Vec<IndexSectionData> = config
        .sections
        .iter()
        .map(|section| {
            let pages = section
                .pages
                .iter()
                .map(|page| {
                    let audio_dir = page
                        .audio_dir
                        .clone()
                        .unwrap_or_else(|| page.slug.clone());
                    let has_audio = output_dir
                        .join("audio")
                        .join(&audio_dir)
                        .join("lines")
                        .is_dir();

                    IndexPageData {
                        slug: page.slug.clone(),
                        title: page.title.clone(),
                        description: page.description.clone(),
                        has_audio,
                    }
                })
                .collect();

            IndexSectionData {
                heading: section.heading.clone(),
                pages,
            }
        })
        .collect();

    let mut ctx = Context::new();
    ctx.insert("chapter", &config.chapter);
    ctx.insert("sections", &sections);
    if let Some(base) = base_url {
        ctx.insert("canonical_url", &format!("{}/index.html", base));
    }

    let html = tera.render("chapter_index.html", &ctx)?;
    std::fs::write(output_dir.join("index.html"), html)?;

    println!("  wrote index.html");
    Ok(())
}

/// Generate a `sitemap.xml` listing all HTML pages under `site_dir`.
///
/// Walks the site directory, collects `.html` pages (skipping `404.html`),
/// and writes a standard XML sitemap. Dialog and chapter index pages get
/// higher priority than translations and reference pages.
pub fn generate_sitemap(
    site_dir: &Path,
    site_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let base = site_url.trim_end_matches('/');

    let mut urls: Vec<SitemapEntry> = Vec::new();

    collect_html_pages(site_dir, site_dir, base, &mut urls)?;
    urls.sort_by(|a, b| a.loc.cmp(&b.loc));

    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );

    for entry in &urls {
        write!(
            xml,
            "  <url>\n    <loc>{}</loc>\n    <priority>{:.1}</priority>\n  </url>\n",
            entry.loc, entry.priority,
        )?;
    }
    xml.push_str("</urlset>\n");

    let out_path = site_dir.join("sitemap.xml");
    std::fs::write(&out_path, &xml)?;
    println!("Wrote sitemap.xml ({} URLs)", urls.len());
    Ok(())
}

struct SitemapEntry {
    loc: String,
    priority: f32,
}

fn collect_html_pages(
    dir: &Path,
    site_root: &Path,
    base_url: &str,
    urls: &mut Vec<SitemapEntry>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Skip audio directories entirely.
            if path.file_name().is_some_and(|n| n == "audio") {
                continue;
            }
            collect_html_pages(&path, site_root, base_url, urls)?;
        } else if path.extension().is_some_and(|e| e == "html") {
            let name = path.file_name().unwrap().to_string_lossy();
            if name == "404.html" {
                continue;
            }

            let rel = path
                .strip_prefix(site_root)?
                .to_string_lossy()
                .replace('\\', "/");

            let priority = classify_priority(&rel);
            urls.push(SitemapEntry {
                loc: format!("{base_url}/{rel}"),
                priority,
            });
        }
    }
    Ok(())
}

fn classify_priority(rel_path: &str) -> f32 {
    if rel_path == "index.html" {
        1.0
    } else if rel_path.ends_with("/index.html") {
        0.8
    } else if rel_path.contains("/translations/") {
        0.3
    } else {
        0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_DIALOG: &str = "\
Visite à la Boulangerie — Les Spécialités de la Maison

Personnages :
- Claire — une cliente curieuse qui entre dans la boulangerie
- Monsieur Duval — le propriétaire et pâtissier de la boulangerie

Claire : Bonjour, monsieur ! Votre vitrine est magnifique.

Monsieur Duval : Bonjour, madame ! Merci, c'est gentil.

Claire : Qu'est-ce que vous me recommandez ?

Monsieur Duval : Notre éclair au chocolat noir, c'est notre meilleure vente.
";

    #[test]
    fn parse_characters_extracts_names_and_descriptions() {
        let chars = parse_characters(SAMPLE_DIALOG);
        assert_eq!(chars.len(), 2);
        assert_eq!(chars[0].name, "Claire");
        assert_eq!(
            chars[0].description,
            "une cliente curieuse qui entre dans la boulangerie"
        );
        assert_eq!(chars[1].name, "Monsieur Duval");
        assert_eq!(
            chars[1].description,
            "le propriétaire et pâtissier de la boulangerie"
        );
    }

    #[test]
    fn parse_characters_handles_en_dash() {
        let input = "- Léa – une touriste à Paris";
        let chars = parse_characters(input);
        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].name, "Léa");
        assert_eq!(chars[0].description, "une touriste à Paris");
    }

    #[test]
    fn parse_characters_skips_non_character_lines() {
        let input = "\
Title Line

Personnages :
- Claire — une cliente
- Marc — un vendeur

Claire : Bonjour !
Marc : Salut !
";
        let chars = parse_characters(input);
        assert_eq!(chars.len(), 2);
    }

    #[test]
    fn parse_characters_empty_input() {
        assert!(parse_characters("").is_empty());
        assert!(parse_characters("Just a title\nNo characters").is_empty());
    }

    #[test]
    fn chapter_config_deserializes() {
        let toml_str = r#"
[chapter]
title = "Test"
subtitle = "Test subtitle"
vocab_page = "vocabulaire"
footer_text = "Footer"
footer_suffix = "B1"

[[sections]]
heading = "Dialogues"

[[sections.pages]]
slug = "01_test"
title = "Test Page"
description = "A test."
type = "dialog"

[[sections.pages]]
slug = "02_static"
title = "Static Page"
description = "Already exists."
type = "static"
"#;
        let config: ChapterConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.chapter.title, "Test");
        assert_eq!(config.sections.len(), 1);
        assert_eq!(config.sections[0].pages.len(), 2);
        assert_eq!(config.sections[0].pages[0].page_type, "dialog");
        assert_eq!(config.sections[0].pages[1].page_type, "static");
    }

    #[test]
    fn chapter_config_optional_fields() {
        let toml_str = r#"
[chapter]
title = "T"
subtitle = "S"
vocab_page = "v"
footer_text = "F"
footer_suffix = "B"

[[sections]]
heading = "H"

[[sections.pages]]
slug = "p"
title = "P"
description = "D"
type = "dialog"
subtitle = "Custom subtitle"
audio_dir = "custom/audio/path"
"#;
        let config: ChapterConfig = toml::from_str(toml_str).unwrap();
        let page = &config.sections[0].pages[0];
        assert_eq!(page.subtitle.as_deref(), Some("Custom subtitle"));
        assert_eq!(page.audio_dir.as_deref(), Some("custom/audio/path"));
    }

    #[test]
    fn dialog_template_renders() {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            (
                "base.html",
                r#"<!DOCTYPE html>
<html lang="{{ lang | default(value='fr') }}">
<head><title>{{ title }}</title></head>
<body>
{% block main %}{% endblock main %}
{% if has_audio %}<script>audio()</script>{% endif %}
</body></html>"#,
            ),
            (
                "dialog.html",
                r#"{% extends "base.html" %}
{% block main %}
<div class="personnages">
{% for c in personnages %}<li>{{ c.name }} — {{ c.description }}</li>
{% endfor %}</div>
{% if has_audio %}<button onclick="playAll(this)">Play all</button>{% endif %}
<div class="dialogue">
{% for line in lines %}<div class="{{ line.speaker_class }}">{{ line.speaker }} : {{ line.text }}</div>
{% endfor %}</div>
{% endblock main %}"#,
            ),
        ])
        .unwrap();

        let characters = parse_characters(SAMPLE_DIALOG);
        let dialog_lines = dialog::parse_dialog(SAMPLE_DIALOG);

        let classes = ["speaker-a", "speaker-b"];
        let mut speaker_classes: HashMap<String, String> = HashMap::new();
        let mut idx = 0;
        for line in &dialog_lines {
            if !speaker_classes.contains_key(&line.speaker) {
                speaker_classes
                    .insert(line.speaker.clone(), classes[idx % classes.len()].to_string());
                idx += 1;
            }
        }

        let lines_data: Vec<DialogLineData> = dialog_lines
            .iter()
            .enumerate()
            .map(|(i, line)| DialogLineData {
                index: format!("{:02}", i + 1),
                speaker: line.speaker.clone(),
                speaker_class: speaker_classes[&line.speaker].clone(),
                text: line.text.clone(),
                audio_file: format!("{:02}_{}.mp3", i + 1, slugify(&line.speaker)),
            })
            .collect();

        let mut ctx = Context::new();
        ctx.insert("title", "Test Dialog");
        ctx.insert("subtitle", &None::<String>);
        ctx.insert("slug", "test");
        ctx.insert("vocab_page", "vocabulaire");
        ctx.insert("has_audio", &true);
        ctx.insert("audio_dir", "test_dialog");
        ctx.insert("personnages", &characters);
        ctx.insert("lines", &lines_data);

        let html = tera.render("dialog.html", &ctx).unwrap();

        assert!(html.contains("Claire — une cliente curieuse"));
        assert!(html.contains("Monsieur Duval — le propriétaire"));
        assert!(html.contains("playAll(this)"));
        assert!(html.contains("speaker-a"));
        assert!(html.contains("speaker-b"));
        assert!(html.contains("Bonjour, monsieur !"));
        assert!(html.contains("Notre éclair au chocolat noir"));
    }

    #[test]
    fn dialog_template_without_audio() {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            (
                "base.html",
                r#"<html>
<body>{% block main %}{% endblock main %}
{% if has_audio %}<script>audio()</script>{% endif %}
</body></html>"#,
            ),
            (
                "dialog.html",
                r#"{% extends "base.html" %}
{% block main %}
{% if has_audio %}<button onclick="playAll(this)">Play all</button>{% endif %}
{% for line in lines %}<div>{{ line.text }}</div>
{% endfor %}
{% endblock main %}"#,
            ),
        ])
        .unwrap();

        let dialog_lines = dialog::parse_dialog(SAMPLE_DIALOG);
        let lines_data: Vec<DialogLineData> = dialog_lines
            .iter()
            .enumerate()
            .map(|(i, line)| DialogLineData {
                index: format!("{:02}", i + 1),
                speaker: line.speaker.clone(),
                speaker_class: "speaker-a".to_string(),
                text: line.text.clone(),
                audio_file: format!("{:02}.mp3", i + 1),
            })
            .collect();

        let mut ctx = Context::new();
        ctx.insert("title", "Test");
        ctx.insert("has_audio", &false);
        ctx.insert("lines", &lines_data);

        let html = tera.render("dialog.html", &ctx).unwrap();

        assert!(!html.contains("<audio"));
        assert!(!html.contains("<script>"));
        assert!(html.contains("Bonjour, monsieur !"));
    }

    #[test]
    fn chapter_index_template_renders() {
        let mut tera = Tera::default();
        tera.add_raw_template(
            "chapter_index.html",
            r#"<h1>{{ chapter.title }}</h1>
{% for section in sections %}
<h2>{{ section.heading }}</h2>
{% for page in section.pages %}
<a href="{{ page.slug }}.html">{{ page.title }}{% if page.has_audio %} [audio]{% endif %}</a>
{% endfor %}
{% endfor %}"#,
        )
        .unwrap();

        let config: ChapterConfig = toml::from_str(
            r#"
[chapter]
title = "Test Chapter"
subtitle = "Sub"
vocab_page = "v"
footer_text = "F"
footer_suffix = "B"

[[sections]]
heading = "Section One"

[[sections.pages]]
slug = "01_page"
title = "First Page"
description = "Desc"
type = "dialog"
"#,
        )
        .unwrap();

        let sections: Vec<IndexSectionData> = config
            .sections
            .iter()
            .map(|s| IndexSectionData {
                heading: s.heading.clone(),
                pages: s
                    .pages
                    .iter()
                    .map(|p| IndexPageData {
                        slug: p.slug.clone(),
                        title: p.title.clone(),
                        description: p.description.clone(),
                        has_audio: false,
                    })
                    .collect(),
            })
            .collect();

        let mut ctx = Context::new();
        ctx.insert("chapter", &config.chapter);
        ctx.insert("sections", &sections);

        let html = tera.render("chapter_index.html", &ctx).unwrap();
        assert!(html.contains("Test Chapter"));
        assert!(html.contains("Section One"));
        assert!(html.contains("01_page.html"));
        assert!(html.contains("First Page"));
        assert!(!html.contains("[audio]"), "no audio flag when has_audio=false");
    }

    #[test]
    fn chapter_index_shows_audio_badge() {
        let mut tera = Tera::default();
        tera.add_raw_template(
            "chapter_index.html",
            r#"{% for section in sections %}{% for page in section.pages %}{{ page.title }}{% if page.has_audio %} [audio]{% endif %}
{% endfor %}{% endfor %}"#,
        )
        .unwrap();

        let chapter = ChapterMeta {
            title: "T".into(),
            subtitle: "S".into(),
            level: "B1".into(),
            vocab_page: "v".into(),
            footer_text: "F".into(),
            footer_suffix: "B".into(),
        };

        let sections = vec![IndexSectionData {
            heading: "H".into(),
            pages: vec![
                IndexPageData {
                    slug: "with".into(),
                    title: "With Audio".into(),
                    description: "d".into(),
                    has_audio: true,
                },
                IndexPageData {
                    slug: "without".into(),
                    title: "Without Audio".into(),
                    description: "d".into(),
                    has_audio: false,
                },
            ],
        }];

        let mut ctx = Context::new();
        ctx.insert("chapter", &chapter);
        ctx.insert("sections", &sections);

        let html = tera.render("chapter_index.html", &ctx).unwrap();
        assert!(html.contains("With Audio [audio]"));
        assert!(html.contains("Without Audio\n"));
    }

    #[test]
    fn speaker_classes_assigned_in_order() {
        let content = "\
- A — un personnage
- B — un autre
- C — une troisième

A : Line 1
B : Line 2
C : Line 3
A : Line 4
";
        let lines = dialog::parse_dialog(content);
        let classes = ["speaker-a", "speaker-b", "speaker-c", "speaker-d"];
        let mut speaker_classes: HashMap<String, String> = HashMap::new();
        let mut idx = 0;
        for line in &lines {
            if !speaker_classes.contains_key(&line.speaker) {
                speaker_classes
                    .insert(line.speaker.clone(), classes[idx % classes.len()].to_string());
                idx += 1;
            }
        }
        assert_eq!(speaker_classes["A"], "speaker-a");
        assert_eq!(speaker_classes["B"], "speaker-b");
        assert_eq!(speaker_classes["C"], "speaker-c");
    }

    #[test]
    fn classify_priority_values() {
        assert_eq!(classify_priority("index.html"), 1.0);
        assert_eq!(classify_priority("chapters/b1-test/index.html"), 0.8);
        assert_eq!(
            classify_priority("chapters/b1-test/translations/01_en.html"),
            0.3
        );
        assert_eq!(classify_priority("chapters/b1-test/01_dialog.html"), 0.5);
    }

    #[test]
    fn generate_sitemap_creates_valid_xml() {
        let tmp = std::env::temp_dir().join("sitemap_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("chapters/ch1/translations")).unwrap();
        std::fs::create_dir_all(tmp.join("chapters/ch1/audio")).unwrap();

        // Create some HTML files.
        std::fs::write(tmp.join("index.html"), "<html></html>").unwrap();
        std::fs::write(tmp.join("404.html"), "<html>404</html>").unwrap();
        std::fs::write(tmp.join("chapters/ch1/index.html"), "<html></html>").unwrap();
        std::fs::write(tmp.join("chapters/ch1/01_page.html"), "<html></html>").unwrap();
        std::fs::write(
            tmp.join("chapters/ch1/translations/01_page_en.html"),
            "<html></html>",
        )
        .unwrap();
        // Audio dir should be skipped even if it somehow has HTML.
        std::fs::write(tmp.join("chapters/ch1/audio/fake.html"), "<html></html>").unwrap();

        generate_sitemap(&tmp, "https://example.com").unwrap();

        let xml = std::fs::read_to_string(tmp.join("sitemap.xml")).unwrap();

        assert!(xml.starts_with("<?xml"));
        assert!(xml.contains("<loc>https://example.com/index.html</loc>"));
        assert!(xml.contains("<priority>1.0</priority>"));
        assert!(xml.contains("chapters/ch1/index.html"));
        assert!(xml.contains("<priority>0.8</priority>"));
        assert!(xml.contains("chapters/ch1/01_page.html"));
        assert!(xml.contains("chapters/ch1/translations/01_page_en.html"));
        assert!(xml.contains("<priority>0.3</priority>"));
        // 404 and audio should be excluded.
        assert!(!xml.contains("404.html"));
        assert!(!xml.contains("audio/fake.html"));

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
