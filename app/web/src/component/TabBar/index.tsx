import { A } from "@solidjs/router";
import { For } from "solid-js";

import "./index.css";

export type Tab = { href: string; label: string };

export default function TabBar(props: { tabs: Tab[] }) {
  return (
    <nav class="g-tab-bar">
      <For each={props.tabs}>
        {(tab) => (
          <A href={tab.href} activeClass="g-on" end>
            {tab.label}
          </A>
        )}
      </For>
    </nav>
  );
}
