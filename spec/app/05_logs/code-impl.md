# Logs page implementation

## Layout

- `app/src/logs.rs` holds `GET /logs-apps` and `GET /logs-events`. `read_config` reads `conf/log-server.json`. `read_env_value` reads the key from `.env` at the workspace root. `fetch` runs `curl` and sends the key on stdin with `-H @-`, so the key is not in the process list. `page_of` reverses the events and takes one page.
- `app/web/logs/context.js` holds the apps, the selected app and date, the result, the load, and `show`. It has no DOM code. `logColumns` and `cellText` make the columns and the cell text. `show` ignores an answer of an old request.
- `app/web/logs/index.js` builds the page with the DOM API.
- `app/web/logs/index.css` holds the styles of the page.
- The tests are in `app/src/logs.rs` and `app/web/test/logs/context.test.js`.

## Do not

- Do not name the API route `/logs`. It is the path of the page, and the router has the page path.
- Do not write an app name in `app/web/`.
- Do not show the key in an error message.
