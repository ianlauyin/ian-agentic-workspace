# Env file

The values in a `.env` file are secrets. Do not read the values. You can use the values in a command.

## Do not read the values

- Do not read a `.env` file or a `.env.*` file with a file tool, `cat`, `head`, `tail`, `less`, `grep`, `sed`, `awk` or a different command that shows its content.
- Do not show the value of an env variable. Do not use `echo "$VAR"`, `printenv`, `env`, `set` or `export -p`.
- Do not write a value from a `.env` file in a file, a log, a commit, a doc or a message.
- You can read a `.env.example` file. It has no real values.

## Use the values

- To use the variables in a command, load the file in a subshell, and do not show the output of the load: `(set -a; source .env >/dev/null 2>&1; set +a; <command>)`.
- If the app loads the `.env` file itself, run the app. Do not load the file again.
- To find the names of the variables, run `grep -oE '^[[:space:]]*(export[[:space:]]+)?[A-Za-z_][A-Za-z0-9_]*=' .env | sed -E 's/^[[:space:]]*(export[[:space:]]+)?//; s/=$//'`. This command shows only the names. Do not use a command that shows a full line. A line that has no `=` can contain a value.
- To find if a variable has a value, run `(set -a; source .env >/dev/null 2>&1; set +a; [ -n "$VAR" ] && echo set || echo empty)`.
- If a command fails, and the error message can contain a value, do not show the full output. Show only the lines that you need, and remove the values.

## Config files

- In a config file in `conf/`, write a value from `.env` as `env:NAME`. `NAME` is the variable name in the `.env` file at the workspace root. Do not write the value. Do not add a different field for the variable name or for the env file.
- To use an `env:NAME` value, load `.env` as "Use the values" tells. Then use `$NAME` in the command.

## Change a value

- Do not change a value in a `.env` file yourself. Tell the user the variable name. The user writes the value.
- You can add a new variable name with an empty value, or change a `.env.example` file.
