export const PRICING = {
  cpuPerCorePerHour: 0.002,
  ramPerGBPerHour: 0.0005,
  baseJobFee: 0.001,
  solToUsd: 140,
};

export type EstimatorInput = {
  cpuCores: number;
  ramGb: number;
  hours: number;
  minutes: number;
  jobCount: number;
  includeBaseFee: boolean;
};

export function durationHours(hours: number, minutes: number) {
  return Math.max(0, hours) + Math.max(0, minutes) / 60;
}

export function estimateCost(input: EstimatorInput) {
  const totalHours = durationHours(input.hours, input.minutes);
  const cpuCost = input.cpuCores * PRICING.cpuPerCorePerHour * totalHours * input.jobCount;
  const ramCost = input.ramGb * PRICING.ramPerGBPerHour * totalHours * input.jobCount;
  const baseFee = input.includeBaseFee ? PRICING.baseJobFee * input.jobCount : 0;
  const totalSol = cpuCost + ramCost + baseFee;
  const totalUsd = totalSol * PRICING.solToUsd;

  const burnRatePerHour = input.cpuCores * PRICING.cpuPerCorePerHour + input.ramGb * PRICING.ramPerGBPerHour;
  const hoursForOneSol = burnRatePerHour > 0 ? 1 / burnRatePerHour : 0;

  return {
    totalHours,
    cpuCost,
    ramCost,
    baseFee,
    totalSol,
    totalUsd,
    hoursForOneSol,
  };
}

export const ESTIMATOR_PRESETS = [
  { key: "small", label: "Small", cpuCores: 1, ramGb: 4, hours: 1, minutes: 0, jobCount: 1 },
  { key: "medium", label: "Medium", cpuCores: 4, ramGb: 16, hours: 4, minutes: 0, jobCount: 2 },
  { key: "large", label: "Large", cpuCores: 12, ramGb: 48, hours: 12, minutes: 0, jobCount: 4 },
] as const;
