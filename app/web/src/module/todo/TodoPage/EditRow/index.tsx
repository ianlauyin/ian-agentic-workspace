import { createSignal, onMount } from "solid-js";

import Button from "../../../../component/Button";
import TextInput from "../../../../component/TextInput";
import type { TodoItem } from "../../context";
import "./index.css";

type Props = {
  item: TodoItem;
  onSave: (text: string) => void;
  onCancel: () => void;
};

export default function EditRow(props: Props) {
  // the input starts with the saved text, and later changes to the item do not reset it
  // eslint-disable-next-line solid/reactivity
  const [text, setText] = createSignal(props.item.text);
  let input: HTMLInputElement | undefined;

  onMount(() => input?.focus());

  function keyDown(e: KeyboardEvent) {
    if (e.key === "Enter") props.onSave(text());
    if (e.key === "Escape") props.onCancel();
  }

  return (
    <li class="todo-page-edit-row">
      <TextInput ref={input} value={text()} onInput={(e) => setText(e.currentTarget.value)} onKeyDown={keyDown} />
      <span class="actions">
        <Button small onClick={() => props.onSave(text())}>
          Save
        </Button>
        <Button small danger onClick={() => props.onCancel()}>
          Cancel
        </Button>
      </span>
    </li>
  );
}
