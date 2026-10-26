# TODO page implementation

## Layout

- `app/web/src/module/todo/context.ts` holds the list, the load, and the actions that save the list.
- `app/web/src/module/todo/TodoPage/` is the entry component. It holds the UI state: the row in edit and the drag.
- `app/web/src/module/todo/TodoPage/AddForm/`, `ItemRow/`, and `EditRow/` are the sub-components.
- The reorder uses the HTML drag and drop API. A row is draggable only while the mouse button is down on its handle. Thus, the user can still select the text.
- The tests are in `app/web/test/module/todo/context.test.ts`.

## Persist

- The list is the page data `todo-items` in table `page`. Refer to `spec/app/01_server/code-impl.md`.
- The list changes only after a save is successful.
