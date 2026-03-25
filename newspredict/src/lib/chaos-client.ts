import { redis } from './redis';

const CHAOS_URL = process.env.CHAOS_API_URL || 'http://localhost:3117';
const CHAOS_KEY = process.env.CHAOS_API_KEY || '';

interface CacheOptions {
  ttl?: number;
  key?: string;
}

async function chaosFetch(path: string, cache?: CacheOptions): Promise<unknown> {
  const cacheKey = cache?.key || `chaos:${path}`;

  if (cache?.ttl) {
    try {
      const cached = await redis.get(cacheKey);
      if (cached) return cached;
    } catch {
      // Redis failure is non-fatal
    }
  }

  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  if (CHAOS_KEY) headers['X-CHAOS-Key'] = CHAOS_KEY;

  const res = await fetch(`${CHAOS_URL}${path}`, {
    headers,
    next: { revalidate: cache?.ttl || 300 },
  });
  if (!res.ok) throw new Error(`CHAOS API ${res.status}: ${path}`);

  const data: unknown = await res.json();

  if (cache?.ttl) {
    try {
      await redis.setex(cacheKey, cache.ttl, JSON.stringify(data));
    } catch {
      // Redis failure is non-fatal
    }
  }

  return data;
}

export const chaosClient = {
  getData: () => chaosFetch('/api/v1/data', { ttl: 300 }),
  getHealth: () => chaosFetch('/api/v1/health', { ttl: 60 }),
  getEvents: () => chaosFetch('/api/v1/events', { ttl: 300 }),
  getCorrelations: () => chaosFetch('/api/v1/correlations', { ttl: 300 }),
  getMarketSeeds: () => chaosFetch('/api/v1/market-seeds', { ttl: 1800 }),
  getTrends: () => chaosFetch('/api/v1/trends', { ttl: 900 }),
  getSources: () => chaosFetch('/api/v1/sources', { ttl: 300 }),
};
