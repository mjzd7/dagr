import { strict } from "node:assert";
import { createUser } from "./user.ts";
strict.deepEqual(createUser({ email: "a@b.co", age: 30 }), { email: "a@b.co", age: 30 });
strict.throws(() => createUser({ email: "", age: 30 }));
strict.throws(() => createUser({ email: "not-an-email", age: 30 }));
strict.throws(() => createUser({ email: "x@y.z", age: -1 }));
console.log("ALL-PASS");
