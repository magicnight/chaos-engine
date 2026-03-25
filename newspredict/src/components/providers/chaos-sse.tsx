'use client';

import { useEffect, useRef } from 'react';
import { useRouter } from 'next/navigation';

const CHAOS_URL = process.env.NEXT_PUBLIC_CHAOS_URL || 'http://localhost:3117';

/**
 * Invisible component that listens to CHAOS Engine SSE events.
 * When a sweep completes, triggers market re-seed and page refresh.
 */
export function ChaosSSE() {
  const router = useRouter();
  const reconnectTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    let es: EventSource | null = null;

    function connect() {
      try {
        es = new EventSource(`${CHAOS_URL}/api/v1/sse`);

        es.onmessage = async (e) => {
          try {
            const msg = JSON.parse(e.data);
            if (msg.type === 'update') {
              // CHAOS sweep completed — trigger server-side seed + resolve
              try {
                await fetch('/api/internal/sweep-hook', { method: 'POST' });
              } catch {}
              router.refresh();
            }
          } catch {}
        };

        es.onerror = () => {
          es?.close();
          reconnectTimer.current = setTimeout(connect, 10000);
        };
      } catch {
        reconnectTimer.current = setTimeout(connect, 10000);
      }
    }

    connect();

    return () => {
      es?.close();
      if (reconnectTimer.current !== null) clearTimeout(reconnectTimer.current);
    };
  }, [router]);

  return null;
}
