'use client';
import React, { useState } from 'react';

export function QuarantineModal() {
  const [approved, setApproved] = useState(false);

  return (
    <div className="p-6 rounded-2xl bg-zinc-950/80 border border-white/10 shadow-2xl">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-base font-bold text-white flex items-center gap-2">
            Human-in-the-Loop (HITL) PR Quarantine
          </h2>
          <p className="text-xs text-zinc-400 font-mono mt-0.5">Automated Multi-Agent Chaos & Layer Boundary Interception</p>
        </div>
        <span className="px-2.5 py-1 rounded-full text-xs font-mono font-medium bg-white/10 text-white border border-white/20">
          1 PR In Quarantine
        </span>
      </div>

      <div className="mt-4 p-4 rounded-xl bg-black border border-white/10 text-xs font-mono">
        <div className="flex items-center justify-between text-zinc-300">
          <span className="font-bold text-sm text-white">PR #42: feat(checkout): parallelize inventory deduction</span>
          <span className="text-zinc-500">Author: agent-coder-v2</span>
        </div>

        <div className="mt-3 space-y-2 text-zinc-400">
          <div className="flex items-center gap-2">
            <span className="text-white font-bold">✓ AST Slicing:</span> Sliced 42 lines (96.2% compression ratio)
          </div>
          <div className="flex items-center gap-2">
            <span className="text-zinc-300 font-bold">✗ Chaos Matrix:</span> Lock contention failure detected during 30% CPU throttling simulation
          </div>
          <div className="flex items-center gap-2">
            <span className="text-zinc-400 font-bold">🔒 Cryptographic Proof:</span> proof_b7f83e291a84fa0c (Status: QUARANTINED)
          </div>
        </div>

        <div className="mt-4 flex items-center justify-end gap-3">
          <button 
            onClick={() => alert("Diagnostic trace dispatched to LLM agent for self-repair loop.")}
            className="px-3.5 py-2 rounded-lg bg-zinc-900 hover:bg-zinc-800 text-zinc-200 font-semibold transition border border-white/10 text-xs"
          >
            Trigger Agent Self-Repair
          </button>
          <button 
            onClick={() => setApproved(true)}
            className="px-3.5 py-2 rounded-lg bg-white hover:bg-zinc-200 text-black font-semibold transition text-xs shadow"
          >
            {approved ? "✓ Approved & Released" : "Override & Approve Merge"}
          </button>
        </div>
      </div>
    </div>
  );
}
