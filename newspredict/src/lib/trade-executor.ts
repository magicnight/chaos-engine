import { db } from './db';
import { users, markets, trades, positions } from './db/schema';
import { eq, and, sql } from 'drizzle-orm';
import { calculateCost, getPrice, sharesForCost } from './market-engine';
import { sendNotification } from './notify';
import { checkAchievements } from './achievements';

export interface TradeResult {
  success: boolean;
  tradeId?: string;
  shares?: number;
  cost?: number;
  newPrice?: { yes: number; no: number };
  error?: string;
}

/**
 * Execute a trade using optimistic concurrency control.
 *
 * NOTE: The Neon HTTP driver does not support interactive transactions.
 * We use optimistic concurrency via the `version` column on markets and
 * a conditional balance check on users to ensure consistency:
 * 1. Debit balance with WHERE balance >= cost (atomic check-and-set)
 * 2. Update market with WHERE version = expected (optimistic lock)
 * 3. If market update fails, refund the balance deduction
 * 4. Insert trade and upsert position only after both succeed
 */
export async function executeTrade(
  userId: string,
  marketId: string,
  side: 'YES' | 'NO',
  amount: number
): Promise<TradeResult> {
  try {
    // Step 1: Read current state
    const [market] = await db.select().from(markets).where(eq(markets.id, marketId));
    if (!market || market.status !== 'open') {
      return { success: false, error: 'Market not available' };
    }

    const [user] = await db.select().from(users).where(eq(users.id, userId));
    if (!user) return { success: false, error: 'User not found' };
    if (Number(user.balance) < amount) {
      return { success: false, error: 'Insufficient balance' };
    }

    // Step 2: Compute trade parameters
    const yesShares = Number(market.yesShares);
    const noShares = Number(market.noShares);
    const b = Number(market.liquidityParam);
    const shares = sharesForCost(yesShares, noShares, side, amount, b);
    const cost =
      side === 'YES'
        ? calculateCost(yesShares, noShares, shares, 0, b)
        : calculateCost(yesShares, noShares, 0, shares, b);

    const newYes = side === 'YES' ? yesShares + shares : yesShares;
    const newNo = side === 'NO' ? noShares + shares : noShares;
    const newPrice = getPrice(newYes, newNo, b);

    // Step 3: Atomic balance deduction (WHERE balance >= cost)
    const balanceResult = await db
      .update(users)
      .set({
        balance: sql`balance - ${cost.toFixed(2)}`,
        totalTrades: sql`total_trades + 1`,
      })
      .where(and(eq(users.id, userId), sql`CAST(balance AS DECIMAL) >= ${cost.toFixed(2)}`))
      .returning();

    if (balanceResult.length === 0) {
      return { success: false, error: 'Insufficient balance' };
    }

    // Step 4: Optimistic market update (WHERE version = expected)
    const updated = await db
      .update(markets)
      .set({
        yesShares: newYes.toFixed(4),
        noShares: newNo.toFixed(4),
        volume: sql`volume + ${cost.toFixed(2)}`,
        traderCount: sql`trader_count + 1`,
        version: sql`version + 1`,
      })
      .where(and(eq(markets.id, marketId), eq(markets.version, market.version)))
      .returning();

    if (updated.length === 0) {
      // Rollback: refund the balance deduction since market state changed
      await db
        .update(users)
        .set({
          balance: sql`balance + ${cost.toFixed(2)}`,
          totalTrades: sql`total_trades - 1`,
        })
        .where(eq(users.id, userId));
      return { success: false, error: 'Market state changed, please retry' };
    }

    // Step 5: Record the trade
    const [trade] = await db
      .insert(trades)
      .values({
        userId,
        marketId,
        side,
        shares: shares.toFixed(4),
        price: newPrice[side.toLowerCase() as 'yes' | 'no'].toFixed(4),
        cost: cost.toFixed(2),
      })
      .returning();

    // Step 6: Upsert position
    const [existing] = await db
      .select()
      .from(positions)
      .where(
        and(
          eq(positions.userId, userId),
          eq(positions.marketId, marketId),
          eq(positions.side, side)
        )
      );

    if (existing) {
      const totalShares = Number(existing.shares) + shares;
      const totalCost = Number(existing.avgPrice) * Number(existing.shares) + cost;
      const newAvg = totalCost / totalShares;
      await db
        .update(positions)
        .set({
          shares: totalShares.toFixed(4),
          avgPrice: newAvg.toFixed(4),
        })
        .where(eq(positions.id, existing.id));
    } else {
      await db.insert(positions).values({
        userId,
        marketId,
        side,
        shares: shares.toFixed(4),
        avgPrice: (cost / shares).toFixed(4),
      });
    }

    sendNotification(
      userId,
      'trade_confirmed',
      `Trade confirmed: ${side} on market`,
      `${shares.toFixed(1)} shares @ $${(cost / shares).toFixed(2)}`,
      `/markets/${marketId}`
    );

    // Check achievements (non-blocking)
    checkAchievements(userId).catch(() => {});

    return { success: true, tradeId: trade.id, shares, cost, newPrice };
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Trade execution failed';
    return { success: false, error: message };
  }
}
