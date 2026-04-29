import type { Metadata } from "next";
import { JobDetailPageClient } from "@/components/jobs/JobDetailPageClient";

type Props = {
  params: Promise<{ id: string }>;
};

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { id } = await params;

  return {
    title: `Job ${id} | NodeUnion`,
    description: "Live NodeUnion job logs and execution metadata.",
  };
}

export default async function JobDetailPage({ params }: Props) {
  const { id } = await params;

  return <JobDetailPageClient jobId={id} />;
}
