import { createOpenBehaviour, decisionKey, groupsByStatus, withKind } from "./context.js";

const CSS = "/open-behaviour/index.css";
const KINDS = { bug: "Bug", decision: "Needs decision", cleanup: "Cleanup" };
const PROGRESS_MS = 3000;
const PROGRESS_STATUS = {
  running: "The agent is working.",
  finished: "The agent finished.",
  failed: "The agent failed.",
  stopped: "The agent stopped before it finished.",
};

// the open-behaviour files mark code with backticks
function appendText(parent, text) {
  text.split("`").forEach((part, index) => {
    if (index % 2 === 0) {
      parent.append(part);
      return;
    }
    const code = document.createElement("code");
    code.textContent = part;
    parent.append(code);
  });
}

function tags(parent, values) {
  parent.append(
    ...values.map((value) => {
      const tag = document.createElement("span");
      tag.className = "tag";
      tag.textContent = value;
      return tag;
    }),
  );
}

export function mount(main, api) {
  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = CSS;
  document.head.append(link);

  const behaviour = createOpenBehaviour(api, render);
  const cards = new Map();
  const panels = new Map();
  let selected = "";
  let kind = "";
  let built = false;

  main.innerHTML = `
    <div id="open-behaviour" class="open-behaviour-page">
      <h1>Open Behaviour</h1>
      <div class="open-behaviour-page-tabs"></div>
      <div class="open-behaviour-page-kinds" hidden></div>
      <p class="stats" hidden></p>
      <p class="error" hidden></p>
    </div>`;
  const page = main.querySelector(".open-behaviour-page");
  const tabsBar = page.querySelector(".open-behaviour-page-tabs");
  const kindsBar = page.querySelector(".open-behaviour-page-kinds");
  const stats = page.querySelector(".stats");
  const error = page.querySelector(".error");

  const kindButtons = Object.entries({ "": "All", ...KINDS }).map(([value, label]) => {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = label;
    button.addEventListener("click", () => {
      kind = value;
      render();
    });
    kindsBar.append(button);
    return { value, button };
  });

  function groupSection(group) {
    const section = document.createElement("section");
    section.className = "open-behaviour-page-group";
    section.hidden = true;
    section.innerHTML = `<h2></h2><p class="source"></p>`;
    section.querySelector("h2").textContent = group.title;
    const source = section.querySelector(".source");
    source.textContent = `${group.spec} · `;
    appendText(source, group.context ?? "");
    return section;
  }

  // the cards are built one time, so the text of an input stays when the page renders again
  function build() {
    for (const tab of behaviour.state.tabs) {
      const button = document.createElement("button");
      button.type = "button";
      button.textContent = tab.name;
      const badge = document.createElement("span");
      badge.className = "badge";
      button.append(badge);
      button.addEventListener("click", () => {
        selected = tab.id;
        render();
      });
      tabsBar.append(button);

      const panel = document.createElement("div");
      panel.className = "open-behaviour-page-panel";
      panel.hidden = true;
      for (const message of tab.errors) {
        const p = document.createElement("p");
        p.className = "error";
        p.textContent = message;
        panel.append(p);
      }
      const slots = [];
      for (const group of tab.groups) {
        const openSection = groupSection(group);
        const decidedSection = groupSection(group);
        panel.append(openSection);
        slots.push({ group, openSection, decidedSection });
        for (const item of group.items) {
          const key = decisionKey(tab, group, item);
          cards.set(key, itemCard(key, tab, group, item));
        }
      }
      for (const slot of slots) panel.append(slot.decidedSection);
      const empty = document.createElement("p");
      empty.className = "empty";
      empty.textContent = "No open item.";
      panel.append(empty);
      page.append(panel);
      panels.set(tab.id, { button, badge, panel, slots, empty });
    }
    built = true;
  }

  function placeCards(tab) {
    const panel = panels.get(tab.id);
    for (const slot of panel.slots) {
      slot.openSection.hidden = true;
      slot.decidedSection.hidden = true;
    }
    const { open, decided } = groupsByStatus(tab, behaviour.state.decisions);
    place(panel, tab, open, "openSection");
    place(panel, tab, decided, "decidedSection");
  }

  function place(panel, tab, groups, sectionName) {
    for (const group of groups) {
      const slot = panel.slots.find((item) => item.group.spec === group.spec);
      slot[sectionName].hidden = false;
      for (const item of group.items) slot[sectionName].append(cards.get(decisionKey(tab, group, item)).view);
    }
  }

  function itemCard(key, tab, group, item) {
    const view = document.createElement("article");
    view.className = "open-behaviour-page-card";
    view.innerHTML = `
      <div class="head">
        <span class="num"></span>
        <h3></h3>
        <span class="kind"></span>
        <span class="status"></span>
      </div>
      <dl class="fields">
        <dt>Scope</dt>
        <dd class="scope"></dd>
        <dt>Now</dt>
        <dd class="now"></dd>
        <dt>Waits on</dt>
        <dd class="waiting-on"></dd>
        <dt>Next step</dt>
        <dd class="next-step"></dd>
        <dt>Recommendations</dt>
        <dd><ol class="options"></ol></dd>
      </dl>
      <form class="decide-form">
        <label>Final decision
          <textarea class="g-text-input" rows="2" placeholder="Write the final decision"></textarea>
        </label>
        <button class="g-button" type="submit" disabled>Save</button>
        <button class="g-button resolve" type="button" hidden>Resolve</button>
      </form>
      <p class="resolve-chat" hidden></p>
      <div class="resolve-progress" hidden>
        <p class="state"></p>
        <ol class="steps"></ol>
      </div>`;
    view.querySelector(".num").textContent = `#${item.id}`;
    appendText(view.querySelector("h3"), item.item);
    const kind = view.querySelector(".kind");
    kind.textContent = KINDS[item.kind] ?? item.kind;
    kind.classList.add(item.kind);
    tags(view.querySelector(".scope"), item.scope ?? []);
    const now = view.querySelector(".now");
    appendText(now, item.now);
    if (item.note) {
      const note = document.createElement("div");
      note.className = "note";
      appendText(note, item.note);
      now.append(note);
    }
    tags(view.querySelector(".waiting-on"), item.waitingOn ?? []);
    appendText(view.querySelector(".next-step"), item.nextStep ?? "");
    view.querySelector(".options").append(
      ...(item.options ?? []).map((option, index) => {
        const li = document.createElement("li");
        if (index === 0) {
          li.className = "recommended";
          const label = document.createElement("span");
          label.className = "label";
          label.textContent = "Recommended";
          li.append(label);
        }
        appendText(li, option);
        return li;
      }),
    );

    const form = view.querySelector(".decide-form");
    const input = form.querySelector("textarea");
    const saveButton = form.querySelector("button[type=submit]");
    const resolveButton = form.querySelector(".resolve");
    input.value = behaviour.state.decisions[key] ?? "";
    input.addEventListener("input", () => updateCard(key));
    form.addEventListener("submit", (e) => {
      e.preventDefault();
      void behaviour.decide(key, input.value);
    });
    input.addEventListener("keydown", (e) => {
      if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) form.requestSubmit();
    });
    resolveButton.addEventListener("click", () => void behaviour.resolve(tab, group, item));

    return {
      view,
      kind: item.kind,
      input,
      saveButton,
      resolveButton,
      resolveChat: view.querySelector(".resolve-chat"),
      resolveProgress: view.querySelector(".resolve-progress"),
      status: view.querySelector(".status"),
      shownProgress: null,
    };
  }

  // the steps are built again only when a step or the status is new, so the scroll stays when nothing changes
  function updateProgress(card, progress) {
    card.resolveProgress.hidden = !progress;
    const shown = progress ? `${progress.status}/${progress.steps.length}` : null;
    if (!progress || shown === card.shownProgress) return;
    card.shownProgress = shown;
    const state = card.resolveProgress.querySelector(".state");
    state.textContent = PROGRESS_STATUS[progress.status] ?? `Failed to load the progress: ${progress.error}`;
    state.className = `state ${progress.status}`;
    const steps = card.resolveProgress.querySelector(".steps");
    steps.replaceChildren(
      ...progress.steps.map((step) => {
        const li = document.createElement("li");
        li.className = step.kind;
        appendText(li, step.text);
        return li;
      }),
    );
    steps.scrollTop = steps.scrollHeight;
  }

  function updateCard(key) {
    const card = cards.get(key);
    const { view, input, saveButton, resolveButton, resolveChat, status } = card;
    const saved = behaviour.state.decisions[key] ?? "";
    const resolve = behaviour.state.resolves[key] ?? "";
    const edited = input.value.trim() !== saved;
    view.classList.toggle("decided", saved !== "");
    status.textContent = saved ? "Decided" : "Open";
    saveButton.disabled = !edited;
    resolveButton.hidden = saved === "";
    resolveButton.disabled = edited || resolve !== "";
    resolveButton.textContent = resolve === "" ? "Resolve" : resolve === "starting" ? "Starting…" : "Started";
    resolveChat.hidden = resolve === "" || resolve === "starting";
    resolveChat.textContent = `The agent ${resolve} is in the Cursor sidebar. Open https://cursor.com/agents/${resolve}. If the sidebar hides it, turn on Filter, Source, API.`;
    updateProgress(card, behaviour.state.progress[key]);
  }

  function render() {
    const { tabs, decisions, loaded } = behaviour.state;
    error.textContent = behaviour.state.error;
    error.hidden = !behaviour.state.error;
    if (!loaded) return;
    if (!built) build();
    if (!selected) selected = (tabs.find((tab) => tab.groups.some((group) => group.items.length > 0)) ?? tabs[0])?.id ?? "";
    kindsBar.hidden = false;
    for (const { value, button } of kindButtons) button.classList.toggle("on", value === kind);
    for (const [key, card] of cards) {
      updateCard(key);
      card.view.hidden = kind !== "" && card.kind !== kind;
    }
    const shown = tabs.map((tab) => withKind(tab, kind));
    for (const tab of shown) {
      const panel = panels.get(tab.id);
      placeCards(tab);
      panel.panel.hidden = tab.id !== selected;
      panel.button.classList.toggle("on", tab.id === selected);
      panel.empty.hidden = tab.groups.some((group) => group.items.length > 0);
      const open = groupsByStatus(tab, decisions).open.reduce((sum, group) => sum + group.items.length, 0);
      panel.badge.textContent = open;
      panel.badge.hidden = open === 0;
    }
    const tab = shown.find((item) => item.id === selected);
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
  void behaviour.load().then(behaviour.refreshProgress);
  setInterval(() => void behaviour.refreshProgress(), PROGRESS_MS);
}
