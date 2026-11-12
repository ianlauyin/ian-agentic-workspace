# Notes page

## Notes

- The page shows one tab for each project in `conf/projects.json`, in the order of that file. The tab name is the project name.
- Each tab shows a badge with the number of notes of that project. A tab with no note shows no badge.
- When the page opens, it selects the first tab.
- Under the tabs, the page shows an input for a new note and an Add button. The Add button is disabled when the text is empty.
- Add or Cmd+Enter saves the note at the end of the list of the selected tab. The input clears after a successful save.
- Each note is one card. The card shows the note text, an "Assign task" button, an Edit button, and a Delete button.
- A tab with no notes shows "No note."
- A note is plain text.
- Edit replaces the note text on the card with a text input that holds the note text, a Save button and a Cancel button. Save or Cmd+Enter saves the new text in the same place in the list. Cancel closes the input and keeps the text.
- Save trims the text and does nothing when the text is empty. The note keeps its id and its assigned task. A task that is already started keeps the old text.
- Delete removes the note and its assigned task.

## Assign task

- The Assign task button starts one local Cursor agent in the local project folder of the tab. It starts no cloud agent. The agent calls no subagent. The model is Claude Sonnet 5.5 with medium effort.
- The prompt gives the project, the project path, and the note text as the task, and tells the agent to obey `AGENTS.md`.
- When the first agent fails, the server starts a local Cursor agent with Grok 4.7 and high effort, in the same folder with the same prompt. The progress shows "The local agent failed. A local Grok agent takes over now."
- An agent fails when it gives a failed result, or when it exits with an error code before a result. When the first agent cannot start, the server starts the Grok agent. When the Grok agent also cannot start, the start fails.
- The card keeps the chat id that the server gives at the start. The progress shows the progress of the first agent and then of the Grok agent. An agent that stopped or finished does not start the Grok agent.
- The agent runs without a person. It can edit files and run commands without approval.
- The agent continues when the page closes or the server restarts.
- After the start, the button shows "Assigned" and stays disabled. The card shows the chat id and tells that the local agent works in the project folder. A reload keeps this state.
- When the progress gives the status "the agent failed", the page removes the assigned task. This occurs only when the Grok agent also fails, or when the Grok agent cannot start. The button shows "Assign task" again, and the card still shows the progress of the failed agent. A new click starts a new agent. An agent that stopped, or a progress that cannot be loaded, keeps the "Assigned" state.
- Under the agent id, the card shows the progress of the agent: the status, each tool that the agent starts, and each message of the agent. The page loads the progress each 3 seconds while the agent works.
- The status is one of these: the agent is working, the agent finished, the agent failed, or the agent stopped before it finished. When the status is not "working", the page does not load the progress again.
- An error text of the agent shows as a step of the progress.
- If the progress cannot be loaded, the card shows the error. A reload tries again.
- The note stays after the agent finishes. The user deletes it.
- The server starts an agent only for a project in `conf/projects.json` and a note text that is not empty. A different request returns 400.
- If the start fails, the page shows the error, and the user can click the button again.

## Follow-up

- Under the progress, the card shows a follow-up input and a Send button. They show only when the started agent finished or stopped before it finished.
- Send or Cmd+Enter sends the text to the same agent session. The Send button is disabled when the text is empty or while the text is sent.
- The follow-up goes to the current agent: the Grok agent when it started, or else the first agent.
- The server starts the local agent again with the same model and resumes its last session.
- The progress shows the follow-up text as a step. Then the status is "the agent is working" again, and the page loads the progress each 3 seconds until the new run ends.
- A failed follow-up run does not start the Grok agent. The status is "the agent failed", and the page removes the assigned task.
- When the agent still works, the server does not send the follow-up and returns 400.
- If the send fails, the page shows the error and keeps the text in the input.

## Persist

- The page loads the tabs with `GET /projects`.
- The notes are the page data `notes`, as `{"<tab id>": [{"id": "<note id>", "text": "<text>"}]}`. The page changes the notes only after a save is successful.
- The assigned tasks are the page data `notes-assigns`, as `{"<tab id>/<note id>": "<chat id>"}`. A task is saved only after the start is successful.
- The progress and the follow-up use `GET /open-behaviour-progress` and `POST /open-behaviour-followup`.
- If a load or a save fails, the page shows the error and keeps the last saved data.
