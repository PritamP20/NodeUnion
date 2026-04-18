"use client";

import { useEffect, useMemo, useState } from "react";
import {
  AlertTriangle,
  Check,
  ChevronDown,
  ChevronRight,
  Code2,
  Copy,
  Info,
  Lightbulb,
  Search,
} from "lucide-react";
import { docSections } from "@/lib/dashboard-data";

type CalloutTone = "info" | "tip" | "warning";

function toneClasses(tone: CalloutTone) {
  switch (tone) {
    case "tip":
      return "border-emerald-500/20 bg-emerald-500/10 text-emerald-200";
    case "warning":
      return "border-amber-500/20 bg-amber-500/10 text-amber-100";
    default:
      return "border-sky-500/20 bg-sky-500/10 text-sky-100";
  }
}

function CalloutIcon({ tone }: { tone: CalloutTone }) {
  if (tone === "tip") return <Lightbulb size={16} className="text-emerald-300" />;
  if (tone === "warning") return <AlertTriangle size={16} className="text-amber-300" />;
  return <Info size={16} className="text-sky-300" />;
}

function CodeBlock({ label, language, code }: { label: string; language: string; code: string }) {
  const [copied, setCopied] = useState(false);

  async function handleCopy() {
    await navigator.clipboard.writeText(code);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1200);
  }

  const lines = code.split("\n");

  return (
    <div className="code-block overflow-hidden rounded-2xl">
      <div className="flex items-center justify-between border-b border-white/5 px-4 py-3 text-xs uppercase tracking-[0.24em] text-slate-400">
        <div className="flex items-center gap-2">
          <Code2 size={14} className="text-sky-300" />
          <span>{label}</span>
          <span className="rounded-full border border-white/5 bg-white/5 px-2 py-1 text-[10px] tracking-[0.2em] text-slate-500">
            {language}
          </span>
        </div>
        <button
          onClick={handleCopy}
          className="inline-flex items-center gap-1 rounded-full border border-white/5 bg-white/5 px-3 py-1.5 text-[11px] text-slate-300 transition hover:border-sky-400/40 hover:bg-sky-500/10"
        >
          {copied ? <Check size={14} className="text-emerald-300" /> : <Copy size={14} />}
          {copied ? "Copied" : "Copy"}
        </button>
      </div>
      <pre className="overflow-x-auto p-4 font-mono text-sm leading-6 text-slate-200">
        {lines.map((line, index) => (
          <div key={`${label}-${index}`} className="flex gap-4 whitespace-pre-wrap">
            <span className="w-8 shrink-0 select-none text-right text-slate-600">{index + 1}</span>
            <span className={language === "bash" ? "text-slate-200" : language === "json" ? "text-sky-200" : "text-emerald-200"}>
              {line}
            </span>
          </div>
        ))}
      </pre>
    </div>
  );
}

export default function DocsPage() {
  const [search, setSearch] = useState("");
  const [selectedSectionId, setSelectedSectionId] = useState(docSections[0].id);
  const [openGroups, setOpenGroups] = useState<Record<string, boolean>>({
    "Getting Started": true,
    "Orchestrator Setup": true,
    "Agent Setup": true,
    "Job Submission": true,
    "Solana Billing": true,
    "API Reference": true,
    Troubleshooting: true,
  });

  const filteredSections = useMemo(() => {
    const query = search.trim().toLowerCase();
    if (!query) return docSections;

    return docSections.filter((section) => {
      const haystack = [section.group, section.title, section.summary, ...section.steps, ...section.codeBlocks.map((block) => block.code)]
        .join(" ")
        .toLowerCase();
      return haystack.includes(query);
    });
  }, [search]);

  useEffect(() => {
    if (!filteredSections.some((section) => section.id === selectedSectionId) && filteredSections[0]) {
      setSelectedSectionId(filteredSections[0].id);
    }
  }, [filteredSections, selectedSectionId]);

  const selectedSection = useMemo(
    () => filteredSections.find((section) => section.id === selectedSectionId) ?? filteredSections[0] ?? docSections[0],
    [filteredSections, selectedSectionId],
  );

  const groupedSections = useMemo(() => {
    return filteredSections.reduce<Record<string, typeof docSections>>((groups, section) => {
      groups[section.group] ??= [];
      groups[section.group].push(section);
      return groups;
    }, {});
  }, [filteredSections]);

  return (
    <main className="mx-auto w-full max-w-7xl px-4 py-6 sm:px-6 lg:px-8 lg:py-10">
      <section className="glass-card rounded-[2rem] p-6 sm:p-8">
        <p className="font-mono text-xs uppercase tracking-[0.3em] text-sky-300/90">Documentation</p>
        <h1 className="mt-3 text-4xl font-semibold tracking-tight sm:text-5xl">NodeUnion deployment guide</h1>
        <p className="mt-3 max-w-3xl text-sm leading-6 text-slate-300 sm:text-base">
          Search the guide, expand the section groups on the left, and read the selected topic in a structured reference panel on the right.
        </p>
      </section>

      <section className="mt-6 grid gap-6 xl:grid-cols-[0.34fr_0.66fr]">
        <aside className="glass-card rounded-[1.75rem] p-5">
          <label className="flex items-center gap-3 rounded-2xl border border-white/5 bg-black/20 px-4 py-3">
            <Search size={16} className="text-slate-400" />
            <input
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="Search docs"
              className="w-full bg-transparent text-sm outline-none placeholder:text-slate-500"
            />
          </label>

          <div className="mt-5 space-y-3">
            {Object.entries(groupedSections).map(([group, sections]) => {
              const open = openGroups[group] ?? true;
              return (
                <div key={group} className="rounded-[1.25rem] border border-white/5 bg-white/5 p-3">
                  <button
                    onClick={() => setOpenGroups((current) => ({ ...current, [group]: !open }))}
                    className="flex w-full items-center justify-between gap-3 rounded-xl px-2 py-2 text-left"
                  >
                    <span className="text-xs uppercase tracking-[0.26em] text-slate-400">{group}</span>
                    {open ? <ChevronDown size={14} className="text-slate-400" /> : <ChevronRight size={14} className="text-slate-400" />}
                  </button>

                  {open && (
                    <div className="mt-2 space-y-2">
                      {sections.map((section) => (
                        <button
                          key={section.id}
                          onClick={() => setSelectedSectionId(section.id)}
                          className={`sidebar-pill w-full rounded-xl border px-3 py-3 text-left transition ${
                            selectedSection.id === section.id
                              ? "border-sky-400/30 bg-sky-500/10"
                              : "border-white/5 bg-black/10"
                          }`}
                        >
                          <p className="text-sm font-semibold text-slate-100">{section.title}</p>
                          <p className="mt-1 text-xs leading-5 text-slate-400">{section.summary}</p>
                        </button>
                      ))}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </aside>

        <article className="glass-card rounded-[1.75rem] p-6">
          <div className="flex flex-wrap items-center gap-3 text-[11px] uppercase tracking-[0.28em] text-slate-400">
            <span className="rounded-full border border-white/5 bg-white/5 px-3 py-1 text-slate-300">
              {selectedSection.group}
            </span>
            <span className="rounded-full border border-white/5 bg-white/5 px-3 py-1 text-slate-300">
              {selectedSection.codeBlocks.length} code blocks
            </span>
          </div>

          <h2 className="mt-4 text-3xl font-semibold tracking-tight">{selectedSection.title}</h2>
          <p className="mt-3 max-w-3xl text-sm leading-6 text-slate-300">{selectedSection.summary}</p>

          {selectedSection.callout && (
            <div className={`mt-5 rounded-2xl border p-4 ${toneClasses(selectedSection.callout.tone)}`}>
              <div className="flex items-start gap-3">
                <CalloutIcon tone={selectedSection.callout.tone} />
                <div>
                  <p className="font-semibold text-slate-100">{selectedSection.callout.title}</p>
                  <p className="mt-1 text-sm leading-6 text-slate-300">{selectedSection.callout.body}</p>
                </div>
              </div>
            </div>
          )}

          <div className="mt-6 rounded-[1.5rem] border border-white/5 bg-white/5 p-5">
            <h3 className="text-sm uppercase tracking-[0.26em] text-slate-400">Step-by-step</h3>
            <div className="mt-4 space-y-3">
              {selectedSection.steps.map((step, index) => (
                <div key={step} className="flex gap-3 rounded-2xl border border-white/5 bg-black/20 p-4">
                  <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-sky-500/15 font-semibold text-sky-300">
                    {index + 1}
                  </div>
                  <p className="text-sm leading-6 text-slate-300">{step}</p>
                </div>
              ))}
            </div>
          </div>

          <div className="mt-6 space-y-4">
            {selectedSection.codeBlocks.map((block) => (
              <CodeBlock key={block.label} {...block} />
            ))}
          </div>
        </article>
      </section>
    </main>
  );
}
