"use client";

import { useMemo, useState } from "react";

export type WizardStep = 0 | 1 | 2 | 3 | 4;

export type MachineDetails = {
  nodeName: string;
  region: string;
  cpuCapacity: string;
  ramGb: string;
  storageGb: string;
};

export type AgentSetup = {
  endpointUrl: string;
};

export type WizardFormState = {
  machine: MachineDetails;
  selectedNetworkId: string;
  agent: AgentSetup;
};

const MAX_STEP: WizardStep = 4;

export function useWizardState() {
  const [step, setStep] = useState<WizardStep>(0);
  const [direction, setDirection] = useState(1);
  const [form, setForm] = useState<WizardFormState>({
    machine: {
      nodeName: "",
      region: "",
      cpuCapacity: "8",
      ramGb: "16",
      storageGb: "",
    },
    selectedNetworkId: "",
    agent: {
      endpointUrl: "",
    },
  });

  const next = () => {
    setDirection(1);
    setStep((current) => Math.min(MAX_STEP, current + 1) as WizardStep);
  };

  const back = () => {
    setDirection(-1);
    setStep((current) => Math.max(0, current - 1) as WizardStep);
  };

  const updateMachine = (nextMachine: MachineDetails) => {
    setForm((current) => ({ ...current, machine: nextMachine }));
  };

  const updateNetwork = (nextNetworkId: string) => {
    setForm((current) => ({ ...current, selectedNetworkId: nextNetworkId }));
  };

  const updateAgent = (nextAgent: AgentSetup) => {
    setForm((current) => ({ ...current, agent: nextAgent }));
  };

  const machineValid = useMemo(() => {
    const cpu = Number(form.machine.cpuCapacity);
    const ram = Number(form.machine.ramGb);

    return (
      form.machine.nodeName.trim().length > 0 &&
      form.machine.region.trim().length > 0 &&
      Number.isFinite(cpu) &&
      cpu > 0 &&
      Number.isFinite(ram) &&
      ram > 0
    );
  }, [form.machine]);

  return {
    step,
    direction,
    form,
    setStep,
    next,
    back,
    updateMachine,
    updateNetwork,
    updateAgent,
    machineValid,
  };
}
