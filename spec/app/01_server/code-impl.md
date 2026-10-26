# Server implementation

## Layout

- `app/src/main.rs` builds the router and starts the server. It also changes each plain-text error of axum to JSON.
- `app/src/page/` holds the `/page` routes and the DB access.
- `app/web/index.html` holds the tab bar. `app/web/index.js` routes each page path in the browser.
- `app/web/index.css` holds the styles that all pages share.
- Each page is a folder `app/web/<page>/`, with `index.html`, `index.css`, and `index.js`.
- The `index.html` of a page holds only the content of its `<main>`.
- The `index.js` of a page exports a function. The router calls it with a new `<main>`, which holds the HTML and the CSS of the page.

## Persist

- The local DB is SQLite at `app/data/app.db`. The server creates it on start.
- Table `page` keeps one JSON value for each page name, with the time of the save.
- Git ignores `app/data/` and `app/target/`.

## Run

- Start the server from `app/` with `cargo watch -w src -w Cargo.toml -x run`.
- The server reads the web files from the disk for each request. Thus, an edit of a web file does not need a rebuild or a restart.

## Do not

- Do not put the DB in `app/src/`. A write to it then restarts `cargo watch`.
- Do not add a framework or a build step to a page. A page is plain HTML and JS.
