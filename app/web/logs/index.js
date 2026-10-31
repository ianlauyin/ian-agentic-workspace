import { cellText, createLogs, logColumns, pageCount } from "./context.js";

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
      <div class="logs-page-bar">
        <label>Date (HKT)
          <input class="g-text-input date" type="date">
        </label>
        <button class="g-button prev" type="button">Previous</button>
        <span class="status"></span>
        <button class="g-button next" type="button">Next</button>
      </div>
      <div class="logs-page-list"></div>
      <div class="logs-page-popup" hidden></div>
    </div>`;
  const tabsBar = main.querySelector(".logs-page-tabs");
  const error = main.querySelector(".error");
  const dateInput = main.querySelector(".date");
  const prev = main.querySelector(".prev");
  const next = main.querySelector(".next");
  const status = main.querySelector(".status");
  const list = main.querySelector(".logs-page-list");
  const popup = main.querySelector(".logs-page-popup");

  // a cell with an ellipsis shows its full text in the popup
  list.addEventListener("mouseover", (e) => {
    const cell = e.target.closest("td, th");
    if (!cell || cell.scrollWidth <= cell.clientWidth) return;
    popup.textContent = cell.textContent;
    popup.hidden = false;
    const box = cell.getBoundingClientRect();
    popup.style.left = `${Math.min(box.left, window.innerWidth - popup.offsetWidth - 8)}px`;
    popup.style.top = `${box.bottom + 4}px`;
  });
  list.addEventListener("mouseout", (e) => {
    if (!e.relatedTarget?.closest("td, th")) popup.hidden = true;
  });
  list.addEventListener("scroll", () => (popup.hidden = true));

  dateInput.addEventListener("change", () => {
    if (dateInput.value) void logs.show(logs.state.app, dateInput.value, 1);
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
    const pages = result ? pageCount(result.total, result.page_size) : 1;
    prev.disabled = loading || !result || result.page <= 1;
    next.disabled = loading || !result || result.page >= pages;
    status.textContent = loading ? "Loading…" : result ? `Page ${result.page} of ${pages} · ${result.total} events` : "";
    if (loading) return;
    popup.hidden = true;
    const empty = document.createElement("p");
    empty.className = "empty";
    empty.textContent = "No log.";
    list.replaceChildren(...(result?.events.length ? [eventsTable(result.events)] : result ? [empty] : []));
  }

  render();
  void logs.loadApps();
}
