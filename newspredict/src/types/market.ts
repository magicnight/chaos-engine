export interface Market {
  id: string;
  question: string;
  description?: string;
  category: string;
  imageUrl?: string;
  status: 'open' | 'closed' | 'resolved' | 'cancelled';
  creatorType: 'system' | 'user';
  yesPrice: number;
  noPrice: number;
  volume: number;
  traderCount: number;
  resolutionResult?: 'YES' | 'NO' | 'CANCELLED';
  closeAt: string;
  createdAt: string;
  tags?: string[];
  relatedSources?: string[];
}

export interface Trade {
  id: string;
  marketId: string;
  side: 'YES' | 'NO';
  shares: number;
  price: number;
  cost: number;
  createdAt: string;
}

export interface Position {
  id: string;
  marketId: string;
  side: 'YES' | 'NO';
  shares: number;
  avgPrice: number;
  realizedPnl: number;
  market?: Market;
}
