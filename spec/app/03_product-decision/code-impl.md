# Product Decision page implementation

## Layout

- The items are the page data `product-decision-items` in table `page`. Refer to `spec/app/01_server/code-impl.md`. The data is a list of tabs. Each tab has `id`, `name`, and `groups`. Each group has `source`, `spec`, `title`, and `items`. Each item has `id`, `item`, `scope`, `issue`, and `options`. The first option is the recommended one.
- Do not write a project name or an item of a project in `app/web/`. The page reads those values from the DB.
- `app/web/product-decision/context.js` holds the tabs, the saved decisions, the load, the rename, and the save. It has no DOM code. After each change, it calls the `onChange` function of the page. `decisionKey` gives the key of an item. `groupsByStatus` splits the groups of one tab into open items and decided items.
- `app/web/product-decision/index.js` builds the page with the DOM API. It builds the tabs, the name inputs, and the cards one time after the load. On each render, it moves each decided card to the bottom of its tab. Thus, an input keeps its text.
- `app/web/product-decision/index.css` holds the styles of the page, the tabs, the groups, and the cards.
- The tests are in `app/web/test/product-decision/context.test.js`.

## Update the items

- When the open behaviour of a project changes, replace the groups of that tab in `product-decision-items` with `PUT /page`. Copy only the items of kind `decision`. Copy `now` as `issue`.
- Do not change the `id` of a tab or the `id` of an item. The saved decisions use them in their key.

## Persist

- The decisions are the page data `product-decisions` in table `page`. Refer to `spec/app/01_server/code-impl.md`.
- The decisions change only after a save is successful.
