const PAGE_NAME = "open-behaviour-decisions";

// a spec folder name is unique only in its package, so the key has the package
export function decisionKey(tab, group, item) {
  const spec = group.package ? `${group.package}/${group.spec}` : group.spec;
  return `${tab.id}/${spec}/${item.id}`;
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
  const state = { tabs: [], decisions: {}, loaded: false, error: "" };

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

  return { state, load, decide };
}
