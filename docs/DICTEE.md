# Dictée Mode

A **dictée** plays one spoken line of a dialog, the learner types what they
hear, and the engine shows a deterministic word-level diff against the known
transcript. There is no LLM in the loop: the transcript is the answer key and
grading is a fixed alignment formula.

## Where the data lives

Dictée data is **generated** (not authored) and fetched at runtime by the quiz
engine (`connect-src 'self'` permits this — see [CSP.md](CSP.md)). One file per
chapter:

```
site/chapters/<chapter>/dictee-data.json
```

It is regenerated on every site build for chapters that opt in.

## Opting a chapter in

Add `dictee = true` to the chapter's `[chapter]` table in `chapter.toml`:

```toml
[chapter]
title = "…"
# …
dictee = true
```

On the next `cargo run -- build`, the generator
([`crates/site-gen/src/dictee.rs`](../crates/site-gen/src/dictee.rs)):

1. parses every `type = "dialog"` page's `content/<ch>/<slug>.txt`,
2. reconstructs each line's audio filename with the shared
   [`dialog::line_audio_filename`](../crates/site-gen/src/dialog.rs) convention
   (`NN_speaker.mp3`), so the path can never drift from what the TTS writer
   produced,
3. **includes a line only if its MP3 already exists** under the chapter's
   output directory (a missing MP3 is skipped with a warning, never linked as a
   404), and
4. writes `dictee-data.json` — or nothing, if no dialog produced any lines.

Because the transcript ships in the JSON, dictée is a self-study aid, not a
proctored test — the same trade-off already accepted for reading data.

## Validation

```
cargo run -- verify-quiz                 # scan site/ for all exercise data
cargo run -- verify-quiz path/to.json    # validate a single file
```

For `dictee-data.json` the validator checks:

- the `schema` marker is `dictee/v1`;
- exercise ids are unique;
- line numbers are **dense and 1-based** (`1, 2, 3, …`);
- every `text` is non-empty;
- every `audio` path resolves to a file that exists on disk (relative to the
  JSON file's own directory);
- French typography (typographic apostrophes `’`, ellipsis `…`) on every
  `text`, with `--fix` support.

## File schema

```jsonc
{
  "schema": "dictee/v1",
  "exercises": [
    {
      "id": "01_paris_metro",              // unique; the dialog page slug
      "title": "Naviguer dans le métro",   // optional
      "lines": [
        {
          "n": 1,                          // dense, 1-based, ordered
          "speaker": "Léa",
          "audio": "audio/01_paris_metro/lines/01_lea.mp3",  // chapter-relative
          "text": "Excusez-moi, monsieur, vous pouvez m’aider ?"
        }
      ]
    }
  ]
}
```

## Grading contract

The grading functions are pure and live in
[`site/shared/quiz.js`](../site/shared/quiz.js) (`dicteeTokenize`,
`dicteeAlign`, `scoreDictee`); they are covered by `runTests()` in that file.

**Tokenize.** Split on whitespace; strip leading/trailing punctuation but keep
word-internal hyphens and apostrophes (`Excusez-moi`, `m’aider`); fold the
typographic apostrophe `’` → `'` and the ellipsis `…` → `...` before comparing.
Learners type ASCII from a real keyboard, so we normalize the *punctuation*
they can't easily type — the deliberate inverse of the authoring rule, which
requires typographic punctuation in source content.

**Align.** Word-level global alignment (Wagner–Fischer, O(n·m); lines are short
so cost is trivial) classifies each reference token:

| State     | Meaning                                                        |
|-----------|----------------------------------------------------------------|
| `correct` | exact match after punctuation/case folding                     |
| `accent`  | matches only once accents are folded (`a`/`à`, `parle`/`parlé`) — **half credit**, shown distinctly |
| `wrong`   | aligned to an attempt token, but a different word              |
| `missing` | no aligned attempt token (a word the learner left out)         |

Attempt tokens with no reference partner are shown as `extra`.

**Score.**

```
points × (correct + 0.5 · accent) / refTokens        (floored at 0)
```

Extra tokens do **not** subtract — a missed word already costs through
`missing`, and double-penalizing insertions is unfair to a learner who typed a
little too much. Accents are load-bearing in French, so an accent slip is a
*visible half-credit near-miss*, never silently accepted.

## Authoring / rollout checklist

1. Ensure the chapter's dialogs already have generated per-line audio.
2. Add `dictee = true` to `chapter.toml` and rebuild.
3. Add a Dictée tab button to the chapter's hand-authored `quiz.html`:
   `<button class="quiz-tab" data-type="dictee">Dictée</button>`.
4. Run `cargo run -- verify-quiz` and fix every reported problem.
5. Ship the tab. (Optionally stage behind a feature flag first — add
   `class="flag-hidden" data-flag="dictee"` to the button and load
   `shared/flags.js` — see [FLAGS.md](FLAGS.md).)
