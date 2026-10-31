# Logs page

## Logs

- The page shows one tab for each app in the `apps` map of `conf/log-server.json`, in the order of that file. The tab name is the app name.
- When the page opens, it selects the first tab and the current HKT date.
- Under the tabs, the page shows a date input, a Previous button, a Next button, and the text "Page N of M · T events".
- The page shows the events of the selected app on the selected date. The newest event is first.
- Each page has at most 50 events. A date with no events shows "No log." and "Page 1 of 1 · 0 events".
- The events are a table. Each event is one row.
- Each field of an event owns a column. Each field of the `extra` object of an event also owns a column, with the header `extra.<field>`. The extra columns are after the other columns.
- The columns are the fields of the events on the shown page, in the order that the page first sees them. A cell is empty when the event has no such field.
- The page uses the full width of the browser window. It does not have the width limit of the other pages.
- The `id` field has no column.
- The `ts` cell shows the date and the time to the second, as `YYYY-MM-DD HH:MM:SS`, in the time of the event.
- The `user_agent` column is narrow.
- A cell that is too long for its column shows an ellipsis. The full text shows in a popup under the cell when the user holds the pointer on the cell.
- A value that is an object or a list shows as JSON text. The table scrolls sideways when it is wider than the page.
- The first cell of a row of level `warn` or `error` has a colored left border.
- Previous is disabled on the first page. Next is disabled on the last page.
- A change of the tab or of the date selects page 1.
- The log server gives at most the last 1000 events of a date. The page does not show an older event of that date.
- While the page loads, the buttons are disabled and the status text is "Loading…". If a load fails, the page shows the error.
- An answer that is not for the last request is not shown.

## Server

- `GET /logs-apps` returns the apps of `conf/log-server.json` as `[{"id": "<app>", "name": "<app>"}]`.
- `GET /logs-events` takes `app`, `date` as `YYYY-MM-DD`, and `page` from 1. It returns `{"page": N, "page_size": 50, "total": T, "events": [...]}`.
- An app that is not in the `apps` map, a date that has a wrong format, or a page that is not a number of 1 or more returns 400.
- The server reads the events with the method of `skills/fetch-logs.md`. It uses the key `LOG_SERVER_READ_API_KEY` that `conf/log-server.json` names, from `.env`, and the limit 1000.
- A failed read of `conf/log-server.json` or of the key returns 500. A failed request to the log server returns 500 with the reason. The reason does not contain the key.
