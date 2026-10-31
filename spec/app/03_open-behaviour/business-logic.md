# Open Behaviour page

## Items

- The page shows one tab for each project in `conf/projects.json`, in the order of that file. The tab name is the project name. The user cannot change the tab name.
- Each tab shows a badge with the number of open items of that project. The badge changes after each save of a final decision. A tab with no open item shows no badge.
- The items of a tab come from `spec/open-behaviour.json` of that project, in the order of that file. If the file is not there, the tab has no items.
- The page reads the files each time it opens. Thus, a change to a file shows after a reload.
- Under the tabs, the page shows a kind filter with these buttons: "All", "Bug", "Needs decision", and "Cleanup". When the page opens, "All" is selected.
- When a kind is selected, the page shows only the items of that kind in all tabs. The badges, the number of items under the title, and the "No open item." text count only the items of that kind. "All" shows the items of all kinds.
- The kind filter is not in the local DB. A reload selects "All" again.
- When the page opens, it selects the first tab that has items. If no tab has items, it selects the first tab.
- The page shows the items of the selected tab grouped by spec entry, in the order of the source. A saved final decision does not move the item.
- A group with no item of the selected kind is not shown.
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
- Resolve starts one local Cursor agent in the local project folder of the item. Resolve starts no cloud agent. The agent calls no subagent. The model is Claude Sonnet 5.5 with medium effort.
- The agent resolves the item as the saved final decision tells. It obeys `AGENTS.md` of the project. When the item is resolved, the agent removes the full item from the `items` list of its spec entry in `spec/open-behaviour.json` of the local project folder. It does not keep the item with a resolved status or a note. The page reads the local file, so the item goes from the page after a reload.
- When the first agent fails, the server starts a local Cursor agent with Grok 4.7 and high effort, in the same folder with the same prompt. The progress shows "The local agent failed. A local Grok agent takes over now."
- An agent fails when it gives a failed result, or when it exits with an error code before a result. When the first agent cannot start, the server starts the Grok agent. When the Grok agent also cannot start, the start fails.
- The card keeps the chat id that the server gives at the start. The progress shows the progress of the first agent and then of the Grok agent. An agent that stopped or finished does not start the Grok agent.
- The agent runs without a person. It can edit files and run commands without approval.
- The agent continues when the page closes or the server restarts.
- After the start, the button shows "Started" and stays disabled. The card shows the chat id and tells that the local agent works in the project folder. A reload keeps this state.
- When the progress gives the status "the agent failed", the page removes the started resolve. This occurs only when the Grok agent also fails, or when the Grok agent cannot start. The button shows "Resolve" again, and the card still shows the progress of the failed agent. A new click starts a new agent. An agent that stopped, or a progress that cannot be loaded, keeps the "Started" state.
- Under the agent id, the card shows the progress of the agent: the status, each tool that the agent starts, and each message of the agent. The page loads the progress each 3 seconds while the agent works.
- The status is one of these: the agent is working, the agent finished, the agent failed, or the agent stopped before it finished. When the status is not "working", the page does not load the progress again.
- An error text of the agent shows as a step of the progress.
- If the progress cannot be loaded, the card shows the error. A reload tries again.
- If the start fails, the page shows the error, and the user can click Resolve again.
- The server starts an agent only for an item that is in `spec/open-behaviour.json` of a project in `conf/projects.json`. A different item returns 400.

## Follow-up

- Under the progress, the card shows a follow-up input and a Send button. They show only when the started agent finished or stopped before it finished.
- Send or Cmd+Enter sends the text to the same agent session. The Send button is disabled when the text is empty or while the text is sent.
- The follow-up goes to the current agent: the Grok agent when it started, or else the first agent.
- The server starts the local agent again with the same model and resumes its last session.
- The progress shows the follow-up text as a step. Then the status is "the agent is working" again, and the page loads the progress each 3 seconds until the new run ends.
- A failed follow-up run does not start the Grok agent. The status is "the agent failed", and the page removes the started resolve.
- When the agent still works, the server does not send the follow-up and returns 400.
- If the send fails, the page shows the error and keeps the text in the input.

## Persist

- The page loads the tabs with `GET /open-behaviour-items`. Refer to `spec/app/01_server/business-logic.md`.
- Each save writes all final decisions with `PUT /page`, as `open-behaviour-decisions`.
- The key of a decision is `<tab id>/<spec>/<item id>`. `<spec>` is the `spec` field of the entry, as `<package>/NN_name`. The data is `{"<key>": "<decision>"}`.
- When the page opens, it loads `open-behaviour-decisions` with `GET /page`.
- Each successful resolve writes all started resolves with `PUT /page`, as `open-behaviour-resolves`.
- The key of a resolve is the same as the key of a decision. The data is `{"<key>": "<chat id>"}`.
- When the page opens, it loads `open-behaviour-resolves` with `GET /page`.
- The page loads the progress of a started resolve with `GET /open-behaviour-progress`. The progress is not in the DB.
- When the page opens, it removes each decision and each resolve whose item is not in the file of its project. A new item that uses the id of a resolved item thus does not get the old decision or the old resolve. The page writes the clean data with `PUT /page`. The page does this only for a project whose file was read with no error. The entries of a project with an error stay.
- If the save of the clean data fails, the page shows the error and shows the clean data.
- A resolve is saved only after the start is successful. The page does not save the value `starting`.
- When an agent fails, the page writes the started resolves without that item with `PUT /page`.
- If a save of the resolves fails, the page shows the error and keeps the last saved resolves.
- If the file of a project cannot be read or parsed, the tab of that project shows the error. The other tabs still show their items.
- If a save fails, the page shows the error and keeps the last saved decisions.
- If the load fails, the page shows the error and shows no card.
