# Call subagent

- To work in a different repo, call a subagent through Orca. Follow `orca skills get orchestration`.
- The subagent works on the default branch of that repo.
- The repos are in `conf/projects.json`.

## Task prompt

The subagent does not start in this workspace. Thus, in each task prompt, write these items:

- The absolute path of this workspace.
- An instruction to read the "Rules" table in `AGENTS.md` of this workspace, and to obey each rule file at the given time.
- An instruction to obey the rules of this workspace when they are different from the instructions of the target repo.
- An instruction to write the report as `rules/spec-update.md` tells.
