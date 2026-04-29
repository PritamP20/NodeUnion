import { Suspense } from "react";
import type { Metadata } from "next";
import { NetworksPageClient } from "./NetworksPageClient";

export const metadata: Metadata = {
  title: "Networks | NodeUnion",
  description: "Live network topology and node health dashboard.",
};

function NetworksPageSkeleton() {
  return (
    <div className="space-y-6 pb-8">
      <div className="h-8 w-32 animate-pulse rounded bg-white/10" />
      <div className="space-y-3">
        <div className="h-40 animate-pulse rounded bg-white/10" />
        <div className="h-60 animate-pulse rounded bg-white/10" />
      </div>
    </div>
  );
}

export default function NetworksPage() {
  return (
    <Suspense fallback={<NetworksPageSkeleton />}>
      <NetworksPageClient />
    </Suspense>
  );
}