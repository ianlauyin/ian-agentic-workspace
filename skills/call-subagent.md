# Call subagent

- To work in a different repo, call a subagent with the `cursor-agent` CLI.
- The subagent works on the default branch of that repo.
- The repos are in `conf/projects.json`.

## Start a subagent

- Run `cursor-agent create-chat` one time for each subagent. Keep the chat id that it prints.
- Run `cursor-agent --print --force --trust --model grok-4.7-high --workspace "$REPO_PATH" --resume "$CHAT_ID" "$PROMPT"`. `$REPO_PATH` is the path of the target repo.
- Run the command in the background, and wait until it stops. Its output is the report of the subagent.
- To run two or more subagents at the same time, give each subagent its own chat id.

## Task prompt

The subagent does not start in this workspace. Thus, in each task prompt, write these items:

- The absolute path of this workspace.
- An instruction to read the "Rules" table in `AGENTS.md` of this workspace, and to obey each rule file at the given time.
- An instruction to obey the instructions of the target repo when they are different from the rules of this workspace.
- An instruction to write the report as `rules/report.md` tells.
- An instruction to stop and write the open questions at the end of the report when the subagent cannot continue without an answer.

## Continue a subagent

- To answer a question or to give a fix after the code review, run the start command again with the same chat id and a new prompt.
- The subagent keeps the history of its chat.
