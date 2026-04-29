import type { Metadata } from "next";
import { CLIBuilderPageClient } from "@/components/cli-builder/CLIBuilderPageClient";

export const metadata: Metadata = {
  title: "CLI Command Builder | NodeUnion",
  description: "Build NodeUnion CLI commands visually and copy runnable output.",
};

export default function CLIBuilderPage() {
  return <CLIBuilderPageClient />;
}
