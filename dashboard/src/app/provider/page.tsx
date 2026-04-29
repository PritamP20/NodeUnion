import { Suspense } from "react";
import type { Metadata } from "next";
import { ProviderPageClient } from "./ProviderPageClient";

export const metadata: Metadata = {
  title: "Provider | NodeUnion",
  description: "Launch workloads with the web dashboard.",
};

function ProviderPageSkeleton() {
  return (
    <div className="space-y-6 pb-8">
      <div className="h-32 animate-pulse rounded bg-white/10" />
      <div className="grid gap-6 xl:grid-cols-2">
        <div className="h-96 animate-pulse rounded bg-white/10" />
        <div className="h-96 animate-pulse rounded bg-white/10" />
      </div>
    </div>
  );
}

export default function ProviderPage() {
  return (
    <Suspense fallback={<ProviderPageSkeleton />}>
      <ProviderPageClient />
    </Suspense>
  );
}