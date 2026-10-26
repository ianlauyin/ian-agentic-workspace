# Spec

A spec gives the important design choices, the behaviour, and the requirements. The code gives the implementation details. A spec refers to the source code. A spec does not repeat the source code.

Each spec has one folder. The spec tree has the same structure as the package paths.

- Put a spec in the app where the user sees the behaviour. Do not put the spec where the code is.
- If a flow occurs in two apps, write one spec for each app.
- Use the same `NN_kebab-name/` folder name in the two apps.
- Each spec gives only the data for its app.
- Each spec refers to the spec for the other app.

Each spec has two files:

| File                | Contents                                                                                       | When to read                              |
| ------------------- | ---------------------------------------------------------------------------------------------- | ----------------------------------------- |
| `business-logic.md` | What the feature does. The rules that the feature must obey. The user-visible behaviour and the contract. | Before you decide if a change is permitted |
| `code-impl.md`      | How the current code does the feature: Prototype, Layout, Persist, Do not                      | Before you make the change                |

- After a rewrite of the feature, `business-logic.md` is still correct.
- After a rewrite of the feature, `code-impl.md` is not correct.
- If only one package uses a term, define the term in `spec/<package>/00_context.md` of that package.
- Other specs refer to the file that defines the term.

## Specs

Each package folder has a `spec/<package>/00_context.md` file. This file gives the Vocab of the package. Then it gives the list of the specs in the package.

Before your first edit in a package, read `spec/<package>/00_context.md` of that package. Do this for each package that the change has an effect on.
