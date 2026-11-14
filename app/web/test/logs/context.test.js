import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { cellText, createLogs, logColumns, pageCount } from "../../logs/context.js";

function fakeApi() {
  const calls = [];
  const api = {
    loadLogApps: async () => [
      {
        id: "one",
        name: "one",
        apps: [
          { id: "my-app", name: "my-app" },
          { id: "other-app", name: "other-app" },
        ],
      },
      { id: "two", name: "two", apps: [{ id: "third-app", name: "third-app" }] },
    ],
    loadLogs: async (params) => {
      calls.push(params);
      return { page: params.page, page_size: 50, total: 120, events: [] };
    },
  };
  return { api, calls };
}

describe("logs", () => {
  it("makes a column for each field and each context field, with info last", () => {
    const events = [
      { ts: "t1", id: "i1", level: "info", context: { ip: "1.1.1.1" }, info: { "body.a": 1 } },
      { ts: "t2", action: "API/GET:/x", context: { user_agent: "ua" }, info: { code: 5 } },
    ];
    const columns = logColumns(events);
    assert.deepEqual(columns.map((c) => c.label), ["ts", "level", "action", "context.ip", "context.user_agent", "info"]);
    assert.equal(cellText(events[0], columns[3]), "1.1.1.1");
    assert.equal(cellText(events[0], columns[4]), "");
    assert.equal(cellText(events[1], columns[4]), "ua");
    assert.equal(cellText(events[1], columns[1]), "");
    assert.equal(cellText({ ts: "2026-10-31T22:08:03.077818073+08:00" }, { key: "ts" }), "2026-10-31 22:08:03");
    assert.equal(cellText(events[1], columns[5]), '{"code":5}');
  });

  it("shows plain context text and an empty cell for a null field", () => {
    const events = [{ id: "i1", level: null, context: "plain" }];
    const columns = logColumns(events);
    assert.deepEqual(
      columns.map((c) => c.label),
      ["level", "context"],
    );
    assert.equal(cellText(events[0], columns[0]), "");
    assert.equal(cellText(events[0], columns[1]), "plain");
    assert.deepEqual(
      logColumns([{ level: "info", context: null }]).map((c) => c.label),
      ["level"],
    );
  });

  it("counts the pages", () => {
    assert.equal(pageCount(0, 50), 1);
    assert.equal(pageCount(50, 50), 1);
    assert.equal(pageCount(51, 50), 2);
  });

  it("selects the first server and its first app", async () => {
    const fake = fakeApi();
    const logs = createLogs(fake.api);
    await logs.loadApps();
    assert.equal(logs.state.server, "one");
    assert.equal(logs.state.app, "my-app");
    assert.equal(fake.calls[0].page, 1);
    assert.equal(logs.state.result.total, 120);
  });

  it("loads another page and another app", async () => {
    const fake = fakeApi();
    const logs = createLogs(fake.api);
    await logs.loadApps();
    await logs.show("one", "my-app", logs.state.date, 2);
    assert.equal(logs.state.result.page, 2);
    await logs.show("one", "other-app", "2026-10-30", 1);
    assert.deepEqual(fake.calls.at(-1), { server: "one", app: "other-app", date: "2026-10-30", page: 1, page_size: 50, filters: "[]" });
  });

  it("selects the first app of a server and keeps the filters", async () => {
    const fake = fakeApi();
    const logs = createLogs(fake.api);
    await logs.loadApps();
    await logs.addFilter("level", "warn");
    await logs.show("one", "other-app", logs.state.date, 2);
    await logs.selectServer("two");
    assert.equal(logs.state.server, "two");
    assert.equal(logs.state.app, "third-app");
    assert.deepEqual(logs.state.filters, [{ field: "level", value: "warn" }]);
    assert.equal(fake.calls.at(-1).page, 1);
    assert.equal(fake.calls.at(-1).server, "two");
    assert.equal(fake.calls.at(-1).filters, '[{"field":"level","value":"warn"}]');
    const calls = fake.calls.length;
    await logs.selectServer("two");
    assert.equal(fake.calls.length, calls);
  });

  it("does not load events for a server that has no app", async () => {
    let calls = 0;
    const logs = createLogs({
      loadLogApps: async () => [
        { id: "one", name: "one", apps: [{ id: "my-app", name: "my-app" }] },
        { id: "two", name: "two", apps: [] },
      ],
      loadLogs: async () => {
        calls += 1;
        return { page: 1, page_size: 50, total: 1, events: [{ level: "info" }] };
      },
    });
    await logs.loadApps();
    assert.equal(calls, 1);
    await logs.selectServer("two");
    assert.equal(logs.state.server, "two");
    assert.equal(logs.state.app, "");
    assert.equal(logs.state.result, null);
    assert.equal(calls, 1);
  });

  it("loads page 1 again when the rows or the filters change", async () => {
    const fake = fakeApi();
    const logs = createLogs(fake.api);
    await logs.loadApps();
    await logs.show("one", "my-app", logs.state.date, 2);
    await logs.setPageSize(100);
    assert.deepEqual([fake.calls.at(-1).page, fake.calls.at(-1).page_size], [1, 100]);
    await logs.addFilter("level", "warn");
    await logs.addFilter("level", "error");
    assert.deepEqual(logs.state.filters, [{ field: "level", value: "error" }]);
    assert.equal(fake.calls.at(-1).filters, '[{"field":"level","value":"error"}]');
    await logs.removeFilter(0);
    assert.equal(fake.calls.at(-1).filters, "[]");
  });

  it("keeps the newest answer when an old answer is slow", async () => {
    const resolvers = [];
    const api = {
      loadLogApps: async () => [{ id: "one", name: "one", apps: [{ id: "my-app", name: "my-app" }] }],
      loadLogs: (params) => new Promise((resolve) => resolvers.push(() => resolve({ page: params.page, page_size: 50, total: 1, events: [] }))),
    };
    const logs = createLogs(api);
    const first = logs.show("one", "my-app", "2026-10-30", 1);
    const second = logs.show("one", "my-app", "2026-10-30", 2);
    resolvers[1]();
    await second;
    resolvers[0]();
    await first;
    assert.equal(logs.state.result.page, 2);
  });

  it("does not load events when there is no app", async () => {
    let calls = 0;
    const logs = createLogs({
      loadLogApps: async () => [],
      loadLogs: async () => {
        calls += 1;
        return { page: 1, page_size: 50, total: 0, events: [] };
      },
    });
    await logs.loadApps();
    assert.equal(logs.state.app, "");
    assert.equal(logs.state.loaded, true);
    assert.equal(calls, 0);
  });

  it("shows the error when the apps fail to load", async () => {
    const logs = createLogs({ loadLogApps: async () => { throw new Error("down"); } });
    await logs.loadApps();
    assert.equal(logs.state.loaded, false);
    assert.equal(logs.state.error, "Failed to load the apps: down");
  });

  it("shows the error when the load fails", async () => {
    const logs = createLogs({ loadLogApps: async () => [], loadLogs: async () => { throw new Error("down"); } });
    await logs.show("one", "my-app", "2026-10-30", 1);
    assert.match(logs.state.error, /down/);
    assert.equal(logs.state.result, null);
  });
});
