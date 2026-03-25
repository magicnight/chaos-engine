import type { Metadata, Viewport } from 'next';
import { BottomNav } from '@/components/layout/bottom-nav';
import { Providers } from '@/components/providers';
import './globals.css';

export const metadata: Metadata = {
  title: 'NewsPredict',
  description: 'Predict the future with OSINT intelligence',
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
    <html lang="en" className="dark">
      <body className="min-h-screen bg-[var(--background)] text-[var(--foreground)] antialiased">
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
