import { createProductDecision, decisionKey } from "./context.js";

const CSS = "/product-decision/index.css";
const DECISIONS = "/decisions";

async function loadGroups() {
  const res = await fetch(DECISIONS);
  if (!res.ok) throw new Error(`${DECISIONS} returned ${res.status}`);
  return res.json();
}

export function mount(main, api) {
  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = CSS;
  document.head.append(link);

  const decision = createProductDecision(api, loadGroups, render);
  const cards = new Map();

  main.innerHTML = `
    <div id="product-decision" class="product-decision-page">
      <h1>Product Decision</h1>
      <p class="stats" hidden></p>
      <p class="error" hidden></p>
    </div>`;
  const page = main.querySelector(".product-decision-page");
  const stats = page.querySelector(".stats");
  const error = page.querySelector(".error");

  // the cards are built one time, so an input keeps its text when the page renders again
  function buildCards() {
    for (const group of decision.state.groups) {
      const section = document.createElement("section");
      section.className = "product-decision-page-group";
      section.innerHTML = `<h2></h2><p class="source"></p>`;
      section.querySelector("h2").textContent = `${group.project} · ${group.title}`;
      section.querySelector(".source").textContent = `${group.source} · ${group.spec}`;
      for (const item of group.items) {
        const key = decisionKey(group, item);
        const card = itemCard(key, item);
        cards.set(key, card);
        section.append(card.view);
      }
      page.append(section);
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
    const { groups, decisions, loaded } = decision.state;
    error.textContent = decision.state.error;
    error.hidden = !decision.state.error;
    if (!loaded) return;
    if (cards.size === 0) buildCards();
    for (const key of cards.keys()) updateCard(key);
    const total = groups.reduce((sum, group) => sum + group.items.length, 0);
    const decided = [...cards.keys()].filter((key) => decisions[key]).length;
    stats.textContent = `${decided} of ${total} decided`;
    stats.hidden = false;
  }

  render();
  void decision.load();
}
