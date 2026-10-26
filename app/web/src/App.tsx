import { Navigate, Route, Router, useLocation, type RouteSectionProps } from "@solidjs/router";
import { For, createEffect } from "solid-js";

import TabBar from "./component/TabBar";
import { AppProvider, httpApi } from "./context";
import TodoPage from "./module/todo/TodoPage";

const PAGES = [{ path: "/todo", title: "TODO", component: TodoPage }];
const HOME = "/todo";

function Layout(props: RouteSectionProps) {
  const location = useLocation();
  createEffect(() => {
    document.title = PAGES.find((page) => page.path === location.pathname)?.title ?? "Agentic Workspace";
  });
  return (
    <>
      <TabBar tabs={PAGES.map((page) => ({ href: page.path, label: page.title }))} />
      <main>{props.children}</main>
    </>
  );
}

export default function App() {
  return (
    <AppProvider api={httpApi}>
      <Router root={Layout}>
        <For each={PAGES}>{(page) => <Route path={page.path} component={page.component} />}</For>
        <Route path="*" component={() => <Navigate href={HOME} />} />
      </Router>
    </AppProvider>
  );
}
