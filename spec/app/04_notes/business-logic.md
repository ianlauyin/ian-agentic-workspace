# Notes page

## Notes

- The page shows one tab for each project in `conf/projects.json`, in the order of that file. The tab name is the project name.
- Each tab shows a badge with the number of notes of that project. A tab with no note shows no badge.
- When the page opens, it selects the first tab.
- Under the tabs, the page shows an input for a new note and an Add button. The Add button is disabled when the text is empty.
- Add or Cmd+Enter saves the note at the end of the list of the selected tab. The input clears after a successful save.
- Each note is one card. The card shows the note text, an "Assign task" button, and a Delete button.
- A tab with no notes shows "No note."
- A note is plain text. The note is not changed after it is saved.
- Delete removes the note and its assigned task.

## Assign task

- The Assign task button starts one local Cursor agent in the local project folder of the tab. It uses the same agents as the Resolve button of `spec/app/03_open-behaviour/business-logic.md`: the first agent, the Grok agent when the first agent fails, the progress, and the follow-up.
- The prompt gives the project, the project path, and the note text as the task, and tells the agent to obey `AGENTS.md`.
- After the start, the button shows "Assigned" and stays disabled. The card shows the chat id and the progress. A reload keeps this state.
- When the progress gives the status "the agent failed", the page removes the assigned task. The button shows "Assign task" again.
- The note stays after the agent finishes. The user deletes it.
- The server starts an agent only for a project in `conf/projects.json` and a note text that is not empty. A different request returns 400.
- If the start fails, the page shows the error, and the user can click the button again.

## Persist

- The page loads the tabs with `GET /projects`.
- The notes are the page data `notes`, as `{"<tab id>": [{"id": "<note id>", "text": "<text>"}]}`. The page changes the notes only after a save is successful.
- The assigned tasks are the page data `notes-assigns`, as `{"<tab id>/<note id>": "<chat id>"}`. A task is saved only after the start is successful.
- The progress and the follow-up use `GET /open-behaviour-progress` and `POST /open-behaviour-followup`.
- If a load or a save fails, the page shows the error and keeps the last saved data.
