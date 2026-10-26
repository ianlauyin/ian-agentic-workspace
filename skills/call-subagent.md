# Call subagent

- To work in a different repo, call a subagent through Orca. Follow `orca skills get orchestration`.
- The subagent works on the default branch of that repo.
- The repos are in `conf/projects.json`.

## Task prompt

The subagent does not start in this workspace. Thus, in each task prompt, write these items:

- The absolute path of this workspace.
- An instruction to read the "Rules" table in `AGENTS.md` of this workspace, and to obey each rule file at the given time.
- An instruction to obey the instructions of the target repo when they are different from the rules of this workspace.
- An instruction to write the report as `rules/spec-update.md` tells.

## Worker messages

- To read, answer and ack the messages of a worker, use `scripts/orca-inbox.sh`. Do not parse the output of `orca orchestration check` with `grep`.
- Run `scripts/orca-inbox.sh read <run>` to wait for messages. Then run `scripts/orca-inbox.sh reply <run> <msg_id> <body>` for each open question. Then run `scripts/orca-inbox.sh ack <run>`. The ack stops if a question has no reply.
- Use one run for each coordinator terminal. A new `orca orchestration run-create` binds the terminal to the new run, and Orca then refuses a `check` for the older run. To go back to an older run, use `orca orchestration run-use --id <run>`.
