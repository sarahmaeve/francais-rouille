# Content Security Policy

The site uses a strict Content Security Policy defined in `site/_headers`,
served by Cloudflare Pages. This policy prevents cross-site scripting (XSS)
and data injection attacks.

## Active Policy

```
Content-Security-Policy:
  default-src 'none';
  script-src  'self';
  style-src   'self';
  connect-src 'self';
  media-src   'self';
  img-src     'self';
  font-src    'self';
  base-uri    'self';
  form-action 'none';
  frame-ancestors 'none'
```

## Rules for Generated HTML

### No inline styles

`style-src 'self'` blocks all inline CSS — both `<style>` blocks and
`style="..."` attributes on HTML elements.

**Do:**
- Put all CSS in the chapter's `style.css` file.
- Use CSS classes to style elements.

**Don't:**
- Create companion `.css` files in `content/` (the fragment template no
  longer inlines them).
- Use `style="..."` on any HTML element.

**Exception:** SVG presentation attributes (`font-style`, `fill`, `stroke`,
`font-family`, `transform`, etc.) are **not** CSS inline styles. They are
part of the SVG spec and are unaffected by `style-src`.

### No inline scripts

`script-src 'self'` blocks all inline JavaScript.

**Do:**
- Put JavaScript in external files under `site/shared/`.
- Load scripts via `<script src="..."></script>`.

**Don't:**
- Use `<script>...</script>` with inline code.
- Use event handler attributes (`onclick`, `onload`, `onerror`, etc.).
- Use `javascript:` URLs.

### No external resources

All directives are set to `'self'`, meaning every resource (scripts, styles,
images, fonts, audio) must be served from the same origin.

**Don't:**
- Link to external CDNs for fonts, CSS frameworks, or JS libraries.
- Embed external images or iframes.

## Verifying Compliance

After generating HTML, check for violations:

```bash
# Inline style blocks or attributes (ignore SVG presentation attributes)
grep -rn '<style>\|style="' site/chapters/YOUR_CHAPTER/*.html

# Inline scripts or event handlers
grep -rn '<script>[^<]\|onclick\|onload\|onerror\|javascript:' site/chapters/YOUR_CHAPTER/*.html
```

The first command will also match SVG `font-style` — these are false
positives and can be ignored.
