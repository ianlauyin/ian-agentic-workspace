# Open Behaviour page

## Items

- The page shows one tab for each project in `conf/projects.json`, in the order of that file. The tab name is the project name. The user cannot change the tab name.
- Each tab shows a badge with the number of open items of that project. The badge changes after each save of a final decision. A tab with no open item shows no badge.
- The items of a tab come from the open-behaviour files of that project: `spec/open-behaviour.json` first, then each `spec/<package>/open-behaviour.json` in package name order. The file rules are in `template/spec/structure.md`.
- The page reads the files each time it opens. Thus, a change to a file shows after a reload.
- The page shows the items of all kinds.
- When the page opens, it selects the first tab that has items. If no tab has items, it selects the first tab.
- The page shows the open items of the selected tab first. It groups those items by spec entry, in the order of the source.
- The page shows the decided items of the selected tab at the bottom. It groups those items by spec entry, in the order of the source.
- A group with no open item is not in the open part. A group with no decided item is not at the bottom.
- Each group shows the spec title, the source file, the spec folder, and the context of the entry.
- A tab with no items shows "No open item."
- Under the title, the page shows the number of decided items and the number of all items in the selected tab.

## Card

Each item is one card. The card has these parts, in this order:

- The item number, the item name, the kind, and the status. The status is "Open" or "Decided".
- The scope: the apps and services that the item touches.
- Now: what the code does now, with evidence. The note of the item is under it.
- Waits on: the teams that make the decision.
- Next step: the recorded decision in the file.
- Three recommendations. The first recommendation has the "Recommended" label.
- An input for the final decision, and a Save button.

## Final decision

- Save or Cmd+Enter keeps the final decision of that card.
- The Save button is disabled when the input text is the same as the saved decision.
- An empty text removes the saved decision. The card is then "Open".
- The final decision stays only in the local DB. The page does not change the open-behaviour files.

## Persist

- The page loads the tabs with `GET /open-behaviour-items`. Refer to `spec/app/01_server/business-logic.md`.
- Each save writes all final decisions with `PUT /page`, as `open-behaviour-decisions`.
- The key of a decision is `<tab id>/<spec>/<item id>` for an item in `spec/open-behaviour.json`, and `<tab id>/<package>/<spec>/<item id>` for an item in `spec/<package>/open-behaviour.json`. The data is `{"<key>": "<decision>"}`.
- When the page opens, it loads `open-behaviour-decisions` with `GET /page`.
- If a file of a project cannot be read or parsed, the tab of that project shows the error. The tab still shows the items of the other files.
- If a save fails, the page shows the error and keeps the last saved decisions.
- If the load fails, the page shows the error and shows no card.
