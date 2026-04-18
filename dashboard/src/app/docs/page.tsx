export default function DocsPage() {
  return (
    <main className="mx-auto w-full max-w-6xl px-4 py-8 sm:px-6 lg:px-8">
      <header className="glass-card rounded-3xl p-6 sm:p-8">
        <p className="font-mono text-xs tracking-[0.2em] text-cyan-200/70">
          NODEUNION / END-TO-END DOCUMENTATION
        </p>
        <h1 className="mt-2 text-3xl font-semibold tracking-tight sm:text-4xl">
          NodeUnion platform documentation
        </h1>
        <p className="mt-3 text-sm text-slate-300 sm:text-base">
          This page documents architecture, user/provider journeys, API surface,
          and operating procedures from first deployment to daily operations.
        </p>
      </header>

      <section className="mt-6 grid grid-cols-1 gap-4 xl:grid-cols-2">
        <article className="doc-card p-5">
          <h2 className="section-title">1. Platform Overview</h2>
          <p className="mt-3 text-sm text-slate-300">
            NodeUnion is a decentralized compute exchange with an orchestrator control
            plane, wallet-linked users/providers, and blockchain-backed settlement data.
            Networks represent logical compute pools where providers register nodes and users
            deploy workloads.
          </p>
        </article>

        <article className="doc-card p-5">
          <h2 className="section-title">2. Core Components</h2>
          <ul className="mt-3 list-disc space-y-2 pl-5 text-sm text-slate-300">
            <li>Orchestrator API: admission, dispatch, node registry, and billing hooks.</li>
            <li>Provider Agent: executes assigned jobs and reports status/usage.</li>
            <li>Dashboard: operations UI for provider, user, and documentation workflows.</li>
            <li>Solana Program Layer: network/provider registry and escrow/settlement primitives.</li>
            <li>Postgres Data Layer: persistence for jobs, nodes, entitlements, and settlements.</li>
          </ul>
        </article>
      </section>

      <section className="mt-4 space-y-4">
        <article className="doc-card p-5">
          <h2 className="section-title">3. Role Journeys</h2>
          <div className="mt-3 grid grid-cols-1 gap-4 xl:grid-cols-2">
            <div className="rounded-xl border border-cyan-900/40 bg-slate-950/50 p-4">
              <h3 className="text-base font-semibold">Provider Journey</h3>
              <ol className="mt-2 list-decimal space-y-1 pl-5 text-sm text-slate-300">
                <li>Create or select a network from Provider + Deploy page.</li>
                <li>Register provider node with wallet, agent URL, and region metadata.</li>
                <li>Keep nodes idle-ready so orchestrator can schedule incoming jobs.</li>
                <li>Monitor provider portfolio to track assigned workloads by node.</li>
              </ol>
            </div>

            <div className="rounded-xl border border-cyan-900/40 bg-slate-950/50 p-4">
              <h3 className="text-base font-semibold">User Journey</h3>
              <ol className="mt-2 list-decimal space-y-1 pl-5 text-sm text-slate-300">
                <li>Connect wallet from Provider + Deploy page.</li>
                <li>Select desired network and submit image + command workload payload.</li>
                <li>Track job status from dashboard and orchestrator queue surfaces.</li>
                <li>Inspect credits and settlement ledger in Portfolio user mode.</li>
              </ol>
            </div>
          </div>
        </article>

        <article className="doc-card p-5">
          <h2 className="section-title">4. Dashboard Page Map</h2>
          <ul className="mt-3 list-disc space-y-2 pl-5 text-sm text-slate-300">
            <li>Landing: purpose, system flow, and live high-level metrics.</li>
            <li>Provider + Deploy: network creation, provider registration, and user deployment forms.</li>
            <li>Portfolio: wallet-role aware view for provider operations or user financial state.</li>
            <li>Docs: complete technical and operational guidance.</li>
          </ul>
        </article>

        <article className="doc-card p-5">
          <h2 className="section-title">5. API Endpoints Consumed by UI</h2>
          <div className="overflow-x-auto">
            <table className="mt-3 min-w-full text-left text-sm text-slate-300">
              <thead>
                <tr className="border-b border-cyan-900/40 text-slate-400">
                  <th className="px-2 py-2">Method</th>
                  <th className="px-2 py-2">Path</th>
                  <th className="px-2 py-2">Purpose</th>
                </tr>
              </thead>
              <tbody>
                <tr className="border-b border-cyan-950/60">
                  <td className="px-2 py-2">GET</td>
                  <td className="px-2 py-2">/networks</td>
                  <td className="px-2 py-2">List logical compute networks.</td>
                </tr>
                <tr className="border-b border-cyan-950/60">
                  <td className="px-2 py-2">POST</td>
                  <td className="px-2 py-2">/networks/create</td>
                  <td className="px-2 py-2">Register a network on-chain and in DB.</td>
                </tr>
                <tr className="border-b border-cyan-950/60">
                  <td className="px-2 py-2">GET</td>
                  <td className="px-2 py-2">/nodes</td>
                  <td className="px-2 py-2">List registered providers/nodes.</td>
                </tr>
                <tr className="border-b border-cyan-950/60">
                  <td className="px-2 py-2">POST</td>
                  <td className="px-2 py-2">/nodes/register</td>
                  <td className="px-2 py-2">Register provider node and wallet.</td>
                </tr>
                <tr className="border-b border-cyan-950/60">
                  <td className="px-2 py-2">GET</td>
                  <td className="px-2 py-2">/jobs</td>
                  <td className="px-2 py-2">List jobs and assignment status.</td>
                </tr>
                <tr className="border-b border-cyan-950/60">
                  <td className="px-2 py-2">POST</td>
                  <td className="px-2 py-2">/jobs/submit</td>
                  <td className="px-2 py-2">Submit workload for network scheduling.</td>
                </tr>
                <tr className="border-b border-cyan-950/60">
                  <td className="px-2 py-2">GET</td>
                  <td className="px-2 py-2">/users/:wallet/entitlements</td>
                  <td className="px-2 py-2">Fetch wallet credit lanes and usage.</td>
                </tr>
                <tr>
                  <td className="px-2 py-2">GET</td>
                  <td className="px-2 py-2">/users/:wallet/settlements</td>
                  <td className="px-2 py-2">Fetch wallet settlement transaction trail.</td>
                </tr>
              </tbody>
            </table>
          </div>
        </article>

        <article className="doc-card p-5">
          <h2 className="section-title">6. Deployment and Runtime Notes</h2>
          <ul className="mt-3 list-disc space-y-2 pl-5 text-sm text-slate-300">
            <li>Run orchestrator and dashboard together for proxy-based API access.</li>
            <li>Set ORCHESTRATOR_URL in dashboard environment when not using localhost defaults.</li>
            <li>Use devnet-compatible wallet/browser extension for end-to-end chain tests.</li>
            <li>Ensure provider agent URL is reachable by orchestrator for successful dispatch.</li>
            <li>Keep signer keys and RPC credentials in secure secret storage for production.</li>
          </ul>
        </article>

        <article className="doc-card p-5">
          <h2 className="section-title">7. Troubleshooting</h2>
          <ul className="mt-3 list-disc space-y-2 pl-5 text-sm text-slate-300">
            <li>Wallet connect fails: verify browser wallet extension is installed and unlocked.</li>
            <li>Provider registration fails: validate network exists and provider wallet format is correct.</li>
            <li>Job deploy fails: ensure wallet is connected and target network has idle providers.</li>
            <li>Portfolio data missing: confirm the same wallet was used for job/deployment activity.</li>
            <li>No live metrics: confirm orchestrator service is running and API proxy route is healthy.</li>
          </ul>
        </article>
      </section>
    </main>
  );
}
