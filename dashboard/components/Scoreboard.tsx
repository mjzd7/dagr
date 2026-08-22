import React from 'react';

export function Scoreboard() {
  return (
    <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
      <div className="p-5 rounded-2xl bg-zinc-950/80 border border-white/10 shadow-xl relative overflow-hidden group">
        <div className="text-xs font-mono text-zinc-400 uppercase tracking-wider">Avg Token Reduction</div>
        <div className="mt-2 flex items-baseline gap-2">
          <span className="text-3xl font-black text-white font-mono">96.8%</span>
          <span className="text-xs text-zinc-400 font-mono font-semibold">● 95%+ Target</span>
        </div>
        <div className="mt-2 text-xs text-zinc-500 font-mono">~35 lines per slice vs 1,200 in file</div>
      </div>

      <div className="p-5 rounded-2xl bg-zinc-950/80 border border-white/10 shadow-xl relative overflow-hidden group">
        <div className="text-xs font-mono text-zinc-400 uppercase tracking-wider">AST Traversal P99</div>
        <div className="mt-2 flex items-baseline gap-2">
          <span className="text-3xl font-black text-white font-mono">1.8ms</span>
          <span className="text-xs text-zinc-400 font-mono font-semibold">&lt;5ms SLA</span>
        </div>
        <div className="mt-2 text-xs text-zinc-500 font-mono">Tree-sitter native symbolic slicer</div>
      </div>

      <div className="p-5 rounded-2xl bg-zinc-950/80 border border-white/10 shadow-xl relative overflow-hidden group">
        <div className="text-xs font-mono text-zinc-400 uppercase tracking-wider">CoW Sandbox Rollback</div>
        <div className="mt-2 flex items-baseline gap-2">
          <span className="text-3xl font-black text-white font-mono">6.4ms</span>
          <span className="text-xs text-zinc-400 font-mono font-semibold">APFS Shadow</span>
        </div>
        <div className="mt-2 text-xs text-zinc-500 font-mono">Atomic rollback on failure</div>
      </div>

      <div className="p-5 rounded-2xl bg-zinc-950/80 border border-white/10 shadow-xl relative overflow-hidden group">
        <div className="text-xs font-mono text-zinc-400 uppercase tracking-wider">Team FinOps Savings</div>
        <div className="mt-2 flex items-baseline gap-2">
          <span className="text-3xl font-black text-white font-mono">$1,420</span>
          <span className="text-xs text-zinc-400 font-mono font-semibold">This Month</span>
        </div>
        <div className="mt-2 text-xs text-zinc-500 font-mono">Pruned 48.2M tokens across 12 repos</div>
      </div>
    </div>
  );
}
