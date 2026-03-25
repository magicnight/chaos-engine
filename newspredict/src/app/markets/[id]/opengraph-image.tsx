import { ImageResponse } from 'next/og';
import { db } from '@/lib/db';
import { markets } from '@/lib/db/schema';
import { eq } from 'drizzle-orm';
import { getPrice } from '@/lib/market-engine';

export const runtime = 'nodejs';
export const size = { width: 1200, height: 630 };
export const contentType = 'image/png';

export default async function OGImage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;

  const [market] = await db.select().from(markets).where(eq(markets.id, id));
  if (!market) {
    return new ImageResponse(
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', width: '100%', height: '100%', background: '#0b1220', color: '#e2e8f0', fontSize: 48 }}>
        C.H.A.O.S. | Market Not Found
      </div>,
      { ...size }
    );
  }

  const price = getPrice(Number(market.yesShares), Number(market.noShares), Number(market.liquidityParam));
  const yesPercent = Math.round(price.yes * 100);
  const noPercent = 100 - yesPercent;

  return new ImageResponse(
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      width: '100%',
      height: '100%',
      background: 'linear-gradient(135deg, #0b1220 0%, #1a2332 100%)',
      padding: '60px',
      fontFamily: 'system-ui, sans-serif',
    }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '24px' }}>
        <div style={{ color: '#00d4ff', fontSize: '20px', fontWeight: 700, letterSpacing: '0.1em' }}>
          C.H.A.O.S.
        </div>
        <div style={{ color: '#64748b', fontSize: '16px' }}>|</div>
        <div style={{ color: '#00d4ff', fontSize: '14px', textTransform: 'uppercase', letterSpacing: '0.15em', background: 'rgba(0,212,255,0.1)', padding: '4px 12px', borderRadius: '9999px' }}>
          {market.category}
        </div>
      </div>

      <div style={{ color: '#e2e8f0', fontSize: '42px', fontWeight: 700, lineHeight: 1.3, marginBottom: '40px', maxWidth: '900px' }}>
        {market.question}
      </div>

      <div style={{ display: 'flex', gap: '24px', marginTop: 'auto' }}>
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', background: 'rgba(34,197,94,0.1)', borderRadius: '16px', padding: '20px 40px' }}>
          <div style={{ color: '#22c55e', fontSize: '48px', fontWeight: 700 }}>{yesPercent}%</div>
          <div style={{ color: '#22c55e', fontSize: '16px', fontWeight: 600 }}>YES</div>
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', background: 'rgba(239,68,68,0.1)', borderRadius: '16px', padding: '20px 40px' }}>
          <div style={{ color: '#ef4444', fontSize: '48px', fontWeight: 700 }}>{noPercent}%</div>
          <div style={{ color: '#ef4444', fontSize: '16px', fontWeight: 600 }}>NO</div>
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', justifyContent: 'center', marginLeft: 'auto', color: '#64748b', fontSize: '14px' }}>
          <div>Volume: ${Number(market.volume).toLocaleString()}</div>
          <div>{market.traderCount} traders</div>
        </div>
      </div>
    </div>,
    { ...size }
  );
}
