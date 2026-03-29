import { db } from './db';
import { achievements, users, trades, positions } from './db/schema';
import { eq, sql, and } from 'drizzle-orm';
import { sendNotification } from './notify';

interface AchievementDef {
  key: string;
  title: string;
  titleZh: string;
  emoji: string;
  check: (userId: string) => Promise<boolean>;
}

const ACHIEVEMENTS: AchievementDef[] = [
  {
    key: 'first_trade',
    title: 'First Trade',
    titleZh: '首次交易',
    emoji: '🎯',
    check: async (userId) => {
      const [row] = await db.select({ count: sql<number>`count(*)::int` }).from(trades).where(eq(trades.userId, userId));
      return row.count >= 1;
    },
  },
  {
    key: 'trade_10',
    title: '10 Trades',
    titleZh: '10 笔交易',
    emoji: '📊',
    check: async (userId) => {
      const [row] = await db.select({ count: sql<number>`count(*)::int` }).from(trades).where(eq(trades.userId, userId));
      return row.count >= 10;
    },
  },
  {
    key: 'trade_50',
    title: '50 Trades',
    titleZh: '50 笔交易',
    emoji: '🔥',
    check: async (userId) => {
      const [row] = await db.select({ count: sql<number>`count(*)::int` }).from(trades).where(eq(trades.userId, userId));
      return row.count >= 50;
    },
  },
  {
    key: 'first_win',
    title: 'First Win',
    titleZh: '首次获胜',
    emoji: '🏆',
    check: async (userId) => {
      const [user] = await db.select({ wins: users.wins }).from(users).where(eq(users.id, userId));
      return (user?.wins || 0) >= 1;
    },
  },
  {
    key: 'win_5',
    title: '5 Wins',
    titleZh: '5 次获胜',
    emoji: '⭐',
    check: async (userId) => {
      const [user] = await db.select({ wins: users.wins }).from(users).where(eq(users.id, userId));
      return (user?.wins || 0) >= 5;
    },
  },
  {
    key: 'profit_1k',
    title: '$1K Profit',
    titleZh: '盈利 $1K',
    emoji: '💰',
    check: async (userId) => {
      const [user] = await db.select({ balance: users.balance }).from(users).where(eq(users.id, userId));
      return Number(user?.balance || 0) - 1000 >= 1000;
    },
  },
  {
    key: 'multi_market',
    title: 'Diversified',
    titleZh: '分散投资',
    emoji: '🌐',
    check: async (userId) => {
      const [row] = await db.select({ count: sql<number>`count(distinct ${positions.marketId})::int` }).from(positions).where(eq(positions.userId, userId));
      return row.count >= 5;
    },
  },
];

export async function checkAchievements(userId: string) {
  // Get existing achievements
  const existing = await db
    .select({ key: achievements.key })
    .from(achievements)
    .where(eq(achievements.userId, userId));
  const existingKeys = new Set(existing.map((a) => a.key));

  for (const def of ACHIEVEMENTS) {
    if (existingKeys.has(def.key)) continue;
    try {
      const earned = await def.check(userId);
      if (earned) {
        await db.insert(achievements).values({ userId, key: def.key }).onConflictDoNothing();
        sendNotification(userId, 'trade_confirmed', `${def.emoji} ${def.title}`, `Achievement unlocked!`, '/profile');
      }
    } catch {
      // Non-fatal
    }
  }
}

export function getAchievementInfo(key: string) {
  return ACHIEVEMENTS.find((a) => a.key === key) || { key, title: key, titleZh: key, emoji: '🏅' };
}

export { ACHIEVEMENTS };
