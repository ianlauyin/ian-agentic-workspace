import { createSignal } from "solid-js";

import Button from "../../../../component/Button";
import type { TodoItem } from "../../context";
import "./index.css";

export type DropSide = "before" | "after";

type Props = {
  item: TodoItem;
  dragging: boolean;
  dropSide: DropSide | undefined;
  onEdit: () => void;
  onDelete: () => void;
  onDragStart: () => void;
  onDragOver: (side: DropSide) => void;
  onDrop: () => void;
  onDragEnd: () => void;
};

export default function ItemRow(props: Props) {
  // only the handle starts a drag, so the user can still select the text
  const [draggable, setDraggable] = createSignal(false);

  function dragStart(e: DragEvent) {
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", props.item.text);
    }
    props.onDragStart();
  }

  function dragOver(e: DragEvent & { currentTarget: HTMLLIElement }) {
    e.preventDefault();
    const box = e.currentTarget.getBoundingClientRect();
    props.onDragOver(e.clientY > box.top + box.height / 2 ? "after" : "before");
  }

  function drop(e: DragEvent) {
    e.preventDefault();
    props.onDrop();
  }

  function dragEnd() {
    setDraggable(false);
    props.onDragEnd();
  }

  return (
    <li
      class="todo-page-item-row"
      classList={{ dragging: props.dragging, before: props.dropSide === "before", after: props.dropSide === "after" }}
      draggable={draggable()}
      onDragStart={dragStart}
      onDragOver={dragOver}
      onDrop={drop}
      onDragEnd={dragEnd}
    >
      <span class="handle" title="Drag to reorder" onMouseDown={() => setDraggable(true)} onMouseUp={() => setDraggable(false)}>
        ⋮⋮
      </span>
      <span class="text">{props.item.text}</span>
      <span class="actions">
        <Button small onClick={() => props.onEdit()}>
          Edit
        </Button>
        <Button small danger onClick={() => props.onDelete()}>
          Delete
        </Button>
      </span>
    </li>
  );
}
