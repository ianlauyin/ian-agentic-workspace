// the router gives each visit a fresh <main> with index.html and index.css in it, so a late callback writes to a detached one
export default function mount(root) {
  const $ = (id) => root.querySelector(`#${id}`);

  function el(tag, attrs, ...children) {
    const e = document.createElement(tag);
    Object.assign(e, attrs);
    e.append(...children);
    return e;
  }

  // a GET sends its params in the query, an object one as JSON
  async function api(method, path, body) {
    let init = { method, headers: { "content-type": "application/json" }, body: JSON.stringify(body) };
    if (method === "GET") {
      const q = Object.entries(body ?? {}).map(([k, v]) => [k, typeof v === "object" ? JSON.stringify(v) : String(v)]);
      if (q.length) path += `?${new URLSearchParams(q)}`;
      init = { method };
    }
    const res = await fetch(path, init);
    const json = await res.json();
    if (!res.ok) throw new Error(json.error);
    return json;
  }

  // my own TODOs, saved as one list in page todo-items
  let items = [];

  async function save(next) {
    try {
      await api("PUT", "/page", { name: "todo-items", data: { items: next } });
      items = next;
      $("items-status").textContent = "";
    } catch (e) {
      $("items-status").textContent = `Failed to save: ${e.message}`;
    }
    renderItems();
  }

  function renderItems(editing) {
    const box = $("items");
    box.replaceChildren();
    if (items.length === 0) { box.append(el("p", { className: "note" }, "No TODO yet.")); return; }
    const rows = items.map((it, i) => {
      if (i === editing) {
        const input = el("input", { value: it.text });
        const done = () => { const text = input.value.trim(); text ? save(items.with(i, { ...it, text })) : renderItems(); };
        input.onkeydown = (e) => { if (e.key === "Enter") done(); if (e.key === "Escape") renderItems(); };
        setTimeout(() => input.focus());
        return el("tr", {},
          el("td", { className: "text" }, input),
          el("td", { className: "act" },
            el("button", { onclick: done }, "Save"),
            el("button", { className: "del", onclick: () => renderItems() }, "Cancel")));
      }
      return el("tr", {},
        el("td", { className: "text" }, it.text),
        el("td", { className: "act" },
          el("button", { onclick: () => renderItems(i) }, "Edit"),
          el("button", { className: "del", onclick: () => confirm(`Delete "${it.text}"?`) && save(items.toSpliced(i, 1)) }, "Delete")));
    });
    box.append(el("div", { className: "wrap" }, el("table", {}, el("tbody", {}, ...rows))));
  }

  $("add").onsubmit = (e) => {
    e.preventDefault();
    const text = $("new").value.trim();
    if (!text) return;
    $("new").value = "";
    save([...items, { text }]);
  };

  api("GET", "/page", { name: "todo-items" }).then((saved) => {
    items = saved?.items ?? [];
    renderItems();
  }, (e) => { $("items-status").textContent = `Failed to load my TODOs: ${e.message}`; });
}
