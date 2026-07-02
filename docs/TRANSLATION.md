# Translation Practice (FR → EN, self-checked)

The learner translates one French line at a time; on reveal, the engine shows
the maintained reference English translation beside the attempt, plus a
deterministic *missing-keyword checklist*, and the learner **self-grades**.

There is deliberately **no auto-grading of free English prose** — that would
require an LLM judge, and the whole point of this project is to stay
deterministic. The only machine help is a checklist prompt, never a score.

## Where the data lives

Translation data is **generated** from the parallel content files and fetched
at runtime (`connect-src 'self'` — see [CSP.md](CSP.md)). One file per chapter:

```
site/chapters/<chapter>/translation-data.json
```

**Key asset:** every dialog ships both `content/<ch>/<slug>.txt` (French) and
`<slug>_en.txt` (English), mirroring each other line-for-line in the same
`Speaker : text` format. The generator parses both and pairs them by index.

## Opting a chapter in

Add `translation = true` to the chapter's `[chapter]` table in `chapter.toml`.
On the next build, the generator
([`crates/site-gen/src/translation.rs`](../crates/site-gen/src/translation.rs))
aligns each dialog with its `_en.txt`, zips the two by index, attaches each
line's audio filename when the MP3 exists, and writes `translation-data.json`.

## Alignment: the content-hardening check

Before pairing, the generator runs `check_alignment` on every dialog and its
translation, and **fails the build** (for opted-in chapters) on drift:

- **Line-count mismatch** — the two files must have the same number of spoken
  lines.
- **Turn-taking mismatch** — the *shape* of the speaker alternation (each
  speaker mapped to the order it first appears, e.g. `[0,1,0]`) must match
  position for position.

Both signals are intentionally robust to the fact that translations legitimately
**rename role-based speakers** (*Le guichetier* → *The clerk*, *Vendeuse* →
*Saleswoman*) while keeping proper names (*Léa*): the check never compares
speaker *names* across languages. A pure adjacent swap in a strictly
alternating two-speaker dialog is not structurally detectable once names are
translated; the line-count check remains the strong guard against the common
drift.

The same check also runs as a **warning** in the normal build for *every*
dialog that has an `_en.txt` (whether or not the chapter opts in to translation
data), so it hardens all parallel content in the repo — nothing else catches a
translation that has silently fallen out of sync with its source.

## Validation

```
cargo run -- verify-quiz                 # scan site/ for all exercise data
cargo run -- verify-quiz path/to.json    # validate a single file
```

For `translation-data.json` the validator checks:

- the `schema` marker is `translation/v1`;
- exercise ids are unique;
- line numbers are dense and 1-based;
- every `fr` and `en` is non-empty;
- French typography on the **`fr` side only** — `en` is English and is
  explicitly exempt from French typographic rules. `--fix` rewrites only the
  `fr` fields, never touching English text or JSON structure.

## File schema

```jsonc
{
  "schema": "translation/v1",
  "exercises": [
    {
      "id": "01_paris_metro",
      "title": "Naviguer dans le métro",
      "audio_base": "audio/01_paris_metro/lines/",   // optional; when any line has audio
      "lines": [
        {
          "n": 1,
          "speaker": "Léa",
          "fr": "Excusez-moi, monsieur…",
          "en": "Excuse me, sir…",
          "audio": "01_lea.mp3"                       // optional; relative to audio_base
        }
      ]
    }
  ]
}
```

## Deterministic assist (never a score)

Pure functions in [`site/shared/quiz.js`](../site/shared/quiz.js), covered by
`runTests()`:

- `keywordSet(reference)` — the reference's *content words*: case-folded,
  length ≥ 4, minus a small English stoplist (~30 function words).
- `missingKeywords(reference, attempt)` — content words present in the
  reference but absent from the attempt.

These drive a checklist — *« À vérifier : avez-vous rendu … ? »* — shown after
the learner reveals the reference. It is a prompt, not a grade: cheap,
deterministic, and honest about what it is.

## Self-grading and the practice loop

After **Comparer** reveals the reference, the learner marks each line *Juste* /
*Presque* / *À revoir*. Grades (and, optionally, the last attempt) are stored
per line in `localStorage` under `tr:<chapter>:<exercise>:<n>`, following the
flashcard state conventions, and all access is guarded for private-browsing.

The summary screen counts each grade and offers a **Revoir** mode that replays
only the lines marked *Presque* / *À revoir* — turning a one-shot exercise into
a practice loop.

## Explicit non-goal: typed EN → FR grading

Full-sentence EN → FR grading is out of scope. Sentence-level exact matching is
too strict to be fair, and anything looser is a judge. EN → FR practice stays at
the *word / phrase* level, where accent-aware exact matching works — i.e. the
existing fill-in-the-blank and flashcard machinery, not this engine.

## Authoring / rollout checklist

1. Confirm the chapter's dialogs each have a line-aligned `_en.txt`
   (the build warns on drift; fix any warnings first).
2. Add `translation = true` to `chapter.toml` and rebuild.
3. Add a tab button to `quiz.html`:
   `<button class="quiz-tab" data-type="translation">Traduction</button>`.
4. Run `cargo run -- verify-quiz` and fix every reported problem.
5. Ship the tab. (Optionally stage behind a feature flag first — add
   `class="flag-hidden" data-flag="translation"` to the button and load
   `shared/flags.js` — see [FLAGS.md](FLAGS.md).)
