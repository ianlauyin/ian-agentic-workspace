# Coding style

## General

- After a refactor, fix all the warnings.
- Readability is more important than strict DRY. Do not add a helper, a generic type or an abstraction only to remove duplication.
- Write a comment only when the code cannot show the reason.
- Write tests only for functions that hold important logic. For each of these functions, test each edge case. Do not test a trivial function, a getter or a pass-through. This rule applies to the Rust code and to the frontend code.

## Backend (Rust + Axum)

The backend crate is `backend/`.

- Use JSON for the request and the response. Use a different format only for a route that needs a body that is not JSON: a file upload, a stream or a webhook that has its own format.
- Do not write doc comments. This includes `///` and `//!`.
- Run `cargo fmt` and `cargo clippy` after you change the code. Fix all the warnings that they show.
- Do not write a request log event for a website static file request. This includes the SPA fallback that sends `index.html`. Write a request log event for each API request.
- This project reads its env variables and gives the values to `ian_framework`.
- When a cargo feature only adds a step to a function, write one function. Put the step in a `#[cfg(feature = "...")]` block, with an early return when the step gives the result. Do not write a second copy of the function for `#[cfg(not(feature = "..."))]`.

### Configuration

- The project reads a value that is not a secret, such as a bucket name, a list, a path, a URL or a log level, only from the process environment. Do not read that value from Secret Manager. When a value is not a secret, do not create a secret for it.

### Files

- In a source file, put the imports and the constants first. Then put each public type, its inherent `impl` block with the public methods first, and its trait impls (`Debug`, `Display`). Put the private types, the private functions and their impl blocks after the public items.
- In a file two or more levels deep, use `crate::` paths. Do not use `super::super::`.
- Do not use `self` in an import, neither in an import list nor at the start of an import path. To use a module and an item of that module, import only the module. Then use the module name to refer to the item.
- Keep a helper, a constant or a type in the file that uses it. When a different file also uses it, put it in the public module of its domain. Put a time or time zone helper in the `time` module.
- To use an item of `ian_framework`, import the public module that exports the item. Then use the module name in the code to refer to the item. Do not import the item directly. Do not write the full `ian_framework::` path in the code.
- If a module of this project has the same name as the `ian_framework` module, import the functions and the types of the project module directly. Do not import the project module. Do not give an alias to the `ian_framework` module.

### Safety

- Do not show a secret, a token or a query string in `Debug` output or in an error message. For a type that holds a secret, write its `Debug` impl by hand.
- Do not use `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!` or an index that can be out of range outside tests. Use `get` or return an error.
- Read a secret, such as a key, a token or a password, from Secret Manager with `env::required` or `env::optional`.

### Tests

- In the tests, do not use a name from a project of the user. This includes repo names, app ids, env variable names, secret names, GCP project ids, service accounts and bucket names. Use generic names: `my-app`, `my-project` and `APP_KEY`.
- Put the tests in one `#[cfg(test)] mod tests` at the end of the source file that holds the tested item.
- Keep each test helper in the test module that uses it. If the tests of two or more files use a test-only item, put the item in the `#[cfg(test)] mod tests` of the parent module. The other test modules can use it.
- Keep all test-only code in a test module. This includes test constructors, test-only methods and test doubles. Do not add a `#[cfg(test)]` item to the production code.
