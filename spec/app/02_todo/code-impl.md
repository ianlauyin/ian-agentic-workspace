# TODO page implementation

## Layout

- `app/web/todo/index.html` holds the add form and the list box.
- `app/web/todo/index.js` loads, renders, reorders, and saves the list.
- The reorder uses the HTML drag and drop API. A row is draggable only while the mouse button is down on its handle. Thus, the user can still select the text.
- `app/web/todo/index.css` has one root selector, `.todo-page`.

## Persist

- The list is the page data `todo-items` in table `page`. Refer to `spec/app/01_server/code-impl.md`.
