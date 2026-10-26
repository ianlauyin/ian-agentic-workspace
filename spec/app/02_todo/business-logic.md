# TODO page

## My TODO

- The "My TODO" section is the only section of the page.
- The user adds a TODO with the input and the Add button, or with Enter.
- Each TODO is a row with its text, an Edit button, and a Delete button.
- Edit changes the text to an input. Save or Enter keeps the change. Cancel or Escape discards it.
- An empty text does not add or change a TODO.
- Delete asks for a confirmation before it removes the TODO.
- Each change saves the full list with `PUT /page`, as `todo-items`, with the data `{"items": [{"text": "..."}]}`.
- When the page opens, it loads `todo-items` with `GET /page`.
- If a save fails, the page shows the error and keeps the last saved list.
