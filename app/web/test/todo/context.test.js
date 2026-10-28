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

const texts = (todo, section) => todo.state.lists[section].map((item) => item.text);

describe("createTodo", () => {
  it("loads the saved lists", async () => {
    const todo = createTodo(fakeApi({ important: [{ text: "a" }], pending: [{ text: "b" }] }).api);
    await todo.load();
    assert.equal(todo.state.loaded, true);
    assert.deepEqual(texts(todo, "important"), ["a"]);
    assert.deepEqual(texts(todo, "pending"), ["b"]);
  });

  it("loads the old list as pending", async () => {
    const todo = createTodo(fakeApi({ items: [{ text: "a" }] }).api);
    await todo.load();
    assert.deepEqual(texts(todo, "important"), []);
    assert.deepEqual(texts(todo, "pending"), ["a"]);
  });

  it("loads empty lists when there is no save", async () => {
    const todo = createTodo(fakeApi(null).api);
    await todo.load();
    assert.deepEqual(todo.state.lists, { important: [], pending: [] });
  });

  it("shows the error when the load fails", async () => {
    const api = { loadPage: async () => Promise.reject(new Error("db down")), savePage: async () => {} };
    const todo = createTodo(api);
    await todo.load();
    assert.equal(todo.state.loaded, false);
    assert.equal(todo.state.error, "Failed to load my TODOs: db down");
  });

  it("saves the full lists as todo-items on add", async () => {
    const fake = fakeApi({ important: [], pending: [{ text: "a" }] });
    const todo = createTodo(fake.api);
    await todo.load();
    await todo.add("important", "  b  ");
    assert.deepEqual(fake.saves, [
      { name: "todo-items", data: { important: [{ text: "b" }], pending: [{ text: "a" }] } },
    ]);
  });

  it("does not add or change a TODO with an empty text", async () => {
    const fake = fakeApi({ important: [{ text: "a" }], pending: [] });
    const todo = createTodo(fake.api);
    await todo.load();
    await todo.add("pending", "   ");
    await todo.edit("important", 0, "");
    assert.deepEqual(fake.saves, []);
    assert.deepEqual(texts(todo, "important"), ["a"]);
  });

  it("edits and removes a TODO", async () => {
    const todo = createTodo(fakeApi({ important: [], pending: [{ text: "a" }, { text: "b" }] }).api);
    await todo.load();
    await todo.edit("pending", 1, "c");
    assert.deepEqual(texts(todo, "pending"), ["a", "c"]);
    await todo.remove("pending", 0);
    assert.deepEqual(texts(todo, "pending"), ["c"]);
  });

  it("moves a TODO to a gap in the same section", async () => {
    const fake = fakeApi({ important: [], pending: [{ text: "a" }, { text: "b" }, { text: "c" }] });
    const todo = createTodo(fake.api);
    await todo.load();
    await todo.move("pending", 0, "pending", 3);
    assert.deepEqual(texts(todo, "pending"), ["b", "c", "a"]);
    await todo.move("pending", 2, "pending", 0);
    assert.deepEqual(texts(todo, "pending"), ["a", "b", "c"]);
    await todo.move("pending", 1, "pending", 1);
    await todo.move("pending", 1, "pending", 2);
    assert.equal(fake.saves.length, 2);
  });

  it("moves a TODO to a gap in the other section", async () => {
    const todo = createTodo(fakeApi({ important: [{ text: "a" }], pending: [{ text: "b" }, { text: "c" }] }).api);
    await todo.load();
    await todo.move("pending", 1, "important", 0);
    assert.deepEqual(texts(todo, "important"), ["c", "a"]);
    assert.deepEqual(texts(todo, "pending"), ["b"]);
    await todo.move("important", 0, "pending", 1);
    assert.deepEqual(texts(todo, "important"), ["a"]);
    assert.deepEqual(texts(todo, "pending"), ["b", "c"]);
  });

  it("keeps the last saved lists when a save fails", async () => {
    const fake = fakeApi({ important: [], pending: [{ text: "a" }] });
    const todo = createTodo(fake.api);
    await todo.load();
    fake.failSaves();
    await todo.add("pending", "b");
    assert.deepEqual(texts(todo, "pending"), ["a"]);
    assert.equal(todo.state.error, "Failed to save: db down");
  });

  it("calls onChange after each change", async () => {
    let changes = 0;
    const todo = createTodo(fakeApi(null).api, () => changes++);
    await todo.load();
    await todo.add("important", "a");
    assert.equal(changes, 2);
  });
});
