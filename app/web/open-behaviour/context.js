const DECISIONS_PAGE = "open-behaviour-decisions";
const RESOLVES_PAGE = "open-behaviour-resolves";
const ENDED = ["finished", "failed", "stopped", "error"];

export function decisionKey(tab, group, item) {
  return `${tab.id}/${group.spec}/${item.id}`;
}

export function withKind(tab, kind) {
  if (!kind) return tab;
  const groups = tab.groups
    .map((group) => ({ ...group, items: group.items.filter((item) => item.kind === kind) }))
    .filter((group) => group.items.length > 0);
  return { ...tab, groups };
}

export function groupsByStatus(tab, decisions) {
  const open = [];
  const decided = [];
  for (const group of tab.groups) {
    const openItems = [];
    const decidedItems = [];
    for (const item of group.items) {
      if (decisions[decisionKey(tab, group, item)]) decidedItems.push(item);
      else openItems.push(item);
    }
    if (openItems.length > 0) open.push({ ...group, items: openItems });
    if (decidedItems.length > 0) decided.push({ ...group, items: decidedItems });
  }
  return { open, decided };
}

// the in-progress value is not a started chat, so a reload can start the item again
function savedResolves(resolves) {
  return Object.fromEntries(Object.entries(resolves).filter(([, value]) => value && value !== "starting"));
}

// onChange runs after each change of the state, so the page can render again
export function createOpenBehaviour(api, onChange = () => {}) {
  // resolves maps a decision key to "starting" or to the Cursor chat id
  const state = { tabs: [], decisions: {}, resolves: {}, progress: {}, loaded: false, error: "" };
  let refreshing = false;

  async function load() {
    try {
      const [tabs, decisions, resolves] = await Promise.all([
        api.loadOpenBehaviour(),
        api.loadPage(DECISIONS_PAGE),
        api.loadPage(RESOLVES_PAGE),
      ]);
      state.tabs = tabs;
      state.decisions = decisions ?? {};
      state.resolves = resolves ?? {};
      state.loaded = true;
      state.error = "";
    } catch (e) {
      state.error = `Failed to load the open behaviour: ${e.message}`;
    }
    onChange();
  }

  // decisions changes only after the save, so a failed save keeps the last saved decisions
  async function decide(key, text) {
    const trimmed = text.trim();
    const { [key]: _, ...rest } = state.decisions;
    const next = trimmed ? { ...rest, [key]: trimmed } : rest;
    try {
      await api.savePage(DECISIONS_PAGE, next);
      state.decisions = next;
      state.error = "";
    } catch (e) {
      state.error = `Failed to save: ${e.message}`;
    }
    onChange();
  }

  async function resolve(tab, group, item) {
    const key = decisionKey(tab, group, item);
    const decision = state.decisions[key];
    if (!decision || state.resolves[key]) return;
    state.resolves = { ...state.resolves, [key]: "starting" };
    const { [key]: _ended, ...progress } = state.progress;
    state.progress = progress;
    onChange();
    let chat;
    try {
      ({ chat } = await api.resolveOpenBehaviour({
        tab: tab.id,
        spec: group.spec,
        id: item.id,
        decision,
      }));
    } catch (e) {
      const { [key]: _, ...rest } = state.resolves;
      state.resolves = rest;
      state.error = `Failed to start the resolve: ${e.message}`;
      onChange();
      return;
    }
    const next = { ...state.resolves, [key]: chat };
    try {
      await api.savePage(RESOLVES_PAGE, savedResolves(next));
      state.resolves = next;
      state.error = "";
    } catch (e) {
      const { [key]: _, ...rest } = state.resolves;
      state.resolves = rest;
      state.error = `Failed to save: ${e.message}`;
    }
    onChange();
  }

  // the returned progress has the status "running", so refreshProgress loads the new steps
  async function followup(key, text) {
    const chat = state.resolves[key];
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
    const running = Object.entries(state.resolves).filter(
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
    // a failed agent leaves the item as it was, so the user can start it again
    const failed = loaded.filter(([, progress]) => progress.status === "failed").map(([key]) => key);
    if (failed.length > 0) {
      const remove = (resolves) => Object.fromEntries(Object.entries(resolves).filter(([key]) => !failed.includes(key)));
      try {
        await api.savePage(RESOLVES_PAGE, savedResolves(remove(state.resolves)));
        state.resolves = remove(state.resolves);
      } catch (e) {
        state.error = `Failed to save: ${e.message}`;
      }
    }
    refreshing = false;
    onChange();
  }

  return { state, load, decide, resolve, followup, refreshProgress };
}
