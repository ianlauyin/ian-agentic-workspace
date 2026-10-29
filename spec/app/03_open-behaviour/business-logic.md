# Open Behaviour page

## Items

- The page shows one tab for each project in `conf/projects.json`, in the order of that file. The tab name is the project name. The user cannot change the tab name.
- Each tab shows a badge with the number of open items of that project. The badge changes after each save of a final decision. A tab with no open item shows no badge.
- The items of a tab come from `spec/open-behaviour.json` of that project, in the order of that file. The file rules are in `template/spec/structure.md`. If the file is not there, the tab has no items.
- The page reads the files each time it opens. Thus, a change to a file shows after a reload.
- Under the tabs, the page shows a kind filter with these buttons: "All", "Bug", "Needs decision", and "Cleanup". When the page opens, "All" is selected.
- When a kind is selected, the page shows only the items of that kind in all tabs. The badges, the number of items under the title, and the "No open item." text count only the items of that kind. "All" shows the items of all kinds.
- The kind filter is not in the local DB. A reload selects "All" again.
- When the page opens, it selects the first tab that has items. If no tab has items, it selects the first tab.
- The page shows the open items of the selected tab first. It groups those items by spec entry, in the order of the source.
- The page shows the decided items of the selected tab at the bottom. It groups those items by spec entry, in the order of the source.
- A group with no open item is not in the open part. A group with no decided item is not at the bottom.
- Each group shows the spec title, the spec folder path, and the context of the entry.
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
- An input for the final decision, a Save button, and a Resolve button.

## Final decision

- Save or Cmd+Enter keeps the final decision of that card.
- The Save button is disabled when the input text is the same as the saved decision.
- An empty text removes the saved decision. The card is then "Open".
- The final decision stays only in the local DB. The page does not change `spec/open-behaviour.json` of a project.

## Resolve

- The Resolve button shows only on a card that has a saved final decision.
- The Resolve button is disabled when the input text is not the same as the saved decision. Save the decision first.
- Resolve starts one Cursor agent in the background, in this workspace, in a new Cursor chat. The model is Claude Opus 5.5 with medium effort. The agent resolves the item as the saved final decision tells, and then deletes the item from `spec/open-behaviour.json` of the project. It obeys `AGENTS.md`.
- The agent runs without a person. It can edit files and run commands without approval.
- The agent continues when the page closes or the server restarts.
- After the start, the button shows "Started" and stays disabled. The card shows the chat id of the agent and the `cursor-agent --resume` command that opens the chat. A reload keeps this state.
- The page does not show when the agent stops. After the agent deletes the item from the file, the item is not on the page after a reload.
- If the start fails, the page shows the error, and the user can click Resolve again.
- The server starts an agent only for an item that is in `spec/open-behaviour.json` of a project in `conf/projects.json`. A different item returns 400.

## Persist

- The page loads the tabs with `GET /open-behaviour-items`. Refer to `spec/app/01_server/business-logic.md`.
- Each save writes all final decisions with `PUT /page`, as `open-behaviour-decisions`.
- The key of a decision is `<tab id>/<spec>/<item id>`. `<spec>` is the `spec` field of the entry, as `<package>/NN_name`. The data is `{"<key>": "<decision>"}`.
- When the page opens, it loads `open-behaviour-decisions` with `GET /page`.
- Each successful resolve writes all started resolves with `PUT /page`, as `open-behaviour-resolves`.
- The key of a resolve is the same as the key of a decision. The data is `{"<key>": "<chat id>"}`.
- When the page opens, it loads `open-behaviour-resolves` with `GET /page`.
- A resolve is saved only after the start is successful. The page does not save the value `starting`.
- If a save of the resolves fails, the page shows the error and keeps the last saved resolves.
- If the file of a project cannot be read or parsed, the tab of that project shows the error. The other tabs still show their items.
- If a save fails, the page shows the error and keeps the last saved decisions.
- If the load fails, the page shows the error and shows no card.
