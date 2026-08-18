import React from 'react';
import { Scoreboard } from '../components/Scoreboard';
import { DependencyGraph } from '../components/DependencyGraph';
import { QuarantineModal } from '../components/QuarantineModal';
import { LiveFeed } from '../components/LiveFeed';

export default function DashboardPage() {
  return (
    <div className="space-y-6">
      {/* 1. FinOps & Latency Scoreboard */}
      <Scoreboard />

      {/* 2. 3D AST Dependency Graph & Blast Radius Canvas */}
      <DependencyGraph />

      {/* 3. Grid for Quarantine & Live Event Stream */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <QuarantineModal />
        <LiveFeed />
      </div>
    </div>
  );
}
