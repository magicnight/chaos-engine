/**
 * Simple in-memory IP rate limiter.
 * Suitable for single-instance deployments (Docker).
 * For multi-instance, replace with Upstash Ratelimit.
 */

const store = new Map<string, { count: number; resetAt: number }>();

// Clean stale entries every 5 minutes
setInterval(() => {
  const now = Date.now();
  for (const [key, val] of store) {
    if (val.resetAt < now) store.delete(key);
  }
}, 5 * 60 * 1000);

export function rateLimit(
  ip: string,
  key: string,
  limit: number,
  windowMs: number = 60_000
): { allowed: boolean; remaining: number } {
  const id = `${key}:${ip}`;
  const now = Date.now();
  const entry = store.get(id);

  if (!entry || entry.resetAt < now) {
    store.set(id, { count: 1, resetAt: now + windowMs });
    return { allowed: true, remaining: limit - 1 };
  }

  if (entry.count >= limit) {
    return { allowed: false, remaining: 0 };
  }

  entry.count++;
  return { allowed: true, remaining: limit - entry.count };
}
