# Design system

The design system gives the app the default style of Claude: a warm ivory page, a clay accent, and serif headings.

## Tokens

- `app/web/design-system.css` holds all tokens as CSS variables on `:root`. A `prefers-color-scheme: dark` block gives the dark values.
- Color tokens: `--bg`, `--panel`, `--text`, `--muted`, `--line`, `--accent`, `--on-accent`, `--bad`, `--warn`, `--overlay`.
- `--on-accent` is the text color on an `--accent` background.
- Shadow token: `--shadow-raised`.
- Gradient tokens: `--gradient-accent` for the accent buttons and badges, `--gradient-panel` for the panels and cards, `--gradient-page` for the page background, `--gradient-title` for the `h1` text, and `--gradient-active` for the active nav link, and `--gradient-dock` for the nav dock. `--shadow-float` is the shadow of the nav dock. `--shadow-glow` is the hover shadow of an accent button.
- Font tokens: `--font-sans` for the body, `--font-serif` for `h1` and `h2`, and `--font-mono` for code and log text.
- Radius tokens: `--radius-xs` (4px), `--radius-sm` (8px), `--radius-md` (12px), and `--radius-pill`.

## Use

- `app/web/index.html` loads `/design-system.css` before `/index.css`.
- `app/web/index.css` and the `index.css` of each page use only tokens for color, font family, radius and shadow.
- To add a value, add a token to `app/web/design-system.css`, with its dark value. Then use the token.
- The UI controls `g-button`, `g-text-input` and `g-nav` are in `app/web/index.css`. The accent button uses `--accent` and `--on-accent`.
