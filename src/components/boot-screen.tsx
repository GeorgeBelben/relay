import { useEffect, useRef, type ReactNode } from "react";
import { RiLoader4Line } from "@remixicon/react";
import { toast } from "sonner";
import { Logo } from "@/components/logo";
import { useRescan, useScanStatus, type ScanStatus } from "@/hooks/use-ingest";

type BootScreenProps = {
  children: ReactNode;
};

function describeStatus(status: ScanStatus): string {
  switch (status.state) {
    case "idle":
      return "Starting up…";
    case "scanning-files":
      return "Scanning ROM library…";
    case "enriching-art":
      return `Fetching artwork… (${status.current}/${status.total})`;
    default:
      return "";
  }
}

// Kicks off the library scan as soon as the app launches and holds the kiosk on a logo/loader
// screen until it's done -- nothing in the real library UI has a sensible partial-data state mid
// scan, so there's no reason to let the user in early. Falls through to the app on error too: a
// failed scan shouldn't strand the kiosk on a boot screen forever.
export function BootScreen({ children }: BootScreenProps) {
  const status = useScanStatus();
  const rescan = useRescan();
  const triggered = useRef(false);

  useEffect(() => {
    if (status.state === "idle" && !triggered.current) {
      triggered.current = true;
      rescan.mutate();
    }
  }, [status.state, rescan]);

  useEffect(() => {
    if (status.state === "error") {
      toast.error(`Library scan failed: ${status.message}`);
    }
  }, [status]);

  if (status.state === "done" || status.state === "error") {
    return <>{children}</>;
  }

  return (
    <div className="h-svh w-full flex flex-col items-center justify-center gap-8">
      <Logo className="w-24" />
      <div className="flex items-center gap-3 text-white/60">
        <RiLoader4Line className="size-5 animate-spin" />
        <p className="font-lexend-deca text-sm">{describeStatus(status)}</p>
      </div>
    </div>
  );
}
