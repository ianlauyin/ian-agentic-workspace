import { createTodo, SECTIONS } from "./context.js";

const CSS = "/todo/index.css";
const TITLES = { important: "Important", pending: "Pending" };

export function mount(main, api) {
  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = CSS;
  document.head.append(link);

  const todo = createTodo(api, render);
  let editing = null;
  let dragFrom = null;
  let dropTo = null;

  main.innerHTML = `
    <div id="todo" class="todo-page">
      <h1>TODO</h1>
      <p class="error" hidden></p>
    </div>`;
  const page = main.querySelector(".todo-page");
  const error = page.querySelector(".error");
  const sections = Object.fromEntries(SECTIONS.map((section) => [section, sectionView(section)]));

  function sectionView(section) {
    const view = document.createElement("section");
    view.className = "todo-page-section";
    view.innerHTML = `
      <h2></h2>
      <form class="add-form">
        <input class="g-text-input" type="text" autocomplete="off" placeholder="New TODO">
        <button class="g-button" type="submit" disabled>Add</button>
      </form>
      <p class="note" hidden>No TODO yet.</p>
      <ul class="list" hidden></ul>`;
    view.querySelector("h2").textContent = TITLES[section];
    page.append(view);
    const form = view.querySelector(".add-form");
    const input = form.querySelector("input");

    form.addEventListener("submit", (e) => {
      e.preventDefault();
      if (!input.value.trim()) return;
      void todo.add(section, input.value);
      input.value = "";
    });

    // a drop outside the rows puts the row at the end, so the user can drop on an empty section
    view.addEventListener("dragover", (e) => {
      if (dragFrom === null || e.target.closest(".todo-page-item-row")) return;
      e.preventDefault();
      dropTo = { section, gap: todo.state.lists[section].length };
      clearDropMarks();
      view.classList.add("drop-end");
    });
    view.addEventListener("drop", (e) => {
      e.preventDefault();
      if (dragFrom !== null && dropTo !== null) void todo.move(dragFrom.section, dragFrom.index, dropTo.section, dropTo.gap);
      dragFrom = null;
      dropTo = null;
      clearDropMarks();
    });

    return { addButton: form.querySelector("button"), note: view.querySelector(".note"), list: view.querySelector(".list") };
  }

  function clearDropMarks() {
    for (const row of page.querySelectorAll(".todo-page-item-row")) row.classList.remove("before", "after");
    for (const view of page.querySelectorAll(".todo-page-section")) view.classList.remove("drop-end");
  }

  function render() {
    const { lists, loaded } = todo.state;
    error.textContent = todo.state.error;
    error.hidden = !todo.state.error;
    for (const section of SECTIONS) {
      const { addButton, note, list } = sections[section];
      const items = lists[section];
      addButton.disabled = !loaded;
      note.hidden = !loaded || items.length > 0;
      list.hidden = !loaded || items.length === 0;
      list.replaceChildren(
        ...items.map((item, index) =>
          editing?.section === section && editing.index === index ? editRow(section, item, index) : itemRow(section, item, index),
        ),
      );
    }
    page.querySelector(".todo-page-edit-row input")?.focus();
  }

  function itemRow(section, item, index) {
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
      editing = { section, index };
      render();
    });
    deleteButton.addEventListener("click", () => {
      if (confirm(`Delete "${item.text}"?`)) void todo.remove(section, index);
    });

    // only the handle starts a drag, so the user can still select the text
    const handle = row.querySelector(".handle");
    handle.addEventListener("mousedown", () => (row.draggable = true));
    handle.addEventListener("mouseup", () => (row.draggable = false));
    row.addEventListener("dragstart", (e) => {
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", item.text);
      dragFrom = { section, index };
      row.classList.add("dragging");
    });
    row.addEventListener("dragover", (e) => {
      if (dragFrom === null) return;
      e.preventDefault();
      const box = row.getBoundingClientRect();
      const after = e.clientY > box.top + box.height / 2;
      dropTo = { section, gap: index + (after ? 1 : 0) };
      clearDropMarks();
      row.classList.add(after ? "after" : "before");
    });
    row.addEventListener("dragend", () => {
      row.draggable = false;
      row.classList.remove("dragging");
      dragFrom = null;
      dropTo = null;
      clearDropMarks();
    });
    return row;
  }

  function editRow(section, item, index) {
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
      await todo.edit(section, index, text.value);
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
