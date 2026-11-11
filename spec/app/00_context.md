# App context

The app is a local server for the user. It is modeled on the ub-command-center app.

## Rules

- The frontend in `app/web/` uses plain HTML, JavaScript, and CSS.
- The app obeys `rules/coding-style.md`.

## Vocab


| Term      | Meaning                                                             |
| --------- | ------------------------------------------------------------------- |
| Page      | A browser view in the nav. Each page is a folder in `app/web/`. |
| Page data | One JSON value that a page saves in the local DB, by page name.     |


## Specs

- `spec/app/01_server.md` gives the server, the routes, the page shell, and the DB.
- `spec/app/02_todo.md` gives the `/todo` page.
- `spec/app/03_open-behaviour.md` gives the `/open-behaviour` page.
- `spec/app/04_notes.md` gives the `/notes` page.
- `spec/app/05_logs.md` gives the `/logs` page.
