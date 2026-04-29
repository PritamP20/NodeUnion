"use client";

import { useEffect, useMemo, useState } from "react";
import confetti from "canvas-confetti";
import useSWR from "swr";
import { WizardShell } from "@/components/onboarding/WizardShell";
import { StepWelcome } from "@/components/onboarding/StepWelcome";
import { StepMachineDetails } from "@/components/onboarding/StepMachineDetails";
import { StepNetworkSelect } from "@/components/onboarding/StepNetworkSelect";
import { StepAgentSetup } from "@/components/onboarding/StepAgentSetup";
import { StepReviewFinish } from "@/components/onboarding/StepReviewFinish";
import { useWizardState } from "@/components/onboarding/useWizardState";
import { buildRegisterCommand } from "@/components/cli-builder/useCommandBuilder";
import { REGION_COORDINATES } from "@/lib/region-coordinates";
import { fetchMainSnapshot, fetchNodes } from "@/lib/orchestrator-realtime";

type Snapshot = Awaited<ReturnType<typeof fetchMainSnapshot>>;
type VerifyStatus = "idle" | "checking" | "success" | "failed";

const TITLES = [
  "Start Earning with Your Idle Compute",
  "Machine Details",
  "Network Selection",
  "Agent Setup",
  "Review & Finish",
] as const;

export function OnboardingPageClient() {
  const { data, error, isLoading } = useSWR<Snapshot>("/api/main/snapshot", () => fetchMainSnapshot(), {
    refreshInterval: 30000,
    revalidateOnFocus: true,
  });

  const {
    step,
    direction,
    form,
    next,
    back,
    updateMachine,
    updateNetwork,
    updateAgent,
    machineValid,
  } = useWizardState();

  const [verifyStatus, setVerifyStatus] = useState<VerifyStatus>("idle");

  const regions = useMemo(() => Object.keys(REGION_COORDINATES).sort(), []);
  const selectedNetwork = data?.networks.find((network) => network.network_id === form.selectedNetworkId);

  const generatedRegisterCommand = useMemo(
    () =>
      buildRegisterCommand({
        name: form.machine.nodeName,
        region: form.machine.region,
        cpu: form.machine.cpuCapacity,
        ramGb: form.machine.ramGb,
        network: form.selectedNetworkId,
        endpoint: form.agent.endpointUrl,
      }),
    [form],
  );

  useEffect(() => {
    if (step !== 4) {
      return;
    }

    void confetti({
      particleCount: 120,
      spread: 80,
      origin: { y: 0.7 },
    });
  }, [step]);

  const canNext = (() => {
    if (step === 0) return true;
    if (step === 1) return machineValid;
    if (step === 2) return form.selectedNetworkId.length > 0;
    if (step === 3) return verifyStatus === "success";
    return false;
  })();

  const onVerify = async () => {
    setVerifyStatus("checking");

    try {
      const nodes = await fetchNodes();
      const found = nodes.some((node) => node.node_id === form.machine.nodeName.trim());
      setVerifyStatus(found ? "success" : "failed");
    } catch {
      setVerifyStatus("failed");
    }
  };

  const installCommand = "curl -fsSL https://get.nodeunion.dev/agent | bash";
  const runCommand = `nodeunion-agent --node ${form.machine.nodeName || "provider-node-1"} --network ${form.selectedNetworkId || "<network-id>"} --region ${form.machine.region || "<region>"}`;

  return (
    <main className="mx-auto w-full max-w-7xl px-4 py-6 sm:px-6 lg:px-8 lg:py-10">
      <WizardShell
        step={step}
        direction={direction}
        title={TITLES[step]}
        canGoBack={step > 0}
        canGoNext={canNext}
        nextLabel={step === 0 ? "Get Started ->" : step === 3 ? "Finish" : "Next"}
        onBack={back}
        onNext={next}
        hideNext={step === 4}
      >
        {step === 0 ? <StepWelcome /> : null}

        {step === 1 ? (
          <StepMachineDetails machine={form.machine} regions={regions} onChange={updateMachine} />
        ) : null}

        {step === 2 ? (
          <StepNetworkSelect
            networks={data?.networks ?? []}
            selectedNetworkId={form.selectedNetworkId}
            onSelect={updateNetwork}
            isLoading={isLoading}
            error={error ? "Could not load networks." : undefined}
          />
        ) : null}

        {step === 3 ? (
          <StepAgentSetup
            installCommand={installCommand}
            runCommand={runCommand}
            endpointUrl={form.agent.endpointUrl}
            verifyStatus={verifyStatus}
            onEndpointChange={(value) => updateAgent({ endpointUrl: value })}
            onVerify={() => void onVerify()}
          />
        ) : null}

        {step === 4 ? (
          <StepReviewFinish form={form} selectedNetwork={selectedNetwork} command={generatedRegisterCommand} />
        ) : null}
      </WizardShell>
    </main>
  );
}
