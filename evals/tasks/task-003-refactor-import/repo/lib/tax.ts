export function taxFor(cents: number): number {
  return Math.round(cents * 0.2);
}
