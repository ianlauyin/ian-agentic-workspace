# Server

## Server

- The server listens on `http://127.0.0.1:47322`.
- A route returns JSON. If it has no data to return, it returns `null`.
- A `GET` route takes its params in the query. A `PUT` route and a `POST` route take a JSON body.
- A body without the JSON content type returns 415.
- A JSON body without a necessary field returns 422.
- A bad request returns 400.
- A failed DB call returns 502.
- A failed read of `conf/projects.json` returns 500.
- An error returns JSON as `{"error": "<reason>"}`. This includes the errors that axum returns.

## Pages

- `GET /` and all page paths return `app/web/index.html`.
- All other `GET` requests return a file from `app/web/`.
- The tab bar has a link to each page. It marks the current page.
- An unknown page path in the browser shows `/todo`.

| Page | Description |
| --- | --- |
| `GET /` | Shows `/todo`. |
| `GET /todo` | Shows the TODOs of the user. Refer to `spec/app/02_todo/business-logic.md`. |
| `GET /open-behaviour` | Shows the open behaviour of each project, the final decisions of the user, and the started resolves. Refer to `spec/app/03_open-behaviour/business-logic.md`. |

## Routes

| Route | Description |
| --- | --- |
| `GET /page` | Returns the saved JSON of a page by `name`. If there is no save, returns `null`. |
| `PUT /page` | Saves the JSON `data` of a page by `name`, and replaces the last save. Returns `null`. |
| `GET /open-behaviour-items` | Returns the open-behaviour items of each project in `conf/projects.json`. Refer to `spec/app/03_open-behaviour/business-logic.md`. |
| `POST /open-behaviour-resolve` | Starts an agent that resolves one decided item. Returns `{"chat": "<chat id>"}`. Refer to `spec/app/03_open-behaviour/business-logic.md`. |
| `GET /open-behaviour-progress` | Returns the progress of the agent of a `chat` as `{"status": "running" \| "finished" \| "failed" \| "stopped", "steps": [{"kind": "message" \| "tool" \| "output" \| "reply", "text": "..."}]}`. A chat id with a character that is not a letter, a digit, or `-` returns 400. A chat with no log returns 400. Refer to `spec/app/03_open-behaviour/business-logic.md`. |
| `POST /open-behaviour-followup` | Sends `{"chat": "<chat id>", "text": "..."}` to the agent of that chat as a follow-up. Returns the progress, as `GET /open-behaviour-progress` does. A bad chat id, an empty text, a chat with no log, or an agent that still works returns 400. Refer to `spec/app/03_open-behaviour/business-logic.md`. |

- A page name has only letters, digits, and `-`. Other names return 400.
