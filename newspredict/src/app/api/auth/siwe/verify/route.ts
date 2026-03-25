import { NextResponse } from 'next/server';
import { redis } from '@/lib/redis';
import { SiweMessage } from 'siwe';
import { db } from '@/lib/db';
import { users } from '@/lib/db/schema';
import { eq } from 'drizzle-orm';

export async function POST(request: Request) {
  try {
    const { message, signature } = await request.json();

    if (!message || !signature) {
      return NextResponse.json({ error: 'Missing message or signature' }, { status: 400 });
    }

    // Parse and verify SIWE message
    const siweMessage = new SiweMessage(message);
    const result = await siweMessage.verify({ signature });

    if (!result.success) {
      return NextResponse.json({ error: 'Invalid signature' }, { status: 401 });
    }

    // Verify domain matches (EIP-4361 security)
    const expectedDomain = process.env.NEXTAUTH_URL
      ? new URL(process.env.NEXTAUTH_URL).host
      : 'localhost:3000';
    if (siweMessage.domain !== expectedDomain) {
      return NextResponse.json({ error: 'Domain mismatch' }, { status: 401 });
    }

    // Verify and consume nonce (one-time use)
    const nonceKey = `siwe:nonce:${siweMessage.nonce}`;
    const nonceValid = await redis.get(nonceKey);
    if (!nonceValid) {
      return NextResponse.json({ error: 'Invalid or expired nonce' }, { status: 401 });
    }
    await redis.del(nonceKey); // Consume nonce - prevents replay

    const address = siweMessage.address.toLowerCase();

    // Find or create user by wallet address
    let [user] = await db.select().from(users).where(eq(users.walletAddress, address));

    if (!user) {
      [user] = await db.insert(users).values({
        walletAddress: address,
        name: `${address.slice(0, 6)}...${address.slice(-4)}`,
      }).returning();
    }

    return NextResponse.json({
      ok: true,
      user: { id: user.id, address, name: user.name },
    });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Verification failed';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
