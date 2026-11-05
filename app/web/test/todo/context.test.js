import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { createTodo, normalize } from "../../todo/context.js";

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

const card = (title, detail = "", deadline = "") => ({ title, detail, deadline });
const titles = (todo, section) => todo.state.lists[section].map((item) => item.title);

describe("normalize", () => {
  it("returns three empty lists when there is no data", () => {
    assert.deepEqual(normalize(null), { pending: [], ongoing: [], reviewing: [] });
  });

  it("fills the missing detail and deadline", () => {
    assert.deepEqual(normalize({ ongoing: [{ title: "a" }] }).ongoing, [card("a")]);
  });

  it("loads the old lists as pending, with text as title", () => {
    const lists = normalize({ important: [{ text: "a" }], pending: [{ text: "b" }] });
    assert.deepEqual(lists.pending, [card("a"), card("b")]);
    assert.deepEqual(normalize({ items: [{ text: "c" }] }).pending, [card("c")]);
  });
});

describe("createTodo", () => {
  it("shows the error when the load fails", async () => {
    const api = { loadPage: async () => Promise.reject(new Error("db down")), savePage: async () => {} };
    const todo = createTodo(api);
    await todo.load();
    assert.equal(todo.state.loaded, false);
    assert.equal(todo.state.error, "Failed to load my TODOs: db down");
  });

  it("saves the full lists as todo-items on add", async () => {
    const fake = fakeApi({ pending: [card("a")] });
    const todo = createTodo(fake.api);
    await todo.load();
    await todo.add("ongoing", { title: "  b  ", detail: " d ", deadline: "2026-12-01" });
    assert.deepEqual(fake.saves, [
      { name: "todo-items", data: { pending: [card("a")], ongoing: [card("b", "d", "2026-12-01")], reviewing: [] } },
    ]);
  });

  it("does not add or change a TODO with an empty title", async () => {
    const fake = fakeApi({ pending: [card("a")] });
    const todo = createTodo(fake.api);
    await todo.load();
    await todo.add("pending", { title: "   ", detail: "x", deadline: "" });
    await todo.edit("pending", 0, { title: "", detail: "x", deadline: "" });
    assert.deepEqual(fake.saves, []);
    assert.deepEqual(titles(todo, "pending"), ["a"]);
  });

  it("edits the title, the detail and the deadline, and clears the deadline", async () => {
    const todo = createTodo(fakeApi({ pending: [card("a", "", "2026-12-01"), card("b")] }).api);
    await todo.load();
    await todo.edit("pending", 0, { title: " c ", detail: " d ", deadline: "" });
    assert.deepEqual(todo.state.lists.pending, [card("c", "d", ""), card("b")]);
  });

  it("removes a TODO", async () => {
    const todo = createTodo(fakeApi({ pending: [card("a"), card("b")] }).api);
    await todo.load();
    await todo.remove("pending", 0);
    assert.deepEqual(titles(todo, "pending"), ["b"]);
  });

  it("moves a TODO to a gap in the same column", async () => {
    const fake = fakeApi({ pending: [card("a"), card("b"), card("c")] });
    const todo = createTodo(fake.api);
    await todo.load();
    await todo.move("pending", 0, "pending", 3);
    assert.deepEqual(titles(todo, "pending"), ["b", "c", "a"]);
    await todo.move("pending", 2, "pending", 0);
    assert.deepEqual(titles(todo, "pending"), ["a", "b", "c"]);
    await todo.move("pending", 1, "pending", 1);
    await todo.move("pending", 1, "pending", 2);
    assert.equal(fake.saves.length, 2);
  });

  it("moves a TODO to a gap in another column and keeps its detail and deadline", async () => {
    const todo = createTodo(fakeApi({ pending: [card("a"), card("b", "d", "2026-12-01")], ongoing: [card("c")] }).api);
    await todo.load();
    await todo.move("pending", 1, "ongoing", 0);
    assert.deepEqual(todo.state.lists.ongoing, [card("b", "d", "2026-12-01"), card("c")]);
    assert.deepEqual(titles(todo, "pending"), ["a"]);
    await todo.move("ongoing", 0, "reviewing", 0);
    assert.deepEqual(titles(todo, "reviewing"), ["b"]);
  });

  it("keeps the last saved lists when a save fails", async () => {
    const fake = fakeApi({ pending: [card("a")] });
    const todo = createTodo(fake.api);
    await todo.load();
    fake.failSaves();
    await todo.add("pending", { title: "b", detail: "", deadline: "" });
    assert.deepEqual(titles(todo, "pending"), ["a"]);
    assert.equal(todo.state.error, "Failed to save: db down");
  });

  it("calls onChange after each change", async () => {
    let changes = 0;
    const todo = createTodo(fakeApi(null).api, () => changes++);
    await todo.load();
    await todo.add("pending", { title: "a", detail: "", deadline: "" });
    assert.equal(changes, 2);
  });
});
