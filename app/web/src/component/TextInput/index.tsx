import { splitProps, type JSX } from "solid-js";

import "./index.css";

export default function TextInput(props: JSX.InputHTMLAttributes<HTMLInputElement>) {
  const [local, rest] = splitProps(props, ["class"]);
  return <input type="text" autocomplete="off" {...rest} class={local.class ? `g-text-input ${local.class}` : "g-text-input"} />;
}
