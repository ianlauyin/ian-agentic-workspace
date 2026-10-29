import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { createProductDecision, decisionKey } from "../../product-decision/context.js";

const GROUPS = [{ project: "my-app", spec: "01_login", title: "Login", items: [{ id: 1, item: "Sign-in" }] }];
const KEY = "my-app/01_login/1";

function fakeApi(saved) {
  const saves = [];
  let failSave = false;
  const api = {
    loadPage: async () => saved,
    savePage: async (name, data) => {
      if (failSave) throw new Error("db down");
      saves.push({ name, data });
    },
  };
  return { api, saves, failSaves: () => (failSave = true) };
}

const loadGroups = async () => GROUPS;

describe("createProductDecision", () => {
  it("makes the key from the project, the spec, and the item id", () => {
    assert.equal(decisionKey(GROUPS[0], GROUPS[0].items[0]), KEY);
  });

  it("loads the groups and the saved decisions", async () => {
    const decision = createProductDecision(fakeApi({ [KEY]: "a" }).api, loadGroups);
    await decision.load();
    assert.equal(decision.state.loaded, true);
    assert.deepEqual(decision.state.groups, GROUPS);
    assert.deepEqual(decision.state.decisions, { [KEY]: "a" });
  });

  it("loads no decisions when there is no save", async () => {
    const decision = createProductDecision(fakeApi(null).api, loadGroups);
    await decision.load();
    assert.deepEqual(decision.state.decisions, {});
  });

  it("shows the error when the load fails", async () => {
    const decision = createProductDecision(fakeApi(null).api, async () => Promise.reject(new Error("404")));
    await decision.load();
    assert.equal(decision.state.loaded, false);
    assert.equal(decision.state.error, "Failed to load the product decisions: 404");
  });

  it("saves all decisions as product-decisions", async () => {
    const fake = fakeApi({ "my-app/01_login/2": "b" });
    const decision = createProductDecision(fake.api, loadGroups);
    await decision.load();
    await decision.decide(KEY, "  a  ");
    assert.deepEqual(fake.saves, [{ name: "product-decisions", data: { "my-app/01_login/2": "b", [KEY]: "a" } }]);
  });

  it("removes the decision when the text is empty", async () => {
    const fake = fakeApi({ [KEY]: "a" });
    const decision = createProductDecision(fake.api, loadGroups);
    await decision.load();
    await decision.decide(KEY, "   ");
    assert.deepEqual(decision.state.decisions, {});
    assert.deepEqual(fake.saves, [{ name: "product-decisions", data: {} }]);
  });

  it("keeps the last saved decisions when a save fails", async () => {
    const fake = fakeApi({ [KEY]: "a" });
    const decision = createProductDecision(fake.api, loadGroups);
    await decision.load();
    fake.failSaves();
    await decision.decide(KEY, "b");
    assert.deepEqual(decision.state.decisions, { [KEY]: "a" });
    assert.equal(decision.state.error, "Failed to save: db down");
  });
});
