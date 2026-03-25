import NextAuth from 'next-auth';
import GitHub from 'next-auth/providers/github';
import Google from 'next-auth/providers/google';
import Credentials from 'next-auth/providers/credentials';
import { SiweMessage } from 'siwe';
import { db } from './db';
import { users } from './db/schema';
import { eq } from 'drizzle-orm';
import { consumeNonce } from './nonce-store';

export const { handlers, signIn, signOut, auth } = NextAuth({
  providers: [
    GitHub({
      clientId: process.env.GITHUB_CLIENT_ID,
      clientSecret: process.env.GITHUB_CLIENT_SECRET,
    }),
    Google({
      clientId: process.env.GOOGLE_CLIENT_ID,
      clientSecret: process.env.GOOGLE_CLIENT_SECRET,
    }),
    Credentials({
      id: 'guest',
      name: 'Guest',
      credentials: {},
      async authorize() {
        // Create anonymous guest user with starting balance
        const [user] = await db.insert(users).values({
          name: `Guest_${Math.random().toString(36).slice(2, 8)}`,
          balance: '1000.00',
        }).returning();
        return { id: user.id, name: user.name, email: null };
      },
    }),
    Credentials({
      id: 'siwe',
      name: 'Ethereum',
      credentials: {
        message: { type: 'text' },
        signature: { type: 'text' },
      },
      async authorize(credentials) {
        if (!credentials?.message || !credentials?.signature) return null;

        try {
          const siweMessage = new SiweMessage(credentials.message as string);

          // Verify domain to prevent phishing relay attacks
          const expectedDomain = new URL(
            process.env.NEXTAUTH_URL || 'http://localhost:3000'
          ).host;
          if (siweMessage.domain !== expectedDomain) return null;

          const result = await siweMessage.verify({
            signature: credentials.signature as string,
          });

          if (!result.success) return null;

          // Verify and consume nonce (one-time use)
          const nonceValid = await consumeNonce(siweMessage.nonce);
          if (!nonceValid) return null;

          const address = siweMessage.address.toLowerCase();

          // Find or create user
          let [user] = await db.select().from(users).where(eq(users.walletAddress, address));
          if (!user) {
            [user] = await db.insert(users).values({
              walletAddress: address,
              name: `${address.slice(0, 6)}...${address.slice(-4)}`,
            }).returning();
          }

          return { id: user.id, name: user.name, email: user.email };
        } catch {
          return null;
        }
      },
    }),
  ],
  session: { strategy: 'jwt' },
  pages: {
    signIn: '/sign-in',
  },
  callbacks: {
    async signIn({ user, account }) {
      // Guest and SIWE users are already created in authorize()
      if (account?.provider === 'guest' || account?.provider === 'siwe') return true;
      if (!user.email) return true;
      const existing = await db.select().from(users).where(eq(users.email, user.email)).limit(1);
      if (existing.length === 0) {
        const [newUser] = await db.insert(users).values({
          email: user.email,
          name: user.name || null,
          avatarUrl: user.image || null,
          balance: '1000.00',
        }).returning();
        user.id = newUser.id;
      } else {
        user.id = existing[0].id;
      }
      return true;
    },
    session({ session, token }) {
      if (token?.sub) session.user.id = token.sub;
      return session;
    },
    jwt({ token, user }) {
      if (user?.id) token.sub = user.id;
      return token;
    },
  },
});
