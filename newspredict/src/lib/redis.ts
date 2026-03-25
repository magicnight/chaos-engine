import { Redis } from '@upstash/redis';

const url = process.env.UPSTASH_REDIS_REST_URL || '';
const token = process.env.UPSTASH_REDIS_REST_TOKEN || '';

const noop = { get: async () => null, setex: async () => {} } as unknown as Redis;

export const redis = url && token ? new Redis({ url, token }) : noop;
