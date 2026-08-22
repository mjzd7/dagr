import React from 'react';

export function LiveFeed() {
  const events = [
    { time: '15:32:04', event: 'ProofGenerated', detail: 'Proof-of-Correctness issued for commit a1b2c3 (Status: PASS)', badge: 'bg-white/10 text-white border-white/20' },
    { time: '15:31:45', event: 'ChaosRunner', detail: 'Injected 500ms network jitter on microVM shadow sandbox', badge: 'bg-white/5 text-zinc-300 border-white/10' },
    { time: '15:30:12', event: 'CommitIngested', detail: 'Parsed 14 nodes from repo://mjzd7/dagr (Idempotency verified)', badge: 'bg-white/5 text-zinc-300 border-white/10' },
    { time: '15:28:50', event: 'MCPToolCall', detail: 'Cursor IDE invoked dagr_get_context_slice on chargeCustomer (34 lines)', badge: 'bg-white/5 text-zinc-300 border-white/10' },
  ];

  return (
    <div className="p-6 rounded-2xl bg-zinc-950/80 border border-white/10 shadow-2xl">
      <h2 className="text-base font-bold text-white flex items-center gap-2 mb-4">
        Live CDC Event Stream (Redpanda)
      </h2>
      <div className="space-y-3 font-mono text-xs">
        {events.map((e, idx) => (
          <div key={idx} className="flex items-center justify-between p-3 rounded-lg bg-black border border-white/10">
            <div className="flex items-center gap-3">
              <span className={`px-2 py-0.5 rounded text-[11px] font-bold border ${e.badge}`}>{e.event}</span>
              <span className="text-zinc-300">{e.detail}</span>
            </div>
            <span className="text-zinc-500">{e.time}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
