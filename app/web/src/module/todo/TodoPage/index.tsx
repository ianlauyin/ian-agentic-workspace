import { For, Show, createSignal, onMount } from "solid-js";

import { useApp } from "../../../context";
import { TodoContext, createTodo } from "../context";
import AddForm from "./AddForm";
import EditRow from "./EditRow";
import ItemRow, { type DropSide } from "./ItemRow";
import "./index.css";

type DropTarget = { index: number; side: DropSide };

export default function TodoPage() {
  const todo = createTodo(useApp().api);
  const [editing, setEditing] = createSignal<number | null>(null);
  const [dragFrom, setDragFrom] = createSignal<number | null>(null);
  const [dropTarget, setDropTarget] = createSignal<DropTarget | null>(null);

  onMount(() => void todo.load());

  async function saveEdit(index: number, text: string) {
    await todo.edit(index, text);
    setEditing(null);
  }

  function remove(index: number) {
    if (confirm(`Delete "${todo.items()[index].text}"?`)) void todo.remove(index);
  }

  function dragOver(index: number, side: DropSide) {
    if (dragFrom() !== null) setDropTarget({ index, side });
  }

  function drop() {
    const from = dragFrom();
    const target = dropTarget();
    if (from !== null && target) void todo.move(from, target.index + (target.side === "after" ? 1 : 0));
  }

  function dragEnd() {
    setDragFrom(null);
    setDropTarget(null);
  }

  return (
    <TodoContext.Provider value={todo}>
      <div id="todo" class="todo-page">
        <h1>TODO</h1>
        <h2>My TODO</h2>
        <AddForm />
        <Show when={todo.error()}>
          <p class="error">{todo.error()}</p>
        </Show>
        <Show when={todo.loaded()}>
          <Show when={todo.items().length > 0} fallback={<p class="note">No TODO yet.</p>}>
            <ul class="list">
              <For each={todo.items()}>
                {(item, index) => (
                  <Show
                    when={editing() === index()}
                    fallback={
                      <ItemRow
                        item={item}
                        dragging={dragFrom() === index()}
                        dropSide={dropTarget()?.index === index() ? dropTarget()?.side : undefined}
                        onEdit={() => setEditing(index())}
                        onDelete={() => remove(index())}
                        onDragStart={() => setDragFrom(index())}
                        onDragOver={(side) => dragOver(index(), side)}
                        onDrop={drop}
                        onDragEnd={dragEnd}
                      />
                    }
                  >
                    <EditRow item={item} onSave={(text) => saveEdit(index(), text)} onCancel={() => setEditing(null)} />
                  </Show>
                )}
              </For>
            </ul>
          </Show>
        </Show>
      </div>
    </TodoContext.Provider>
  );
}
