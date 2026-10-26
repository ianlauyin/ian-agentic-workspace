import { describe, expect, it } from "vitest";

import type { Api } from "../../../src/context";
import { createTodo, type TodoItem } from "../../../src/module/todo/context";

function fakeApi(saved: { items: TodoItem[] } | null) {
  const saves: unknown[] = [];
  let failSave = false;
  const api: Api = {
    loadPage: async <T>() => saved as T | null,
    savePage: async (name, data) => {
      if (failSave) throw new Error("db down");
      saves.push({ name, data });
    },
  };
  return { api, saves, failSaves: () => (failSave = true) };
}

const texts = (todo: ReturnType<typeof createTodo>) => todo.items().map((item) => item.text);

describe("createTodo", () => {
  it("loads the saved list", async () => {
    const todo = createTodo(fakeApi({ items: [{ text: "a" }] }).api);
    await todo.load();
    expect(todo.loaded()).toBe(true);
    expect(texts(todo)).toEqual(["a"]);
  });

  it("loads an empty list when there is no save", async () => {
    const todo = createTodo(fakeApi(null).api);
    await todo.load();
    expect(todo.items()).toEqual([]);
  });

  it("shows the error when the load fails", async () => {
    const api: Api = { loadPage: async () => Promise.reject(new Error("db down")), savePage: async () => {} };
    const todo = createTodo(api);
    await todo.load();
    expect(todo.loaded()).toBe(false);
    expect(todo.error()).toBe("Failed to load my TODOs: db down");
  });

  it("saves the full list as todo-items on add", async () => {
    const fake = fakeApi({ items: [{ text: "a" }] });
    const todo = createTodo(fake.api);
    await todo.load();
    await todo.add("  b  ");
    expect(fake.saves).toEqual([{ name: "todo-items", data: { items: [{ text: "a" }, { text: "b" }] } }]);
    expect(texts(todo)).toEqual(["a", "b"]);
  });

  it("does not add or change a TODO with an empty text", async () => {
    const fake = fakeApi({ items: [{ text: "a" }] });
    const todo = createTodo(fake.api);
    await todo.load();
    await todo.add("   ");
    await todo.edit(0, "");
    expect(fake.saves).toEqual([]);
    expect(texts(todo)).toEqual(["a"]);
  });

  it("edits and removes a TODO", async () => {
    const todo = createTodo(fakeApi({ items: [{ text: "a" }, { text: "b" }] }).api);
    await todo.load();
    await todo.edit(1, "c");
    expect(texts(todo)).toEqual(["a", "c"]);
    await todo.remove(0);
    expect(texts(todo)).toEqual(["c"]);
  });

  it("moves a TODO to a gap", async () => {
    const fake = fakeApi({ items: [{ text: "a" }, { text: "b" }, { text: "c" }] });
    const todo = createTodo(fake.api);
    await todo.load();
    await todo.move(0, 3);
    expect(texts(todo)).toEqual(["b", "c", "a"]);
    await todo.move(2, 0);
    expect(texts(todo)).toEqual(["a", "b", "c"]);
    await todo.move(1, 1);
    await todo.move(1, 2);
    expect(fake.saves).toHaveLength(2);
  });

  it("keeps the last saved list when a save fails", async () => {
    const fake = fakeApi({ items: [{ text: "a" }] });
    const todo = createTodo(fake.api);
    await todo.load();
    fake.failSaves();
    await todo.add("b");
    expect(texts(todo)).toEqual(["a"]);
    expect(todo.error()).toBe("Failed to save: db down");
  });
});
