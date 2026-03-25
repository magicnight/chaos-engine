import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

// Lightweight middleware — check for NextAuth session cookie.
// Does NOT import auth.ts (which pulls in pg/drizzle, incompatible with Edge Runtime).
export function middleware(request: NextRequest) {
  const sessionCookie =
    request.cookies.get('authjs.session-token') ||
    request.cookies.get('__Secure-authjs.session-token') ||
    request.cookies.get('next-auth.session-token') ||
    request.cookies.get('__Secure-next-auth.session-token');

  if (!sessionCookie) {
    const signInUrl = new URL('/sign-in', request.url);
    signInUrl.searchParams.set('callbackUrl', request.nextUrl.pathname);
    return NextResponse.redirect(signInUrl);
  }

  return NextResponse.next();
}

export const config = {
  matcher: ['/create', '/portfolio', '/activity', '/profile/:path*'],
};
