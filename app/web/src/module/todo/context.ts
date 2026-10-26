import { createContext, createSignal, useContext } from "solid-js";

import type { Api } from "../../context";

const PAGE_NAME = "todo-items";

export type TodoItem = { text: string };

type TodoData = { items: TodoItem[] };

export function createTodo(api: Api) {
  const [items, setItems] = createSignal<TodoItem[]>([]);
  const [loaded, setLoaded] = createSignal(false);
  const [error, setError] = createSignal("");

  async function load() {
    try {
      const data = await api.loadPage<TodoData>(PAGE_NAME);
      setItems(data?.items ?? []);
      setLoaded(true);
      setError("");
    } catch (e) {
      setError(`Failed to load my TODOs: ${messageOf(e)}`);
    }
  }

  // items changes only after the save, so a failed save keeps the last saved list
  async function save(next: TodoItem[]) {
    try {
      await api.savePage(PAGE_NAME, { items: next } satisfies TodoData);
      setItems(next);
      setError("");
    } catch (e) {
      setError(`Failed to save: ${messageOf(e)}`);
    }
  }

  async function add(text: string) {
    const trimmed = text.trim();
    if (trimmed) await save([...items(), { text: trimmed }]);
  }

  async function edit(index: number, text: string) {
    const trimmed = text.trim();
    if (trimmed) await save(items().with(index, { ...items()[index], text: trimmed }));
  }

  async function remove(index: number) {
    await save(items().toSpliced(index, 1));
  }

  // gap is the index of the gap between the rows: 0 is before the first row, items().length is after the last row
  async function move(from: number, gap: number) {
    if (gap === from || gap === from + 1) return;
    const next = items().toSpliced(from, 1);
    next.splice(gap > from ? gap - 1 : gap, 0, items()[from]);
    await save(next);
  }

  return { items, loaded, error, load, add, edit, remove, move };
}

export type Todo = ReturnType<typeof createTodo>;

export const TodoContext = createContext<Todo>();

export function useTodo() {
  const todo = useContext(TodoContext);
  if (!todo) throw new Error("useTodo is used outside TodoContext.");
  return todo;
}

function messageOf(e: unknown) {
  return e instanceof Error ? e.message : String(e);
}
