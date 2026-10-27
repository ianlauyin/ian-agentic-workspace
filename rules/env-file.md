# Env file

The values in a `.env` file are secrets. Do not read the values. You can use the values in a command.

## Do not read the values

- Do not read a `.env` file or a `.env.*` file with a file tool, `cat`, `head`, `tail`, `less`, `grep`, `sed`, `awk` or a different command that shows its content.
- Do not show the value of an env variable. Do not use `echo "$VAR"`, `printenv`, `env`, `set` or `export -p`.
- Do not write a value from a `.env` file in a file, a log, a commit, a doc or a message.
- You can read a `.env.example` file. It has no real values.

## Use the values

- To use the variables in a command, load the file in a subshell, and do not show the output of the load: `(set -a; source .env; set +a; <command>)`.
- If the app loads the `.env` file itself, for example with `dotenvy`, run the app. Do not load the file again.
- To find the names of the variables, run `sed 's/=.*//' .env`. This command shows only the names.
- To find if a variable has a value, run `(set -a; source .env; set +a; [ -n "$VAR" ] && echo set || echo empty)`.
- If a command fails, and the error message can contain a value, do not show the full output. Show only the lines that you need, and remove the values.

## Change a value

- Do not change a value in a `.env` file yourself. Tell the user the variable name. The user writes the value.
- You can add a new variable name with an empty value, or change a `.env.example` file.
