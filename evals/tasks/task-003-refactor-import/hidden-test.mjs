import { strict } from "node:assert";
import { cartTotal } from "./cart.ts";
strict.equal(cartTotal([1000, 500]), 1800);
console.log("ALL-PASS");
