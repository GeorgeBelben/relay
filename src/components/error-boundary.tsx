import { Component, type ErrorInfo, type ReactNode } from "react";
import { Logo } from "@/components/logo";
import { logger } from "@/lib/better-stack";

type Props = { children: ReactNode };
type State = { hasError: boolean };

// Catches render-time errors anywhere in the tree below it and reports them to BetterStack --
// without this, a thrown render error leaves the kiosk on a blank screen forever, with nobody
// around to notice or a devtools console to check. React error boundaries must be class
// components; there's no hooks equivalent (as of React 19).
export class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false };

  static getDerivedStateFromError(): State {
    return { hasError: true };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    logger.error(error, { componentStack: info.componentStack });
    // Logtail batches log entries rather than sending each one immediately -- force this one out
    // now rather than trusting a process that just hit a render error to flush it on its own schedule.
    void logger.flush();
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="h-svh w-full flex flex-col items-center justify-center gap-6 bg-[#111111] text-white">
          <Logo className="w-24 opacity-60" />
          <p className="font-lexend-deca text-sm text-white/60">Something went wrong.</p>
        </div>
      );
    }
    return this.props.children;
  }
}
