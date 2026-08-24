//! Compliance-grade audit export over the effect journal.
//!
//! Three formats, one source of truth (`EffectJournal`):
//! - `Jsonl`        — one `EffectRecord` per line (machine replay/analysis)
//! - `Otlp`         — OTLP/v1 JSON traces so agent actions land in existing
//!                    observability stacks (answers MCP OTel proposal #269)
//! - `Soc2Evidence` — per-action evidence lines with ISO timestamps and an
//!                    integrity hash chain suitable for auditor review
//!
//! ponytail: hand-emitted OTLP JSON instead of opentelemetry crates; upgrade
//! only if attribute/semantic-convention drift becomes a real problem.

use crate::error::{DagrError, Result};
use crate::journal::EffectJournal;
use serde::Serialize;
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditFormat {
    Jsonl,
    Otlp,
    Soc2Evidence,
}

impl EffectJournal {
    /// Exports every recorded effect to `writer`, returning the row count.
    pub fn export_audit(
        &self,
        fmt: AuditFormat,
        writer: &mut dyn Write,
    ) -> Result<usize> {
        let records = self.fetch_all()?;
        match fmt {
            AuditFormat::Jsonl => export_jsonl(&records, writer)?,
            AuditFormat::Otlp => export_otlp(&records, writer)?,
            AuditFormat::Soc2Evidence => export_soc2(&records, writer)?,
        }
        Ok(records.len())
    }

    pub(crate) fn fetch_all(&self) -> Result<Vec<crate::journal::EffectRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT effect_id, run_id, step_index, effect_type, input_blake3,
                        output_payload, timestamp_utc
                 FROM effect_journal ORDER BY timestamp_utc, step_index",
            )
            .map_err(|e| DagrError::Internal(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Vec<u8>>(4)?,
                    row.get::<_, Vec<u8>>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            })
            .map_err(|e| DagrError::Internal(e.to_string()))?;

        let mut out = Vec::new();
        for r in rows {
            let (eid, rid, step, etype, hash, payload, ts) =
                r.map_err(|e| DagrError::Internal(e.to_string()))?;
            let mut hash_arr = [0u8; 32];
            if hash.len() == 32 {
                hash_arr.copy_from_slice(&hash);
            } else {
                hash_arr.copy_from_slice(&blake3::hash(&hash).as_bytes()[..32]);
            }
            out.push(crate::journal::EffectRecord {
                effect_id: uuid::Uuid::parse_str(&eid).unwrap_or_else(|_| uuid::Uuid::new_v4()),
                run_id: crate::event_store::RunId(
                    uuid::Uuid::parse_str(&rid).unwrap_or_else(|_| uuid::Uuid::new_v4()),
                ),
                step_index: step as u32,
                effect_type: etype,
                input_blake3: hash_arr,
                output_payload: payload,
                timestamp_utc: ts as u64,
            });
        }
        Ok(out)
    }
}

fn export_jsonl(records: &[crate::journal::EffectRecord], w: &mut dyn Write) -> Result<()> {
    for r in records {
        serde_json::to_writer(&mut *w, r)?;
        writeln!(w)?;
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn export_otlp(records: &[crate::journal::EffectRecord], w: &mut dyn Write) -> Result<()> {
    #[derive(Serialize)]
    struct Attr {
        key: &'static str,
        value: Value,
    }
    #[derive(Serialize)]
    struct Value {
        #[serde(rename = "stringValue")]
        string_value: String,
    }

    let spans: Vec<_> = records
        .iter()
        .map(|r| {
            let trace_id = hex(&r.input_blake3[..16]);
            let span_id = hex(&r.effect_id.as_bytes()[..8]);
            let start_ns = r.timestamp_utc.saturating_mul(1_000_000_000);
            json_span(
                &trace_id,
                &span_id,
                &format!("effect.{}", r.effect_type),
                start_ns,
                vec![
                    Attr {
                        key: "dagr.run.id",
                        value: Value {
                            string_value: r.run_id.0.to_string(),
                        },
                    },
                    Attr {
                        key: "dagr.effect.step",
                        value: Value {
                            string_value: r.step_index.to_string(),
                        },
                    },
                    Attr {
                        key: "dagr.effect.input_blake3",
                        value: Value {
                            string_value: hex(&r.input_blake3),
                        },
                    },
                ],
            )
        })
        .collect();

    fn json_span(
        trace_id: &str,
        span_id: &str,
        name: &str,
        start_ns: u64,
        attributes: Vec<Attr>,
    ) -> impl Serialize {
        serde_json::json!({
            "traceId": trace_id,
            "spanId": span_id,
            "name": name,
            "kind": "SPAN_KIND_INTERNAL",
            "startTimeUnixNano": start_ns.to_string(),
            "endTimeUnixNano": start_ns.to_string(),
            "status": { "code": "STATUS_CODE_OK" },
            "attributes": attributes,
        })
    }

    let doc = serde_json::json!({
        "resourceSpans": [{
            "resource": {
                "attributes": [
                    { "key": "service.name",
                      "value": { "stringValue": "dagr" } },
                    { "key": "service.version",
                      "value": { "stringValue": env!("CARGO_PKG_VERSION") } },
                ]
            },
            "scopeSpans": [{
                "scope": { "name": "dagr.audit", "version": env!("CARGO_PKG_VERSION") },
                "spans": spans,
            }],
        }]
    });
    writeln!(
        w,
        "{}",
        serde_json::to_string(&doc)?
    )?;
    Ok(())
}

fn iso_from_unix(secs: u64) -> String {
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    // Howard Hinnant's civil-from-days algorithm
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

fn export_soc2(records: &[crate::journal::EffectRecord], w: &mut dyn Write) -> Result<()> {
    #[derive(Serialize)]
    #[serde(rename_all = "snake_case")]
    struct Evidence<'a> {
        evidence_type: &'static str,
        actor: String,
        action: &'a str,
        object_integrity: String,
        result: &'static str,
        timestamp_utc_iso: String,
        sequence: usize,
        prior_entry_hash: String,
        entry_hash: String,
    }

    let mut prior = String::from("GENESIS");
    for (i, r) in records.iter().enumerate() {
        let entry_hash = blake3::keyed_hash(
            &prior.as_bytes().try_into().unwrap_or([0u8; 32]),
            &[
                &r.input_blake3[..],
                &r.output_payload,
                &r.timestamp_utc.to_le_bytes(),
            ]
            .concat(),
        )
        .to_hex()[..32]
        .to_string();

        let ev = Evidence {
            evidence_type: "agent_action",
            actor: format!("dagr-agent:{}", r.run_id.0),
            action: &r.effect_type,
            object_integrity: hex(&r.input_blake3),
            result: "completed",
            timestamp_utc_iso: iso_from_unix(r.timestamp_utc),
            sequence: i,
            prior_entry_hash: prior.clone(),
            entry_hash: entry_hash.clone(),
        };
        serde_json::to_writer(&mut *w, &ev)?;
        writeln!(w)?;
        prior = entry_hash;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::RunId;
    use crate::journal::{EffectJournal, ExecutionMode};

    fn seeded_journal() -> EffectJournal {
        let journal = EffectJournal::in_memory().unwrap();
        let run = RunId(uuid::Uuid::new_v4());
        for step in 0..2u32 {
            journal
                .record_effect(&crate::journal::EffectRecord {
                    effect_id: uuid::Uuid::new_v4(),
                    run_id: run,
                    step_index: step,
                    effect_type: format!("tool.call.{step}"),
                    input_blake3: blake3::hash(format!("input-{step}").as_bytes()).into(),
                    output_payload: b"ok".to_vec(),
                    timestamp_utc: 1_750_000_000 + step as u64,
                })
                .unwrap();
        }
        let _ = ExecutionMode::Live;
        journal
    }

    #[test]
    fn jsonl_export_emits_one_parseable_record_per_effect() {
        let j = seeded_journal();
        let mut buf = Vec::new();
        let n = j.export_audit(AuditFormat::Jsonl, &mut buf).unwrap();
        assert_eq!(n, 2);
        let text = String::from_utf8(buf).unwrap();
        let lines: Vec<_> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        for l in lines {
            let v: serde_json::Value = serde_json::from_str(l).unwrap();
            assert!(v["effect_id"].is_string());
            assert!(v["timestamp_utc"].is_u64());
        }
    }

    #[test]
    fn otlp_export_is_structurally_conformant() {
        let j = seeded_journal();
        let mut buf = Vec::new();
        j.export_audit(AuditFormat::Otlp, &mut buf).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&buf).unwrap();

        let rs = &v["resourceSpans"][0];
        assert_eq!(rs["resource"]["attributes"][0]["key"], "service.name");
        let spans = &rs["scopeSpans"][0]["spans"];
        assert_eq!(spans.as_array().unwrap().len(), 2);
        for s in spans.as_array().unwrap() {
            assert_eq!(s["traceId"].as_str().unwrap().len(), 32);
            assert_eq!(s["spanId"].as_str().unwrap().len(), 16);
            assert_eq!(s["kind"], "SPAN_KIND_INTERNAL");
            assert!(s["name"].as_str().unwrap().starts_with("effect."));
        }
    }

    #[test]
    fn soc2_export_chains_hashes_and_carries_required_fields() {
        let j = seeded_journal();
        let mut buf = Vec::new();
        j.export_audit(AuditFormat::Soc2Evidence, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        let mut prior = "GENESIS".to_string();
        for (i, l) in text.lines().enumerate() {
            let v: serde_json::Value = serde_json::from_str(l).unwrap();
            assert_eq!(v["evidence_type"], "agent_action");
            assert_eq!(v["result"], "completed");
            assert!(v["timestamp_utc_iso"]
                .as_str()
                .unwrap()
                .ends_with('Z'));
            assert_eq!(v["sequence"], serde_json::json!(i));
            assert_eq!(v["prior_entry_hash"], serde_json::json!(prior));
            prior = v["entry_hash"].as_str().unwrap().to_string();
        }
    }

    #[test]
    fn iso_conversion_handles_epoch_and_recent() {
        assert_eq!(iso_from_unix(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_from_unix(1_750_000_000), "2025-06-15T15:06:40Z");
    }
}

#[cfg(test)]
mod otlp_snapshot_tests {
    use super::*;
    use crate::event_store::RunId;
    use crate::journal::{EffectJournal, EffectRecord};

    #[test]
    fn otlp_span_shape_is_pinned_against_drift() {
        let j = EffectJournal::in_memory().unwrap();
        let run = RunId(uuid::Uuid::new_v4());
        j.record_effect(&EffectRecord {
            effect_id: uuid::Uuid::new_v4(),
            run_id: run,
            step_index: 0,
            effect_type: "tool.call".into(),
            input_blake3: [7u8; 32],
            output_payload: b"ok".to_vec(),
            timestamp_utc: 1_750_000_000,
        })
        .unwrap();

        let mut buf = Vec::new();
        j.export_audit(AuditFormat::Otlp, &mut buf).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&buf).unwrap();

        // Top-level shape
        let top_keys: Vec<&str> = v.as_object().unwrap().keys().map(|s| s.as_str()).collect();
        assert_eq!(top_keys, vec!["resourceSpans"]);

        let span = &v["resourceSpans"][0]["scopeSpans"][0]["spans"][0];
        let mut span_keys: Vec<&str> =
            span.as_object().unwrap().keys().map(|s| s.as_str()).collect();
        span_keys.sort_unstable();
        assert_eq!(
            span_keys,
            vec![
                "attributes",
                "endTimeUnixNano",
                "kind",
                "name",
                "spanId",
                "startTimeUnixNano",
                "status",
                "traceId",
            ]
        );

        let attr_keys: Vec<&str> = span["attributes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["key"].as_str().unwrap())
            .collect();
        assert_eq!(
            attr_keys,
            vec!["dagr.run.id", "dagr.effect.step", "dagr.effect.input_blake3"]
        );
        for a in span["attributes"].as_array().unwrap() {
            assert!(a["value"]["stringValue"].is_string(), "OTLP values must be wrapped");
        }
        assert_eq!(span["status"]["code"], "STATUS_CODE_OK");
    }
}
