export type BuilderTab = "submit" | "register";

export type SubmitJobForm = {
  wallet: string;
  network: string;
  image: string;
  command: string;
  cpu: string;
  ramGb: string;
  port: string;
  orchestratorUrl: string;
};

export type RegisterNodeForm = {
  name: string;
  region: string;
  cpu: string;
  ramGb: string;
  network: string;
  endpoint: string;
};

function quote(value: string) {
  if (!value) return value;
  if (value.includes(" ")) return `"${value.replaceAll('"', '\\"')}"`;
  return value;
}

const CONTINUATION = "\\";

function pushFlag(lines: string[], flag: string, value: string, suffix = "") {
  if (!value.trim()) {
    return;
  }

  lines.push(`  ${flag} ${quote(value.trim())}${suffix} ${CONTINUATION}`);
}

export function buildSubmitCommand(form: SubmitJobForm) {
  const lines = [`nodeunion job submit ${CONTINUATION}`];

  pushFlag(lines, "--wallet", form.wallet);
  pushFlag(lines, "--network", form.network);
  pushFlag(lines, "--image", form.image);
  pushFlag(lines, "--cmd", form.command);
  pushFlag(lines, "--cpu", form.cpu);
  pushFlag(lines, "--ram", form.ramGb, "GB");
  pushFlag(lines, "--port", form.port);
  pushFlag(lines, "--orchestrator", form.orchestratorUrl);

  if (lines.length > 1) {
    const last = lines[lines.length - 1];
    lines[lines.length - 1] = last.replace(/\s+\\$/, "");
  }

  return lines.join("\n");
}

export function buildRegisterCommand(form: RegisterNodeForm) {
  const lines = [`nodeunion node register ${CONTINUATION}`];

  pushFlag(lines, "--name", form.name);
  pushFlag(lines, "--region", form.region);
  pushFlag(lines, "--cpu", form.cpu);
  pushFlag(lines, "--ram", form.ramGb, "GB");
  pushFlag(lines, "--network", form.network);
  pushFlag(lines, "--endpoint", form.endpoint);

  if (lines.length > 1) {
    const last = lines[lines.length - 1];
    lines[lines.length - 1] = last.replace(/\s+\\$/, "");
  }

  return lines.join("\n");
}

export function useCommandBuilder(tab: BuilderTab, submit: SubmitJobForm, register: RegisterNodeForm) {
  return tab === "submit" ? buildSubmitCommand(submit) : buildRegisterCommand(register);
}
