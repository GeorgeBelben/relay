import { useConnectedGamepads } from "@/lib/gamepad/use-connected-gamepads";
import { cn } from "@/lib/utils";
import { Logo } from "./logo";
import { GlobeIcon } from '@phosphor-icons/react';
import { useDaemonConnection } from "@/lib/daemon";

export function Header() {
    const connectedGamepads = useConnectedGamepads();
    const daemonStatus = useDaemonConnection();

    console.log("daemonStatus", daemonStatus);

    return (
        <div className="p-8 fixed inset-0 bottom-auto grid grid-cols-3">
            <div className="flex items-center justify-start gap-4">

                <Logo className="w-6" />
            </div>
            <div className="flex items-center justify-center">
            </div>
            <div className="flex items-center justify-end gap-4">
                <div className="flex items-center gap-2">
                    {[0, 1, 2, 3].map((index) => {
                        const gamepad = connectedGamepads[index];
                        return (
                            <div key={index} className={cn("size-2.5 bg-neutral-700 flex items-center justify-center", {
                                "bg-white": gamepad?.index === index,
                            })} />
                        )
                    })}
                </div>
                <GlobeIcon size={20} className={cn({
                    "text-green-500": daemonStatus === 'connected',
                    "text-neutral-500": daemonStatus === 'disconnected'
                })} />
            </div>
        </div>
    )
}