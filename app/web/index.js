const PAGES = [
  { path: "/todo", title: "TODO", module: "/todo/index.js", css: "/todo/index.css" },
  { path: "/notes", title: "Notes", module: "/notes/index.js", css: "/notes/index.css" },
  { path: "/logs", title: "Logs", module: "/logs/index.js", css: "/logs/index.css" },
];
const HOME = "/todo";

async function request(path, init) {
  const res = await fetch(path, init);
  const json = await res.json();
  if (!res.ok) throw new Error(json.error);
  return json;
}

const api = {
  loadProjects: () => request("/projects"),
  assignNote: (body) =>
    request("/notes-assign", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    }),
  loadLogApps: () => request("/logs-apps"),
  loadLogs: (params) => request(`/logs-events?${new URLSearchParams(params)}`),
  loadResolveProgress: (chat) => request(`/open-behaviour-progress?${new URLSearchParams({ chat })}`),
  followupResolve: (chat, text) =>
    request("/open-behaviour-followup", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ chat, text }),
    }),
  loadPage: (name) => request(`/page?${new URLSearchParams({ name })}`),
  savePage: (name, data) =>
    request("/page", {
      method: "PUT",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ name, data }),
    }),
};

const nav = document.querySelector(".g-nav");
const main = document.querySelector("main");
let unmount;
let pageStyle;
let routeId = 0;

function loadStyle(href) {
  return new Promise((resolve) => {
    const link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = href;
    link.onload = link.onerror = () => resolve(link);
    document.head.append(link);
  });
}

async function route() {
  const id = ++routeId;
  const page = PAGES.find((p) => p.path === location.pathname);
  if (!page) {
    history.replaceState(null, "", HOME);
    return route();
  }
  document.title = page.title;
  for (const link of nav.children) link.classList.toggle("g-on", link.pathname === page.path);
  const [style, { mount }] = await Promise.all([loadStyle(page.css), import(page.module)]);
  if (id !== routeId) {
    style.remove();
    return;
  }
  unmount?.();
  pageStyle?.remove();
  pageStyle = style;
  main.replaceChildren();
  unmount = mount(main, api);
}

nav.onclick = (e) => {
  const link = e.target.closest("a");
  if (!link || e.metaKey || e.ctrlKey || e.shiftKey) return;
  e.preventDefault();
  if (link.pathname === location.pathname) return;
  history.pushState(null, "", link.pathname);
  void route();
};

window.onpopstate = () => void route();
await route();
