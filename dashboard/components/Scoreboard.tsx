import React from 'react';

export function Scoreboard() {
  return (
    <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
      <div className="p-5 rounded-2xl bg-slate-900/60 border border-slate-800/80 shadow-xl backdrop-blur-sm relative overflow-hidden group">
        <div className="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-emerald-400 to-cyan-500"></div>
        <div className="text-xs font-mono text-slate-400 uppercase tracking-wider">Avg Token Reduction</div>
        <div className="mt-2 flex items-baseline gap-2">
          <span className="text-3xl font-black text-emerald-400 font-mono">96.8%</span>
          <span className="text-xs text-emerald-500 font-semibold">🟢 +2.1%</span>
        </div>
        <div className="mt-2 text-xs text-slate-400">~35 lines per slice vs 1,200 in file</div>
      </div>

      <div className="p-5 rounded-2xl bg-slate-900/60 border border-slate-800/80 shadow-xl backdrop-blur-sm relative overflow-hidden group">
        <div className="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-cyan-400 to-indigo-500"></div>
        <div className="text-xs font-mono text-slate-400 uppercase tracking-wider">Symbol Slicing P99</div>
        <div className="mt-2 flex items-baseline gap-2">
          <span className="text-3xl font-black text-cyan-400 font-mono">1.8ms</span>
          <span className="text-xs text-cyan-500 font-semibold">⚡ Sub-5ms SLA</span>
        </div>
        <div className="mt-2 text-xs text-slate-400">Tree-sitter native AST traversal</div>
      </div>

      <div className="p-5 rounded-2xl bg-slate-900/60 border border-slate-800/80 shadow-xl backdrop-blur-sm relative overflow-hidden group">
        <div className="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-indigo-400 to-purple-500"></div>
        <div className="text-xs font-mono text-slate-400 uppercase tracking-wider">CoW Sandbox Rollback</div>
        <div className="mt-2 flex items-baseline gap-2">
          <span className="text-3xl font-black text-indigo-400 font-mono">6.4ms</span>
          <span className="text-xs text-indigo-400 font-semibold">APFS CoW</span>
        </div>
        <div className="mt-2 text-xs text-slate-400">100% clean zero residual bytes</div>
      </div>

      <div className="p-5 rounded-2xl bg-slate-900/60 border border-slate-800/80 shadow-xl backdrop-blur-sm relative overflow-hidden group">
        <div className="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-purple-400 to-pink-500"></div>
        <div className="text-xs font-mono text-slate-400 uppercase tracking-wider">FinOps Token Savings</div>
        <div className="mt-2 flex items-baseline gap-2">
          <span className="text-3xl font-black text-purple-400 font-mono">$1,420</span>
          <span className="text-xs text-emerald-400 font-semibold">This Month</span>
        </div>
        <div className="mt-2 text-xs text-slate-400">Saved 48.2M tokens across 12 repos</div>
      </div>
    </div>
  );
}
