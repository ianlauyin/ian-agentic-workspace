# TODO page

## Sections

- The page has two sections, in this order: "Important" and "Pending".
- Each section has its own list, its own input and its own Add button.
- If a section has no TODO, it shows "No TODO yet.".

## TODO

- The user adds a TODO to a section with the input and the Add button of that section, or with Enter.
- Each TODO is a row with a drag handle, its text, an Edit button, and a Delete button.
- The user drags a row by its handle to change the order, or to move the TODO to the other section. A line shows where the row goes. A drop on a section, but not on a row, puts the row at the end of that section. The drop saves the new lists.
- Edit changes the text to an input. Save or Enter keeps the change. Cancel or Escape discards it.
- An empty text does not add or change a TODO.
- Delete asks for a confirmation before it removes the TODO.

## Persist

- Each change saves the two lists with `PUT /page`, as `todo-items`, with the data `{"important": [{"text": "..."}], "pending": [{"text": "..."}]}`.
- When the page opens, it loads `todo-items` with `GET /page`.
- If the saved data has the old `items` list, the page loads that list as the "Pending" list. The next save writes the new data.
- If a save fails, the page shows the error and keeps the last saved lists.
- If the load fails, the page shows the error. The Add buttons stay disabled. Thus, a save cannot replace the saved lists with empty lists.
