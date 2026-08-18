import { NextResponse } from 'next/server';

export async function POST(req: Request) {
  const body = await req.json();
  const { prId, action } = body;

  if (action === 'OVERRIDE_APPROVE') {
    return NextResponse.json({
      status: 'APPROVED',
      prId,
      message: `PR #${prId} override approved by human reviewer. De-quarantining commit.`,
      timestamp: new Date().toISOString()
    });
  }

  if (action === 'TRIGGER_AGENT_REPAIR') {
    return NextResponse.json({
      status: 'DISPATCHED_TO_AGENT',
      prId,
      message: `Diagnostic trace and AST slice sent to agent for autonomous self-repair.`,
      timestamp: new Date().toISOString()
    });
  }

  return NextResponse.json({ error: 'Invalid action' }, { status: 400 });
}
