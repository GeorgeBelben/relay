import type { NavAction } from "@/lib/input";

// Excludes "home" -- it's a global, always-available input (see lib/system-menu), not a
// screen-contextual one, so nothing should ever advertise it as a per-screen hint.
export type Hint = { action: Exclude<NavAction, "home">; label: string };
