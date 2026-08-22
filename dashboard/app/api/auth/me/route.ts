import { NextRequest, NextResponse } from 'next/server';
import { verifyUserSession } from '@/lib/auth/security';

export async function GET(request: NextRequest) {
  const sessionCookie = request.cookies.get('dagr_session')?.value;

  if (!sessionCookie) {
    return NextResponse.json({ authenticated: false, user: null }, { status: 401 });
  }

  const user = verifyUserSession(sessionCookie);
  if (!user) {
    return NextResponse.json(
      { authenticated: false, error: 'Invalid or expired session' },
      { status: 401 }
    );
  }

  return NextResponse.json({ authenticated: true, user });
}
