import { useConnectedGamepads } from "@/lib/gamepad/use-connected-gamepads";
import { cn } from "@/lib/utils";
import { Logo } from "./logo";

export function Header() {
    const connectedGamepads = useConnectedGamepads();

    console.log("connectedGamepads", connectedGamepads);

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
            </div>
        </div>
    )
}