const HIDDEN_FIELDS = ["id"];
const HKT_OFFSET_MS = 8 * 60 * 60 * 1000;

export function hktToday() {
  return new Date(Date.now() + HKT_OFFSET_MS).toISOString().slice(0, 10);
}

export function pageCount(total, pageSize) {
  return Math.max(1, Math.ceil(total / pageSize));
}

// each field of an event owns a column, and each field of its extra owns a column after them
export function logColumns(events) {
  const fields = new Set();
  const extras = new Set();
  for (const event of events) {
    for (const key of Object.keys(event)) if (key !== "extra" && !HIDDEN_FIELDS.includes(key)) fields.add(key);
    for (const key of Object.keys(event.extra ?? {})) extras.add(key);
  }
  return [
    ...[...fields].map((key) => ({ label: key, key, extra: false })),
    ...[...extras].map((key) => ({ label: `extra.${key}`, key, extra: true })),
  ];
}

export function cellText(event, column) {
  const value = column.extra ? event.extra?.[column.key] : event[column.key];
  if (value === undefined || value === null) return "";
  if (!column.extra && column.key === "ts") return String(value).slice(0, 19).replace("T", " ");
  return typeof value === "object" ? JSON.stringify(value) : String(value);
}

// onChange runs after each change of the state, so the page can render again
export function createLogs(api, onChange = () => {}) {
  const state = { apps: [], app: "", date: hktToday(), result: null, loading: false, loaded: false, error: "" };
  let requestId = 0;

  async function loadApps() {
    try {
      state.apps = await api.loadLogApps();
      state.app = state.apps[0]?.id ?? "";
      state.loaded = true;
      state.error = "";
    } catch (e) {
      state.error = `Failed to load the apps: ${e.message}`;
    }
    onChange();
    if (state.app) await show(state.app, state.date, 1);
  }

  // a slow answer for an old tab or page does not replace a newer one
  async function show(app, date, page) {
    const id = ++requestId;
    state.app = app;
    state.date = date;
    state.loading = true;
    onChange();
    try {
      const result = await api.loadLogs({ app, date, page });
      if (id !== requestId) return;
      state.result = { ...result, app, date };
      state.error = "";
    } catch (e) {
      if (id !== requestId) return;
      state.result = null;
      state.error = `Failed to load the logs: ${e.message}`;
    }
    state.loading = false;
    onChange();
  }

  return { state, loadApps, show };
}
