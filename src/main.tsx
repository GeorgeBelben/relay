import React from "react";
import ReactDOM from "react-dom/client";
import { QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider, createMemoryHistory, createRouter } from "@tanstack/react-router";
import { getContext, queryClient } from "./router";
import { routeTree } from "./routeTree.gen";
import "./main.css";
import { initFocusEngine } from "@/lib/focus";

// Must happen before any `useFocusable` component mounts.
initFocusEngine();

// Tauri's webview doesn't load from a plain http:// origin, so browser history semantics don't
// apply the same way they would on the web -- memory history keeps the router's notion of
// "current page" in-process instead, same reasoning as the Electron MVP's file:// setup.
const memoryHistory = createMemoryHistory({ initialEntries: ["/"] });

const router = createRouter({ routeTree, history: memoryHistory, context: getContext() });

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </React.StrictMode>,
);
