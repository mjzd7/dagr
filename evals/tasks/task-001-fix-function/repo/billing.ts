export function discountedTotal(cents: number[], discountPct: number): number {
  const subtotal = cents.reduce((a, b) => a + b, 0);
  // BUG: applies discount only to first item
  return Math.round(subtotal - cents[0] * (discountPct / 100));
}
