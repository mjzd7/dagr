const SIMULATOR_SCENARIOS = {
  typescript: {
    name: "TypeScript // Stripe Payment Slicer",
    target: "src/billing/charge.ts:processPayment",
    rawFile: `// src/billing/charge.ts (1,180 lines monolithic file)
import { StripeClient } from "@/lib/stripe";
import { DatabasePool } from "@/db/connection";
import { MetricsCollector } from "@/telemetry/metrics";
import { InvoiceGenerator } from "@/billing/invoicing";
import { NotificationService } from "@/notify/slack";
import { AuditLogger } from "@/security/audit";
import { FeatureFlags } from "@/config/features";
import { CurrencyConverter } from "@/utils/currency";

// ... 850 lines of unrelated helpers, refunds, tax calculations, webhook handlers ...

export interface PaymentPayload {
  customerId: string;
  amountCents: number;
  currency: string;
  idempotencyKey: string;
}

export interface PaymentReceipt {
  transactionId: string;
  status: "succeeded" | "pending" | "failed";
  timestamp: number;
}

export async function processPayment(payload: PaymentPayload, stripe: StripeClient): Promise<PaymentReceipt> {
  validatePaymentPayload(payload);
  const rate = await CurrencyConverter.getExchangeRate(payload.currency, "USD");
  const charge = await stripe.charges.create({
    amount: payload.amountCents,
    currency: payload.currency,
    customer: payload.customerId,
  }, { idempotencyKey: payload.idempotencyKey });

  AuditLogger.log("payment.created", { chargeId: charge.id, amount: payload.amountCents });
  return {
    transactionId: charge.id,
    status: charge.status === "succeeded" ? "succeeded" : "failed",
    timestamp: Date.now(),
  };
}

// ... 300 more lines of ledger balancing, export scripts, and schema migrations ...`,
    slicedFile: `// ⚡ DAGR Hoisted Type Contracts (<0.2ms)
export interface PaymentPayload {
  customerId: string;
  amountCents: number;
  currency: string;
  idempotencyKey: string;
}

export interface PaymentReceipt {
  transactionId: string;
  status: "succeeded" | "pending" | "failed";
  timestamp: number;
}

// ⚡ DAGR Minimal Implementation Slice [L872-L891]
export async function processPayment(payload: PaymentPayload, stripe: StripeClient): Promise<PaymentReceipt> {
  validatePaymentPayload(payload);
  const rate = await CurrencyConverter.getExchangeRate(payload.currency, "USD");
  const charge = await stripe.charges.create({
    amount: payload.amountCents,
    currency: payload.currency,
    customer: payload.customerId,
  }, { idempotencyKey: payload.idempotencyKey });

  AuditLogger.log("payment.created", { chargeId: charge.id, amount: payload.amountCents });
  return {
    transactionId: charge.id,
    status: charge.status === "succeeded" ? "succeeded" : "failed",
    timestamp: Date.now(),
  };
}`,
    rawTokens: 11840,
    slicedTokens: 285,
    latency: "0.24ms"
  },

  python: {
    name: "Python // JWT Auth & Token Verification",
    target: "auth/service.py:verify_token",
    rawFile: `# auth/service.py (940 lines monolith)
import jwt
import hashlib
import time
from typing import Optional, Dict, Any
from dataclasses import dataclass
from database.session import get_db
from models.user import UserRecord
from services.email import send_otp_email
from security.rate_limiter import RateLimiter

# ... 700 lines of user signup, password hashing, 2FA, session revocation ...

@dataclass
class TokenPayload:
    user_id: str
    tenant_id: str
    scopes: list[str]
    exp: int

def verify_token(token_str: str, secret_key: str) -> TokenPayload:
    try:
        raw_payload = jwt.decode(token_str, secret_key, algorithms=["HS256"])
        if raw_payload["exp"] < int(time.time()):
            raise PermissionError("Token has expired")
        return TokenPayload(
            user_id=raw_payload["sub"],
            tenant_id=raw_payload.get("tenant", "default"),
            scopes=raw_payload.get("scopes", []),
            exp=raw_payload["exp"]
        )
    except jwt.PyJWTError as e:
        raise ValueError(f"Invalid token signature: {e}")

# ... 220 more lines of OAuth provider callbacks and JWT refresh rotators ...`,
    slicedFile: `# ⚡ DAGR Hoisted Type Contracts (<0.2ms)
@dataclass
class TokenPayload:
    user_id: str
    tenant_id: str
    scopes: list[str]
    exp: int

# ⚡ DAGR Minimal Implementation Slice [L720-L736]
def verify_token(token_str: str, secret_key: str) -> TokenPayload:
    try:
        raw_payload = jwt.decode(token_str, secret_key, algorithms=["HS256"])
        if raw_payload["exp"] < int(time.time()):
            raise PermissionError("Token has expired")
        return TokenPayload(
            user_id=raw_payload["sub"],
            tenant_id=raw_payload.get("tenant", "default"),
            scopes=raw_payload.get("scopes", []),
            exp=raw_payload["exp"]
        )
    except jwt.PyJWTError as e:
        raise ValueError(f"Invalid token signature: {e}")`,
    rawTokens: 8950,
    slicedTokens: 210,
    latency: "0.19ms"
  },

  rust: {
    name: "Rust // SQLite WAL Symbol Search",
    target: "crates/dagr-core/src/storage.rs:search_symbols",
    rawFile: `// crates/dagr-core/src/storage.rs (1,450 lines)
use rusqlite::{params, Connection};
use serde_json::Value;
use std::path::{Path, PathBuf};
use crate::types::{CodeGraphNode, SymbolKind, SymbolSpan};
use crate::fuzzy::compute_symbol_match_score;
use crate::error::{DagrError, Result};

// ... 1,100 lines of migration schema, journal tables, CoW file maps, lock arbitration ...

impl LocalIndexStore {
    pub fn search_symbols(&self, query: &str, limit: usize) -> Result<Vec<CodeGraphNode>> {
        let mut stmt = self.conn.prepare("SELECT symbol_name, file_path, serialized_payload FROM symbol_index")?;
        let rows = stmt.query_map([], |row| {
            let symbol_name: String = row.get(0)?;
            let file_path: String = row.get(1)?;
            let payload: String = row.get(2)?;
            Ok((symbol_name, file_path, payload))
        })?;

        let mut scored_results: Vec<(CodeGraphNode, usize)> = Vec::new();
        for row in rows {
            let (sym_name, path, payload) = row?;
            if let Ok(node) = serde_json::from_str::<CodeGraphNode>(&payload) {
                let score = compute_symbol_match_score(query, &sym_name, &path, node.docstring.as_deref());
                if score > 0 {
                    scored_results.push((node, score));
                }
            }
        }

        scored_results.sort_by_key(|b| std::cmp::Reverse(b.1));
        Ok(scored_results.into_iter().take(limit).map(|(node, _)| node).collect())
    }
}

// ... 320 lines of Blake3 hash cache invalidation and vacuum helpers ...`,
    slicedFile: `// ⚡ DAGR Hoisted Type Contracts (<0.2ms)
pub struct CodeGraphNode {
    pub id: String,
    pub symbol_name: String,
    pub kind: SymbolKind,
    pub span: SymbolSpan,
}

// ⚡ DAGR Minimal Implementation Slice [L1120-L1145]
impl LocalIndexStore {
    pub fn search_symbols(&self, query: &str, limit: usize) -> Result<Vec<CodeGraphNode>> {
        let mut stmt = self.conn.prepare("SELECT symbol_name, file_path, serialized_payload FROM symbol_index")?;
        let rows = stmt.query_map([], |row| {
            let symbol_name: String = row.get(0)?;
            let file_path: String = row.get(1)?;
            let payload: String = row.get(2)?;
            Ok((symbol_name, file_path, payload))
        })?;

        let mut scored_results: Vec<(CodeGraphNode, usize)> = Vec::new();
        for row in rows {
            let (sym_name, path, payload) = row?;
            if let Ok(node) = serde_json::from_str::<CodeGraphNode>(&payload) {
                let score = compute_symbol_match_score(query, &sym_name, &path, node.docstring.as_deref());
                if score > 0 {
                    scored_results.push((node, score));
                }
            }
        }

        scored_results.sort_by_key(|b| std::cmp::Reverse(b.1));
        Ok(scored_results.into_iter().take(limit).map(|(node, _)| node).collect())
    }
}`,
    rawTokens: 14200,
    slicedTokens: 340,
    latency: "0.22ms"
  }
};
