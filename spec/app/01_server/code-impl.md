# Server implementation

## Layout

- `app/src/main.rs` builds the router and starts the server. It also changes each plain-text error of axum and tower-http to JSON.
- `app/src/page.rs` holds the `/page` routes and the check of the page name.
- `app/src/open_behaviour.rs` holds the `/open-behaviour-items` route, the `/open-behaviour-resolve` route, the `/open-behaviour-progress` route, and the `/open-behaviour-followup` route. Refer to `spec/app/03_open-behaviour/code-impl.md`.
- `app/src/notes.rs` holds the `/projects` route and the `/notes-assign` route. Refer to `spec/app/04_notes/code-impl.md`.
- `app/src/logs.rs` holds the `/logs-apps` route and the `/logs-events` route. Refer to `spec/app/05_logs/code-impl.md`.
- `app/src/db.rs` holds the DB access.
- `app/web/` is the frontend. It uses plain HTML, JavaScript modules, and CSS. It has no build step and no packages.
- `app/web/index.html` is the page shell. It has the nav, with a link to each page, and `main`. The nav is a side panel at the right edge. It slides in when the pointer is at the right edge or the focus is in it.
- `app/web/index.js` holds the list of the pages and the API client for all routes. It routes in the browser: a click on a nav link changes the URL with `history.pushState` and does not reload the document. It loads the CSS and the module of the page first. Then it calls the cleanup of the old page, replaces the old page CSS, and calls `mount`. An unknown path goes to `/todo`.
- `app/web/index.css` holds the color tokens, the styles of the HTML elements, and the UI controls: `g-nav`, `g-button`, and `g-text-input`.
- Each page is a folder `app/web/<page>/`. Its `index.js` exports `mount(main, api)`. `mount` returns a cleanup function when the page starts a timer. Its `index.css` holds its styles. The shell loads `index.css`, so `mount` does not.
- The tests of the Rust code are in `app/src/test/`.
- The tests of the frontend are in `app/web/test/`. They use the Node test runner.

## Persist

- The local DB is SQLite at `app/data/app.db`. The server creates it on start.
- Table `page` keeps one JSON value for each page name, with the time of the save.
- Git ignores `app/data/` and `app/target/`.

## Run

- Start the server from `app/` with `cargo watch -w src -w Cargo.toml -x run`.
- The server reads `app/web/` from the disk for each request. Thus, a frontend edit does not need a restart. Reload the browser.

## Check

- From `app/`, run `cargo test` and `cargo clippy --all-targets`.
- From `app/web/`, run `node --test`.

## Do not

- Do not put the DB in `app/src/`. A write to it then restarts `cargo watch`.
- Do not add a page path only in `PAGES` in `app/web/index.js`. Also add it to `PAGE_PATHS` in `app/src/main.rs`. If not, a reload of the page returns 404.
