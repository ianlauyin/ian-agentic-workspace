import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { createOpenBehaviour, decisionKey, groupsByStatus, withKind } from "../../open-behaviour/context.js";

const TABS = [
  {
    id: "my-app",
    name: "my-app",
    groups: [{ spec: "01_login", title: "Login", items: [{ id: 1, item: "Sign-in" }] }],
    errors: [],
  },
];
const KEY = "my-app/01_login/1";

function fakeApi(saved, tabs = TABS) {
  const saves = [];
  const resolves = [];
  let failSave = false;
  let failLoad = "";
  let failResolve = false;
  const api = {
    resolveOpenBehaviour: async (body) => {
      if (failResolve) throw new Error("no agent");
      resolves.push(body);
      return { terminal: "term_1" };
    },
    loadOpenBehaviour: async () => {
      if (failLoad) throw new Error(failLoad);
      return tabs;
    },
    loadPage: async (name) => {
      if (name === "open-behaviour-decisions") return saved;
      throw new Error(name);
    },
    savePage: async (name, data) => {
      if (failSave) throw new Error("db down");
      saves.push({ name, data });
    },
  };
  return {
    api,
    saves,
    resolves,
    failSaves: () => (failSave = true),
    failLoads: (message) => (failLoad = message),
    failResolves: () => (failResolve = true),
  };
}

describe("decisionKey", () => {
  it("makes the key from the tab id, the spec, and the item id", () => {
    const tab = TABS[0];
    assert.equal(decisionKey(tab, tab.groups[0], tab.groups[0].items[0]), KEY);
  });

  it("keeps the package in the spec path", () => {
    const group = { spec: "site/01_login" };
    assert.equal(decisionKey(TABS[0], group, { id: 2 }), "my-app/site/01_login/2");
  });
});

describe("createOpenBehaviour", () => {
  it("loads the tabs and the saved decisions", async () => {
    const behaviour = createOpenBehaviour(fakeApi({ [KEY]: "a" }).api);
    await behaviour.load();
    assert.equal(behaviour.state.loaded, true);
    assert.deepEqual(behaviour.state.tabs, TABS);
    assert.deepEqual(behaviour.state.decisions, { [KEY]: "a" });
  });

  it("loads no decisions when there is no save", async () => {
    const behaviour = createOpenBehaviour(fakeApi(null).api);
    await behaviour.load();
    assert.deepEqual(behaviour.state.decisions, {});
  });

  it("shows the error when the load fails", async () => {
    const fake = fakeApi(null);
    fake.failLoads("500");
    const behaviour = createOpenBehaviour(fake.api);
    await behaviour.load();
    assert.equal(behaviour.state.loaded, false);
    assert.equal(behaviour.state.error, "Failed to load the open behaviour: 500");
  });

  it("saves all decisions as open-behaviour-decisions", async () => {
    const fake = fakeApi({ "my-app/01_login/2": "b" });
    const behaviour = createOpenBehaviour(fake.api);
    await behaviour.load();
    await behaviour.decide(KEY, "  a  ");
    assert.deepEqual(fake.saves, [{ name: "open-behaviour-decisions", data: { "my-app/01_login/2": "b", [KEY]: "a" } }]);
  });

  it("removes the decision when the text is empty", async () => {
    const fake = fakeApi({ [KEY]: "a" });
    const behaviour = createOpenBehaviour(fake.api);
    await behaviour.load();
    await behaviour.decide(KEY, "   ");
    assert.deepEqual(behaviour.state.decisions, {});
    assert.deepEqual(fake.saves, [{ name: "open-behaviour-decisions", data: {} }]);
  });

  it("keeps the last saved decisions when a save fails", async () => {
    const fake = fakeApi({ [KEY]: "a" });
    const behaviour = createOpenBehaviour(fake.api);
    await behaviour.load();
    fake.failSaves();
    await behaviour.decide(KEY, "b");
    assert.deepEqual(behaviour.state.decisions, { [KEY]: "a" });
    assert.equal(behaviour.state.error, "Failed to save: db down");
  });
});

describe("resolve", () => {
  const tab = TABS[0];
  const group = tab.groups[0];
  const item = group.items[0];

  it("starts the resolve with the saved decision and keeps the terminal", async () => {
    const fake = fakeApi({ [KEY]: "a" });
    const behaviour = createOpenBehaviour(fake.api);
    await behaviour.load();
    await behaviour.resolve(tab, group, item);
    assert.deepEqual(fake.resolves, [{ tab: "my-app", spec: "01_login", id: 1, decision: "a" }]);
    assert.deepEqual(behaviour.state.resolves, { [KEY]: "term_1" });
  });

  it("does not start an item with no decision or an item that started", async () => {
    const fake = fakeApi(null);
    const behaviour = createOpenBehaviour(fake.api);
    await behaviour.load();
    await behaviour.resolve(tab, group, item);
    behaviour.state.decisions = { [KEY]: "a" };
    await behaviour.resolve(tab, group, item);
    await behaviour.resolve(tab, group, item);
    assert.equal(fake.resolves.length, 1);
  });

  it("shows the error and lets the user try again when the start fails", async () => {
    const fake = fakeApi({ [KEY]: "a" });
    const behaviour = createOpenBehaviour(fake.api);
    await behaviour.load();
    fake.failResolves();
    await behaviour.resolve(tab, group, item);
    assert.deepEqual(behaviour.state.resolves, {});
    assert.equal(behaviour.state.error, "Failed to start the resolve: no agent");
  });
});

const SPLIT = {
  id: "my-app",
  name: "my-app",
  groups: [
    {
      spec: "01_login",
      title: "Login",
      items: [
        { id: 1, item: "Sign-in" },
        { id: 2, item: "Home" },
      ],
    },
    {
      spec: "02_pay",
      title: "Pay",
      items: [{ id: 1, item: "Price" }],
    },
  ],
};

describe("groupsByStatus", () => {
  it("puts decided items after open items and keeps the source order", () => {
    const { open, decided } = groupsByStatus(SPLIT, {
      "my-app/01_login/2": "a",
      "my-app/02_pay/1": "b",
    });
    assert.deepEqual(open, [{ ...SPLIT.groups[0], items: [SPLIT.groups[0].items[0]] }]);
    assert.deepEqual(decided, [{ ...SPLIT.groups[0], items: [SPLIT.groups[0].items[1]] }, SPLIT.groups[1]]);
  });

  it("omits a group that has no item of that status", () => {
    const { open, decided } = groupsByStatus(SPLIT, { "my-app/02_pay/1": "b" });
    assert.deepEqual(
      open.map((group) => group.spec),
      ["01_login"],
    );
    assert.deepEqual(
      decided.map((group) => group.spec),
      ["02_pay"],
    );
  });
});

const MIXED = {
  id: "my-app",
  name: "my-app",
  groups: [
    {
      spec: "01_login",
      title: "Login",
      items: [
        { id: 1, item: "Sign-in", kind: "bug" },
        { id: 2, item: "Home", kind: "cleanup" },
      ],
    },
    {
      spec: "02_pay",
      title: "Pay",
      items: [{ id: 1, item: "Price", kind: "decision" }],
    },
  ],
  errors: [],
};

describe("withKind", () => {
  it("keeps all items when no kind is given", () => {
    assert.equal(withKind(MIXED, ""), MIXED);
  });

  it("keeps only the items of the kind and omits a group with no item of the kind", () => {
    assert.deepEqual(withKind(MIXED, "bug"), {
      ...MIXED,
      groups: [{ ...MIXED.groups[0], items: [MIXED.groups[0].items[0]] }],
    });
  });
});
