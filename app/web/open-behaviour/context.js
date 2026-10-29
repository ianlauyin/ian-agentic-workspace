const PAGE_NAME = "open-behaviour-decisions";

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

// onChange runs after each change of the state, so the page can render again
export function createOpenBehaviour(api, onChange = () => {}) {
  // resolves maps a decision key to "starting" or to the Orca terminal handle
  const state = { tabs: [], decisions: {}, resolves: {}, loaded: false, error: "" };

  async function load() {
    try {
      const [tabs, data] = await Promise.all([api.loadOpenBehaviour(), api.loadPage(PAGE_NAME)]);
      state.tabs = tabs;
      state.decisions = data ?? {};
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
      await api.savePage(PAGE_NAME, next);
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
    onChange();
    try {
      const { terminal } = await api.resolveOpenBehaviour({
        tab: tab.id,
        spec: group.spec,
        id: item.id,
        decision,
      });
      state.resolves = { ...state.resolves, [key]: terminal };
      state.error = "";
    } catch (e) {
      const { [key]: _, ...rest } = state.resolves;
      state.resolves = rest;
      state.error = `Failed to start the resolve: ${e.message}`;
    }
    onChange();
  }

  return { state, load, decide, resolve };
}
