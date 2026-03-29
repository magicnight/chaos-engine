import { NextRequest, NextResponse } from 'next/server';
import { auth } from '@/lib/auth';
import { db } from '@/lib/db';
import { achievements } from '@/lib/db/schema';
import { eq } from 'drizzle-orm';
import { getAchievementInfo } from '@/lib/achievements';

export async function GET(request: NextRequest) {
  try {
    const userId = request.nextUrl.searchParams.get('userId');
    const session = await auth();
    const targetId = userId || session?.user?.id;

    if (!targetId) {
      return NextResponse.json({ error: 'User ID required' }, { status: 400 });
    }

    const rows = await db
      .select()
      .from(achievements)
      .where(eq(achievements.userId, targetId));

    const list = rows.map((a: any) => {
      const info = getAchievementInfo(a.key);
      return {
        key: a.key,
        title: info.title,
        titleZh: info.titleZh,
        emoji: info.emoji,
        unlockedAt: a.unlockedAt.toISOString(),
      };
    });

    return NextResponse.json({ achievements: list });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Failed to fetch achievements';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
