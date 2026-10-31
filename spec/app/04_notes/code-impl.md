# Notes page implementation

## Layout

- `app/src/notes.rs` holds `GET /projects` and `POST /notes-assign`. `GET /projects` gives the id and the name of each project in `conf/projects.json`. `POST /notes-assign` takes `tab` and `text`. It finds the project path with `project_root`, makes the prompt with `task_prompt`, and calls `start_agent` of `app/src/open_behaviour.rs`. It returns `{"chat": "<chat id>"}`. An empty text or an unknown project returns 400.
- `app/web/notes/context.js` holds the projects, the notes, the assigned tasks, the progress, the load, the add, the delete, the assign, the follow-up, and the progress load. It has no DOM code. `state.assigns` maps `<tab id>/<note id>` to `"starting"` or to the chat id. `"starting"` is not in the DB. `refreshProgress` and `followup` work as in `app/web/open-behaviour/context.js`.
- `app/web/notes/index.js` builds the page with the DOM API. It builds the cards again only when the selected tab or its note ids change. Thus, the follow-up text stays.
- `app/web/notes/index.css` holds the styles of the page.
- The tests are in `app/src/notes.rs` and `app/web/test/notes/context.test.js`.

## Persist

- The notes are the page data `notes` in table `page`. The assigned tasks are the page data `notes-assigns`. Refer to `spec/app/01_server/code-impl.md`.
- The progress is not in the DB.

## Do not

- Do not write a project name in `app/web/`.
- Do not change the id of a note. The saved task uses it in its key.
