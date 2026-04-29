import type { Metadata } from "next";
import { redirect } from "next/navigation";

export const metadata: Metadata = {
  title: "Network Topology | NodeUnion",
  description: "Interactive force-directed view of NodeUnion networks and provider nodes.",
};

export default function NetworksTopologyPage() {
  redirect("/networks?view=topology");
}
