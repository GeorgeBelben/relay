import { format } from "date-fns";
import { useEffect, useRef, useState } from "react";

export function Clock() {
  const [time, setTime] = useState(format(new Date(), "HH:mm"));
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    intervalRef.current = setInterval(() => {
      setTime(format(new Date(), "HH:mm"));
    }, 1000);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, []);

  return <span className="text-lg">{time}</span>;
}
