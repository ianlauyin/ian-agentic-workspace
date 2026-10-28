# Fetch logs

Use this skill to get the log events of a project from the log server.

## Find the app name

- Read `conf/projects.json`. A project that has `"log": true` sends its log events to the log server.
- Read `conf/log-server.json`. The `apps` map gives the log server app name for each project.
- If the project is not in the `apps` map, tell the user. Do not use a different app name.

## Get the events

- Get the `base_url`, `route` and `query` values from `conf/log-server.json`.
- The `api_key` value is `env:NAME`. Load `.env` in a subshell as `rules/env-file.md` tells.
- Run `curl` in the same subshell, from the workspace root.
- Send the key in the `Authorization: Bearer` header.
- Make the URL from `base_url`, `route`, the app name and the query values.
- The `date` value is a date in HKT. If you do not give a date, the server uses the current day in HKT.
- To get the events of an earlier day, give the date.
- The response is `{"events": [...]}`. If the list is empty, the app has no events on that date.
- To select or change the events in the output, use `jq`.

## Errors

| Status | Cause | Action |
| --- | --- | --- |
| 400 | A query value is not correct. | Make sure that the date has the correct format. |
| 401 | The key is not correct, or it has no value. | Tell the user the variable name. Do not show the key. |
| 403 | The key cannot read that app, or the app name is not correct. | Read the `apps` map again. |

Do not show a response header or a command output that can contain the key.
