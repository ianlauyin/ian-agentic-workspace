import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { assignKey, createNotes } from "../../notes/context.js";

function fakeApi(notes = null, assigns = null, progress = {}) {
  const saves = [];
  const assignments = [];
  const followups = [];
  let failAssign = false;
  let failSave = false;
  let failFollowup = false;
  const api = {
    loadProjects: async () => [{ id: "my-app", name: "my-app" }],
    loadPage: async (name) => (name === "notes" ? notes : assigns),
    savePage: async (name, data) => {
      if (failSave) throw new Error("db down");
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
    followupResolve: async (chat, text) => {
      if (failFollowup) throw new Error("The agent is still working.");
      followups.push({ chat, text });
      return { status: "running", steps: [{ kind: "reply", text }] };
    },
  };
  return {
    api,
    saves,
    assignments,
    followups,
    failAssign: () => (failAssign = true),
    failSaves: () => (failSave = true),
    failFollowups: () => (failFollowup = true),
  };
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

  it("edits a note and keeps its id and its task", async () => {
    const fake = fakeApi();
    const notes = createNotes(fake.api);
    await notes.load();
    await notes.addNote("my-app", "Add a route.");
    const note = notes.state.notes["my-app"][0];
    await notes.assign("my-app", note);
    assert.equal(await notes.editNote("my-app", note, "  Add two routes.  "), true);
    assert.deepEqual(notes.state.notes["my-app"], [{ id: note.id, text: "Add two routes." }]);
    assert.equal(notes.state.assigns[assignKey("my-app", note)], "chat_1");
    assert.equal(await notes.editNote("my-app", note, "  "), false);
    assert.equal(notes.state.notes["my-app"][0].text, "Add two routes.");
  });

  it("assigns a note and saves the chat", async () => {
    const fake = fakeApi();
    const notes = createNotes(fake.api);
    await notes.load();
    await notes.addNote("my-app", "Add a route.");
    const note = notes.state.notes["my-app"][0];
    await notes.assign("my-app", note);
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

  it("shows the error when the load fails and keeps the notes", async () => {
    const notes = createNotes({
      loadProjects: async () => {
        throw new Error("db down");
      },
      loadPage: async () => ({ "my-app": [{ id: "n1", text: "x" }] }),
      savePage: async () => {},
    });
    await notes.load();
    assert.equal(notes.state.loaded, false);
    assert.deepEqual(notes.state.notes, {});
    assert.equal(notes.state.error, "Failed to load the notes: db down");
  });

  it("keeps the last saved notes when a save fails", async () => {
    const fake = fakeApi();
    const notes = createNotes(fake.api);
    await notes.load();
    await notes.addNote("my-app", "Add a route.");
    const note = notes.state.notes["my-app"][0];
    fake.failSaves();
    assert.equal(await notes.addNote("my-app", "Second."), false);
    assert.equal(await notes.editNote("my-app", note, "Changed."), false);
    await notes.removeNote("my-app", note);
    assert.deepEqual(notes.state.notes["my-app"], [{ id: note.id, text: "Add a route." }]);
    assert.equal(notes.state.error, "Failed to save: db down");
  });

  it("keeps the note open when the assign save fails", async () => {
    const fake = fakeApi();
    const notes = createNotes(fake.api);
    await notes.load();
    await notes.addNote("my-app", "Add a route.");
    fake.failSaves();
    await notes.assign("my-app", notes.state.notes["my-app"][0]);
    assert.deepEqual(notes.state.assigns, {});
    assert.equal(notes.state.error, "Failed to save: db down");
  });

  it("does not remove a stopped assign and keeps a failed assign when the save fails", async () => {
    const key = "my-app/n1";
    const stopped = fakeApi({ "my-app": [{ id: "n1", text: "x" }] }, { [key]: "c1" }, { c1: { status: "stopped", steps: [] } });
    const notes = createNotes(stopped.api);
    await notes.load();
    await notes.refreshProgress();
    assert.deepEqual(notes.state.assigns, { [key]: "c1" });
    assert.deepEqual(stopped.saves, []);

    const failed = fakeApi({ "my-app": [{ id: "n1", text: "x" }] }, { [key]: "c1" }, { c1: { status: "failed", steps: [] } });
    const again = createNotes(failed.api);
    await again.load();
    failed.failSaves();
    await again.refreshProgress();
    assert.deepEqual(again.state.assigns, { [key]: "c1" });
    assert.equal(again.state.error, "Failed to save: db down");
  });

  it("sends a follow-up and sends nothing when the reply cannot start", async () => {
    const key = "my-app/n1";
    const fake = fakeApi({ "my-app": [{ id: "n1", text: "x" }] }, { [key]: "c1" }, { c1: { status: "finished", steps: [] } });
    const notes = createNotes(fake.api);
    await notes.load();
    notes.state.assigns[`${key}-start`] = "starting";
    assert.equal(await notes.followup(key, "   "), false);
    assert.equal(await notes.followup("my-app/missing", "More."), false);
    assert.equal(await notes.followup(`${key}-start`, "More."), false);
    assert.equal(await notes.followup(key, "  More.  "), true);
    assert.deepEqual(fake.followups, [{ chat: "c1", text: "More." }]);
    const kept = notes.state.progress[key];
    fake.failFollowups();
    assert.equal(await notes.followup(key, "Again."), false);
    assert.equal(notes.state.error, "Failed to send the follow-up: The agent is still working.");
    assert.deepEqual(notes.state.progress[key], kept);
  });
});
