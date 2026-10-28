# Report

These rules apply to each report file that an agent writes. A subagent also obeys these rules.

## File

- Write the report as an HTML file in the `report/` folder at the root of this workspace. A subagent uses the absolute path of this workspace.
- Use the name `{issue}-{datetime}.html`.
- `{issue}` is a short name of the issue in lowercase words with hyphens.
- `{datetime}` is the HKT date and time when you write the report, in the format `YYYYMMDD-HHMM`.
- Do not write a report in a different folder or in a different format.

## Content

The report must have these sections, in this sequence:

| Section | Content |
| --- | --- |
| Cause | The cause of the issue. If you do not have evidence for the cause, write that the cause is not known. |
| Evidence | The evidence for each claim in the Cause section, as `rules/debugging.md` tells. Give the source of each item. |
| Fix steps | The steps to fix the issue, in sequence. For each step, write who does it: the agent or the user. Write each command in full. |

- If you changed the code, also add a "Spec files" section as `rules/spec-update.md` tells.
- Write the text as `rules/writing.md` tells. This includes the rules for STE and for sensitive data.

## HTML

- Make one file that has all its content. Put the CSS in a `<style>` element. Do not load an external script, stylesheet, font or image.
- Give the `<title>` and the `<h1>` the issue name.
- Put each command, log line and code line in a `<pre>` element. Escape `<`, `>` and `&`.
