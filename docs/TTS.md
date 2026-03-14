# Français Rouille — Text-to-Speech Guide

## Prerequisites

You need a Google Cloud project with the **Cloud Text-to-Speech API** enabled
and an API key.

1. Go to the [Google Cloud Console](https://console.cloud.google.com/).
2. Create or select a project.
3. Enable the **Cloud Text-to-Speech API** under APIs & Services.
4. Create an API key under APIs & Services → Credentials.

## Setting the API Key

Export your API key as an environment variable before running the program:

```bash
export GOOGLE_TTS_API_KEY="your-api-key-here"
```

To persist this across sessions, add the export line to your shell profile
(`~/.zshrc`, `~/.bashrc`, etc.).

## Building

```bash
cargo build --release
```

The binary is at `target/release/francais-rouille`.

## Usage

### Synthesize a plain text file

Converts an entire text file to a single MP3 using a default French female
voice:

```bash
francais-rouille file <input.txt> <output.mp3>
```

Example:

```bash
francais-rouille file content/b1-vie-quotidienne/09_gallimard_evenement.txt output/gallimard.mp3
```

### Synthesize a dialog file

Parses a dialog text file, assigns a distinct voice to each character (based
on gender detected from the character descriptions), and produces one MP3
per dialog line in `<output_dir>/lines/`.

```bash
francais-rouille dialog <input.txt> <output_dir>
```

Pass `--combined` to also generate a single concatenated audio file with
silence between lines:

```bash
francais-rouille dialog <input.txt> <output_dir> --combined
```

Example:

```bash
francais-rouille dialog content/b1-vie-quotidienne/07_boulangerie.txt output/07_boulangerie/
```

Output:

```
output/07_boulangerie/
└── lines/
    ├── 01_claire.mp3
    ├── 02_monsieur_duval.mp3
    ├── 03_claire.mp3
    └── ...
```

## How voice assignment works

The program reads the character description lines in each text file to
detect gender:

```
- Claire — une cliente curieuse qui entre dans la boulangerie
- Monsieur Duval — le propriétaire et pâtissier de la boulangerie
```

French gendered articles (`une`/`la` = female, `un`/`le` = male) after the
em-dash determine which voice pool to draw from. Voices are randomly
selected from the full set of Premium fr-FR Google Cloud voices, so each
run produces a different combination. Within a single run, each character
keeps the same voice throughout the dialog. When two characters share the
same gender, they are assigned distinct voices.

## Dialog text file format

Dialog files should follow this structure:

```
Title of the Dialog

Personnages :
- Speaker Name — une/un description of the character
- Speaker Name — le/la description of the character

Speaker Name : First line of dialog.

Speaker Name : Second line of dialog.
```

The ` : ` (space-colon-space) delimiter separates the speaker name from the
spoken text. Character descriptions on lines starting with `-` provide
gender information for voice assignment.

## Apostrophe handling

Dialog `.txt` files **must** use the typographic apostrophe `'` (U+2019)
rather than the ASCII straight apostrophe `'` (U+0027). Google Cloud TTS
treats U+0027 as a word boundary, which produces an audible pause in French
elisions like `l'église`, `d'abord`, `qu'il`, etc. Using U+2019 ensures
these are pronounced as a single word.

Before generating audio, verify your file contains no ASCII apostrophes:

```bash
grep -Pn "'" content/your-chapter/your_dialog.txt
```

To convert any remaining ASCII apostrophes to typographic ones:

```bash
python3 -c "
import sys; f = sys.argv[1]; open(f, 'w').write(open(f).read().replace(chr(0x27), chr(0x2019)))
" content/your-chapter/your_dialog.txt
```
