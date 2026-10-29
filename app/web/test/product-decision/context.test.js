import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { createProductDecision, decisionKey, groupsByStatus } from "../../product-decision/context.js";

const TABS = [
  {
    id: "my-app",
    name: "My App",
    groups: [{ spec: "01_login", title: "Login", source: "spec/a", items: [{ id: 1, item: "Sign-in" }] }],
  },
];
const KEY = "my-app/01_login/1";

function fakeApi(saved, items = TABS) {
  const saves = [];
  let failSave = false;
  let failLoad = "";
  const api = {
    loadPage: async (name) => {
      if (failLoad) throw new Error(failLoad);
      if (name === "product-decision-items") return items;
      if (name === "product-decisions") return saved;
      throw new Error(name);
    },
    savePage: async (name, data) => {
      if (failSave) throw new Error("db down");
      saves.push({ name, data });
    },
  };
  return { api, saves, failSaves: () => (failSave = true), failLoads: (message) => (failLoad = message) };
}

describe("createProductDecision", () => {
  it("makes the key from the tab id, the spec, and the item id", () => {
    const tab = TABS[0];
    assert.equal(decisionKey(tab, tab.groups[0], tab.groups[0].items[0]), KEY);
  });

  it("loads the tabs and the saved decisions", async () => {
    const decision = createProductDecision(fakeApi({ [KEY]: "a" }).api);
    await decision.load();
    assert.equal(decision.state.loaded, true);
    assert.deepEqual(decision.state.tabs, TABS);
    assert.deepEqual(decision.state.decisions, { [KEY]: "a" });
  });

  it("loads no decisions when there is no save", async () => {
    const decision = createProductDecision(fakeApi(null).api);
    await decision.load();
    assert.deepEqual(decision.state.decisions, {});
  });

  it("loads no items when there is no save", async () => {
    const decision = createProductDecision(fakeApi(null, null).api);
    await decision.load();
    assert.deepEqual(decision.state.tabs, []);
  });

  it("shows the error when the load fails", async () => {
    const fake = fakeApi(null);
    fake.failLoads("404");
    const decision = createProductDecision(fake.api);
    await decision.load();
    assert.equal(decision.state.loaded, false);
    assert.equal(decision.state.error, "Failed to load the product decisions: 404");
  });

  it("saves all decisions as product-decisions", async () => {
    const fake = fakeApi({ "my-app/01_login/2": "b" });
    const decision = createProductDecision(fake.api);
    await decision.load();
    await decision.decide(KEY, "  a  ");
    assert.deepEqual(fake.saves, [{ name: "product-decisions", data: { "my-app/01_login/2": "b", [KEY]: "a" } }]);
  });

  it("removes the decision when the text is empty", async () => {
    const fake = fakeApi({ [KEY]: "a" });
    const decision = createProductDecision(fake.api);
    await decision.load();
    await decision.decide(KEY, "   ");
    assert.deepEqual(decision.state.decisions, {});
    assert.deepEqual(fake.saves, [{ name: "product-decisions", data: {} }]);
  });

  it("keeps the last saved decisions when a save fails", async () => {
    const fake = fakeApi({ [KEY]: "a" });
    const decision = createProductDecision(fake.api);
    await decision.load();
    fake.failSaves();
    await decision.decide(KEY, "b");
    assert.deepEqual(decision.state.decisions, { [KEY]: "a" });
    assert.equal(decision.state.error, "Failed to save: db down");
  });

  it("renames a tab and keeps the id and the groups", async () => {
    const fake = fakeApi({});
    const decision = createProductDecision(fake.api);
    await decision.load();
    await decision.rename("my-app", "  Other  ");
    assert.equal(decision.state.tabs[0].name, "Other");
    assert.equal(decision.state.tabs[0].id, "my-app");
    assert.deepEqual(decision.state.tabs[0].groups, TABS[0].groups);
    assert.deepEqual(fake.saves, [{ name: "product-decision-items", data: [{ ...TABS[0], name: "Other" }] }]);
  });

  it("does not rename a tab when the name is empty", async () => {
    const fake = fakeApi({});
    const decision = createProductDecision(fake.api);
    await decision.load();
    await decision.rename("my-app", "   ");
    assert.equal(decision.state.tabs[0].name, "My App");
    assert.deepEqual(fake.saves, []);
  });

  it("keeps the last tab name when a rename fails", async () => {
    const fake = fakeApi({});
    const decision = createProductDecision(fake.api);
    await decision.load();
    fake.failSaves();
    await decision.rename("my-app", "Other");
    assert.equal(decision.state.tabs[0].name, "My App");
    assert.equal(decision.state.error, "Failed to save: db down");
  });
});

const SPLIT = {
  id: "my-app",
  name: "My App",
  groups: [
    {
      spec: "01_login",
      title: "Login",
      source: "spec/a",
      items: [
        { id: 1, item: "Sign-in" },
        { id: 2, item: "Home" },
      ],
    },
    {
      spec: "02_pay",
      title: "Pay",
      source: "spec/b",
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
