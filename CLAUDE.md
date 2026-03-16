# General

Provide expert advice in Rust.
Always include tests where possible, especially when a changelist is greater than 25 lines.
Simplify code for correctness and focus on idiomatic Rust.

## Project Overview

We are creating a mechanism to teach and test French language comprehension and French to English language translation for learners from the B1 - C2 levels.

## Content Security Policy

The site is served with a strict CSP (`site/_headers`). All generated HTML
**must** comply with these rules:

- **No inline `<style>` blocks.** All CSS must go in the chapter's external
  `style.css`. The fragment template does not support companion `.css` files;
  add any new classes directly to the chapter stylesheet.
- **No inline `style="..."` attributes** on HTML elements. Use CSS classes
  instead. (SVG presentation attributes like `font-style`, `fill`, `stroke`
  are fine — they are not CSS inline styles.)
- **No inline scripts or event handlers** (`onclick`, `onload`, etc.).
  All JavaScript must be in external `.js` files loaded via `src=` from
  the `shared/` directory.
- **No external resources.** All scripts, styles, fonts, images, and media
  must be served from the same origin (`'self'`).

See `docs/CSP.md` for the full policy and rationale.

## French Content Guidelines

All `.txt` and `.md` content files represent **spoken French** — dialogues
between characters or monologues by a narrator/guide addressing an audience.
Written-only registers must not appear in these files.

### Tenses

Use only tenses that occur in modern spoken French:

- **Passé composé** for completed past actions (*il a lancé*, *elle est venue*).
- **Imparfait** for background, description, and habitual past (*il régnait*,
  *elle vivait*).
- **Plus-que-parfait** for events prior to another past event (*il avait
  compris*).
- **Conditionnel** for hypotheticals and politeness (*je voudrais*, *rien ne
  serait pareil*).
- **Present**, **futur simple**, **futur proche** as normal.

**Do not use the passé simple** (*lança*, *descendirent*, *fut*). It belongs
to literary narrative, not spoken French — even in formal or academic speech.

### Typography

Run `verify-language` before committing content changes:

```
cargo run -- verify-language          # check
cargo run -- verify-language --fix    # auto-correct
```

Key rules enforced (fr-FR):
- Use typographic apostrophes `'` (U+2019), not ASCII `'` (U+0027), in
  French elision (*l'homme*, *d'accord*, *aujourd'hui*).
- Use the ellipsis character `…` (U+2026), not three dots `...`.
- With `--strict`: narrow no-break space (U+202F) before `;` `:` `!` `?`.
