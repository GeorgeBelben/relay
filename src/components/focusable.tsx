import { cn } from '@/lib/utils';
import { useFocusable } from '@noriginmedia/norigin-spatial-navigation-react';
import { Link, LinkProps } from "@tanstack/react-router";

type FocusableLinkProps = LinkProps & {

}

export function FocusableLink({ ...props }: FocusableLinkProps) {
    const { ref, focused } = useFocusable();
    return (
        <Link
            ref={ref}
            className={cn(
                "outline-2 outline-offset-8 outline-offset-transparent outline-transparent rounded-2xl transition duration-200",
                {
                    "outline-2 outline-white": focused,
                })}
            {...props}
        />
    )
}