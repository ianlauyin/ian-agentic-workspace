# TODO page implementation

## Layout

- `app/web/todo/context.js` holds the three lists, the load, and the actions that save the lists. It has no DOM code. After each change, it calls the `onChange` function of the page. Each action gets the column name: `pending`, `ongoing` or `reviewing`. `SECTIONS` gives the order of the columns. `normalize` changes the saved data, old or new, to the three lists.
- `app/web/todo/index.js` builds the page with the DOM API. It holds the UI state: the card in edit and the drag. It renders the lists again after each change. The add form is in a `dialog` element, opened with `showModal`.
- `app/web/todo/index.css` holds the styles of the page, the board, the columns, and the cards. The board is a CSS grid with three columns.
- The move uses the HTML drag and drop API. A card is draggable only while the mouse button is down on its top bar (the `handle` element). Thus, the user can still select the text.
- The columns of the board have the same height. Each column element is a drop target, and its drop section fills the free height of the column. Thus, the user can drop a card on an empty column.
- The tests are in `app/web/test/todo/context.test.js`.

## Persist

- The lists are the page data `todo-items` in table `page`. Refer to `spec/app/01_server/code-impl.md`.
- The lists change only after a save is successful.
