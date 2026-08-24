import { taxFor } from "./lib/tax";

export function cartTotal(items: number[]): number {
  const sub = items.reduce((a, b) => a + b, 0);
  return sub + taxFor(sub);
}
