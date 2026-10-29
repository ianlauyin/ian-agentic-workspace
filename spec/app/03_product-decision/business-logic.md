# Product Decision page

## Items

- The page shows the product decisions that the user must make for the projects in `conf/projects.json`.
- The items come from the open behaviour of each project. For sales-tools, the source is the items of kind `decision` in `spec/site/open-behaviour.html` of the sales-tools repo.
- The page groups the items by project and spec folder, in the order of the source.
- Under the title, the page shows the number of decided items and the number of all items.

## Card

Each item is one card. The card has these parts, in this order:

- The item number, the item name, and the status. The status is "Open" or "Decided".
- The scope: the apps and services that the item touches.
- The issue: what the code does now, with evidence.
- Three recommendations. The first recommendation has the "Recommended" label.
- An input for the final decision, and a Save button.

## Final decision

- Save or Cmd+Enter keeps the final decision of that card.
- The Save button is disabled when the input text is the same as the saved decision.
- An empty text removes the saved decision. The card is then "Open".

## Persist

- Each save writes all final decisions with `PUT /page`, as `product-decisions`, with the data `{"<project>/<spec>/<id>": "<decision>"}`.
- When the page opens, it loads the items and `product-decisions` with `GET /page`.
- If a save fails, the page shows the error and keeps the last saved decisions.
- If the load fails, the page shows the error and shows no card.
