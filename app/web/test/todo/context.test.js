import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { createTodo } from "../../todo/context.js";

function fakeApi(saved) {
  const saves = [];
  let failSave = false;
  const api = {
    loadPage: async () => saved,
    savePage: async (name, data) => {
      if (failSave) throw new Error("db down");
      saves.push({ name, data });
    },
  };
  return { api, saves, failSaves: () => (failSave = true) };
}

const texts = (todo) => todo.state.items.map((item) => item.text);

describe("createTodo", () => {
  it("loads the saved list", async () => {
    const todo = createTodo(fakeApi({ items: [{ text: "a" }] }).api);
    await todo.load();
    assert.equal(todo.state.loaded, true);
    assert.deepEqual(texts(todo), ["a"]);
  });

  it("loads an empty list when there is no save", async () => {
    const todo = createTodo(fakeApi(null).api);
    await todo.load();
    assert.deepEqual(todo.state.items, []);
  });

  it("shows the error when the load fails", async () => {
    const api = { loadPage: async () => Promise.reject(new Error("db down")), savePage: async () => {} };
    const todo = createTodo(api);
    await todo.load();
    assert.equal(todo.state.loaded, false);
    assert.equal(todo.state.error, "Failed to load my TODOs: db down");
  });

  it("saves the full list as todo-items on add", async () => {
    const fake = fakeApi({ items: [{ text: "a" }] });
    const todo = createTodo(fake.api);
    await todo.load();
    await todo.add("  b  ");
    assert.deepEqual(fake.saves, [{ name: "todo-items", data: { items: [{ text: "a" }, { text: "b" }] } }]);
    assert.deepEqual(texts(todo), ["a", "b"]);
  });

  it("does not add or change a TODO with an empty text", async () => {
    const fake = fakeApi({ items: [{ text: "a" }] });
    const todo = createTodo(fake.api);
    await todo.load();
    await todo.add("   ");
    await todo.edit(0, "");
    assert.deepEqual(fake.saves, []);
    assert.deepEqual(texts(todo), ["a"]);
  });

  it("edits and removes a TODO", async () => {
    const todo = createTodo(fakeApi({ items: [{ text: "a" }, { text: "b" }] }).api);
    await todo.load();
    await todo.edit(1, "c");
    assert.deepEqual(texts(todo), ["a", "c"]);
    await todo.remove(0);
    assert.deepEqual(texts(todo), ["c"]);
  });

  it("moves a TODO to a gap", async () => {
    const fake = fakeApi({ items: [{ text: "a" }, { text: "b" }, { text: "c" }] });
    const todo = createTodo(fake.api);
    await todo.load();
    await todo.move(0, 3);
    assert.deepEqual(texts(todo), ["b", "c", "a"]);
    await todo.move(2, 0);
    assert.deepEqual(texts(todo), ["a", "b", "c"]);
    await todo.move(1, 1);
    await todo.move(1, 2);
    assert.equal(fake.saves.length, 2);
  });

  it("keeps the last saved list when a save fails", async () => {
    const fake = fakeApi({ items: [{ text: "a" }] });
    const todo = createTodo(fake.api);
    await todo.load();
    fake.failSaves();
    await todo.add("b");
    assert.deepEqual(texts(todo), ["a"]);
    assert.equal(todo.state.error, "Failed to save: db down");
  });

  it("calls onChange after each change", async () => {
    let changes = 0;
    const todo = createTodo(fakeApi({ items: [] }).api, () => changes++);
    await todo.load();
    await todo.add("a");
    assert.equal(changes, 2);
  });
});
