const HIDDEN_FIELDS = ["id"];
export const PAGE_SIZES = [50, 100, 500, 1000];
const HKT_OFFSET_MS = 8 * 60 * 60 * 1000;

export function hktToday() {
  return new Date(Date.now() + HKT_OFFSET_MS).toISOString().slice(0, 10);
}

export function pageCount(total, pageSize) {
  return Math.max(1, Math.ceil(total / pageSize));
}

const isObject = (value) => typeof value === "object" && value !== null && !Array.isArray(value);

// each field of an event owns a column, and each field of the context owns a column too
// the context columns are after the other columns, and the info column is the last one
export function logColumns(events) {
  const fields = new Set();
  const contextFields = new Set();
  let hasPlainContext = false;
  for (const event of events) {
    for (const key of Object.keys(event)) {
      if (HIDDEN_FIELDS.includes(key) || key === "context" || key === "info") continue;
      fields.add(key);
    }
    if (isObject(event.context)) for (const key of Object.keys(event.context)) contextFields.add(key);
    else if (event.context !== undefined && event.context !== null) hasPlainContext = true;
  }
  const columns = [...fields].map((key) => ({ label: key, key }));
  if (hasPlainContext) columns.push({ label: "context", key: "context" });
  for (const sub of contextFields) columns.push({ label: `context.${sub}`, key: "context", sub });
  if (events.some((event) => "info" in event)) columns.push({ label: "info", key: "info" });
  return columns;
}

export function cellText(event, column) {
  let value = event[column.key];
  if (column.sub !== undefined) value = isObject(value) ? value[column.sub] : undefined;
  if (value === undefined || value === null) return "";
  if (column.key === "ts") return String(value).slice(0, 19).replace("T", " ");
  return typeof value === "object" ? JSON.stringify(value) : String(value);
}

// onChange runs after each change of the state, so the page can render again
export function createLogs(api, onChange = () => {}) {
  const state = { servers: [], server: "", app: "", date: hktToday(), pageSize: PAGE_SIZES[0], filters: [], result: null, loading: false, loaded: false, error: "" };
  let requestId = 0;

  async function loadApps() {
    try {
      state.servers = await api.loadLogApps();
      state.server = state.servers[0]?.id ?? "";
      state.app = state.servers[0]?.apps[0]?.id ?? "";
      state.loaded = true;
      state.error = "";
    } catch (e) {
      state.error = `Failed to load the apps: ${e.message}`;
    }
    onChange();
    if (state.server && state.app) await show(state.server, state.app, state.date, 1);
  }

  function selectServer(server) {
    if (server === state.server) return;
    const item = state.servers.find((entry) => entry.id === server);
    return show(server, item?.apps[0]?.id ?? "", state.date, 1);
  }

  // a slow answer for an old tab or page does not replace a newer one
  async function show(server, app, date, page) {
    const id = ++requestId;
    state.server = server;
    state.app = app;
    state.date = date;
    if (!app) {
      state.result = null;
      state.loading = false;
      state.error = "";
      onChange();
      return;
    }
    state.loading = true;
    onChange();
    try {
      const result = await api.loadLogs({ server, app, date, page, page_size: state.pageSize, filters: JSON.stringify(state.filters) });
      if (id !== requestId) return;
      state.result = { ...result, server, app, date };
      state.error = "";
    } catch (e) {
      if (id !== requestId) return;
      state.result = null;
      state.error = `Failed to load the logs: ${e.message}`;
    }
    state.loading = false;
    onChange();
  }

  const reload = () => show(state.server, state.app, state.date, 1);

  // a new filter on a field replaces the old filter on that field
  function addFilter(field, value) {
    state.filters = [...state.filters.filter((f) => f.field !== field), { field, value }];
    return reload();
  }

  function removeFilter(index) {
    state.filters = state.filters.filter((_, i) => i !== index);
    return reload();
  }

  function setPageSize(pageSize) {
    state.pageSize = pageSize;
    return reload();
  }

  return { state, loadApps, selectServer, show, reload, addFilter, removeFilter, setPageSize };
}
