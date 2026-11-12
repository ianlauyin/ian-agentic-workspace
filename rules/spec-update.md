# Spec update

When you change the code, update the spec for that code in the same change. A spec that is not up to date is a bug.

The spec for a feature is the file `spec/<package>/NN_kebab-name.md`. Update that file when the change affects user-visible behaviour, a domain rule, or the wire contract.

If the code does not obey a rule in `spec/<package>/NN_kebab-name.md`, and the task did not tell you to change that rule, stop and ask. The code can be incorrect.

In the "Spec files" section of the report, write the list of the spec files that you updated. If you did not update a spec, write the reason.

Do not update a spec only for these changes:

- Formatting
- Lint fixes
- Comments
- Dependency version updates
