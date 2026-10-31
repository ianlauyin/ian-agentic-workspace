import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { assignKey, createNotes } from "../../notes/context.js";

function fakeApi(notes = null, assigns = null, progress = {}) {
  const saves = [];
  const assignments = [];
  let failAssign = false;
  const api = {
    loadProjects: async () => [{ id: "my-app", name: "my-app" }],
    loadPage: async (name) => (name === "notes" ? notes : assigns),
    savePage: async (name, data) => {
      saves.push({ name, data });
    },
    assignNote: async (body) => {
      if (failAssign) throw new Error("no agent");
      assignments.push(body);
      return { chat: "chat_1" };
    },
    loadResolveProgress: async (chat) => {
      if (!progress[chat]) throw new Error("The chat has no log.");
      return progress[chat];
    },
    followupResolve: async () => ({ status: "running", steps: [] }),
  };
  return { api, saves, assignments, failAssign: () => (failAssign = true) };
}

describe("notes", () => {
  it("adds a note and saves it", async () => {
    const fake = fakeApi();
    const notes = createNotes(fake.api);
    await notes.load();
    assert.equal(await notes.addNote("my-app", "  Add a route.  "), true);
    assert.equal(notes.state.notes["my-app"][0].text, "Add a route.");
    assert.equal(fake.saves[0].name, "notes");
    assert.equal(await notes.addNote("my-app", "  "), false);
  });

  it("assigns a note and saves the chat", async () => {
    const fake = fakeApi();
    const notes = createNotes(fake.api);
    await notes.load();
    await notes.addNote("my-app", "Add a route.");
    const note = notes.state.notes["my-app"][0];
    await notes.assign("my-app", note);
    assert.deepEqual(fake.assignments, [{ tab: "my-app", text: "Add a route." }]);
    assert.equal(notes.state.assigns[assignKey("my-app", note)], "chat_1");
    assert.deepEqual(fake.saves.at(-1), { name: "notes-assigns", data: { [assignKey("my-app", note)]: "chat_1" } });
  });

  it("keeps the note open when the start fails", async () => {
    const fake = fakeApi();
    const notes = createNotes(fake.api);
    await notes.load();
    await notes.addNote("my-app", "Add a route.");
    fake.failAssign();
    await notes.assign("my-app", notes.state.notes["my-app"][0]);
    assert.deepEqual(notes.state.assigns, {});
    assert.match(notes.state.error, /no agent/);
  });

  it("removes the assign when the agent fails", async () => {
    const key = "my-app/n1";
    const fake = fakeApi({ "my-app": [{ id: "n1", text: "x" }] }, { [key]: "c1" }, { c1: { status: "failed", steps: [] } });
    const notes = createNotes(fake.api);
    await notes.load();
    await notes.refreshProgress();
    assert.deepEqual(notes.state.assigns, {});
    assert.equal(notes.state.progress[key].status, "failed");
  });

  it("deletes a note with its assign", async () => {
    const key = "my-app/n1";
    const fake = fakeApi({ "my-app": [{ id: "n1", text: "x" }] }, { [key]: "c1" });
    const notes = createNotes(fake.api);
    await notes.load();
    await notes.removeNote("my-app", { id: "n1", text: "x" });
    assert.deepEqual(notes.state.notes["my-app"], []);
    assert.deepEqual(notes.state.assigns, {});
  });
});
