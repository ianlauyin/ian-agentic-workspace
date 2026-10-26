import { createContext, useContext, type JSX } from "solid-js";

export type Api = {
  loadPage<T>(name: string): Promise<T | null>;
  savePage(name: string, data: unknown): Promise<void>;
};

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(path, init);
  const json = await res.json();
  if (!res.ok) throw new Error(json.error);
  return json;
}

export const httpApi: Api = {
  loadPage: (name) => request(`/page?${new URLSearchParams({ name })}`),
  savePage: (name, data) =>
    request("/page", {
      method: "PUT",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ name, data }),
    }),
};

const AppContext = createContext<{ api: Api }>({ api: httpApi });

export function AppProvider(props: { api: Api; children: JSX.Element }) {
  // the api does not change after the start
  // eslint-disable-next-line solid/reactivity
  return <AppContext.Provider value={{ api: props.api }}>{props.children}</AppContext.Provider>;
}

export function useApp() {
  return useContext(AppContext);
}
