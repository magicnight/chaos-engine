import { NextResponse } from 'next/server';
import crypto from 'crypto';
import { saveNonce } from '@/lib/nonce-store';

export async function GET() {
  const nonce = crypto.randomBytes(16).toString('hex');
  await saveNonce(nonce, 300);
  return NextResponse.json({ nonce });
}
