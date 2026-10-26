import { createTodo } from "./context.js";

const CSS = "/todo/index.css";

export function mount(main, api) {
  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = CSS;
  document.head.append(link);

  const todo = createTodo(api, render);
  let editing = null;
  let dragFrom = null;
  let dropGap = null;

  main.innerHTML = `
    <div id="todo" class="todo-page">
      <h1>TODO</h1>
      <h2>My TODO</h2>
      <form class="todo-page-add-form">
        <input class="g-text-input" type="text" autocomplete="off" placeholder="New TODO">
        <button class="g-button" type="submit" disabled>Add</button>
      </form>
      <p class="error" hidden></p>
      <p class="note" hidden>No TODO yet.</p>
      <ul class="list" hidden></ul>
    </div>`;
  const form = main.querySelector(".todo-page-add-form");
  const input = form.querySelector("input");
  const addButton = form.querySelector("button");
  const error = main.querySelector(".error");
  const note = main.querySelector(".note");
  const list = main.querySelector(".list");

  form.addEventListener("submit", (e) => {
    e.preventDefault();
    if (!input.value.trim()) return;
    void todo.add(input.value);
    input.value = "";
  });

  function render() {
    const { items, loaded } = todo.state;
    addButton.disabled = !loaded;
    error.textContent = todo.state.error;
    error.hidden = !todo.state.error;
    note.hidden = !loaded || items.length > 0;
    list.hidden = !loaded || items.length === 0;
    list.replaceChildren(...items.map((item, index) => (index === editing ? editRow(item, index) : itemRow(item, index))));
    list.querySelector(".todo-page-edit-row input")?.focus();
  }

  function itemRow(item, index) {
    const row = document.createElement("li");
    row.className = "todo-page-item-row";
    row.innerHTML = `
      <span class="handle" title="Drag to reorder">⋮⋮</span>
      <span class="text"></span>
      <span class="actions">
        <button class="g-button g-small" type="button">Edit</button>
        <button class="g-button g-small g-danger" type="button">Delete</button>
      </span>`;
    row.querySelector(".text").textContent = item.text;
    const [editButton, deleteButton] = row.querySelectorAll("button");
    editButton.addEventListener("click", () => {
      editing = index;
      render();
    });
    deleteButton.addEventListener("click", () => {
      if (confirm(`Delete "${item.text}"?`)) void todo.remove(index);
    });

    // only the handle starts a drag, so the user can still select the text
    const handle = row.querySelector(".handle");
    handle.addEventListener("mousedown", () => (row.draggable = true));
    handle.addEventListener("mouseup", () => (row.draggable = false));
    row.addEventListener("dragstart", (e) => {
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", item.text);
      dragFrom = index;
      row.classList.add("dragging");
    });
    row.addEventListener("dragover", (e) => {
      if (dragFrom === null) return;
      e.preventDefault();
      const box = row.getBoundingClientRect();
      const after = e.clientY > box.top + box.height / 2;
      dropGap = index + (after ? 1 : 0);
      for (const other of list.children) other.classList.remove("before", "after");
      row.classList.add(after ? "after" : "before");
    });
    row.addEventListener("drop", (e) => {
      e.preventDefault();
      if (dragFrom !== null && dropGap !== null) void todo.move(dragFrom, dropGap);
    });
    row.addEventListener("dragend", () => {
      row.draggable = false;
      dragFrom = null;
      dropGap = null;
      for (const other of list.children) other.classList.remove("dragging", "before", "after");
    });
    return row;
  }

  function editRow(item, index) {
    const row = document.createElement("li");
    row.className = "todo-page-edit-row";
    row.innerHTML = `
      <input class="g-text-input" type="text" autocomplete="off">
      <span class="actions">
        <button class="g-button g-small" type="button">Save</button>
        <button class="g-button g-small g-danger" type="button">Cancel</button>
      </span>`;
    const text = row.querySelector("input");
    text.value = item.text;
    const [saveButton, cancelButton] = row.querySelectorAll("button");

    async function save() {
      await todo.edit(index, text.value);
      editing = null;
      render();
    }
    function cancel() {
      editing = null;
      render();
    }

    text.addEventListener("keydown", (e) => {
      if (e.key === "Enter") void save();
      if (e.key === "Escape") cancel();
    });
    saveButton.addEventListener("click", () => void save());
    cancelButton.addEventListener("click", cancel);
    return row;
  }

  render();
  void todo.load();
}
