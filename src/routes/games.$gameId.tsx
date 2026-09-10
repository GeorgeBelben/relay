import { useBackHandler } from '@/lib/focus/back-stack';
import { createFileRoute, useRouter } from '@tanstack/react-router'

export const Route = createFileRoute('/games/$gameId')({
    component: RouteComponent,
})

function RouteComponent() {
    const router = useRouter();
    useBackHandler(() => router.history.back());
    return <div>Hello "/games/$gameId"!</div>
}
