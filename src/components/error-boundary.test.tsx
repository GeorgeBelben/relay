import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { ErrorBoundary } from "./error-boundary";

const errorMock = vi.fn();
const flushMock = vi.fn().mockResolvedValue(undefined);
vi.mock("@/lib/better-stack", () => ({
  logger: { error: (...args: unknown[]) => errorMock(...args), flush: () => flushMock() },
}));

function Bomb(): never {
  throw new Error("kaboom");
}

describe("ErrorBoundary", () => {
  it("renders children normally when nothing throws", () => {
    render(
      <ErrorBoundary>
        <p>All good</p>
      </ErrorBoundary>,
    );
    expect(screen.getByText("All good")).toBeInTheDocument();
  });

  it("renders a fallback and reports to BetterStack when a child throws", () => {
    // React logs the caught error to the console by default -- silence it for this test only.
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    render(
      <ErrorBoundary>
        <Bomb />
      </ErrorBoundary>,
    );

    expect(screen.getByText("Something went wrong.")).toBeInTheDocument();
    expect(errorMock).toHaveBeenCalledWith(expect.any(Error), expect.objectContaining({ componentStack: expect.any(String) }));
    expect(flushMock).toHaveBeenCalled();

    consoleSpy.mockRestore();
  });
});
