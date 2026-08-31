import { getAvatarDataUri } from "@/lib/avatar";
import { cn } from "@/lib/cn";

type ProfileAvatarProps = { seed: string; className?: string };

export function ProfileAvatar({ seed, className }: ProfileAvatarProps) {
  return <img src={getAvatarDataUri(seed)} alt="" className={cn("rounded-full object-cover", className)} />;
}
