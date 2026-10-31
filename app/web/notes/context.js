const NOTES_PAGE = "notes";
const ASSIGNS_PAGE = "notes-assigns";
const ENDED = ["finished", "failed", "stopped", "error"];

export function assignKey(tab, note) {
  return `${tab}/${note.id}`;
}

// the in-progress value is not a started chat, so a reload can assign the note again
function savedAssigns(assigns) {
  return Object.fromEntries(Object.entries(assigns).filter(([, value]) => value && value !== "starting"));
}

// onChange runs after each change of the state, so the page can render again
export function createNotes(api, onChange = () => {}) {
  // assigns maps an assign key to "starting" or to the chat id of the local agent
  const state = { projects: [], notes: {}, assigns: {}, progress: {}, loaded: false, error: "" };
  let refreshing = false;

  async function load() {
    try {
      const [projects, notes, assigns] = await Promise.all([api.loadProjects(), api.loadPage(NOTES_PAGE), api.loadPage(ASSIGNS_PAGE)]);
      state.projects = projects;
      state.notes = notes ?? {};
      state.assigns = assigns ?? {};
      state.loaded = true;
      state.error = "";
    } catch (e) {
      state.error = `Failed to load the notes: ${e.message}`;
    }
    onChange();
  }

  // notes changes only after the save, so a failed save keeps the last saved notes
  async function addNote(tab, text) {
    const trimmed = text.trim();
    if (!trimmed) return false;
    const next = { ...state.notes, [tab]: [...(state.notes[tab] ?? []), { id: crypto.randomUUID(), text: trimmed }] };
    try {
      await api.savePage(NOTES_PAGE, next);
      state.notes = next;
      state.error = "";
      onChange();
      return true;
    } catch (e) {
      state.error = `Failed to save: ${e.message}`;
      onChange();
      return false;
    }
  }

  async function removeNote(tab, note) {
    const key = assignKey(tab, note);
    const next = { ...state.notes, [tab]: (state.notes[tab] ?? []).filter((item) => item.id !== note.id) };
    const { [key]: _, ...assigns } = state.assigns;
    try {
      await api.savePage(NOTES_PAGE, next);
      state.notes = next;
      if (key in state.assigns) {
        await api.savePage(ASSIGNS_PAGE, savedAssigns(assigns));
        state.assigns = assigns;
      }
      state.error = "";
    } catch (e) {
      state.error = `Failed to save: ${e.message}`;
    }
    onChange();
  }

  async function assign(tab, note) {
    const key = assignKey(tab, note);
    if (state.assigns[key]) return;
    state.assigns = { ...state.assigns, [key]: "starting" };
    const { [key]: _ended, ...progress } = state.progress;
    state.progress = progress;
    onChange();
    let chat;
    try {
      ({ chat } = await api.assignNote({ tab, text: note.text }));
    } catch (e) {
      const { [key]: _, ...rest } = state.assigns;
      state.assigns = rest;
      state.error = `Failed to assign the task: ${e.message}`;
      onChange();
      return;
    }
    const next = { ...state.assigns, [key]: chat };
    try {
      await api.savePage(ASSIGNS_PAGE, savedAssigns(next));
      state.assigns = next;
      state.error = "";
    } catch (e) {
      const { [key]: _, ...rest } = state.assigns;
      state.assigns = rest;
      state.error = `Failed to save: ${e.message}`;
    }
    onChange();
  }

  // the returned progress has the status "running", so refreshProgress loads the new steps
  async function followup(key, text) {
    const chat = state.assigns[key];
    const trimmed = text.trim();
    if (!chat || chat === "starting" || !trimmed) return false;
    try {
      const progress = await api.followupResolve(chat, trimmed);
      state.progress = { ...state.progress, [key]: progress };
      state.error = "";
      onChange();
      return true;
    } catch (e) {
      state.error = `Failed to send the follow-up: ${e.message}`;
      onChange();
      return false;
    }
  }

  // an ended agent writes no more steps, so its progress is not loaded again
  async function refreshProgress() {
    const running = Object.entries(state.assigns).filter(
      ([key, chat]) => chat !== "starting" && !ENDED.includes(state.progress[key]?.status),
    );
    if (refreshing || running.length === 0) return;
    refreshing = true;
    const loaded = await Promise.all(
      running.map(async ([key, chat]) => {
        try {
          return [key, await api.loadResolveProgress(chat)];
        } catch (e) {
          return [key, { status: "error", steps: [], error: e.message }];
        }
      }),
    );
    state.progress = { ...state.progress, ...Object.fromEntries(loaded) };
    // a failed agent leaves the note as it was, so the user can assign it again
    const failed = loaded.filter(([, progress]) => progress.status === "failed").map(([key]) => key);
    if (failed.length > 0) {
      const remove = (assigns) => Object.fromEntries(Object.entries(assigns).filter(([key]) => !failed.includes(key)));
      try {
        await api.savePage(ASSIGNS_PAGE, savedAssigns(remove(state.assigns)));
        state.assigns = remove(state.assigns);
      } catch (e) {
        state.error = `Failed to save: ${e.message}`;
      }
    }
    refreshing = false;
    onChange();
  }

  return { state, load, addNote, removeNote, assign, followup, refreshProgress };
}
