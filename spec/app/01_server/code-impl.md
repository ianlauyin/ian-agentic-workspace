# Server implementation

## Layout

- `app/src/main.rs` builds the router and starts the server. It also changes each plain-text error of axum and tower-http to JSON.
- `app/src/page.rs` holds the `/page` routes and the check of the page name.
- `app/src/db.rs` holds the DB access.
- `app/web/` is the frontend package. It uses SolidJS, TypeScript, and Vite.
- `app/web/src/App.tsx` holds the list of the pages and the browser routes. An unknown path goes to `/todo`.
- `app/web/src/context.tsx` holds the global context. It gives the API client for `/page`.
- `app/web/src/component/` holds the UI controls: the tab bar, the button, and the text input.
- `app/web/src/global.css` holds the color tokens and the styles of the HTML elements.
- Each page is a module in `app/web/src/module/<page>/`.
- The tests of the frontend are in `app/web/test/`.

## Persist

- The local DB is SQLite at `app/data/app.db`. The server creates it on start.
- Table `page` keeps one JSON value for each page name, with the time of the save.
- Git ignores `app/data/`, `app/target/`, `app/web/node_modules/`, and `app/web/dist/`.

## Run

- Install the frontend packages from `app/web/` with `pnpm install`.
- Build the frontend from `app/web/` with `pnpm dev`. This command builds again after each edit.
- Start the server from `app/` with `cargo watch -w src -w Cargo.toml -x run`.
- The server reads `app/web/dist/` from the disk for each request. Thus, a new frontend build does not need a restart.

## Check

- From `app/`, run `cargo test` and `cargo clippy --all-targets`.
- From `app/web/`, run `pnpm test`, `pnpm typecheck`, and `pnpm lint`.

## Do not

- Do not put the DB in `app/src/`. A write to it then restarts `cargo watch`.
- Do not add a page path only in `app/web/src/App.tsx`. Also add it to `PAGE_PATHS` in `app/src/main.rs`. If not, a reload of the page returns 404.
