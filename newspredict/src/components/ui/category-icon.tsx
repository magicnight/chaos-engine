'use client';

import {
  TrendingUp, Globe, Cpu, AlertTriangle, CloudRain,
  Heart, Atom, Film, Trophy, HelpCircle,
} from 'lucide-react';
import type { LucideIcon } from 'lucide-react';

interface CategoryConfig {
  gradient: [string, string];
  Icon: LucideIcon;
}

const CATEGORY_MAP: Record<string, CategoryConfig> = {
  economics:     { gradient: ['#00d4ff', '#0066ff'], Icon: TrendingUp },
  politics:      { gradient: ['#f59e0b', '#ef4444'], Icon: Globe },
  technology:    { gradient: ['#8b5cf6', '#6366f1'], Icon: Cpu },
  geopolitics:   { gradient: ['#ef4444', '#dc2626'], Icon: AlertTriangle },
  environment:   { gradient: ['#22c55e', '#16a34a'], Icon: CloudRain },
  health:        { gradient: ['#ec4899', '#db2777'], Icon: Heart },
  science:       { gradient: ['#06b6d4', '#0891b2'], Icon: Atom },
  entertainment: { gradient: ['#f97316', '#ea580c'], Icon: Film },
  sports:        { gradient: ['#eab308', '#ca8a04'], Icon: Trophy },
  other:         { gradient: ['#6b7280', '#4b5563'], Icon: HelpCircle },
};

const SIZES = { sm: 24, md: 32, lg: 40 } as const;
const ICON_SIZES = { sm: 12, md: 16, lg: 20 } as const;
const RADII = { sm: 6, md: 8, lg: 10 } as const;

interface CategoryIconProps {
  category: string;
  size?: 'sm' | 'md' | 'lg';
}

export function CategoryIcon({ category, size = 'md' }: CategoryIconProps) {
  const config = CATEGORY_MAP[category] || CATEGORY_MAP.other;
  const { gradient, Icon } = config;
  const dim = SIZES[size];
  const iconDim = ICON_SIZES[size];
  const radius = RADII[size];

  return (
    <div
      aria-hidden="true"
      style={{
        width: dim,
        height: dim,
        borderRadius: radius,
        background: `linear-gradient(135deg, ${gradient[0]}, ${gradient[1]})`,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        flexShrink: 0,
      }}
    >
      <Icon size={iconDim} color="white" strokeWidth={2.5} />
    </div>
  );
}

/** Get the gradient colors for a category (for use in other contexts) */
export function getCategoryGradient(category: string): [string, string] {
  return (CATEGORY_MAP[category] || CATEGORY_MAP.other).gradient;
}
