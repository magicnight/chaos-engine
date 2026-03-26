import { db } from './db';
import { notifications } from './db/schema';

type NotificationType = 'market_resolved' | 'trade_confirmed' | 'new_follower';

export async function sendNotification(
  userId: string,
  type: NotificationType,
  title: string,
  body?: string,
  link?: string
) {
  try {
    await db.insert(notifications).values({
      userId,
      type,
      title,
      body: body || null,
      link: link || null,
    });
  } catch {
    // Non-fatal — don't break the calling operation
  }
}
