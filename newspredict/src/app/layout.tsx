import type { Metadata, Viewport } from 'next';
import { Geist, Geist_Mono } from 'next/font/google';
import { BottomNav } from '@/components/layout/bottom-nav';
import { Providers } from '@/components/providers';
import './globals.css';

const geist = Geist({ subsets: ['latin'], variable: '--font-geist' });
const geistMono = Geist_Mono({ subsets: ['latin'], variable: '--font-geist-mono' });

export const metadata: Metadata = {
  title: 'C.H.A.O.S. | NewsPredict',
  description: 'OSINT-powered prediction markets — trade on real-time intelligence from 44 sources',
  manifest: '/manifest.json',
};

export const viewport: Viewport = {
  width: 'device-width',
  initialScale: 1,
  maximumScale: 1,
  themeColor: '#0b1220',
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning className={`dark ${geist.variable} ${geistMono.variable}`}>
      <body className="min-h-screen bg-[var(--background)] text-[var(--foreground)] antialiased font-[family-name:var(--font-geist)]">
        <Providers>
          <main className="pb-20">{children}</main>
          <BottomNav />
        </Providers>
        <script
          dangerouslySetInnerHTML={{
            __html: "if('serviceWorker'in navigator){navigator.serviceWorker.register('/sw.js')}"
          }}
        />
      </body>
    </html>
  );
}
