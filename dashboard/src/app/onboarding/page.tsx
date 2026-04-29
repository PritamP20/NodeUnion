import type { Metadata } from "next";
import { OnboardingPageClient } from "@/components/onboarding/OnboardingPageClient";

export const metadata: Metadata = {
  title: "Provider Onboarding | NodeUnion",
  description: "Guided setup wizard for registering your machine as a NodeUnion provider.",
};

export default function OnboardingPage() {
  return <OnboardingPageClient />;
}
