const PAGE_NAME = "todo-items";

export const SECTIONS = ["important", "pending"];

// onChange runs after each change of the state, so the page can render again
export function createTodo(api, onChange = () => {}) {
  const state = { lists: { important: [], pending: [] }, loaded: false, error: "" };

  async function load() {
    try {
      const data = await api.loadPage(PAGE_NAME);
      // the old data has only one list, in items
      state.lists = { important: data?.important ?? [], pending: data?.pending ?? data?.items ?? [] };
      state.loaded = true;
      state.error = "";
    } catch (e) {
      state.error = `Failed to load my TODOs: ${e.message}`;
    }
    onChange();
  }

  // lists changes only after the save, so a failed save keeps the last saved lists
  async function save(next) {
    try {
      await api.savePage(PAGE_NAME, next);
      state.lists = next;
      state.error = "";
    } catch (e) {
      state.error = `Failed to save: ${e.message}`;
    }
    onChange();
  }

  async function add(section, text) {
    const trimmed = text.trim();
    if (trimmed) await save({ ...state.lists, [section]: [...state.lists[section], { text: trimmed }] });
  }

  async function edit(section, index, text) {
    const trimmed = text.trim();
    const items = state.lists[section];
    if (trimmed) await save({ ...state.lists, [section]: items.with(index, { ...items[index], text: trimmed }) });
  }

  async function remove(section, index) {
    await save({ ...state.lists, [section]: state.lists[section].toSpliced(index, 1) });
  }

  // gap is the index of the gap between the rows of the target section: 0 is before the first row, items.length is after the last row
  async function move(fromSection, from, toSection, gap) {
    if (fromSection === toSection) {
      if (gap === from || gap === from + 1) return;
      const items = state.lists[fromSection];
      const next = items.toSpliced(from, 1);
      next.splice(gap > from ? gap - 1 : gap, 0, items[from]);
      await save({ ...state.lists, [fromSection]: next });
      return;
    }
    const item = state.lists[fromSection][from];
    await save({
      ...state.lists,
      [fromSection]: state.lists[fromSection].toSpliced(from, 1),
      [toSection]: state.lists[toSection].toSpliced(gap, 0, item),
    });
  }

  return { state, load, add, edit, remove, move };
}
