# TODO page

## My TODO

- The "My TODO" section is the only section of the page.
- The user adds a TODO with the input and the Add button, or with Enter.
- Each TODO is a row with a drag handle, its text, an Edit button, and a Delete button.
- The user drags a row by its handle to change the order. A line shows where the row goes. The drop saves the new order.
- Edit changes the text to an input. Save or Enter keeps the change. Cancel or Escape discards it.
- An empty text does not add or change a TODO.
- Delete asks for a confirmation before it removes the TODO.
- Each change saves the full list with `PUT /page`, as `todo-items`, with the data `{"items": [{"text": "..."}]}`.
- When the page opens, it loads `todo-items` with `GET /page`.
- If a save fails, the page shows the error and keeps the last saved list.
- If the load fails, the page shows the error. The Add button stays disabled. Thus, a save cannot replace the saved list with an empty list.
