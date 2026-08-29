import { QueryClient } from "@tanstack/react-query";

export type RouterContext = {
  queryClient: QueryClient;
};

export const queryClient = new QueryClient();

export const getContext = (): RouterContext => ({
  queryClient,
});
