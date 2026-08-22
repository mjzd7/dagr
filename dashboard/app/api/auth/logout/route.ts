import { NextRequest, NextResponse } from 'next/server';

export async function POST(request: NextRequest) {
  const origin = request.nextUrl.origin;
  const response = NextResponse.json({ success: true, message: 'Logged out' });

  response.cookies.set('dagr_session', '', {
    path: '/',
    expires: new Date(0),
    httpOnly: true,
  });

  return response;
}

export async function GET(request: NextRequest) {
  const origin = request.nextUrl.origin;
  const response = NextResponse.redirect(`${origin}/login`);

  response.cookies.set('dagr_session', '', {
    path: '/',
    expires: new Date(0),
    httpOnly: true,
  });

  return response;
}
