# Reading Comprehension Exercises

B2-level reading-comprehension items use **auto-scorable, deterministic**
formats — every answer is graded by exact match (or a fixed formula) against a
key, with no LLM judge in the loop. This document specifies the data schema,
the item types, and the scoring rules.

The canonical example is
[`examples/reading/neo-ruraux.json`](../examples/reading/neo-ruraux.json).

## Where the data lives

Reading data is authored as JSON and **fetched at runtime** by the quiz engine
(`connect-src 'self'` in the CSP permits this — see [CSP.md](CSP.md)). One file
per chapter:

```
site/chapters/<chapter>/reading-data.json
```

Because it is a single source of truth — passage text, questions, *and* answer
keys all in one file — the same file the browser loads is the file we validate.

## Validation

Run after authoring or editing reading data:

```
cargo run -- verify-quiz                 # scan site/ for reading-data.json
cargo run -- verify-quiz --site DIR      # scan a different directory
cargo run -- verify-quiz path/to.json    # validate a single file
```

The validator parses the JSON and checks that every answer key references real
options, statements, and source sentences — so a malformed key (an answer
pointing at a deleted option, a highlight index past the end of the passage)
cannot silently ship. It exits non-zero on any problem. Implementation:
[`crates/site-gen/src/reading.rs`](../crates/site-gen/src/reading.rs).

## File schema

```jsonc
{
  "schema": "reading/v1",
  "passages": [
    {
      "id": "neo_ruraux_b2_01",   // unique within the file
      "level": "B2",               // optional
      "title": "Les néo-ruraux",  // optional
      "sources": [ /* one Source, or several for paired sources */ ],
      "items":   [ /* the questions */ ]
    }
  ]
}
```

### Source

```jsonc
{
  "id": "A",                         // unique within the passage
  "title": "Quitter la ville",      // optional
  "attribution": "…",               // optional
  "sentences": ["…", "…", "…"]     // ordered; the unit of evidence selection
}
```

Splitting a source into a **`sentences` array** (rather than one blob of HTML)
is what makes evidence-selection deterministic: a `highlight_span` answer
refers to sentences by their **zero-based index** in this array. Render the
sentences joined for reading; render each as an individually selectable span
for highlight items.

### Item — shared fields

| Field        | Required | Notes                                                        |
|--------------|----------|--------------------------------------------------------------|
| `type`       | yes      | Selects the item kind (below).                               |
| `id`         | yes      | Unique within the passage.                                   |
| `points`     | yes      | Maximum points. Must be > 0. Fractional weights allowed.     |
| `prompt`     | no       | The question text shown to the learner.                      |
| `skill`      | no       | Tag (`main_idea`, `inference`, `vocab_in_context`, …) for assembling balanced forms. |
| `difficulty` | no       | 1–5, for assembling balanced forms.                          |
| `source`     | no       | Which source the item refers to. Required only when a passage has more than one source. |

## Item types

What makes an item "AP level" is the **cognitive demand** (inference, author's
purpose, vocab-in-context, implication) — not the format. A 4-option MC asking
for an inference is harder than a 6-option MC asking a literal-retrieval
question. We deliberately do **not** inflate option counts; instead we lower
guessability by mixing in multi-select and True/False/Not-Given.

### `mc_single` — single-answer multiple choice

```jsonc
{ "type": "mc_single", "id": "q1", "points": 1,
  "options": [ {"id":"A","text":"…"}, {"id":"B","text":"…"} ],
  "answer": "B" }
```

### `multi_select` — choose all that apply

```jsonc
{ "type": "multi_select", "id": "q5", "points": 3,
  "options": [ {"id":"A","text":"…"}, … ],
  "answer": ["A", "B", "D"],
  "scoring": "partial" }          // "partial" (default) | "all_or_nothing"
```

The cleanest lever for crushing guessability. **Default scoring is partial
credit** (formula below), which discourages "select everything" gaming without
over-penalizing a single slip the way all-or-nothing does.

### `true_false_notgiven` — Vrai / Faux / Non précisé

```jsonc
{ "type": "true_false_notgiven", "id": "q4", "points": 4,
  "statements": [ {"id":"a","text":"…"}, … ],
  "answer": { "a": "F", "b": "V", "c": "NP", "d": "F" } }   // V | F | NP
```

The third option, **Non précisé**, forces the reader to separate what the text
*says* from what is plausible but *unstated* — exactly the inference skill the
items should test. Scored per sub-statement.

### `highlight_span` — evidence selection

```jsonc
{ "type": "highlight_span", "id": "q6", "points": 1, "source": "A",
  "min_overlap": 0.6,
  "accepted": [ [7], [7, 8] ] }   // sets of sentence indices into the source
```

The learner clicks the sentence(s) supporting a claim. Selection is
**sentence-granular** (not arbitrary character spans): simpler, CSP-clean, and
a better match for "click the sentence that proves it." Multiple accepted
answers allow both a tight single-sentence answer and a looser surrounding
span.

### `matching`

```jsonc
{ "type": "matching", "id": "q7", "points": 4,
  "left":  [ {"id":"1","text":"…"}, … ],
  "right": [ {"id":"a","text":"…"}, … ],   // may include extra distractors
  "answer": { "1": "a", "2": "c", … } }
```

### `ordering`

```jsonc
{ "type": "ordering", "id": "q8", "points": 1,
  "elements": [ {"id":"1","text":"…"}, {"id":"2","text":"…"}, … ],
  "answer": ["3", "1", "2"] }     // must be a permutation of the element ids
```

### `banked_cloze`

```jsonc
{ "type": "banked_cloze", "id": "q9", "points": 2,
  "text": "Le ___ mange la ___ rouge.",   // gaps are ___ (one per blank)
  "bank": [ {"id":"w1","text":"chat"}, … ], // may include extra distractors
  "answer": ["w1", "w3"] }                  // one bank id per gap, in order
```

## Scoring rules

These formulas are the contract the JS quiz engine implements. They are all
deterministic.

| Type                  | Score                                                                 |
|-----------------------|-----------------------------------------------------------------------|
| `mc_single`           | `points` if the selected option equals `answer`, else 0.              |
| `multi_select` (partial) | `points × max(0, (n_correct_selected − n_incorrect_selected) / n_correct)`. |
| `multi_select` (all_or_nothing) | `points` only if the selected set equals `answer` exactly. |
| `true_false_notgiven` | `points × (sub-statements correct / total sub-statements)`.           |
| `highlight_span`      | `points` if Jaccard overlap of the selected sentence set with **any** accepted set ≥ `min_overlap` (default 0.6), else 0. |
| `matching`            | `points × (correct pairs / total left entries)`.                      |
| `ordering`            | `points × (correctly-ordered adjacent pairs / total adjacent pairs)`. |
| `banked_cloze`        | `points × (gaps filled correctly / total gaps)`.                      |

Pick one multi-select policy and keep it **consistent across the whole item
bank**.

## Typed French answers and accents

None of the formats above require the learner to *type* French — they are all
selection-based, which is why they auto-score cleanly. If a future type does
take typed input, **do not silently fold accents**: in French, accents are
semantically load-bearing (*ou*/*où*, *a*/*à*, *sur*/*sûr*, *du*/*dû*), so
folding them marks real errors correct. The existing quiz engine already models
the right behaviour in `site/shared/quiz.js` — exact match, an explicit
"missing accent" near-miss state, and a normalized fallback — rather than
blind accent-folding.

## Paired sources

For AP-style synthesis, give a passage **two (or more) sources** on one theme
and write items whose `source` is set, plus cross-source items that reason
across both. The schema supports this from the start: `sources` is an array,
and single-source items name their `source`. A `highlight_span` item that omits
`source` when a passage has several is a validation error (the sentence-index
reference would be ambiguous); MC-style cross-source items intentionally omit
`source` because they span both texts.

Worked example:
[`examples/reading/semaine-quatre-jours.json`](../examples/reading/semaine-quatre-jours.json)
— two opposing texts on the four-day week. The pair turns on a real French
nuance the items exploit: *semaine **de** quatre jours* (reduced hours, the
"100-80-100" model) versus *semaine **en** quatre jours* (the same hours
compressed into longer days), so the two authors use the same phrase for
different arrangements.

## Authoring checklist

1. Write original passages (clean IP) in spoken-register French per
   [CLAUDE.md](../CLAUDE.md) — passé composé / imparfait, no passé simple.
2. Aim items at inference, author's purpose, and vocab-in-context, not literal
   retrieval. Tag each with `skill` and `difficulty`.
3. Run `cargo run -- verify-quiz` and fix every reported problem.
4. Run `cargo run -- verify-language` over any French prose you add.
