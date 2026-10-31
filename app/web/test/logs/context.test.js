import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { cellText, createLogs, logColumns, pageCount } from "../../logs/context.js";

function fakeApi() {
  const calls = [];
  const api = {
    loadLogApps: async () => [{ id: "my-app", name: "my-app" }, { id: "other-app", name: "other-app" }],
    loadLogs: async (params) => {
      calls.push(params);
      return { page: params.page, page_size: 50, total: 120, events: [] };
    },
  };
  return { api, calls };
}

describe("logs", () => {
  it("makes a column for each field and each extra field", () => {
    const events = [{ ts: "t1", id: "i1", level: "info", extra: { query: { a: 1 } } }, { ts: "t2", action: "GET /x", extra: { code: 5 } }];
    const columns = logColumns(events);
    assert.deepEqual(columns.map((c) => c.label), ["ts", "level", "action", "extra.query", "extra.code"]);
    assert.equal(cellText(events[0], columns[3]), '{"a":1}');
    assert.equal(cellText(events[1], columns[3]), "");
    assert.equal(cellText({ ts: "2026-10-31T22:08:03.077818073+08:00" }, { key: "ts", extra: false }), "2026-10-31 22:08:03");
    assert.equal(cellText(events[1], columns[4]), "5");
  });

  it("counts the pages", () => {
    assert.equal(pageCount(0, 50), 1);
    assert.equal(pageCount(50, 50), 1);
    assert.equal(pageCount(51, 50), 2);
  });

  it("selects the first app and loads its first page", async () => {
    const fake = fakeApi();
    const logs = createLogs(fake.api);
    await logs.loadApps();
    assert.equal(logs.state.app, "my-app");
    assert.equal(fake.calls[0].page, 1);
    assert.equal(logs.state.result.total, 120);
  });

  it("loads another page and another app", async () => {
    const fake = fakeApi();
    const logs = createLogs(fake.api);
    await logs.loadApps();
    await logs.show("my-app", logs.state.date, 2);
    assert.equal(logs.state.result.page, 2);
    await logs.show("other-app", "2026-10-30", 1);
    assert.deepEqual(fake.calls.at(-1), { app: "other-app", date: "2026-10-30", page: 1 });
  });

  it("keeps the newest answer when an old answer is slow", async () => {
    const resolvers = [];
    const api = {
      loadLogApps: async () => [{ id: "my-app", name: "my-app" }],
      loadLogs: (params) => new Promise((resolve) => resolvers.push(() => resolve({ page: params.page, page_size: 50, total: 1, events: [] }))),
    };
    const logs = createLogs(api);
    const first = logs.show("my-app", "2026-10-30", 1);
    const second = logs.show("my-app", "2026-10-30", 2);
    resolvers[1]();
    await second;
    resolvers[0]();
    await first;
    assert.equal(logs.state.result.page, 2);
  });

  it("shows the error when the load fails", async () => {
    const logs = createLogs({ loadLogApps: async () => [], loadLogs: async () => { throw new Error("down"); } });
    await logs.show("my-app", "2026-10-30", 1);
    assert.match(logs.state.error, /down/);
    assert.equal(logs.state.result, null);
  });
});
