import { query } from "./db/client";
export const total = () => query("SELECT total FROM cart").length;
