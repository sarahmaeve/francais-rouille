---
name: create-chapter
description: Create a new French language learning chapter from an English prompt file. Generates French content, English translations, audio, and HTML in three phases with a review gate.
user_invocable: true
---

# Create Chapter

Generate a complete chapter for the Français Rouillé language learning site
from an English-language prompt file.

## Usage

```
/create-chapter <prompt-file> <chapter-slug>
```

Example:
```
/create-chapter testdata/b2-librarie.md b2-librairie-occitane
```

## Arguments

- `<prompt-file>` — Path to a markdown file describing the chapter content
  (dialogues, fragments, monologues, characters, themes)
- `<chapter-slug>` — Directory name for the chapter (e.g. `b2-librairie-occitane`).
  Must be lowercase with hyphens.

## Workflow

This skill runs in three phases with a review gate between content creation
and asset generation. **Do not proceed to the next phase without explicit
user confirmation.**

### Phase 1: Content Generation

1. **Read the prompt file** and identify each content piece (dialogs,
   fragments, monologues) with its description.

2. **Determine the CEFR level** (B1, B2, etc.) from the prompt file.

3. **Create the chapter directory** at `content/<chapter-slug>/`.

4. **Generate content files** for each piece described in the prompt:

   - **Dialogs and monologues:** Create three files each:
     - `<slug>.txt` — French dialog in the project's dialog format
     - `<slug>.md` files are no longer maintained manually
     - `<slug>_en.txt` — English translation in the same dialog format
   - **Fragments:** Create an `.html` file in the content directory.
     For prose articles, use `<article class="prose-article">` markup.
     For maps, use inline SVG in a `<div class="map-container">`.
   - **Fragment translations:** Create `<slug>_en.html` in the content
     directory with just the content (no `<html>`, `<head>`, or footer).
     The build system wraps it with the `fragment_translation.html`
     template automatically.

5. **Dialog format** (`.txt` files):
   ```
   Title of Dialog

   Personnages :
   - Character Name — une description en français
   - Other Character — une autre description

   Character Name : Spoken text here.

   Other Character : Response text here.
   ```

6. **Generate `chapter.toml`** with proper sections, page types, slugs,
   and descriptions. Use these page types:
   - `"dialog"` for dialogs and monologues
   - `"fragment"` for HTML content (maps, articles)
   - `"static"` for hand-authored pages (vocabulary, quiz)

7. **Generate a vocabulary page** as static HTML at
   `site/chapters/<chapter-slug>/vocabulaire.html`. Group vocabulary by
   theme. Include French-English pairs in tables wrapped with
   `<div class="vocab-table-wrap">`.

8. **Create a chapter stylesheet** at
   `site/chapters/<chapter-slug>/style.css`. Each chapter has its own
   `style.css` for chapter-specific overrides (the shared styles live in
   `shared/chapter.css`). **Do not copy another chapter's stylesheet
   wholesale.** Instead:
   - Check which CSS classes the chapter's fragments actually use.
   - Look in existing chapter stylesheets (`site/chapters/*/style.css`)
     for definitions of those classes and copy only the ones needed.
   - If the chapter has no fragment-specific classes, write a minimal
     placeholder:
     ```css
     /* <chapter-slug> — no chapter-specific overrides */
     /* All styles provided by shared/chapter.css */
     ```

9. **Add the chapter** to `content/site.toml` under the appropriate level
   section. Set `new = true` for the new chapter.

10. **Run typography fix:**
    ```bash
    cargo run -- verify-language <chapter-slug> --fix
    ```

11. **Present a summary** to the user:
    - List of all files created with types and line counts
    - Character names and genders
    - Number of dialog lines per file
    - Any content decisions made (character names, historical details)

12. **Ask the user to review** the content in `content/<chapter-slug>/`
    before proceeding. **Stop here and wait for confirmation.**

### Phase 2: Validation

After the user confirms the content looks good:

1. **Run typography verification** (check mode):
   ```bash
   cargo run -- verify-language <chapter-slug>
   ```

2. **Review the French content** for:
   - Grammar errors (agreement, conjugation, prepositions)
   - No passé simple (only passé composé, imparfait, plus-que-parfait,
     conditionnel, present, futur simple, futur proche)
   - Spelling errors
   - Historical or factual accuracy
   - Natural spoken register

3. **Review the English translations** for:
   - Accuracy against the French originals
   - Idiomatic English (not literal translation)
   - Consistent terminology across files
   - No missing or added content

4. **Report any issues found** with specific file names, line numbers,
   and suggested fixes. Apply fixes if the user approves.

### Phase 3: Asset Generation

After the user approves the validated content:

1. **Check for the TTS API key.** Audio generation requires the
   `GOOGLE_TTS_API_KEY` environment variable. Before running any TTS
   commands, ask the user to provide the key if it is not already set.
   Pass it inline with the command (`GOOGLE_TTS_API_KEY='...' cargo run
   -- dialog ...`) rather than persisting it to any file or settings.

2. **Generate audio** for each dialog and monologue:
   ```bash
   cargo run -- dialog content/<chapter-slug>/<slug>.txt \
       site/chapters/<chapter-slug>/audio/<slug>
   ```

3. **Build the chapter HTML:**
   ```bash
   cargo run -- build <chapter-slug>
   ```

4. **Rebuild the site index** (full build):
   ```bash
   cargo run -- build
   ```

5. **Run CSP validation:**
   ```bash
   cargo run -- check-csp
   ```

6. **Report completion** with:
   - Total audio files generated
   - Total HTML pages generated
   - CSP check result
   - Any warnings

## Content Guidelines

### French Language Rules

- All `.txt` and `.md` content represents **spoken French** — dialogues
  or monologues, never written-only registers.
- **No passé simple.** Use passé composé for completed past actions.
- Use imparfait for background/description, plus-que-parfait for events
  prior to another past event, conditionnel for hypotheticals.
- Target the CEFR level specified in the prompt. B2 content should use
  more complex subordinate clauses, relative pronouns (dont, où), and
  plus-que-parfait compared to B1.

### Typography

- The `verify-language --fix` command will convert ASCII apostrophes to
  typographic apostrophes (U+2019) in elision contexts and fix ellipsis
  characters. Always run it after creating content.

### TTS Considerations

- Roman numerals in royal names and century notation are automatically
  converted to spoken French by the TTS system (e.g. "Guillaume IX" →
  "Guillaume neuf", "XIIIe siècle" → "treizième siècle").
- Only Studio voices (fr-FR-Studio-A female, fr-FR-Studio-D male) are
  used as preferred voices. Additional speakers of the same gender draw
  from fallback voices.

### File Naming

- Use numbered slugs: `00_`, `01_`, `02_`, etc.
- Use lowercase with underscores for slugs.
- Fragment slugs should be descriptive: `00_carte_occitanie`, `01_histoire_occitan`.
- Dialog slugs should reflect the scene: `02_decouverte_librairie`.
- Vocabulary page is always `vocabulaire`.

### Resources

To add external links (YouTube videos, books, websites) to a page, add
a `resources` array to its entry in `chapter.toml`:

```toml
resources = [
    { type = "video", title = "Video Title", url = "https://...", note = "description" },
    { type = "book", title = "Author, Book Title", note = "Publisher, Year" },
    { type = "link", title = "Site Name", url = "https://...", note = "description" },
]
```

### Feature Flags

To make a page experimental, add `flag = "flag-name"` to its entry in
`chapter.toml`. The page will be hidden from the chapter index unless
the user has the flag enabled. A butter bar warning will appear on
flagged pages.

Register new flags in `site/shared/flags-ui.js` in the `KNOWN_FLAGS` array.
