# Logs page

## Logs

- The page shows one tab for each app in the `apps` map of `conf/log-server.json`, in the order of that file. The tab name is the app name.
- When the page opens, it selects the first tab and the current HKT date.
- Under the tabs, the page shows a control panel and a pager. The pager has a Previous button, a Next button, and the text "Page N of M · T events".
- The page shows the events of the selected app on the selected date. The newest event is first.
- Each page has at most the number of events that the rows list gives. A date with no events shows "No log." and "Page 1 of 1 · 0 events".
- The events are a table. Each event is one row.
- Each field of an event owns a column. Each field of the `extra` object of an event also owns a column, with the header `extra.<field>`. The extra columns are after the other columns.
- The columns are the fields of the events on the shown page, in the order that the page first sees them. A cell is empty when the event has no such field.
- The page uses the full width of the browser window. It does not have the width limit of the other pages.
- The `id` field has no column.
- The `ts` cell shows the date and the time to the second, as `YYYY-MM-DD HH:MM:SS`, in the time of the event.
- The `user_agent` column is narrow.
- A cell that is too long for its column shows an ellipsis. The full text shows in a modal when the user clicks the cell. The user closes the modal with the Close button, with the Escape key, or with a click outside the modal.
- A value that is an object or a list shows as JSON text. The table scrolls sideways when it is wider than the page.
- The first cell of a row of level `warn` or `error` has a colored left border.
- Previous is disabled on the first page. Next is disabled on the last page.
- A change of the tab or of the date selects page 1.
- The log server gives at most the last 1000 events of a date. The page does not show an older event of that date.
- While the page loads, the buttons are disabled and the status text is "Loading…". If a load fails, the page shows the error.
- An answer that is not for the last request is not shown.

## Control panel

- The first bar of the panel has a date input, a rows list, and a Refresh button. The rows list has 50, 100, 500, and 1000 rows. The default is 50. A change of the rows list selects page 1.
- Refresh loads page 1 again with the same app, date, rows, and filters.
- The second bar of the panel has a field input, a value input, and an Add filter button. The field list gives the column names of the events on the shown page. A column name is the first part of a path.
- A filter is a field and a value. A field is a path of keys that are joined with dots, from the event to a value, for example `extra.query.limit`. The path goes into objects at any depth. When a value on the path is a text that holds JSON, the path goes into that JSON. An event passes a filter when the text of the value at the path is equal to the filter value.
- An event must pass all filters. The page shows each filter as a pill with a remove button. A new filter on a field replaces the old filter on that field.
- A filter with no field or no value is not added.
- A change of a filter selects page 1. A change of the tab keeps the filters and the rows.
- The panel does not save its state.

## Server

- `GET /logs-apps` returns the apps of `conf/log-server.json` as `[{"id": "<app>", "name": "<app>"}]`.
- `GET /logs-events` takes `app`, `date` as `YYYY-MM-DD`, `page` from 1, the optional `page_size` (50, 100, 500, or 1000, default 50), and the optional `filters` as a JSON list of `{"field", "value"}`. The server applies the filters to the events of the date, then makes the pages. It returns `{"page": N, "page_size": S, "total": T, "events": [...]}`.
- An app that is not in the `apps` map, a date that has a wrong format, or a page that is not a number of 1 or more, a page size that is not in the list, or a `filters` value that is not correct returns 400.
- The server reads the events with the method of `skills/fetch-logs.md`. It uses the key `LOG_SERVER_READ_API_KEY` that `conf/log-server.json` names, from `.env`, and the limit 1000.
- A failed read of `conf/log-server.json` or of the key returns 500. A failed request to the log server returns 500 with the reason. The reason does not contain the key.
