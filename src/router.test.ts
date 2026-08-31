import { describe, expect, it, vi } from "vitest";

const errorMock = vi.fn();
vi.mock("@/lib/better-stack", () => ({
  logger: { error: (...args: unknown[]) => errorMock(...args) },
}));

describe("queryClient error logging", () => {
  it("logs a failed query to BetterStack with its query key", async () => {
    const { queryClient } = await import("./router");
    const error = new Error("invoke failed");

    await queryClient.fetchQuery({
      queryKey: ["games"],
      queryFn: () => Promise.reject(error),
      retry: false,
    }).catch(() => {});

    expect(errorMock).toHaveBeenCalledWith(error, { queryKey: ["games"] });
  });

  it("logs a failed mutation to BetterStack with its mutation key", async () => {
    const { queryClient } = await import("./router");
    const error = new Error("mutation failed");

    await queryClient
      .getMutationCache()
      .build(queryClient, { mutationKey: ["create_game"], mutationFn: () => Promise.reject(error) })
      .execute(undefined)
      .catch(() => {});

    expect(errorMock).toHaveBeenCalledWith(error, { mutationKey: ["create_game"] });
  });
});
