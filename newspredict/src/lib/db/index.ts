import * as schema from './schema';

// Use standard pg for local PostgreSQL, neon for serverless (Vercel)
const isServerless = !!process.env.VERCEL || process.env.DB_DRIVER === 'neon';

let db: any;

if (isServerless) {
  // Neon serverless (Vercel deployment)
  const { neon } = require('@neondatabase/serverless');
  const { drizzle } = require('drizzle-orm/neon-http');
  const sql = neon(process.env.DATABASE_URL!);
  db = drizzle(sql, { schema });
} else {
  // Standard pg (local / Docker / Podman)
  const { drizzle } = require('drizzle-orm/node-postgres');
  const { Pool } = require('pg');
  const pool = new Pool({ connectionString: process.env.DATABASE_URL! });
  db = drizzle(pool, { schema });
}

export { db };
export type Database = typeof db;
