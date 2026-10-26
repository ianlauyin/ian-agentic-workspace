const PAGE_NAME = "todo-items";

// onChange runs after each change of the state, so the page can render again
export function createTodo(api, onChange = () => {}) {
  const state = { items: [], loaded: false, error: "" };

  async function load() {
    try {
      const data = await api.loadPage(PAGE_NAME);
      state.items = data?.items ?? [];
      state.loaded = true;
      state.error = "";
    } catch (e) {
      state.error = `Failed to load my TODOs: ${e.message}`;
    }
    onChange();
  }

  // items changes only after the save, so a failed save keeps the last saved list
  async function save(next) {
    try {
      await api.savePage(PAGE_NAME, { items: next });
      state.items = next;
      state.error = "";
    } catch (e) {
      state.error = `Failed to save: ${e.message}`;
    }
    onChange();
  }

  async function add(text) {
    const trimmed = text.trim();
    if (trimmed) await save([...state.items, { text: trimmed }]);
  }

  async function edit(index, text) {
    const trimmed = text.trim();
    if (trimmed) await save(state.items.with(index, { ...state.items[index], text: trimmed }));
  }

  async function remove(index) {
    await save(state.items.toSpliced(index, 1));
  }

  // gap is the index of the gap between the rows: 0 is before the first row, items.length is after the last row
  async function move(from, gap) {
    if (gap === from || gap === from + 1) return;
    const next = state.items.toSpliced(from, 1);
    next.splice(gap > from ? gap - 1 : gap, 0, state.items[from]);
    await save(next);
  }

  return { state, load, add, edit, remove, move };
}
