# Directory Structure

Each directory has a target learner level (B1, B2, etc.), for example `B1_texts`.
Additional chapters are `<level>_<subject>_texts`, like `B1_appart_texts`.

## Contents

For each dialog, email, sign, or other example, there should be:
- A markdown file of the item in French (e.g. `01_example.md`)
- A markdown file of the English translation (e.g. `01_example_en.md`)
- A plaintext file of the item in French (e.g. `01_example.txt`)

There should also be a `vocabulaire.md` file that contains unusual or potentially
new vocabulary for the learner at that level.

## HTML

The `html/` subdirectory contains:
- `index.html` — navigation page listing all items in the chapter
- `style.css` — shared stylesheet (may extend the base theme with chapter-specific styles)
- One HTML file per item (e.g. `01_example.html`)
- `translations/` — English HTML versions (e.g. `01_example_en.html`)
- `vocabulaire.html` — vocabulary reference page

## Audio

The `audio/` directory (at the chapter root, not inside `html/`) may contain
subdirectories of generated TTS audio, one per dialog:
- `audio/<dialog_name>/combined.mp3`
- `audio/<dialog_name>/lines/01_speaker.mp3`, etc.

## Existing Chapters

- `B1_texts/` — General B1 dialogs (metro, bakery, taxi, school, etc.)
- `B1_appart_texts/` — B1 apartment life in Lyon (Maeve & Irene in Croix-Rousse)
