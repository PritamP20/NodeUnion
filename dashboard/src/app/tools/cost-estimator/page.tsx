import type { Metadata } from "next";
import { CostEstimatorPageClient } from "@/components/cost-estimator/CostEstimatorPageClient";

export const metadata: Metadata = {
  title: "Cost Estimator | NodeUnion",
  description: "Estimate NodeUnion job cost in SOL and USD before submitting workloads.",
};

export default function CostEstimatorPage() {
  return <CostEstimatorPageClient />;
}
