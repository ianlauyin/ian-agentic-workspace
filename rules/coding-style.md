# Coding style

## General

- After a refactor, fix all the warnings.
- Readability is more important than strict DRY. Do not add a helper, a generic type or an abstraction only to remove duplication.
- Write a comment only when the code cannot show the reason.

## Backend (Rust + Axum)

- Use JSON for the request and the response unless the route needs special handling.
- Do not write doc comments.

### Files

- In a source file, put the imports and the constants first. Then put each public type, its inherent `impl` block with the public methods first, and its trait impls (`Debug`, `Display`).
- In a file two or more levels deep, use `crate::` paths. Do not use `super::super::`.
- In a library crate, make the public path of an item follow its module folders. Each folder in `src/` is a public module. Each folder directly in that folder is also a public module. Export each item from the deepest of these public modules that contains its file. Do not export the item again from a parent module.
- To use an item of `ian_framework`, import the public module that exports the item. Then use the module name in the code to refer to the item. Do not import the item directly. Do not write the full `ian_framework::` path in the code.
- If a module of the project has the same name as the `ian_framework` module, import the functions and the types of the project module directly. Do not import the project module. Do not give an alias to the `ian_framework` module.
- Keep a helper, a constant or a type in the file that uses it.
- In a library crate, put a public helper in the public module of its domain. Put a time or time zone helper in the `time` module. Do not put the helper in the module of the first item that uses it.
- Put the tests in one `#[cfg(test)] mod tests` at the end of the source file that holds the tested item. Keep each test helper in that test module. Do not share test helpers between test modules.
- Keep all test-only code in a test module. This includes test constructors, test-only methods and test doubles. Do not add a `#[cfg(test)]` item to the production code. An exception is a `#[cfg(test)]` enum variant and its match arms, because they cannot go in a different module.
- If the tests of two or more files use a test-only item, put the item in the `#[cfg(test)] mod tests` of the parent module. The other test modules can use it.

### Safety

- Do not show a secret, a token or a query string in `Debug` output or in an error message. For a type that holds a secret, write its `Debug` impl by hand.
- Do not use `unwrap`, `expect` or `panic!` outside tests.

## Frontend (SolidJS + TypeScript)

### Modules

- Put UI primitives and controls that have no domain logic in `src/component/**`.
- A module is one feature. It has its own state, its own actions and its own entry component.
- Put each module in `src/module/{module}`. To group the modules of one business area, you can use `src/module/{area}/{module}`.
- Give each module an id selector.
- The global context is in `src/context.tsx`.
- Each module has one context in `context.ts` in the module folder. Put the actions that are not UI logic in this context.
- Put all tests in a `test/` folder next to `src/`, with the same folder paths as in `src/`. Do not put tests in `src/`.

### Components

- If only one component uses a sub-component, put the sub-component in the folder of that component.
- Put the constants before the component.
- Use the file `Xxx.tsx`. If the component has styles, use the files `Xxx/index.tsx` and `Xxx/index.css`.
- Put the images for the component in an `asset/` folder next to the component.

### Styles

- Each `index.css` file has only one root selector.
- Nest each child selector inside the block of its parent.
- Give the outermost tag of each module component and sub-component a class name. Use this format: `{module}-{component}-{sub-component}...`.
- Give the outermost tag of each component in `src/component` a class name with the `g-` prefix. Use the `g-` prefix also for the modifier classes on that tag.
- If you override the internal parts of a different component, put the `g-` class of that component in the selector.
