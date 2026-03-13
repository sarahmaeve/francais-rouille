# Directory Structure

The project separates deployable web content (`site/`) from source content (`content/`)
and build tooling (`src/`).

## `content/` — Source Material (not deployed)

Each chapter directory (e.g. `content/b1-vie-quotidienne/`) contains:
- A markdown file of the item in French (e.g. `01_example.md`)
- A markdown file of the English translation (e.g. `01_example_en.md`)
- A plaintext file of the item in French (e.g. `01_example.txt`) — used as TTS input
- A `vocabulaire.md` file with vocabulary for that level

## `site/` — Deployable Web Content

Everything under `site/` is what gets deployed. Point your hosting provider at this directory.

### `site/shared/`
Shared JavaScript and CSS used across chapters (quiz engine, crossword engine).

### `site/chapters/<chapter-name>/`
Each chapter directory contains:
- `index.html` — navigation page listing all items in the chapter
- `style.css` — chapter stylesheet
- One HTML file per item (e.g. `01_example.html`)
- `translations/` — English HTML versions (e.g. `01_example_en.html`)
- `vocabulaire.html` — vocabulary reference page
- `quiz.html` + `quiz-data.js` — interactive quiz
- `mots-croises.html` — crossword puzzle (where applicable)
- `audio/` — generated TTS audio
  - `audio/<dialog_name>/combined.mp3`
  - `audio/<dialog_name>/lines/01_speaker.mp3`, etc.

## `src/` — Rust CLI (build tool, not deployed)

The Rust CLI parses dialog files from `content/` and generates TTS audio into `site/`.

## Existing Chapters

- `b1-vie-quotidienne` — General B1 dialogs (metro, bakery, taxi, school, etc.)
- `b1-appartement` — B1 apartment life in Lyon (Maeve & Irene in Croix-Rousse)
