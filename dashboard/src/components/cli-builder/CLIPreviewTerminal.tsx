"use client";

import { useMemo } from "react";
import { useRouter } from "next/navigation";
import type { BuilderTab, SubmitJobForm } from "@/components/cli-builder/useCommandBuilder";
import { TerminalHeader } from "@/components/jobs/TerminalHeader";

function tokenize(line: string) {
  return line.match(/"[^"]*"|\S+/g) ?? [];
}

type Props = {
  tab: BuilderTab;
  commandText: string;
  submitForm: SubmitJobForm;
};

export function CLIPreviewTerminal({ tab, commandText, submitForm }: Props) {
  const router = useRouter();
  const lines = useMemo(() => commandText.split("\n"), [commandText]);

  const onCopy = async () => {
    await navigator.clipboard.writeText(commandText);
  };

  const onRunViaWeb = () => {
    if (tab !== "submit") {
      return;
    }

    const params = new URLSearchParams();
    if (submitForm.wallet) params.set("wallet", submitForm.wallet);
    if (submitForm.network) params.set("network", submitForm.network);
    if (submitForm.image) params.set("image", submitForm.image);
    if (submitForm.command) params.set("command", submitForm.command);
    if (submitForm.cpu) params.set("cpu", submitForm.cpu);
    if (submitForm.ramGb) params.set("ramGb", submitForm.ramGb);
    if (submitForm.port) params.set("port", submitForm.port);
    if (submitForm.orchestratorUrl) params.set("orchestratorUrl", submitForm.orchestratorUrl);

    router.push(`/provider?${params.toString()}`);
  };

  return (
    <section className="overflow-hidden rounded-xl border border-white/10 bg-[#0a0a0f]">
      <TerminalHeader label="Generated Command" />

      <div className="flex items-center justify-end gap-2 border-b border-white/10 px-4 py-3 text-xs">
        <button type="button" onClick={onCopy} className="rounded-md border border-white/10 bg-white/5 px-2 py-1 text-slate-200 hover:bg-white/10">
          Copy Command
        </button>
        {tab === "submit" ? (
          <button type="button" onClick={onRunViaWeb} className="rounded-md border border-indigo-500/30 bg-indigo-500/20 px-2 py-1 text-indigo-200 hover:bg-indigo-500/30">
            Run via Web
          </button>
        ) : null}
      </div>

      <div className="min-h-[420px] space-y-1 px-4 py-4 font-mono text-sm">
        {lines.map((line, index) => {
          const tokens = tokenize(line);

          return (
            <p key={`${index}-${line.slice(0, 20)}`} className="leading-6">
              {tokens.map((token, tokenIndex) => {
                let className = "text-white";

                if (tokenIndex === 0 && token.startsWith("nodeunion")) {
                  className = "text-indigo-400";
                } else if (token.startsWith("--")) {
                  className = "text-cyan-400";
                } else if (token === "\\") {
                  className = "text-slate-500";
                }

                return (
                  <span key={`${token}-${tokenIndex}`} className={className}>
                    {token}{" "}
                  </span>
                );
              })}
            </p>
          );
        })}
      </div>
    </section>
  );
}
