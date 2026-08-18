import React from 'react';

export function LiveFeed() {
  const events = [
    { time: '15:32:04', event: 'ProofGenerated', detail: 'Proof-of-Correctness issued for commit a1b2c3 (Status: GREEN)', badge: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20' },
    { time: '15:31:45', event: 'ChaosRunner', detail: 'Injected 500ms network jitter on microVM shadow sandbox', badge: 'bg-purple-500/10 text-purple-400 border-purple-500/20' },
    { time: '15:30:12', event: 'CommitIngested', detail: 'Parsed 14 nodes from repo://mjzd7/dagr (Idempotency verified)', badge: 'bg-cyan-500/10 text-cyan-400 border-cyan-500/20' },
    { time: '15:28:50', event: 'MCPToolCall', detail: 'Cursor IDE invoked dagr_get_context_slice on chargeCustomer (34 lines)', badge: 'bg-slate-800 text-slate-300 border-slate-700' },
  ];

  return (
    <div className="p-6 rounded-2xl bg-slate-900/60 border border-slate-800/80 shadow-2xl">
      <h2 className="text-lg font-bold text-slate-100 flex items-center gap-2 mb-4">
        ⚡ Live CDC Event Stream (Redpanda)
      </h2>
      <div className="space-y-3 font-mono text-xs">
        {events.map((e, idx) => (
          <div key={idx} className="flex items-center justify-between p-3 rounded-lg bg-slate-950/60 border border-slate-800/60">
            <div className="flex items-center gap-3">
              <span className={`px-2 py-0.5 rounded text-[11px] font-bold border ${e.badge}`}>{e.event}</span>
              <span className="text-slate-300">{e.detail}</span>
            </div>
            <span className="text-slate-500">{e.time}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
