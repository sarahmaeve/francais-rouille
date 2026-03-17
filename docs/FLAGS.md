# Feature Flags

Feature flags let you stage content on the live site without making it
visible to all users. Flagged pages are deployed and accessible by direct
URL, but hidden from chapter indexes unless the viewer has the flag
enabled.

This is useful for:

- Reviewing new content before it goes live
- Sharing draft pages with specific reviewers via a link
- A/B testing different content variants

## How It Works

1. A page in `chapter.toml` is marked with `flag = "flag-name"`
2. The build system renders the page normally but adds `class="flag-hidden"`
   and `data-flag="flag-name"` to its card in the chapter index
3. `shared/flags.js` (loaded on every page) checks for active flags and
   unhides matching elements by removing the `flag-hidden` class
4. Flags are read from two sources:
   - **`localStorage`** key `fr-flags` (comma-separated flag names)
   - **URL parameter** `?flags=name1,name2` (session override)

## Flagging a Page

Add `flag = "flag-name"` to any page entry in `chapter.toml`:

```toml
[[sections.pages]]
slug = "11_restaurant_commande"
title = "Commander au Restaurant"
description = "Émilie commande le menu du jour."
type = "dialog"
flag = "new-dining"
```

The page itself is always generated and accessible at its direct URL
(`11_restaurant_commande.html`). Only the listing in the chapter index
is hidden.

## Enabling Flags

### For reviewers: shared link

Append `?flags=` to any page URL:

```
https://site.example/chapters/b1-vie-quotidienne/index.html?flags=new-dining
```

Multiple flags can be comma-separated:

```
?flags=new-dining,new-quiz
```

URL flags are active for that page load only. They do not persist.

### For persistent testing: flags UI

Visit `shared/flags.html` on the site to toggle flags on and off. Changes
are saved to `localStorage` and persist across sessions.

### Programmatically

```js
// Enable
localStorage.setItem('fr-flags', 'new-dining,another-flag');

// Disable all
localStorage.removeItem('fr-flags');
```

## Registering Known Flags

When you create a new flag, add it to the `KNOWN_FLAGS` array in
`site/shared/flags-ui.js` so it appears in the management UI:

```js
var KNOWN_FLAGS = [
    { name: 'new-dining', description: 'Experimental dining dialog (B1)' },
    { name: 'my-new-flag', description: 'Description for reviewers' }
];
```

Flags not in this list can still be enabled manually via the UI or URL
parameter — they just won't appear as checkboxes by default.

## Promoting Content

When a flagged page is ready for all users, remove the `flag` line from
its entry in `chapter.toml` and rebuild:

```bash
cargo run -- build
```

The page will appear in the chapter index for everyone. No changes to
JavaScript or CSS are needed.

## Adding flag-hidden to New Chapters

Each chapter's `style.css` must include the `.flag-hidden` class:

```css
.flag-hidden { display: none; }
```

This ensures flagged content is hidden before JavaScript runs (no flash
of hidden content). All existing chapters already have this class.

## Technical Details

### Files

| File | Purpose |
|------|---------|
| `site/shared/flags.js` | Reads flags, unhides matching `data-flag` elements |
| `site/shared/flags-ui.js` | Management UI logic, known flags registry |
| `site/shared/flags.html` | Flag toggle page |
| `site/shared/flags.css` | Styles for the flags page |

### CSP Compliance

The flags system is fully compliant with the site's Content Security
Policy (`site/_headers`):

- No inline scripts — all JS is in external files under `shared/`
- No inline styles — `flag-hidden` is defined in chapter `style.css` files
- No form elements — the flags UI uses button click handlers, not `<form>`
- No external resources — everything is same-origin

### Security Considerations

Feature flags provide **visibility control**, not access control. Flagged
pages are deployed and accessible by direct URL. Do not use flags to gate
sensitive content — use server-side access control (e.g. Cloudflare Access)
for that purpose.
