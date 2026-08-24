import { strict } from "node:assert";
import { discountedTotal } from "./billing.ts";
strict.equal(discountedTotal([1000, 2000, 3000], 10), 5400);
strict.equal(discountedTotal([500], 20), 400);
strict.equal(discountedTotal([], 50), 0);
console.log("ALL-PASS");
