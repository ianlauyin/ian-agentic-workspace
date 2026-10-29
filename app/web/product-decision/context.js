const PAGE_NAME = "product-decisions";

export function decisionKey(group, item) {
  return `${group.project}/${group.spec}/${item.id}`;
}

// onChange runs after each change of the state, so the page can render again
export function createProductDecision(api, loadGroups, onChange = () => {}) {
  const state = { groups: [], decisions: {}, loaded: false, error: "" };

  async function load() {
    try {
      const [groups, data] = await Promise.all([loadGroups(), api.loadPage(PAGE_NAME)]);
      state.groups = groups;
      state.decisions = data ?? {};
      state.loaded = true;
      state.error = "";
    } catch (e) {
      state.error = `Failed to load the product decisions: ${e.message}`;
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
