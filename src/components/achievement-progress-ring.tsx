import { cn } from "@/lib/cn";

type AchievementProgressRingProps = {
  percent: number; // 0-100
  className?: string;
};

// A ring, not a filled disc, so it reads against whatever box art sits behind it -- see
// game-tile.tsx for the circular dark badge this sits inside of, matching the "Beaten" checkmark
// badge's own footprint on the opposite corner.
const RADIUS = 7;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

export function AchievementProgressRing({ percent, className }: AchievementProgressRingProps) {
  const clamped = Math.max(0, Math.min(100, percent));
  const offset = CIRCUMFERENCE * (1 - clamped / 100);

  return (
    <svg viewBox="0 0 18 18" className={cn("-rotate-90", className)} aria-hidden="true">
      <circle cx="9" cy="9" r={RADIUS} fill="none" stroke="white" strokeOpacity="0.25" strokeWidth="2.5" />
      <circle
        cx="9"
        cy="9"
        r={RADIUS}
        fill="none"
        stroke="currentColor"
        strokeWidth="2.5"
        strokeLinecap="round"
        strokeDasharray={CIRCUMFERENCE}
        strokeDashoffset={offset}
        className="transition-[stroke-dashoffset] duration-300 ease-out"
      />
    </svg>
  );
}
