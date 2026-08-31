import React from "react";
import ReactDOM from "react-dom/client";
import { QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider, createMemoryHistory, createRouter } from "@tanstack/react-router";
import { getContext, queryClient } from "./router";
import { routeTree } from "./routeTree.gen";
import "./main.css";
import { ErrorBoundary } from "@/components/error-boundary";
import { initFocusEngine } from "@/lib/focus";
import { logger } from "@/lib/better-stack";

// Must happen before any `useFocusable` component mounts.
initFocusEngine();

// Catches everything React's own error boundary can't: errors thrown outside a render (event
// handlers, timers, plain JS bugs) and rejected promises nobody attached a .catch to. Same "if
// the app errors, know about it" goal as the ErrorBoundary/QueryCache logging below -- this is a
// kiosk with no console anyone's watching.
window.addEventListener("error", (event) => {
  logger.error(event.error ?? event.message, { filename: event.filename, lineno: event.lineno, colno: event.colno });
});
window.addEventListener("unhandledrejection", (event) => {
  logger.error(event.reason instanceof Error ? event.reason : String(event.reason));
});

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
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </ErrorBoundary>
  </React.StrictMode>,
);
