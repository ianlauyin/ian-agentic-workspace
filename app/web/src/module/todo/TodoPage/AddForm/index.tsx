import { createSignal } from "solid-js";

import Button from "../../../../component/Button";
import TextInput from "../../../../component/TextInput";
import { useTodo } from "../../context";
import "./index.css";

export default function AddForm() {
  const todo = useTodo();
  const [text, setText] = createSignal("");

  function submit(e: SubmitEvent) {
    e.preventDefault();
    if (!text().trim()) return;
    void todo.add(text());
    setText("");
  }

  return (
    <form class="todo-page-add-form" onSubmit={submit}>
      <TextInput placeholder="New TODO" value={text()} onInput={(e) => setText(e.currentTarget.value)} />
      <Button type="submit" disabled={!todo.loaded()}>
        Add
      </Button>
    </form>
  );
}
