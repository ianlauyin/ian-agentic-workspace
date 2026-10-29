# Server implementation

## Layout

- `app/src/main.rs` builds the router and starts the server. It also changes each plain-text error of axum and tower-http to JSON.
- `app/src/page.rs` holds the `/page` routes and the check of the page name.
- `app/src/open_behaviour.rs` holds the `/open-behaviour-items` route. Refer to `spec/app/03_open-behaviour/code-impl.md`.
- `app/src/db.rs` holds the DB access.
- `app/web/` is the frontend. It uses plain HTML, JavaScript modules, and CSS. It has no build step and no packages.
- `app/web/index.html` is the page shell. It has the tab bar and `main`.
- `app/web/index.js` holds the list of the pages and the API client for `/page`. It fills the tab bar and loads the module of the current page. An unknown path goes to `/todo`.
- `app/web/index.css` holds the color tokens, the styles of the HTML elements, and the UI controls: `g-tab-bar`, `g-button`, and `g-text-input`.
- Each page is a folder `app/web/<page>/`. Its `index.js` exports `mount(main, api)`, and its `index.css` holds its styles.
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
