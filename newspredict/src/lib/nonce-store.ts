// Nonce store: uses Redis if available, falls back to in-memory Map
import { redis } from './redis';

const memoryStore = new Map<string, number>(); // nonce -> expiry timestamp

export async function saveNonce(nonce: string, ttlSeconds: number = 300): Promise<void> {
  try {
    if (process.env.UPSTASH_REDIS_REST_URL) {
      await redis.setex(`siwe:nonce:${nonce}`, ttlSeconds, 'valid');
      return;
    }
  } catch {}

  // Fallback: in-memory
  memoryStore.set(nonce, Date.now() + ttlSeconds * 1000);
  // Prune expired entries
  const now = Date.now();
  for (const [k, exp] of memoryStore) {
    if (exp < now) memoryStore.delete(k);
  }
}

export async function consumeNonce(nonce: string): Promise<boolean> {
  try {
    if (process.env.UPSTASH_REDIS_REST_URL) {
      const val = await redis.get(`siwe:nonce:${nonce}`);
      if (!val) return false;
      await redis.del(`siwe:nonce:${nonce}`);
      return true;
    }
  } catch {}

  // Fallback: in-memory
  const exp = memoryStore.get(nonce);
  if (!exp || exp < Date.now()) {
    memoryStore.delete(nonce);
    return false;
  }
  memoryStore.delete(nonce);
  return true;
}
