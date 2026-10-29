# Ian's Personal Agentic Workspace

This repo is a workspace for a primary agent. The primary agent gives the work to subagents. Each subagent does its work in a different repo. All agents must obey the rules in this workspace.

## Roles

| Agent | Work | Rules |
| --- | --- | --- |
| Primary agent | Splits the task, finds the target repo, calls a subagent for each part, does a code review of the changes and reports to the user | The rules in `AGENTS.md` and `rules/` |
| Subagent | Does one part of the task in one target repo | The rules in `rules/` of this workspace, and the instructions of the target repo |

- The primary agent does not edit the code of a different repo. It calls a subagent.
- The primary agent calls each subagent with Grok 4.7 and high effort.
- After a subagent changes the code, the primary agent does a code review of the changes in the target repo. The review must find bugs, and code that does not obey the rules in `rules/`.
- If the review finds a problem, the primary agent does not fix the code. It calls the subagent again to fix the problem. Then it does the review again.
- If a rule in this workspace and an instruction of the target repo are different, obey the rule in the target repo.

## Update the workspace

If the user tells you an error or a preference, update the applicable file in this workspace.

- Write the change for all tasks, not only the current task.
- Before you change a file, read it. Do not write the same instruction two times.
- If the new instruction and an old instruction are different, replace the old instruction.
- If the error is in a different repo, call the subagent to change the code.

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
