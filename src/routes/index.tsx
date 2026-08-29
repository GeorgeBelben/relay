import { createFileRoute } from "@tanstack/react-router";
import { HomeCrudStub } from "@/components/home-crud-stub";

export const Route = createFileRoute("/")({
  component: HomeCrudStub,
});
