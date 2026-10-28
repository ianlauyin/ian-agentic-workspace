# TODO page implementation

## Layout

- `app/web/todo/context.js` holds the two lists, the load, and the actions that save the lists. It has no DOM code. After each change, it calls the `onChange` function of the page. Each action gets the section name, `important` or `pending`. `SECTIONS` gives the order of the sections.
- `app/web/todo/index.js` builds the page with the DOM API. It holds the UI state: the row in edit and the drag. It renders the lists again after each change. The add forms stay, so their inputs keep their text.
- `app/web/todo/index.css` holds the styles of the page, the sections, and the rows.
- The reorder uses the HTML drag and drop API. A row is draggable only while the mouse button is down on its handle. Thus, the user can still select the text.
- Each section element is a drop target. Thus, the user can drop a row on an empty section.
- The tests are in `app/web/test/todo/context.test.js`.

## Persist

- The lists are the page data `todo-items` in table `page`. Refer to `spec/app/01_server/code-impl.md`.
- The lists change only after a save is successful.
