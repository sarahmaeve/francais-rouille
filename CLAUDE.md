# General

Provide expert advice in Rust.
Always include tests where possible, especially when a changelist is greater than 25 lines.
Simplify code for correctness and focus on idiomatic Rust.

After any major code change, run clippy and fix all warnings before considering
the work complete:

```
cargo clippy --workspace --all-targets
```

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

Run `check-csp` after generating HTML to catch violations:

```
cargo run -- check-csp              # check site/
cargo run -- check-csp --site DIR   # check a different directory
```

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

### Image Metadata

Run `strip-metadata` before adding images to the site:

```
cargo run -- strip-metadata <path>              # strip in place
cargo run -- strip-metadata <path> --output DIR # write to DIR
cargo run -- strip-metadata <path> --keep-icc   # preserve ICC profiles
```

This removes EXIF, XMP, IPTC, and comment metadata (GPS coordinates,
device info, timestamps, thumbnails) from JPEG and PNG files. `<path>`
can be a single file or a directory (recursive).

### Feature Flags

Content can be staged behind client-side feature flags. Flagged pages are
deployed but hidden from users who haven't enabled the flag.

To flag a page, add `flag = "flag-name"` to its entry in `chapter.toml`:

```toml
[[sections.pages]]
slug = "11_restaurant_commande"
title = "Commander au Restaurant"
description = "..."
type = "dialog"
flag = "new-dining"
```

The page will render with `class="flag-hidden"` and `data-flag="flag-name"`
in the chapter index. `shared/flags.js` (loaded on every page) checks
`localStorage` and the `?flags=` URL parameter to unhide matching elements.

- **Enable for a reviewer:** share a link with `?flags=new-dining`
- **Toggle flags persistently:** visit `shared/flags.html`
- **Register known flags:** add entries to the `KNOWN_FLAGS` array in
  `shared/flags-ui.js`
- **Promote to production:** remove the `flag` line from `chapter.toml`
