# Spec update

When you change the code, update the spec for that code in the same change. A spec that is not up to date is a bug.

Update `code-impl.md` when the change touches an item that `code-impl.md` describes.

Update `business-logic.md` when the change affects user-visible behaviour, a domain rule, or the wire contract.

If the code does not obey a rule in `business-logic.md`, and the task did not tell you to change that rule, stop and ask. The code can be incorrect.

If no spec describes the changed flow, write this in the report. Do not make a new spec unless the user tells you to. To make a new spec, read `spec/writing-specs.md`.

In the report, write the list of the spec files that you updated. If you did not update a spec, write the reason.

Do not update a spec only for these changes:

- Formatting
- Lint fixes
- Comments
- Dependency version updates
