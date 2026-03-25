export interface ChaosSweepData {
  chaos: {
    version: string;
    timestamp: string;
    totalDurationMs: number;
    sourcesQueried: number;
    sourcesOk: number;
    sourcesFailed: number;
  };
  sources: Record<string, unknown>;
  errors: Array<{ name: string; error: string }>;
  timing: Record<string, { status: string; ms: number }>;
  delta?: unknown;
  correlations?: unknown[];
  analysis?: string;
}

export interface MarketSeed {
  id: string;
  question: string;
  category: string;
  options: string[];
  resolution_criteria: string;
  resolution_source: string;
  confidence: number;
  context: string;
}

export interface ChaosEvent {
  id: string;
  category: string;
  title: string;
  source: string;
  lat?: number;
  lon?: number;
  timestamp: string;
  url?: string;
}
