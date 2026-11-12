# App context

The app is a local server for the user. It is modeled on the ub-command-center app.

## Rules

- The frontend in `app/web/` uses plain HTML, JavaScript, and CSS.
- The app obeys `rules/coding-style.md`.
- The CSS uses the design tokens of `app/web/design-system.css`. It does not use a color, a font family, a radius or a shadow value that is not a token.

## Vocab


| Term      | Meaning                                                             |
| --------- | ------------------------------------------------------------------- |
| Page      | A browser view in the nav. Each page is a folder in `app/web/`. |
| Page data | One JSON value that a page saves in the local DB, by page name.     |


## Specs

- `spec/app/01_server/` gives the server, the routes, the page shell, and the DB.
- `spec/app/02_todo/` gives the `/todo` page.
- `spec/app/04_notes/` gives the `/notes` page.
- `spec/app/05_logs/` gives the `/logs` page.
- `spec/app/06_design-system/` gives the design system.
