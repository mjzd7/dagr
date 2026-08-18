'use client';
import React, { useState } from 'react';

export function QuarantineModal() {
  const [approved, setApproved] = useState(false);

  return (
    <div className="p-6 rounded-2xl bg-slate-900/60 border border-slate-800/80 shadow-2xl">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-bold text-slate-100 flex items-center gap-2">
            🛡️ Human-in-the-Loop (HITL) PR Quarantine
          </h2>
          <p className="text-xs text-slate-400 font-mono mt-0.5">Automated Multi-Agent Chaos & Layer Boundary Interception</p>
        </div>
        <span className="px-3 py-1 rounded-full text-xs font-mono font-bold bg-amber-500/10 text-amber-400 border border-amber-500/30">
          1 PR Pending Review
        </span>
      </div>

      <div className="mt-4 p-4 rounded-xl bg-slate-950/80 border border-slate-800 text-xs font-mono">
        <div className="flex items-center justify-between text-slate-300">
          <span className="font-bold text-sm text-cyan-400">PR #42: feat(checkout): parallelize inventory deduction</span>
          <span className="text-slate-500">Author: agent-coder-v2</span>
        </div>

        <div className="mt-3 space-y-2 text-slate-400">
          <div className="flex items-center gap-2">
            <span className="text-emerald-400 font-bold">✓ AST Slicing:</span> Sliced 42 lines (96.2% compression ratio)
          </div>
          <div className="flex items-center gap-2">
            <span className="text-rose-400 font-bold">✗ Chaos Matrix:</span> Lock contention failure detected during 30% CPU throttling simulation
          </div>
          <div className="flex items-center gap-2">
            <span className="text-indigo-400 font-bold">🔒 Cryptographic Proof:</span> proof_b7f83e291a84fa0c (Status: QUARANTINED)
          </div>
        </div>

        <div className="mt-4 flex items-center justify-end gap-3">
          <button 
            onClick={() => alert("Diagnostic trace dispatched to LLM agent for self-repair loop.")}
            className="px-4 py-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 font-semibold transition"
          >
            Trigger Agent Self-Repair
          </button>
          <button 
            onClick={() => setApproved(true)}
            className="px-4 py-2 rounded-lg bg-cyan-600 hover:bg-cyan-500 text-white font-semibold transition shadow-lg shadow-cyan-600/30"
          >
            {approved ? "✓ Approved & De-quarantined" : "Override & Approve Merge"}
          </button>
        </div>
      </div>
    </div>
  );
}
