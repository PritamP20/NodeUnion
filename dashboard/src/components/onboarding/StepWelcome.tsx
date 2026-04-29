const BENEFITS = [
  { title: "Earn per job", body: "Get paid when your machine processes real workloads." },
  { title: "Full control", body: "Choose your region, resources, and network participation." },
  { title: "Real-time monitoring", body: "Track node health and activity from the dashboard." },
];

export function StepWelcome() {
  return (
    <div>
      <p className="max-w-2xl text-sm leading-7 text-slate-300">
        NodeUnion connects your idle machine to decentralized compute demand. In a few steps, you will register your node and start receiving jobs.
      </p>

      <div className="mt-6 grid gap-3 sm:grid-cols-3">
        {BENEFITS.map((benefit) => (
          <article key={benefit.title} className="rounded-xl border border-white/10 bg-white/5 p-4">
            <h2 className="text-sm font-semibold text-slate-100">{benefit.title}</h2>
            <p className="mt-2 text-sm text-slate-300">{benefit.body}</p>
          </article>
        ))}
      </div>
    </div>
  );
}
