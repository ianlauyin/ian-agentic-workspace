// path: title; each page is a folder with index.html, index.css, and index.js
const PAGES = {
  "/todo": "TODO",
};

async function route() {
  if (!PAGES[location.pathname]) history.replaceState(null, "", "/todo");
  const path = location.pathname;
  document.title = PAGES[path];
  for (const a of document.querySelectorAll("nav a")) a.classList.toggle("on", a.pathname === path);
  const main = document.createElement("main");
  document.querySelector("main").replaceWith(main);
  const [html, mod] = await Promise.all([fetch(`${path}/index.html`).then((r) => r.text()), import(`${path}/index.js`)]);
  if (!main.isConnected) return;
  main.innerHTML = `<link rel="stylesheet" href="${path}/index.css">${html}`;
  mod.default(main);
}

document.querySelector("nav").onclick = (e) => {
  const a = e.target.closest("a");
  if (!a || e.metaKey || e.ctrlKey || e.shiftKey) return;
  e.preventDefault();
  if (a.pathname !== location.pathname) history.pushState(null, "", a.pathname);
  route();
};

window.onpopstate = route;
route();
