# Server

## Server

- The server listens on `http://127.0.0.1:47322`.
- A route returns JSON. If it has no data to return, it returns `null`.
- A `GET` route takes its params in the query. A `PUT` route takes a JSON body.
- A body without the JSON content type returns 415.
- A JSON body without a necessary field returns 422.
- A bad request returns 400.
- A failed DB call returns 502.
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
| `GET /product-decision` | Shows the product decisions that the user must make. Refer to `spec/app/03_product-decision/business-logic.md`. |

## Routes

| Route | Description |
| --- | --- |
| `GET /page` | Returns the saved JSON of a page by `name`. If there is no save, returns `null`. |
| `PUT /page` | Saves the JSON `data` of a page by `name`, and replaces the last save. Returns `null`. |

- A page name has only letters, digits, and `-`. Other names return 400.
