# Product Decision page implementation

## Layout

- `conf/decisions.json` holds the items. The page loads it with `GET /decisions`. It is a list of groups. Each group has `project`, `source`, `spec`, `title`, and `items`. Each item has `id`, `item`, `scope`, `issue`, and `options`. The first option is the recommended one.
- `app/web/product-decision/context.js` holds the groups, the saved decisions, the load, and the save. It has no DOM code. After each change, it calls the `onChange` function of the page. `decisionKey` gives the key of an item.
- `app/web/product-decision/index.js` builds the page with the DOM API. It builds the cards one time after the load. Thus, an input keeps its text when the page renders again.
- `app/web/product-decision/index.css` holds the styles of the page, the groups, and the cards.
- The tests are in `app/web/test/product-decision/context.test.js`.

## Update the items

- When the open behaviour of a project changes, update `conf/decisions.json` from the source file. Copy only the items of kind `decision`. Copy `now` as `issue`.
- Do not change the `id` of an item. The saved decisions use it in their key.

## Persist

- The decisions are the page data `product-decisions` in table `page`. Refer to `spec/app/01_server/code-impl.md`.
- The decisions change only after a save is successful.
