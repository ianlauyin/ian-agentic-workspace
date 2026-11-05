const PAGE_NAME = "todo-items";

export const SECTIONS = ["pending", "ongoing", "reviewing"];

function card(title, detail, deadline) {
  return { title, detail: detail.trim(), deadline };
}

// the old data has the lists important and pending, or only items, and the cards have text in place of title
export function normalize(data) {
  const old = [...(data?.important ?? []), ...(data?.items ?? [])];
  const lists = { pending: [...old, ...(data?.pending ?? [])], ongoing: data?.ongoing ?? [], reviewing: data?.reviewing ?? [] };
  return Object.fromEntries(
    SECTIONS.map((section) => [
      section,
      lists[section].map((item) => card(item.title ?? item.text ?? "", item.detail ?? "", item.deadline ?? "")),
    ]),
  );
}

// onChange runs after each change of the state, so the page can render again
export function createTodo(api, onChange = () => {}) {
  const state = { lists: { pending: [], ongoing: [], reviewing: [] }, loaded: false, error: "" };

  async function load() {
    try {
      const data = await api.loadPage(PAGE_NAME);
      state.lists = normalize(data);
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

  async function add(section, { title, detail, deadline }) {
    const trimmed = title.trim();
    if (trimmed) await save({ ...state.lists, [section]: [...state.lists[section], card(trimmed, detail, deadline)] });
  }

  async function edit(section, index, { title, detail, deadline }) {
    const trimmed = title.trim();
    if (trimmed) await save({ ...state.lists, [section]: state.lists[section].with(index, card(trimmed, detail, deadline)) });
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
