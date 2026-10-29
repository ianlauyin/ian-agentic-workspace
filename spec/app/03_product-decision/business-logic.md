# Product Decision page

## Items

- The page shows one tab for each project in the page data `product-decision-items`.
- The tab shows the saved name of that project.
- Save or Enter keeps a new tab name. The Save button is disabled when the input text is empty or the same as the saved name.
- An empty text does not change the tab name.
- The first tab is selected when the page opens.
- The page shows the groups of the selected tab.
- The items of a tab come from the open behaviour of that project. For sales-tools, the source is the items of kind `decision` in `spec/site/open-behaviour.json` of the sales-tools repo.
- The page shows the open items of the selected tab first. It groups those items by spec folder, in the order of the source.
- The page shows the decided items of the selected tab at the bottom. It groups those items by spec folder, in the order of the source.
- A group with no open item is not in the open part. A group with no decided item is not at the bottom.
- Under the title, the page shows the number of decided items and the number of all items in the selected tab.

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

- The items are the page data `product-decision-items`. The data is a list of tabs. Each tab has `id`, `name`, and `groups`.
- A rename writes the list of tabs with `PUT /page`, as `product-decision-items`. The rename changes `name` only.
- Do not change the `id` of a tab. The saved decisions use it in their key.
- Each save writes all final decisions with `PUT /page`, as `product-decisions`, with the data `{"<tab id>/<spec>/<item id>": "<decision>"}`.
- When the page opens, it loads `product-decision-items` and `product-decisions` with `GET /page`.
- If a rename fails, the page shows the error and keeps the last saved name.
- If there is no `product-decision-items` save, the page shows no card.
- If a save fails, the page shows the error and keeps the last saved decisions.
- If the load fails, the page shows the error and shows no card.
