import { pgTable, uuid, text, decimal, integer, timestamp, jsonb, uniqueIndex, primaryKey, check } from 'drizzle-orm/pg-core';
import { sql } from 'drizzle-orm';

export const users = pgTable('users', {
  id: uuid('id').defaultRandom().primaryKey(),
  email: text('email').unique(),
  name: text('name'),
  avatarUrl: text('avatar_url'),
  walletAddress: text('wallet_address').unique(),
  balance: decimal('balance', { precision: 18, scale: 2 }).default('1000.00').notNull(),
  totalTrades: integer('total_trades').default(0).notNull(),
  wins: integer('wins').default(0).notNull(),
  createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
  updatedAt: timestamp('updated_at', { withTimezone: true }).defaultNow().notNull(),
}, (t) => [
  check('balance_non_negative', sql`${t.balance} >= 0`),
]);

export const markets = pgTable('markets', {
  id: uuid('id').defaultRandom().primaryKey(),
  question: text('question').notNull(),
  description: text('description'),
  category: text('category').notNull(),
  imageUrl: text('image_url'),
  status: text('status').default('open').notNull(),
  creatorId: uuid('creator_id').references(() => users.id),
  creatorType: text('creator_type').default('system').notNull(),
  yesShares: decimal('yes_shares', { precision: 18, scale: 4 }).default('0').notNull(),
  noShares: decimal('no_shares', { precision: 18, scale: 4 }).default('0').notNull(),
  liquidityParam: decimal('liquidity_param', { precision: 8, scale: 2 }).default('100.00').notNull(),
  volume: decimal('volume', { precision: 18, scale: 2 }).default('0').notNull(),
  traderCount: integer('trader_count').default(0).notNull(),
  version: integer('version').default(0).notNull(),
  resolutionCriteria: text('resolution_criteria').notNull(),
  resolutionSource: text('resolution_source'),
  resolutionResult: text('resolution_result'),
  resolvedAt: timestamp('resolved_at', { withTimezone: true }),
  closeAt: timestamp('close_at', { withTimezone: true }).notNull(),
  createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
  crucixSeedId: text('crucix_seed_id').unique(),
  relatedSources: text('related_sources').array(),
  tags: text('tags').array(),
}, (t) => [
  check('market_status_valid', sql`${t.status} IN ('open', 'closed', 'resolved', 'cancelled')`),
  check('market_category_valid', sql`${t.category} IN ('geopolitics', 'economics', 'science', 'technology', 'health', 'environment', 'sports', 'entertainment', 'politics', 'other')`),
  check('market_creator_type_valid', sql`${t.creatorType} IN ('system', 'user', 'crucix')`),
]);

export const trades = pgTable('trades', {
  id: uuid('id').defaultRandom().primaryKey(),
  userId: uuid('user_id').notNull().references(() => users.id),
  marketId: uuid('market_id').notNull().references(() => markets.id),
  side: text('side').notNull(),
  shares: decimal('shares', { precision: 18, scale: 4 }).notNull(),
  price: decimal('price', { precision: 8, scale: 4 }).notNull(),
  cost: decimal('cost', { precision: 18, scale: 2 }).notNull(),
  txHash: text('tx_hash'),
  createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
}, (t) => [
  check('trade_side_valid', sql`${t.side} IN ('YES', 'NO')`),
  check('trade_shares_positive', sql`${t.shares} > 0`),
]);

export const positions = pgTable('positions', {
  id: uuid('id').defaultRandom().primaryKey(),
  userId: uuid('user_id').notNull().references(() => users.id),
  marketId: uuid('market_id').notNull().references(() => markets.id),
  side: text('side').notNull(),
  shares: decimal('shares', { precision: 18, scale: 4 }).notNull(),
  avgPrice: decimal('avg_price', { precision: 8, scale: 4 }).notNull(),
  realizedPnl: decimal('realized_pnl', { precision: 18, scale: 2 }).default('0').notNull(),
}, (t) => [
  uniqueIndex('positions_user_market_side').on(t.userId, t.marketId, t.side),
  check('position_side_valid', sql`${t.side} IN ('YES', 'NO')`),
  check('position_shares_positive', sql`${t.shares} > 0`),
]);

export const leaderboardSnapshots = pgTable('leaderboard_snapshots', {
  id: integer('id').primaryKey().generatedAlwaysAsIdentity(),
  period: text('period').notNull(),
  rankings: jsonb('rankings').notNull(),
  createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
});

export const comments = pgTable('comments', {
  id: uuid('id').defaultRandom().primaryKey(),
  marketId: uuid('market_id').notNull().references(() => markets.id),
  userId: uuid('user_id').notNull().references(() => users.id),
  content: text('content').notNull(),
  createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
}, (t) => [
  check('comment_content_length', sql`length(${t.content}) > 0 AND length(${t.content}) <= 2000`),
]);

export const notifications = pgTable('notifications', {
  id: uuid('id').defaultRandom().primaryKey(),
  userId: uuid('user_id').notNull().references(() => users.id),
  type: text('type').notNull(), // 'market_resolved' | 'trade_confirmed' | 'new_follower'
  title: text('title').notNull(),
  body: text('body'),
  link: text('link'),
  read: integer('read').default(0).notNull(),
  createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
});

export const achievements = pgTable('achievements', {
  id: uuid('id').defaultRandom().primaryKey(),
  userId: uuid('user_id').notNull().references(() => users.id),
  key: text('key').notNull(), // e.g. 'first_trade', 'win_streak_5'
  unlockedAt: timestamp('unlocked_at', { withTimezone: true }).defaultNow().notNull(),
});

export const follows = pgTable('follows', {
  followerId: uuid('follower_id').notNull().references(() => users.id),
  followingId: uuid('following_id').notNull().references(() => users.id),
  createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
}, (t) => [
  primaryKey({ columns: [t.followerId, t.followingId] }),
  check('no_self_follow', sql`${t.followerId} != ${t.followingId}`),
]);
