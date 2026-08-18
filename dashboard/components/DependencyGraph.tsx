'use client';
import React, { useState } from 'react';

export function DependencyGraph() {
  const [selectedNode, setSelectedNode] = useState('processPayment');

  const nodes = [
    { id: 'processPayment', file: 'src/billing/charge.ts', type: 'Function', color: 'border-cyan-500 text-cyan-400 bg-cyan-950/40' },
    { id: 'PaymentIntent', file: 'src/billing/types.ts', type: 'Interface', color: 'border-indigo-500 text-indigo-400 bg-indigo-950/40' },
    { id: 'PaymentResult', file: 'src/billing/types.ts', type: 'Interface', color: 'border-indigo-500 text-indigo-400 bg-indigo-950/40' },
    { id: 'StripeGateway', file: 'src/infra/stripe.ts', type: 'Class', color: 'border-emerald-500 text-emerald-400 bg-emerald-950/40' },
    { id: 'dbClient', file: 'src/db/client.ts', type: 'DbTable', color: 'border-rose-500 text-rose-400 bg-rose-950/40' },
  ];

  return (
    <div className="p-6 rounded-2xl bg-slate-900/60 border border-slate-800/80 shadow-2xl relative">
      <div className="flex items-center justify-between mb-4">
        <div>
          <h2 className="text-lg font-bold text-slate-100 flex items-center gap-2">
            🕸️ 3D Blast Radius & AST Dependency Graph
          </h2>
          <p className="text-xs text-slate-400 font-mono mt-0.5">Memgraph Bolt Query: 2-Hop APOC Transitive Closure</p>
        </div>
        <div className="flex items-center gap-2 text-xs font-mono">
          <span className="px-2.5 py-1 rounded-md bg-slate-800 text-slate-300">Seed: {selectedNode}</span>
        </div>
      </div>

      <div className="h-72 w-full rounded-xl bg-black/40 border border-slate-800/60 flex items-center justify-center relative overflow-hidden p-6">
        <div className="absolute inset-0 bg-[radial-gradient(#1e293b_1px,transparent_1px)] [background-size:16px_16px] opacity-40"></div>
        
        {/* Visual Graph Nodes */}
        <div className="grid grid-cols-3 gap-8 w-full max-w-xl z-10">
          <div className="col-span-1 flex flex-col gap-4">
            <div 
              onClick={() => setSelectedNode('PaymentIntent')}
              className="p-3 rounded-lg border border-indigo-500/40 bg-indigo-950/30 text-indigo-300 text-xs font-mono cursor-pointer hover:border-indigo-400 transition"
            >
              <div className="text-[10px] text-slate-400">INTERFACE</div>
              <div className="font-bold">PaymentIntent</div>
            </div>
            <div 
              onClick={() => setSelectedNode('PaymentResult')}
              className="p-3 rounded-lg border border-indigo-500/40 bg-indigo-950/30 text-indigo-300 text-xs font-mono cursor-pointer hover:border-indigo-400 transition"
            >
              <div className="text-[10px] text-slate-400">INTERFACE</div>
              <div className="font-bold">PaymentResult</div>
            </div>
          </div>

          <div className="col-span-1 flex items-center justify-center">
            <div 
              onClick={() => setSelectedNode('processPayment')}
              className="p-4 rounded-xl border-2 border-cyan-400 bg-cyan-950/60 text-cyan-200 text-center font-mono cursor-pointer shadow-lg shadow-cyan-500/20 scale-110"
            >
              <div className="text-[10px] text-cyan-400 font-bold">TARGET SYMBOL</div>
              <div className="font-bold text-sm">processPayment</div>
              <div className="text-[10px] text-slate-400 mt-1">34 lines • 342 tokens</div>
            </div>
          </div>

          <div className="col-span-1 flex flex-col gap-4">
            <div 
              onClick={() => setSelectedNode('StripeGateway')}
              className="p-3 rounded-lg border border-emerald-500/40 bg-emerald-950/30 text-emerald-300 text-xs font-mono cursor-pointer hover:border-emerald-400 transition"
            >
              <div className="text-[10px] text-slate-400">CALLS</div>
              <div className="font-bold">StripeGateway</div>
            </div>
            <div 
              onClick={() => setSelectedNode('dbClient')}
              className="p-3 rounded-lg border border-rose-500/40 bg-rose-950/30 text-rose-300 text-xs font-mono cursor-pointer hover:border-rose-400 transition"
            >
              <div className="text-[10px] text-rose-400">BLOCKED BOUNDARY</div>
              <div className="font-bold">@db/client</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
