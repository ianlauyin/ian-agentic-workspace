import { assignKey, createNotes } from "./context.js";

const CSS = "/notes/index.css";
const PROGRESS_MS = 3000;
const REPLY_STATUS = ["finished", "stopped"];
const PROGRESS_STATUS = {
  running: "The agent is working.",
  finished: "The agent finished.",
  failed: "The agent failed.",
  stopped: "The agent stopped before it finished.",
};

export function mount(main, api) {
  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = CSS;
  document.head.append(link);

  const notes = createNotes(api, render);
  let selected = "";
  let shownTab = "";
  let shownKeys = "";
  let cards = new Map();

  main.innerHTML = `
    <div id="notes" class="notes-page">
      <h1>Notes</h1>
      <div class="notes-page-tabs"></div>
      <p class="error" hidden></p>
      <form class="notes-page-add" hidden>
        <label>New note
          <textarea class="g-text-input" rows="3" placeholder="Write a note or a task"></textarea>
        </label>
        <button class="g-button" type="submit" disabled>Add</button>
      </form>
      <div class="notes-page-list"></div>
    </div>`;
  const tabsBar = main.querySelector(".notes-page-tabs");
  const error = main.querySelector(".error");
  const addForm = main.querySelector(".notes-page-add");
  const addInput = addForm.querySelector("textarea");
  const addButton = addForm.querySelector("button");
  const list = main.querySelector(".notes-page-list");

  addInput.addEventListener("input", () => (addButton.disabled = !addInput.value.trim()));
  addInput.addEventListener("keydown", (e) => {
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) addForm.requestSubmit();
  });
  addForm.addEventListener("submit", async (e) => {
    e.preventDefault();
    if (await notes.addNote(selected, addInput.value)) {
      addInput.value = "";
      addButton.disabled = true;
    }
  });

  function noteCard(tab, note) {
    const key = assignKey(tab, note);
    const view = document.createElement("article");
    view.className = "notes-page-card";
    view.innerHTML = `
      <p class="text"></p>
      <div class="actions">
        <button class="g-button assign" type="button">Assign task</button>
        <button class="g-button remove" type="button">Delete</button>
      </div>
      <p class="assign-chat" hidden></p>
      <div class="assign-progress" hidden>
        <p class="state"></p>
        <ol class="steps"></ol>
        <form class="followup-form" hidden>
          <label>Follow-up
            <textarea class="g-text-input" rows="2" placeholder="Reply to the agent"></textarea>
          </label>
          <button class="g-button" type="submit" disabled>Send</button>
        </form>
      </div>`;
    view.querySelector(".text").textContent = note.text;
    const followupForm = view.querySelector(".followup-form");
    const followupInput = followupForm.querySelector("textarea");
    const card = {
      view,
      key,
      assignButton: view.querySelector(".assign"),
      removeButton: view.querySelector(".remove"),
      chat: view.querySelector(".assign-chat"),
      progress: view.querySelector(".assign-progress"),
      followupForm,
      followupInput,
      followupButton: followupForm.querySelector("button"),
      sending: false,
      shownProgress: null,
    };
    card.assignButton.addEventListener("click", () => void notes.assign(tab, note));
    card.removeButton.addEventListener("click", () => void notes.removeNote(tab, note));
    followupInput.addEventListener("input", () => updateCard(card));
    followupInput.addEventListener("keydown", (e) => {
      if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) followupForm.requestSubmit();
    });
    followupForm.addEventListener("submit", async (e) => {
      e.preventDefault();
      if (card.sending || !followupInput.value.trim()) return;
      card.sending = true;
      updateCard(card);
      const sent = await notes.followup(key, followupInput.value);
      card.sending = false;
      if (sent) followupInput.value = "";
      updateCard(card);
    });
    return card;
  }

  // the steps are built again only when a step or the status is new, so the scroll stays when nothing changes
  function updateProgress(card, progress) {
    card.progress.hidden = !progress;
    const shown = progress ? `${progress.status}/${progress.steps.length}` : null;
    if (!progress || shown === card.shownProgress) return;
    card.shownProgress = shown;
    const state = card.progress.querySelector(".state");
    state.textContent = PROGRESS_STATUS[progress.status] ?? `Failed to load the progress: ${progress.error}`;
    state.className = `state ${progress.status}`;
    const steps = card.progress.querySelector(".steps");
    steps.replaceChildren(
      ...progress.steps.map((step) => {
        const li = document.createElement("li");
        li.className = step.kind;
        li.textContent = step.text;
        return li;
      }),
    );
    steps.scrollTop = steps.scrollHeight;
  }

  function updateCard(card) {
    const assign = notes.state.assigns[card.key] ?? "";
    const started = assign !== "" && assign !== "starting";
    card.assignButton.disabled = assign !== "";
    card.assignButton.textContent = assign === "" ? "Assign task" : assign === "starting" ? "Starting…" : "Assigned";
    card.chat.hidden = !started;
    card.chat.textContent = `The local agent ${assign} works in the project folder.`;
    const progress = notes.state.progress[card.key];
    updateProgress(card, progress);
    card.followupForm.hidden = !started || !REPLY_STATUS.includes(progress?.status);
    card.followupButton.disabled = card.sending || !card.followupInput.value.trim();
    card.followupButton.textContent = card.sending ? "Sending…" : "Send";
  }

  // the cards are built again only when the tab or its notes change, so the follow-up text stays
  function renderList() {
    const items = notes.state.notes[selected] ?? [];
    const keys = `${selected}|${items.map((note) => note.id).join(",")}`;
    if (keys !== shownKeys) {
      shownKeys = keys;
      cards = new Map(items.map((note) => [note.id, noteCard(selected, note)]));
      list.replaceChildren(...[...cards.values()].map((card) => card.view));
      if (items.length === 0) {
        const empty = document.createElement("p");
        empty.className = "empty";
        empty.textContent = "No note.";
        list.append(empty);
      }
    }
    for (const card of cards.values()) updateCard(card);
  }

  function render() {
    const { projects, loaded } = notes.state;
    error.textContent = notes.state.error;
    error.hidden = !notes.state.error;
    if (!loaded) return;
    if (!selected) selected = projects[0]?.id ?? "";
    if (tabsBar.childElementCount === 0) {
      for (const project of projects) {
        const button = document.createElement("button");
        button.type = "button";
        button.dataset.tab = project.id;
        button.textContent = project.name;
        const badge = document.createElement("span");
        badge.className = "badge";
        button.append(badge);
        button.addEventListener("click", () => {
          selected = project.id;
          render();
        });
        tabsBar.append(button);
      }
    }
    for (const button of tabsBar.children) {
      const id = button.dataset.tab;
      const count = (notes.state.notes[id] ?? []).length;
      button.classList.toggle("on", id === selected);
      button.querySelector(".badge").textContent = count;
      button.querySelector(".badge").hidden = count === 0;
    }
    if (selected !== shownTab) {
      shownTab = selected;
      addInput.value = "";
      addButton.disabled = true;
    }
    addForm.hidden = !selected;
    renderList();
  }

  render();
  void notes.load().then(notes.refreshProgress);
  setInterval(() => void notes.refreshProgress(), PROGRESS_MS);
}
