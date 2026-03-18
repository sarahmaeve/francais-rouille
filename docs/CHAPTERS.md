# Creating New Chapters

## Quick start

```bash
/create-chapter testdata/my-chapter-prompt.md b2-my-chapter-slug
```

## The prompt file

Write a markdown file describing the chapter you want. Include:

- The target CEFR level (B1, B2, etc.)
- A description of each content piece (dialog, fragment, monologue)
- Character names and descriptions (or let them be generated)
- Themes and cultural context
- Any specific grammar or vocabulary you want covered

### Example prompt

```markdown
# chapter: at a specialist bookstore

This chapter should be for a learner of French at or near the B2 level.

Three items should be created per translation:
- a French language version in a .txt file
- the same French language version as a formatted Markdown (.md) file
- an English language translation as a markdown (.md) file

## fragment one
A generated, SVG map of historical Occitania.

## dialogue one
Irene, an American tourist, speaks with the owner of a bookstore.

## monologue one
A schoolteacher explains Occitan history to students.

# validation
After you create these documents, examine the French for correctness.
```

## What gets created

For a chapter with slug `b2-my-chapter`:

```
content/b2-my-chapter/
├── chapter.toml              # Chapter config
├── 00_map.html               # Fragment (SVG map)
├── 01_article.html           # Fragment (prose, French)
├── 01_article_en.html        # Fragment (prose, English)
├── 02_dialog.txt             # French dialog
├── 02_dialog_en.txt          # English translation
├── 03_dialog.txt/_en.txt     # Another dialog set
└── ...

site/chapters/b2-my-chapter/
├── style.css                 # Chapter-specific CSS overrides
├── vocabulaire.html          # Vocabulary page
└── (translations/ created by build)
```

Note: Common CSS is in `site/shared/chapter.css`. Chapter `style.css`
files contain only chapter-specific overrides.

After audio generation and build:

```
site/chapters/b2-my-chapter/
├── index.html                # Chapter landing page
├── 00_map.html               # Built fragment
├── 01_article.html           # Built fragment
├── 02_dialog.html            # Built dialog with audio
├── translations/
│   ├── 01_article_en.html    # Fragment translation
│   ├── 02_dialog_en.html     # Generated dialog translation
│   └── ...
└── audio/
    └── 02_dialog/lines/      # Per-line MP3 files
        ├── 01_speaker.mp3
        └── ...
```

## The three phases

### Phase 1: Content generation

The skill generates all content files, runs typography fixes, adds the
chapter to `site.toml`, and presents a summary. **It stops here for
your review.**

Review the content in `content/<slug>/` — check that dialogs sound
natural, characters are well-defined, and the cultural/historical
content is accurate.

### Phase 2: Validation

After you confirm, the skill:
- Runs `verify-language` to check typography
- Reviews French for grammar, tense, and register
- Reviews English translations for accuracy and idiom
- Reports any issues for your approval

### Phase 3: Asset generation

After you approve:
- Generates TTS audio for all dialogs and monologues
- Builds chapter HTML
- Rebuilds the site index
- Runs CSP validation

## Adding resources to pages

Add external links (videos, books, websites) to any page via
`chapter.toml`:

```toml
resources = [
    { type = "video", title = "Video Title", url = "https://...", note = "duration" },
    { type = "book", title = "Author, Title", note = "Publisher, Year" },
]
```

These appear as a "Pour aller plus loin" section on the French page
and "Further resources" on the English translation.

## Site index

New chapters are automatically added to the site index via
`content/site.toml`. Set `new = true` to show a "Nouveau" badge:

```toml
[[levels.chapters]]
slug = "b2-my-chapter"
title = "Chapter Title"
description = "Chapter description."
meta = "6 textes · Audio disponible"
new = true
```

The site index is regenerated on full builds (`cargo run -- build`).
Single-chapter builds do not modify it.
