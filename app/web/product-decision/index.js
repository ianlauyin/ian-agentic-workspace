import { createProductDecision, decisionKey, groupsByStatus } from "./context.js";

const CSS = "/product-decision/index.css";

export function mount(main, api) {
  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = CSS;
  document.head.append(link);

  const decision = createProductDecision(api, render);
  const cards = new Map();
  const panels = new Map();
  let selected = "";
  let built = false;

  main.innerHTML = `
    <div id="product-decision" class="product-decision-page">
      <h1>Product Decision</h1>
      <div class="product-decision-page-tabs"></div>
      <p class="stats" hidden></p>
      <p class="error" hidden></p>
    </div>`;
  const page = main.querySelector(".product-decision-page");
  const tabsBar = page.querySelector(".product-decision-page-tabs");
  const stats = page.querySelector(".stats");
  const error = page.querySelector(".error");

  function groupSection(group) {
    const section = document.createElement("section");
    section.className = "product-decision-page-group";
    section.hidden = true;
    section.innerHTML = `<h2></h2><p class="source"></p>`;
    section.querySelector("h2").textContent = group.title;
    section.querySelector(".source").textContent = `${group.source} · ${group.spec}`;
    return section;
  }

  function nameEditor(tab) {
    const view = document.createElement("form");
    view.className = "product-decision-page-name";
    view.innerHTML = `
      <label>Name
        <input class="g-text-input" type="text" autocomplete="off">
      </label>
      <button class="g-button" type="submit" disabled>Save</button>`;
    const input = view.querySelector("input");
    const saveButton = view.querySelector("button");
    input.value = tab.name;
    input.addEventListener("input", () => updateName(tab.id));
    view.addEventListener("submit", (e) => {
      e.preventDefault();
      void decision.rename(tab.id, input.value);
    });
    input.addEventListener("keydown", (e) => {
      if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) view.requestSubmit();
    });
    return { view, input, saveButton };
  }

  function updateName(id) {
    const panel = panels.get(id);
    const saved = decision.state.tabs.find((tab) => tab.id === id)?.name ?? "";
    panel.button.textContent = saved;
    const typed = panel.input.value.trim();
    panel.saveButton.disabled = !typed || typed === saved;
  }

  // the cards and the name inputs are built one time, so the text stays when the page renders again
  function build() {
    for (const tab of decision.state.tabs) {
      const button = document.createElement("button");
      button.type = "button";
      button.textContent = tab.name;
      button.addEventListener("click", () => {
        selected = tab.id;
        render();
      });
      tabsBar.append(button);

      const panel = document.createElement("div");
      panel.className = "product-decision-page-panel";
      panel.hidden = true;
      const editor = nameEditor(tab);
      panel.append(editor.view);
      const slots = [];
      for (const group of tab.groups) {
        const openSection = groupSection(group);
        const decidedSection = groupSection(group);
        panel.append(openSection);
        slots.push({ spec: group.spec, openSection, decidedSection });
        for (const item of group.items) {
          const key = decisionKey(tab, group, item);
          cards.set(key, itemCard(key, item));
        }
      }
      for (const slot of slots) panel.append(slot.decidedSection);
      page.append(panel);
      panels.set(tab.id, { button, panel, slots, input: editor.input, saveButton: editor.saveButton });
    }
    built = true;
  }

  function placeCards(tab) {
    const panel = panels.get(tab.id);
    for (const slot of panel.slots) {
      slot.openSection.hidden = true;
      slot.decidedSection.hidden = true;
    }
    const { open, decided } = groupsByStatus(tab, decision.state.decisions);
    place(panel, tab, open, "openSection");
    place(panel, tab, decided, "decidedSection");
  }

  function place(panel, tab, groups, sectionName) {
    for (const group of groups) {
      const slot = panel.slots.find((item) => item.spec === group.spec);
      slot[sectionName].hidden = false;
      for (const item of group.items) slot[sectionName].append(cards.get(decisionKey(tab, group, item)).view);
    }
  }

  function itemCard(key, item) {
    const view = document.createElement("article");
    view.className = "product-decision-page-card";
    view.innerHTML = `
      <div class="head">
        <span class="num"></span>
        <h3></h3>
        <span class="status"></span>
      </div>
      <dl class="fields">
        <dt>Scope</dt>
        <dd class="scope"></dd>
        <dt>Issue</dt>
        <dd class="issue"></dd>
        <dt>Recommendations</dt>
        <dd><ol class="options"></ol></dd>
      </dl>
      <form class="decide-form">
        <label>Final decision
          <textarea class="g-text-input" rows="2" placeholder="Write the final decision"></textarea>
        </label>
        <button class="g-button" type="submit" disabled>Save</button>
      </form>`;
    view.querySelector(".num").textContent = `#${item.id}`;
    view.querySelector("h3").textContent = item.item;
    view.querySelector(".issue").textContent = item.issue;
    view.querySelector(".scope").append(
      ...item.scope.map((scope) => {
        const tag = document.createElement("span");
        tag.className = "tag";
        tag.textContent = scope;
        return tag;
      }),
    );
    view.querySelector(".options").append(
      ...item.options.map((option, index) => {
        const li = document.createElement("li");
        if (index === 0) {
          li.className = "recommended";
          const label = document.createElement("span");
          label.className = "label";
          label.textContent = "Recommended";
          li.append(label);
        }
        li.append(option);
        return li;
      }),
    );

    const form = view.querySelector(".decide-form");
    const input = form.querySelector("textarea");
    const saveButton = form.querySelector("button");
    input.value = decision.state.decisions[key] ?? "";
    input.addEventListener("input", () => updateCard(key));
    form.addEventListener("submit", (e) => {
      e.preventDefault();
      void decision.decide(key, input.value);
    });
    input.addEventListener("keydown", (e) => {
      if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) form.requestSubmit();
    });

    return { view, input, saveButton, status: view.querySelector(".status") };
  }

  function updateCard(key) {
    const { view, input, saveButton, status } = cards.get(key);
    const saved = decision.state.decisions[key] ?? "";
    view.classList.toggle("decided", saved !== "");
    status.textContent = saved ? "Decided" : "Open";
    saveButton.disabled = input.value.trim() === saved;
  }

  function render() {
    const { tabs, decisions, loaded } = decision.state;
    error.textContent = decision.state.error;
    error.hidden = !decision.state.error;
    if (!loaded) return;
    if (!built) build();
    if (!selected) selected = tabs[0]?.id ?? "";
    for (const key of cards.keys()) updateCard(key);
    for (const tab of tabs) {
      const panel = panels.get(tab.id);
      placeCards(tab);
      panel.panel.hidden = tab.id !== selected;
      panel.button.classList.toggle("on", tab.id === selected);
      updateName(tab.id);
    }
    const tab = tabs.find((item) => item.id === selected);
    const total = tab ? tab.groups.reduce((sum, group) => sum + group.items.length, 0) : 0;
    const decided = tab
      ? tab.groups.reduce(
          (sum, group) => sum + group.items.filter((item) => decisions[decisionKey(tab, group, item)]).length,
          0,
        )
      : 0;
    stats.textContent = `${decided} of ${total} decided`;
    stats.hidden = false;
  }

  render();
  void decision.load();
}
