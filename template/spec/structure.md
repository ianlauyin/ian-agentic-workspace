# Spec

A spec gives the important design choices, the behaviour, and the requirements. The code gives the implementation details. A spec refers to the source code. A spec does not repeat the source code.

Each spec has one folder. The spec tree has the same structure as the package paths.

- Put a spec in the app where the user sees the behaviour. Do not put the spec where the code is.
- If a flow occurs in two apps, write one spec for each app.
- Use the same `NN_kebab-name/` folder name in the two apps.
- Each spec gives only the data for its app.
- Each spec refers to the spec for the other app.

Each spec has two files:

| File                | Contents                                                                                                  | When to read                               |
| ------------------- | --------------------------------------------------------------------------------------------------------- | ------------------------------------------ |
| `business-logic.md` | What the feature does. The rules that the feature must obey. The user-visible behaviour and the contract. | Before you decide if a change is permitted |
| `code-impl.md`      | How the current code does the feature: Prototype, Layout, Persist, Do not                                 | Before you make the change                 |

- After a rewrite of the feature, `business-logic.md` is still correct.
- After a rewrite of the feature, `code-impl.md` is not correct.
- If only one package uses a term, define the term in `spec/<package>/00_context.md` of that package.
- Other specs refer to the file that defines the term.

## Open behaviour

An open item is a known behaviour that is not resolved. The repo has one file for open items: `spec/open-behaviour.json`. A package or a spec folder does not have its own file for open items. Before you change a behaviour, read the open items of its spec.

- The file has only the JSON data: a list of entries. Do not add an HTML page or other text to the file.
- Write one entry for each spec folder that has open items, in package order, then in folder order. Each entry has `spec`, `title`, `context` and `items`.

Each entry has these fields:

| Field     | Contents                                                                              |
| --------- | ------------------------------------------------------------------------------------- |
| `spec`    | The spec folder path under `spec/`, as `<package>/NN_name`.                           |
| `title`   | The title of the spec.                                                                |
| `context` | The repo, the commit and the date of the last check, and the path conventions.        |
| `items`   | The open items of the spec.                                                           |

Each item has these fields:

| Field       | Contents                                                                       |
| ----------- | ------------------------------------------------------------------------------ |
| `id`        | The row number in the spec. Do not change it. Do not use it again.             |
| `item`      | A short name.                                                                  |
| `now`       | What the code does now, with the evidence as `file:line`.                      |
| `waitingOn` | The teams that make the decision.                                              |
| `nextStep`  | The recorded decision. Change it only when a person makes a new decision.      |
| `kind`      | `bug`, `decision` or `cleanup`.                                                |
| `scope`     | The packages and services that the item has an effect on.                      |
| `options`   | Three ways to resolve the item. Put the recommended way first.                 |
| `note`      | Optional. A reference to a copy of the item in a different spec, or a date.    |

- Put an item in the entry of the spec where the user sees the behaviour.
- If an item occurs in two or more apps, put a copy in the entry of the spec of each app. In `note`, give the `spec` and the `id` of each other copy.
- `business-logic.md` and `code-impl.md` refer to their items as `` `spec/open-behaviour.json` (`<package>/NN_name`) ``.
- To resolve an item, fix the code and the spec in the same change. Then delete the item.
- If a spec entry has no items, delete the entry and its references in `business-logic.md` and `code-impl.md`.
- If no spec has items, keep `spec/open-behaviour.json` as an empty list.

## Specs

Each package folder has a `spec/<package>/00_context.md` file. This file gives the Vocab of the package. Then it gives the list of the specs in the package.

Before your first edit in a package, read `spec/<package>/00_context.md` of that package. Do this for each package that the change has an effect on.
