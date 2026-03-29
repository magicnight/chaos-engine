import { db } from './db';
import { copySettings } from './db/schema';
import { eq, and } from 'drizzle-orm';
import { executeTrade } from './trade-executor';
import { sendNotification } from './notify';

/**
 * After a leader executes a trade, mirror it to all active copy-followers.
 * Non-blocking — failures don't affect the leader's trade.
 */
export async function processCopyTrades(
  leaderId: string,
  marketId: string,
  side: 'YES' | 'NO'
) {
  try {
    const followers = await db
      .select()
      .from(copySettings)
      .where(and(eq(copySettings.leaderId, leaderId), eq(copySettings.active, 1)));

    for (const setting of followers) {
      try {
        const amount = setting.maxAmount;
        const result = await executeTrade(setting.followerId, marketId, side, amount);
        if (result.success) {
          sendNotification(
            setting.followerId,
            'trade_confirmed',
            'Copy trade executed',
            `Copied ${side} trade on market (${amount} credits)`,
            `/markets/${marketId}`
          );
        }
      } catch {
        // Individual copy-trade failure is non-fatal
      }
    }
  } catch {
    // DB query failure is non-fatal
  }
}
