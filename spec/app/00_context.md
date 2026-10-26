# App context

The app is a local server for the user. It is modeled on the ub-command-center app.

## Vocab

| Term | Meaning |
| --- | --- |
| Page | A browser view in the tab bar. Each page is a folder in `app/web/`. |
| Page data | One JSON value that a page saves in the local DB, by page name. |

## Specs

- `spec/app/01_server/` gives the server, the routes, the page shell, and the DB.
- `spec/app/02_todo/` gives the `/todo` page.
