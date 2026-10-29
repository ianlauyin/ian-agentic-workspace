# Open Behaviour page implementation

## Layout

- `app/src/open_behaviour.rs` holds `GET /open-behaviour-items`. It reads `conf/projects.json` and changes a leading `~/` of each project path to the home folder. For each project, it gives one tab with `id`, `name`, `groups`, and `errors`. Each group is one entry of `spec/open-behaviour.json` of the project, with no change.
- `app/src/open_behaviour.rs` also holds `POST /open-behaviour-resolve`. The body has `tab`, `spec`, `id`, and `decision`. `find_item` finds the item in `spec/open-behaviour.json` of the project. The route starts `cursor-agent -p --model claude-opus-5-5-medium --force --trust` in the workspace root, with the item JSON and the decision in the prompt. The agent is in its own process group, because `cargo watch` stops the process group of the server on a restart. The stdout and the stderr of the agent go to `app/data/resolve/<unix millis>.log`.
- `app/web/open-behaviour/context.js` holds the tabs, the saved decisions, the started resolves, the load, the save, and the resolve. It has no DOM code. `state.resolves` maps a decision key to `"starting"` or to the log path. It is not in the DB. After each change, it calls the `onChange` function of the page. `decisionKey` gives the key of an item. `groupsByStatus` splits the groups of one tab into open items and decided items.
- `app/web/open-behaviour/index.js` builds the page with the DOM API. It builds the tabs and the cards one time after the load. On each render, it moves each decided card to the bottom of its tab. Thus, an input keeps its text. Text between backticks shows as code.
- `app/web/open-behaviour/index.css` holds the styles of the page, the tabs, the groups, and the cards.
- The tests are in `app/src/open_behaviour.rs` and `app/web/test/open-behaviour/context.test.js`.

## Persist

- The decisions are the page data `open-behaviour-decisions` in table `page`. Refer to `spec/app/01_server/code-impl.md`.
- The decisions change only after a save is successful.
- The items are not in the DB.

## Do not

- Do not write a project name or an item of a project in `app/web/`. The page reads those values from the projects.
- Do not change the `id` of a tab, the `spec` path of an entry, or the `id` of an item. The saved decisions use them in their key.
