import { PAGE_SIZES, cellText, createLogs, logColumns, pageCount } from "./context.js";

const CSS = "/logs/index.css";

export function mount(main, api) {
  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = CSS;
  document.head.append(link);

  const logs = createLogs(api, render);

  main.innerHTML = `
    <div id="logs" class="logs-page">
      <h1>Logs</h1>
      <div class="logs-page-tabs"></div>
      <p class="error" hidden></p>
      <form class="logs-page-bar logs-page-panel">
        <label>Date (HKT)
          <input class="g-text-input date" type="date">
        </label>
        <label>Rows
          <select class="g-text-input page-size"></select>
        </label>
        <button class="g-button refresh">Refresh</button>
      </form>
      <form class="logs-page-bar logs-page-filters">
        <input class="g-text-input field" list="logs-page-fields" placeholder="Field" autocomplete="off">
        <datalist id="logs-page-fields"></datalist>
        <input class="g-text-input value" placeholder="Value" autocomplete="off">
        <button class="g-button add-filter">Add filter</button>
        <span class="pills"></span>
      </form>
      <div class="logs-page-bar logs-page-pager">
        <button class="g-button prev" type="button">Previous</button>
        <span class="status"></span>
        <button class="g-button next" type="button">Next</button>
      </div>
      <div class="logs-page-list"></div>
      <dialog class="logs-page-modal">
        <button class="g-button close" type="button">Close</button>
        <pre class="text"></pre>
      </dialog>
    </div>`;
  const tabsBar = main.querySelector(".logs-page-tabs");
  const error = main.querySelector(".error");
  const dateInput = main.querySelector(".date");
  const pageSize = main.querySelector(".page-size");
  const refresh = main.querySelector(".refresh");
  const fieldInput = main.querySelector(".field");
  const valueInput = main.querySelector(".value");
  const fieldList = main.querySelector("datalist");
  const pills = main.querySelector(".pills");
  const prev = main.querySelector(".prev");
  const next = main.querySelector(".next");
  const status = main.querySelector(".status");
  const list = main.querySelector(".logs-page-list");
  const modal = main.querySelector(".logs-page-modal");
  const modalText = main.querySelector(".logs-page-modal .text");

  // a click on a cell with an ellipsis shows its full text in the modal
  list.addEventListener("click", (e) => {
    const cell = e.target.closest("td, th");
    if (!cell || cell.scrollWidth <= cell.clientWidth) return;
    modalText.textContent = cell.textContent;
    modal.showModal();
  });
  modal.addEventListener("click", (e) => {
    if (e.target === modal || e.target.closest(".close")) modal.close();
  });

  dateInput.addEventListener("change", () => {
    if (dateInput.value) void logs.show(logs.state.app, dateInput.value, 1);
  });
  main.querySelector(".logs-page-panel").addEventListener("submit", (e) => {
    e.preventDefault();
    void logs.reload();
  });
  pageSize.replaceChildren(
    ...PAGE_SIZES.map((n) => Object.assign(document.createElement("option"), { value: n, textContent: `${n} rows` })),
  );
  pageSize.addEventListener("change", () => void logs.setPageSize(Number(pageSize.value)));
  main.querySelector(".logs-page-filters").addEventListener("submit", (e) => {
    e.preventDefault();
    const field = fieldInput.value.trim();
    const value = valueInput.value;
    if (!field || !value) return;
    fieldInput.value = "";
    valueInput.value = "";
    void logs.addFilter(field, value);
  });
  prev.addEventListener("click", () => void logs.show(logs.state.app, logs.state.date, logs.state.result.page - 1));
  next.addEventListener("click", () => void logs.show(logs.state.app, logs.state.date, logs.state.result.page + 1));

  function eventsTable(events) {
    const columns = logColumns(events);
    const table = document.createElement("table");
    const head = table.createTHead().insertRow();
    for (const column of columns) {
      const th = document.createElement("th");
      th.textContent = column.label;
      th.dataset.column = column.label;
      head.append(th);
    }
    const body = table.createTBody();
    for (const event of events) {
      const row = body.insertRow();
      row.className = event.level ?? "";
      for (const column of columns) {
        const cell = row.insertCell();
        cell.textContent = cellText(event, column);
        cell.dataset.column = column.label;
      }
    }
    return table;
  }

  function render() {
    const { apps, app, date, result, loading, loaded } = logs.state;
    error.textContent = logs.state.error;
    error.hidden = !logs.state.error;
    if (!loaded) return;
    if (tabsBar.childElementCount === 0) {
      for (const item of apps) {
        const button = document.createElement("button");
        button.type = "button";
        button.dataset.tab = item.id;
        button.textContent = item.name;
        button.addEventListener("click", () => void logs.show(item.id, logs.state.date, 1));
        tabsBar.append(button);
      }
    }
    for (const button of tabsBar.children) button.classList.toggle("on", button.dataset.tab === app);
    dateInput.value = date;
    pageSize.value = logs.state.pageSize;
    refresh.disabled = loading;
    fieldList.replaceChildren(
      ...(result ? logColumns(result.events).map((c) => Object.assign(document.createElement("option"), { value: c.label })) : []),
    );
    pills.replaceChildren(
      ...logs.state.filters.map((filter, index) => {
        const remove = document.createElement("button");
        remove.type = "button";
        remove.title = "Remove filter";
        remove.textContent = "×";
        remove.addEventListener("click", () => void logs.removeFilter(index));
        const pill = document.createElement("span");
        pill.className = "pill";
        const name = document.createElement("b");
        name.textContent = filter.field;
        pill.append(name, `: ${filter.value}`, remove);
        return pill;
      }),
    );
    const pages = result ? pageCount(result.total, result.page_size) : 1;
    prev.disabled = loading || !result || result.page <= 1;
    next.disabled = loading || !result || result.page >= pages;
    status.textContent = loading ? "Loading…" : result ? `Page ${result.page} of ${pages} · ${result.total} events` : "";
    if (loading) return;
    const empty = document.createElement("p");
    empty.className = "empty";
    empty.textContent = "No log.";
    list.replaceChildren(...(result?.events.length ? [eventsTable(result.events)] : result ? [empty] : []));
  }

  render();
  void logs.loadApps();
}
