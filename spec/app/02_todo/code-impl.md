# TODO page implementation

## Layout

- `app/web/todo/context.js` holds the list, the load, and the actions that save the list. It has no DOM code. After each change, it calls the `onChange` function of the page.
- `app/web/todo/index.js` builds the page with the DOM API. It holds the UI state: the row in edit and the drag. It renders the list again after each change. The add form stays, so its input keeps its text.
- `app/web/todo/index.css` holds the styles of the page, the add form, and the rows.
- The reorder uses the HTML drag and drop API. A row is draggable only while the mouse button is down on its handle. Thus, the user can still select the text.
- The tests are in `app/web/test/todo/context.test.js`.

## Persist

- The list is the page data `todo-items` in table `page`. Refer to `spec/app/01_server/code-impl.md`.
- The list changes only after a save is successful.
