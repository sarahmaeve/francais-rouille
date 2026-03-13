# Français Rouillé

*Rusty French* — a French language learning tool with authentic texts, dialogues, and
text-to-speech audio for B1+ learners.

Browse the content by opening `site/index.html` in a browser.

## What's inside

**HTML chapters** — browsable dialogues, emails, notices, and vocabulary tables, each
with English translations and inline audio playback (combined + per-line).

**Rust CLI** — parses dialog scripts, assigns distinct Google Cloud TTS voices by
character gender, and produces per-line MP3s plus a combined MP3 with pauses between
lines.

## Getting started

### Browse the content

```
open site/index.html
```

### Generate audio with the CLI

Requires a [Google Cloud API key](https://cloud.google.com/text-to-speech/docs/before-you-begin)
with the Cloud Text-to-Speech API enabled.

```
cargo build
export GOOGLE_TTS_API_KEY="your-key"

# Synthesize a dialog — produces lines/*.mp3 and combined.mp3
cargo run -- dialog content/b1-vie-quotidienne/02_viennoiserie.txt site/chapters/b1-vie-quotidienne/audio/02_viennoiserie

# Synthesize a plain text file — produces a single MP3
cargo run -- file content/b1-vie-quotidienne/09_gallimard_evenement.txt output/09_gallimard.mp3
```

Run `cargo run -- --help` for full usage details.

## Dialog file format

```
Title of the Dialog

Personnages :
- Claire — une cliente curieuse
- M. Duval — le propriétaire de la boulangerie

Claire : Bonjour, monsieur ! Votre vitrine est magnifique.
M. Duval : Bonjour, madame ! Qu'est-ce qui vous tente ?
```

Voice gender is inferred from the French article after the em-dash in the
`Personnages` block (`une` / `la` → female, `un` / `le` → male). Each character
keeps the same randomly-selected Premium fr-FR voice throughout the dialog.

## Project structure

```
.
├── site/                           # Deployable web content
│   ├── index.html                  #   Root page linking all chapters
│   ├── shared/                     #   Shared JS/CSS (quiz, crossword engines)
│   └── chapters/
│       ├── b1-vie-quotidienne/     #   Chapter: La Vie Quotidienne
│       │   ├── *.html, style.css   #     French HTML pages
│       │   ├── translations/       #     English HTML translations
│       │   └── audio/              #     Generated MP3s (per-dialog subdirs)
│       └── b1-appartement/         #   Chapter: La Vie en Appartement (Lyon)
│           └── (same structure)
├── content/                        # Source content (not deployed)
│   ├── b1-vie-quotidienne/         #   *.txt, *.md, *_en.md, vocabulaire.md
│   └── b1-appartement/
├── src/                            # Rust CLI source (build tool)
│   ├── main.rs                     #   Entry point, file/dialog modes
│   ├── dialog.rs                   #   Dialog parser and voice assignment
│   └── tts.rs                      #   Google Cloud TTS client
```

## License

Content and code are provided for personal and educational use.
