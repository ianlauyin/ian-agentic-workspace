# Ian's Personal Agentic Workspace

Before you look up business logic, read `spec/catalog.md` for the specification index.

## Work in a different repo

- To edit the code of a different repo, call a subagent that works in that repo. Do not edit the code yourself.
- After the subagent changes the code, do a code review of the changes in the target repo. The review must find bugs, and code that does not obey the instructions of the target repo.
- If the review finds a problem, do not fix the code yourself. Call the subagent again to fix the problem. Then do the review again.

## Update the workspace

If the user tells you an error or a preference, update the applicable file in this workspace.

- Write the change for all tasks, not only the current task.
- Before you change a file, read it. Do not write the same instruction two times.
- If the new instruction and an old instruction are different, replace the old instruction.
- If the error is in a different repo, call the subagent to change the code in that repo.

## Rules

| File | When to read |
| --- | --- |
| `rules/coding-style.md` | Before the first edit |
| `rules/debugging.md` | Before you fix a bug or find the cause of an error |
| `rules/env-file.md` | Before you read a file, run a command or use an env variable |
| `rules/deployment.md` | Before you write or change deploy files, give deploy commands, or need a `gcloud` command |
| `rules/git-convention.md` | Before you make a git commit |
| `rules/report.md` | Before you write a report |
| `rules/spec-update.md` | After you change the code |
| `rules/writing.md` | Before you write or update a markdown file |

## Skills

| File | When to read |
| --- | --- |
| `skills/fetch-logs.md` | Before you get the log events of a project |

## App

The workspace has a local server app in `app/`. Before you change it, read `spec/app/00_context.md`.
