// Intentional violation for demos: UI importing the DB layer.
import { query } from "../db/client";

export function loadOrders() {
  return query("SELECT * FROM orders");
}
