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
- Resolve starts one Cursor cloud agent on the remote of this workspace. The model is Claude Opus 5.5 with medium effort. The server reads `CURSOR_API_KEY` from the process environment. When that value is empty, the server reads `.env` at the workspace root. An empty value is a start failure.
- The cloud agent calls one subagent. The subagent name is `target-repo`. The subagent works on the remote of the project of the item. The cloud agent does not edit that project.
- The remotes are the `origin` remote of this workspace and the `origin` remote of the project. The server sends each remote as an HTTPS URL. The start branch is the default branch of `origin`.
- When the two remotes are the same URL, the server sends that remote one time.
- The subagent resolves the item as the saved final decision tells. It obeys `AGENTS.md` in the target repository. It deletes the item from `spec/open-behaviour.json` on a new branch of the target remote. The page reads the local file, so the item stays on the page until that branch is in the local file.
- When the cloud agent fails, the server starts one local Cursor agent for the same item. The local agent works in this workspace, on the local project folder. It obeys `AGENTS.md` of this workspace. The model is Claude Opus 5.5 with medium effort. The card keeps the same agent id. The progress shows "The cloud agent failed. A local agent resolves the item now.", and then the progress of the local agent. A cloud agent that stopped or finished does not start a local agent.
- The agent runs without a person. It can edit files and run commands without approval.
- The agent continues when the page closes or the server restarts.
- After the start, the button shows "Started" and stays disabled. The card shows the cloud agent id and the agent URL. The agent is in the Cursor agents sidebar. The sidebar shows the agent when the source filter includes API. A reload keeps this state.
- When the progress gives the status "the agent failed", the page removes the started resolve. With a local agent after the cloud agent, this occurs only when the local agent also fails. The button shows "Resolve" again, and the card still shows the progress of the failed agent. A new click starts a new agent. An agent that stopped, or a progress that cannot be loaded, keeps the "Started" state.
- Under the agent id, the card shows the progress of the agent: the status, each tool that the agent starts, and each message of the agent. The page loads the progress each 3 seconds while the agent works.
- The status is one of these: the agent is working, the agent finished, the agent failed, or the agent stopped before it finished. When the status is not "working", the page does not load the progress again.
- An error text of the agent shows as a step of the progress.
- If the progress cannot be loaded, the card shows the error. A reload tries again.
- When the log has no end and no local agent started, the progress route reads the cloud run. A run that has ended adds the end to the log, or starts the local agent when the run failed. A failed read of the cloud run leaves the log as it is.
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
- The page loads the progress of a started resolve with `GET /open-behaviour-progress`. The progress is not in the DB.
- A resolve is saved only after the start is successful. The page does not save the value `starting`.
- When an agent fails, the page writes the started resolves without that item with `PUT /page`.
- If a save of the resolves fails, the page shows the error and keeps the last saved resolves.
- If the file of a project cannot be read or parsed, the tab of that project shows the error. The other tabs still show their items.
- If a save fails, the page shows the error and keeps the last saved decisions.
- If the load fails, the page shows the error and shows no card.
