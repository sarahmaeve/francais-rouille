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
