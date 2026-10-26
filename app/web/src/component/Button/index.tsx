import { splitProps, type JSX } from "solid-js";

import "./index.css";

type Props = JSX.ButtonHTMLAttributes<HTMLButtonElement> & { danger?: boolean; small?: boolean };

export default function Button(props: Props) {
  const [local, rest] = splitProps(props, ["danger", "small", "class"]);
  const className = () => ["g-button", local.danger && "g-danger", local.small && "g-small", local.class].filter(Boolean).join(" ");
  return <button type="button" {...rest} class={className()} />;
}
