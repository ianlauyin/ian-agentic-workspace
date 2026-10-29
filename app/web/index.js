const PAGES = [
  { path: "/todo", title: "TODO", module: "/todo/index.js" },
  { path: "/product-decision", title: "Product Decision", module: "/product-decision/index.js" },
];
const HOME = "/todo";

async function request(path, init) {
  const res = await fetch(path, init);
  const json = await res.json();
  if (!res.ok) throw new Error(json.error);
  return json;
}

const api = {
  loadPage: (name) => request(`/page?${new URLSearchParams({ name })}`),
  savePage: (name, data) =>
    request("/page", {
      method: "PUT",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ name, data }),
    }),
};

const page = PAGES.find((p) => p.path === location.pathname);
if (!page) {
  location.replace(HOME);
} else {
  document.title = page.title;
  const nav = document.querySelector(".g-tab-bar");
  for (const p of PAGES) {
    const link = document.createElement("a");
    link.href = p.path;
    link.textContent = p.title;
    link.classList.toggle("g-on", p === page);
    nav.append(link);
  }
  const { mount } = await import(page.module);
  mount(document.querySelector("main"), api);
}
