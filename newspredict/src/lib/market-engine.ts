const DEFAULT_LIQUIDITY = 100;
const MAX_SHARES_PER_SIDE = 50000;

/** Numerically stable log-sum-exp: log(exp(a) + exp(b)) = max(a,b) + log(1 + exp(-|a-b|)) */
function logSumExp(a: number, b: number): number {
  const max = Math.max(a, b);
  return max + Math.log(1 + Math.exp(-Math.abs(a - b)));
}

/** LMSR cost function (numerically stable) */
export function calculateCost(
  currentYesShares: number,
  currentNoShares: number,
  buyYesShares: number,
  buyNoShares: number,
  b: number = DEFAULT_LIQUIDITY
): number {
  if (
    currentYesShares + buyYesShares > MAX_SHARES_PER_SIDE ||
    currentNoShares + buyNoShares > MAX_SHARES_PER_SIDE
  ) {
    throw new Error('Market share limit exceeded');
  }
  const before = b * logSumExp(currentYesShares / b, currentNoShares / b);
  const after = b * logSumExp(
    (currentYesShares + buyYesShares) / b,
    (currentNoShares + buyNoShares) / b
  );
  return after - before;
}

/** Current YES/NO prices (sigmoid, numerically stable) */
export function getPrice(
  yesShares: number,
  noShares: number,
  b: number = DEFAULT_LIQUIDITY
): { yes: number; no: number } {
  const diff = (yesShares - noShares) / b;
  const yesPrice = 1 / (1 + Math.exp(-diff));
  return { yes: yesPrice, no: 1 - yesPrice };
}

/** Calculate how many shares you get for a given spend amount */
export function sharesForCost(
  currentYesShares: number,
  currentNoShares: number,
  side: 'YES' | 'NO',
  spendAmount: number,
  b: number = DEFAULT_LIQUIDITY
): number {
  let lo = 0;
  let hi = MAX_SHARES_PER_SIDE - (side === 'YES' ? currentYesShares : currentNoShares);
  hi = Math.min(hi, spendAmount * 10);

  for (let i = 0; i < 50; i++) {
    const mid = (lo + hi) / 2;
    const cost =
      side === 'YES'
        ? calculateCost(currentYesShares, currentNoShares, mid, 0, b)
        : calculateCost(currentYesShares, currentNoShares, 0, mid, b);
    if (cost < spendAmount) lo = mid;
    else hi = mid;
  }
  return (lo + hi) / 2;
}
