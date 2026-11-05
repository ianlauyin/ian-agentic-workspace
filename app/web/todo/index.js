import { createTodo, SECTIONS } from "./context.js";

const TITLES = { pending: "Pending", ongoing: "Ongoing", reviewing: "Reviewing" };

export function mount(main, api) {
  const todo = createTodo(api, render);
  let editing = null;
  let dragFrom = null;
  let dropTo = null;

  main.innerHTML = `
    <div id="todo" class="todo-page">
      <div class="header">
        <h1>TODO</h1>
        <button class="g-button add-open" type="button" disabled>Add</button>
      </div>
      <p class="error" hidden></p>
      <div class="board"></div>
      <dialog class="add-dialog">
        <form class="add-form" method="dialog">
          <h2>New TODO</h2>
          <input class="g-text-input title" type="text" autocomplete="off" placeholder="Title" required>
          <textarea class="g-text-input detail" rows="4" placeholder="Detail"></textarea>
          <label class="deadline">Deadline (optional) <input class="g-text-input" type="date"></label>
          <span class="actions">
            <button class="g-button g-danger cancel" type="button">Cancel</button>
            <button class="g-button" type="submit">Add</button>
          </span>
        </form>
      </dialog>
    </div>`;
  const page = main.querySelector(".todo-page");
  const error = page.querySelector(".error");
  const board = page.querySelector(".board");
  const addOpen = page.querySelector(".add-open");
  const addDialog = page.querySelector(".add-dialog");
  const addForm = addDialog.querySelector(".add-form");
  const addTitle = addForm.querySelector("input.title");
  const addDetail = addForm.querySelector("textarea");
  const addDeadline = addForm.querySelector("input[type=date]");

  addOpen.addEventListener("click", () => {
    addForm.reset();
    addDialog.showModal();
    addTitle.focus();
  });
  addForm.querySelector(".cancel").addEventListener("click", () => addDialog.close());
  // a click on the backdrop is a click on the dialog itself
  addDialog.addEventListener("click", (e) => {
    if (e.target === addDialog) addDialog.close();
  });
  addForm.addEventListener("submit", () => {
    void todo.add("pending", { title: addTitle.value, detail: addDetail.value, deadline: addDeadline.value });
  });

  const sections = Object.fromEntries(SECTIONS.map((section) => [section, sectionView(section)]));

  function sectionView(section) {
    const view = document.createElement("section");
    view.className = "todo-page-section";
    view.innerHTML = `
      <h2></h2>
      <ul class="list" hidden></ul>
      <div class="drop-zone">Drop a card here</div>`;
    view.querySelector("h2").textContent = TITLES[section];
    board.append(view);
    // a drop outside the rows puts the row at the end, so the user can drop on an empty section
    view.addEventListener("dragover", (e) => {
      if (dragFrom === null || e.target.closest(".todo-page-card")) return;
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

    return { list: view.querySelector(".list") };
  }

  function clearDropMarks() {
    for (const row of page.querySelectorAll(".todo-page-card")) row.classList.remove("before", "after");
    for (const view of page.querySelectorAll(".todo-page-section")) view.classList.remove("drop-end");
  }

  function render() {
    const { lists, loaded } = todo.state;
    error.textContent = todo.state.error;
    error.hidden = !todo.state.error;
    addOpen.disabled = !loaded;
    for (const section of SECTIONS) {
      const { list } = sections[section];
      const items = lists[section];
      list.hidden = !loaded || items.length === 0;
      list.replaceChildren(
        ...items.map((item, index) =>
          editing?.section === section && editing.index === index ? editCard(section, item, index) : cardView(section, item, index),
        ),
      );
    }
    page.querySelector(".todo-page-edit-card input")?.focus();
  }

  function cardView(section, item, index) {
    const row = document.createElement("li");
    row.className = "todo-page-card";
    row.innerHTML = `
      <div class="handle" title="Drag to reorder or move">⋮⋮⋮</div>
      <div class="body">
        <span class="title"></span>
        <p class="detail" hidden></p>
        <p class="deadline" hidden></p>
        <span class="actions">
          <button class="g-button g-small" type="button">Edit</button>
          <button class="g-button g-small g-danger" type="button">Delete</button>
        </span>
      </div>`;
    row.querySelector(".title").textContent = item.title;
    const detail = row.querySelector(".detail");
    detail.textContent = item.detail;
    detail.hidden = !item.detail;
    const deadline = row.querySelector(".deadline");
    deadline.textContent = `Deadline: ${item.deadline}`;
    deadline.hidden = !item.deadline;
    const [editButton, deleteButton] = row.querySelectorAll("button");
    editButton.addEventListener("click", () => {
      editing = { section, index };
      render();
    });
    deleteButton.addEventListener("click", () => {
      if (confirm(`Delete "${item.title}"?`)) void todo.remove(section, index);
    });

    // only the handle starts a drag, so the user can still select the text
    const handle = row.querySelector(".handle");
    handle.addEventListener("mousedown", () => (row.draggable = true));
    handle.addEventListener("mouseup", () => (row.draggable = false));
    row.addEventListener("dragstart", (e) => {
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", item.title);
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

  function editCard(section, item, index) {
    const row = document.createElement("li");
    row.className = "todo-page-edit-card";
    row.innerHTML = `
      <input class="g-text-input title" type="text" autocomplete="off" placeholder="Title">
      <textarea class="g-text-input detail" rows="3" placeholder="Detail"></textarea>
      <label class="deadline">Deadline (optional) <input class="g-text-input" type="date"></label>
      <span class="actions">
        <button class="g-button g-small" type="button">Save</button>
        <button class="g-button g-small g-danger" type="button">Cancel</button>
      </span>`;
    const title = row.querySelector("input.title");
    const detail = row.querySelector("textarea");
    const deadline = row.querySelector("input[type=date]");
    title.value = item.title;
    detail.value = item.detail;
    deadline.value = item.deadline;
    const [saveButton, cancelButton] = row.querySelectorAll("button");

    async function save() {
      await todo.edit(section, index, { title: title.value, detail: detail.value, deadline: deadline.value });
      editing = null;
      render();
    }
    function cancel() {
      editing = null;
      render();
    }

    row.addEventListener("keydown", (e) => {
      if (e.key === "Enter" && e.target !== detail) void save();
      if (e.key === "Escape") cancel();
    });
    saveButton.addEventListener("click", () => void save());
    cancelButton.addEventListener("click", cancel);
    return row;
  }

  render();
  void todo.load();
}
