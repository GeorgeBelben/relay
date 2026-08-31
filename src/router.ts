import { MutationCache, QueryCache, QueryClient } from "@tanstack/react-query";
import { logger } from "@/lib/better-stack";

export type RouterContext = {
  queryClient: QueryClient;
};

// Every failed query/mutation goes to BetterStack -- this is a kiosk device with no console
// anyone's watching, so a failed invoke() (a Tauri command erroring, a network call failing,
// bad IPC args) needs to surface somewhere other than a toast the user may not read or a
// devtools panel nobody has open. The backend also logs its own command errors independently
// (src-tauri/src/logging.rs) -- this is the frontend-side half of the same "if the app errors,
// know about it" goal, catching failures that never make it to (or originate outside) a Tauri
// command.
export const queryClient = new QueryClient({
  queryCache: new QueryCache({
    onError: (error, query) => {
      logger.error(error, { queryKey: query.queryKey });
    },
  }),
  mutationCache: new MutationCache({
    onError: (error, _variables, _context, mutation) => {
      logger.error(error, { mutationKey: mutation.options.mutationKey });
    },
  }),
});

export const getContext = (): RouterContext => ({
  queryClient,
});
