use std::collections::HashMap;
use std::fs;
use std::path::Path;

use rusqlite::{params, Connection};
use serde_json::Value;

use crate::sources::SourceResult;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS sweeps (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp     TEXT NOT NULL,
    duration_ms   INTEGER NOT NULL,
    sources_ok    INTEGER NOT NULL,
    sources_err   INTEGER NOT NULL,
    total_sources INTEGER NOT NULL,
    data_json     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS analyses (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    sweep_id      INTEGER NOT NULL REFERENCES sweeps(id),
    model         TEXT NOT NULL,
    language      TEXT NOT NULL DEFAULT 'en',
    content       TEXT NOT NULL,
    input_tokens  INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS source_health (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    sweep_id      INTEGER NOT NULL REFERENCES sweeps(id),
    source_name   TEXT NOT NULL,
    status        TEXT NOT NULL,
    duration_ms   INTEGER NOT NULL,
    error         TEXT
);
";

/// A sweep record returned by history queries.
pub struct SweepRecord {
    pub id: i64,
    pub timestamp: String,
    pub duration_ms: u64,
    pub sources_ok: usize,
    pub sources_err: usize,
    pub total_sources: usize,
}

/// Source reliability stats computed from source_health history.
pub struct SourceReliability {
    pub name: String,
    #[allow(dead_code)]
    pub success_count: usize,
    #[allow(dead_code)]
    pub total_count: usize,
    pub success_rate: f64,
}

/// SQLite-backed persistence for sweep data, analyses, and source health.
pub struct Store {
    conn: Connection,
}

impl Store {
    /// Open (or create) the database at `path`, running schema migrations.
    pub fn open(path: &str) -> anyhow::Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = Path::new(path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA busy_timeout=5000;
             PRAGMA synchronous=NORMAL;
             PRAGMA cache_size=-8000;"
        )?;
        conn.execute_batch(SCHEMA)?;

        Ok(Store { conn })
    }

    /// Remove old sweeps keeping only the most recent `keep` entries.
    pub fn prune_old_sweeps(&self, keep: usize) -> anyhow::Result<usize> {
        let deleted: usize = self.conn.execute(
            "DELETE FROM sweeps WHERE id NOT IN (SELECT id FROM sweeps ORDER BY id DESC LIMIT ?1)",
            rusqlite::params![keep],
        )?;
        let _ = self.conn.execute(
            "DELETE FROM source_health WHERE sweep_id NOT IN (SELECT id FROM sweeps)",
            [],
        );
        let _ = self.conn.execute(
            "DELETE FROM analyses WHERE sweep_id NOT IN (SELECT id FROM sweeps)",
            [],
        );
        Ok(deleted)
    }

    /// Save a completed sweep and return its row id.
    pub fn save_sweep(
        &self,
        data: &Value,
        duration_ms: u64,
        sources_ok: usize,
        sources_err: usize,
        total: usize,
    ) -> anyhow::Result<i64> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let json_str = serde_json::to_string(data)?;

        self.conn.execute(
            "INSERT INTO sweeps (timestamp, duration_ms, sources_ok, sources_err, total_sources, data_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                timestamp,
                duration_ms as i64,
                sources_ok as i64,
                sources_err as i64,
                total as i64,
                json_str
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Save per-source health records for a sweep.
    pub fn save_source_health(
        &self,
        sweep_id: i64,
        results: &[SourceResult],
    ) -> anyhow::Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        for r in results {
            tx.execute(
                "INSERT INTO source_health (sweep_id, source_name, status, duration_ms, error)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    sweep_id,
                    r.name,
                    r.status.to_string(),
                    r.duration_ms as i64,
                    r.error,
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Save an LLM analysis linked to a sweep.
    pub fn save_analysis(
        &self,
        sweep_id: i64,
        model: &str,
        lang: &str,
        content: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> anyhow::Result<()> {
        self.conn.execute(
            "INSERT INTO analyses (sweep_id, model, language, content, input_tokens, output_tokens)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                sweep_id,
                model,
                lang,
                content,
                input_tokens as i64,
                output_tokens as i64,
            ],
        )?;
        Ok(())
    }

    /// Get the most recent sweep (id, data_json).
    pub fn get_latest_sweep(&self) -> anyhow::Result<Option<(i64, Value)>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, data_json FROM sweeps ORDER BY id DESC LIMIT 1",
        )?;

        let mut rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let json_str: String = row.get(1)?;
            Ok((id, json_str))
        })?;

        match rows.next() {
            Some(Ok((id, json_str))) => {
                let val: Value = serde_json::from_str(&json_str)?;
                Ok(Some((id, val)))
            }
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// Get the last `limit` sweep records (metadata only, no data_json).
    pub fn get_sweep_history(&self, limit: usize) -> anyhow::Result<Vec<SweepRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, duration_ms, sources_ok, sources_err, total_sources
             FROM sweeps ORDER BY id DESC LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(SweepRecord {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                duration_ms: row.get::<_, i64>(2)? as u64,
                sources_ok: row.get::<_, i64>(3)? as usize,
                sources_err: row.get::<_, i64>(4)? as usize,
                total_sources: row.get::<_, i64>(5)? as usize,
            })
        })?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    /// Get the data_json for a specific sweep by id.
    pub fn get_sweep_data(&self, sweep_id: i64) -> anyhow::Result<Option<Value>> {
        let mut stmt = self
            .conn
            .prepare("SELECT data_json FROM sweeps WHERE id = ?1")?;

        let mut rows = stmt.query_map(params![sweep_id], |row| {
            let json_str: String = row.get(0)?;
            Ok(json_str)
        })?;

        match rows.next() {
            Some(Ok(json_str)) => {
                let val: Value = serde_json::from_str(&json_str)?;
                Ok(Some(val))
            }
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// Get the latest analysis content (most recent by id).
    pub fn get_latest_analysis(&self) -> anyhow::Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT content FROM analyses ORDER BY id DESC LIMIT 1",
        )?;

        let mut rows = stmt.query_map([], |row| {
            let content: String = row.get(0)?;
            Ok(content)
        })?;

        match rows.next() {
            Some(Ok(content)) => Ok(Some(content)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// Compute per-source reliability over the last `lookback` sweeps.
    pub fn source_reliability(&self, lookback: usize) -> anyhow::Result<Vec<SourceReliability>> {
        let mut stmt = self.conn.prepare(
            "SELECT source_name,
                    SUM(CASE WHEN status = 'ok' THEN 1 ELSE 0 END) as success_count,
                    COUNT(*) as total_count
             FROM source_health
             WHERE sweep_id IN (SELECT id FROM sweeps ORDER BY id DESC LIMIT ?1)
             GROUP BY source_name
             ORDER BY source_name",
        )?;

        let rows = stmt.query_map(params![lookback as i64], |row| {
            let name: String = row.get(0)?;
            let success_count: i64 = row.get(1)?;
            let total_count: i64 = row.get(2)?;
            Ok((name, success_count as usize, total_count as usize))
        })?;

        let mut result = Vec::new();
        for row in rows {
            let (name, success_count, total_count) = row?;
            let success_rate = if total_count > 0 {
                success_count as f64 / total_count as f64
            } else {
                0.0
            };
            result.push(SourceReliability {
                name,
                success_count,
                total_count,
                success_rate,
            });
        }
        Ok(result)
    }

    /// Save a migrated sweep from old JSON data.
    pub fn save_migrated_sweep(&self, timestamp: &str, data: &Value) -> anyhow::Result<i64> {
        let ts = if timestamp.is_empty() {
            chrono::Utc::now().to_rfc3339()
        } else {
            timestamp.to_string()
        };
        let json_str = serde_json::to_string(data)?;
        let sources_ok = data
            .get("chaos")
            .and_then(|c| c.get("sourcesOk"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let sources_err = data
            .get("chaos")
            .and_then(|c| c.get("sourcesFailed"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let total = sources_ok + sources_err;
        let duration_ms = data
            .get("chaos")
            .and_then(|c| c.get("totalDurationMs"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        self.conn.execute(
            "INSERT INTO sweeps (timestamp, duration_ms, sources_ok, sources_err, total_sources, data_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![ts, duration_ms, sources_ok, sources_err, total, json_str],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get trend data for specific metric keys over the last N sweeps.
    ///
    /// The `metric_keys` are dot-separated JSON paths relative to the sweep data,
    /// e.g. `["sources.FRED.indicators.VIXCLS", "sources.EIA.data.wti.value"]`.
    /// For FRED indicators the special form `fred:SERIES_ID` is also supported.
    #[allow(dead_code)]
    pub fn get_metric_trends(
        &self,
        metric_keys: &[&str],
        limit: usize,
    ) -> anyhow::Result<Vec<MetricTrend>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, data_json FROM sweeps ORDER BY id DESC LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            let id: i64 = row.get(0)?;
            let timestamp: String = row.get(1)?;
            let json_str: String = row.get(2)?;
            Ok((id, timestamp, json_str))
        })?;

        let mut result = Vec::new();
        for row in rows {
            let (id, timestamp, json_str) = row?;
            let data: Value = serde_json::from_str(&json_str)?;

            let mut metrics = HashMap::new();
            for key in metric_keys {
                let value = extract_metric_value(&data, key);
                if let Some(v) = value {
                    metrics.insert(key.to_string(), v);
                }
            }

            result.push(MetricTrend {
                timestamp,
                sweep_id: id,
                metrics,
            });
        }

        // Return in chronological order (oldest first)
        result.reverse();
        Ok(result)
    }
}

/// A single point in a metric trend time series.
#[allow(dead_code)]
pub struct MetricTrend {
    pub timestamp: String,
    pub sweep_id: i64,
    pub metrics: HashMap<String, f64>,
}

/// Extract a numeric metric value from sweep data given a key.
///
/// Supported key formats:
/// - `fred:SERIES_ID` — looks up a FRED indicator by series ID
/// - `dot.separated.path` — traverses the JSON tree
#[allow(dead_code)]
fn extract_metric_value(data: &Value, key: &str) -> Option<f64> {
    // Special FRED shorthand: "fred:VIXCLS"
    if let Some(series_id) = key.strip_prefix("fred:") {
        let indicators = data
            .get("sources")?
            .get("FRED")?
            .get("indicators")?
            .as_array()?;
        for item in indicators {
            if item.get("id").and_then(|v| v.as_str()) == Some(series_id) {
                return item.get("value").and_then(|v| v.as_f64());
            }
        }
        return None;
    }

    // Dot-separated path traversal
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = data;
    for part in &parts {
        current = current.get(part)?;
    }
    current.as_f64()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let store = Store::open(path.to_str().unwrap()).unwrap();
        (dir, store)
    }

    #[test]
    fn test_open_creates_schema() {
        let (_dir, store) = temp_db();
        // Verify tables exist by querying them
        let count: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM sweeps", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);

        let count: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM analyses", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);

        let count: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM source_health", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_save_and_retrieve_sweep() {
        let (_dir, store) = temp_db();
        let data = serde_json::json!({
            "sources": { "FRED": { "indicators": [{ "id": "VIXCLS", "value": 18.5 }] } }
        });

        let id = store.save_sweep(&data, 1234, 5, 1, 6).unwrap();
        assert_eq!(id, 1);

        let latest = store.get_latest_sweep().unwrap();
        assert!(latest.is_some());
        let (latest_id, latest_data) = latest.unwrap();
        assert_eq!(latest_id, 1);
        assert_eq!(latest_data["sources"]["FRED"]["indicators"][0]["id"], "VIXCLS");

        // Also test get_sweep_data
        let fetched = store.get_sweep_data(id).unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap(), data);
    }

    #[test]
    fn test_sweep_history() {
        let (_dir, store) = temp_db();
        let data = serde_json::json!({"test": true});

        store.save_sweep(&data, 100, 3, 0, 3).unwrap();
        store.save_sweep(&data, 200, 4, 1, 5).unwrap();
        store.save_sweep(&data, 300, 2, 2, 4).unwrap();

        let history = store.get_sweep_history(2).unwrap();
        assert_eq!(history.len(), 2);
        // Most recent first
        assert_eq!(history[0].id, 3);
        assert_eq!(history[0].duration_ms, 300);
        assert_eq!(history[0].sources_ok, 2);
        assert_eq!(history[1].id, 2);
        assert_eq!(history[1].duration_ms, 200);
    }

    #[test]
    fn test_latest_sweep_empty() {
        let (_dir, store) = temp_db();
        let latest = store.get_latest_sweep().unwrap();
        assert!(latest.is_none());
    }

    #[test]
    fn test_save_analysis() {
        let (_dir, store) = temp_db();
        let data = serde_json::json!({"test": true});
        let sweep_id = store.save_sweep(&data, 100, 1, 0, 1).unwrap();

        store
            .save_analysis(sweep_id, "gpt-4o", "en", "Analysis content here", 500, 200)
            .unwrap();

        let count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM analyses WHERE sweep_id = ?1",
                params![sweep_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_source_reliability() {
        let (_dir, store) = temp_db();
        let data = serde_json::json!({"test": true});

        let id1 = store.save_sweep(&data, 100, 2, 0, 2).unwrap();
        let id2 = store.save_sweep(&data, 100, 1, 1, 2).unwrap();

        // Source A: ok in both sweeps
        // Source B: ok in first, error in second
        let results1 = vec![
            SourceResult {
                name: "SourceA".to_string(),
                status: crate::sources::SourceStatus::Ok,
                data: None,
                error: None,
                duration_ms: 50,
                tier: 1,
            },
            SourceResult {
                name: "SourceB".to_string(),
                status: crate::sources::SourceStatus::Ok,
                data: None,
                error: None,
                duration_ms: 60,
                tier: 1,
            },
        ];
        store.save_source_health(id1, &results1).unwrap();

        let results2 = vec![
            SourceResult {
                name: "SourceA".to_string(),
                status: crate::sources::SourceStatus::Ok,
                data: None,
                error: None,
                duration_ms: 55,
                tier: 1,
            },
            SourceResult {
                name: "SourceB".to_string(),
                status: crate::sources::SourceStatus::Error,
                data: None,
                error: Some("connection refused".to_string()),
                duration_ms: 70,
                tier: 2,
            },
        ];
        store.save_source_health(id2, &results2).unwrap();

        let reliability = store.source_reliability(10).unwrap();
        assert_eq!(reliability.len(), 2);

        let a = reliability.iter().find(|r| r.name == "SourceA").unwrap();
        assert_eq!(a.success_count, 2);
        assert_eq!(a.total_count, 2);
        assert!((a.success_rate - 1.0).abs() < f64::EPSILON);

        let b = reliability.iter().find(|r| r.name == "SourceB").unwrap();
        assert_eq!(b.success_count, 1);
        assert_eq!(b.total_count, 2);
        assert!((b.success_rate - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_metric_trends() {
        let (_dir, store) = temp_db();

        // Insert sweeps with FRED VIX data
        let data1 = serde_json::json!({
            "sources": {
                "FRED": {
                    "indicators": [{ "id": "VIXCLS", "value": 18.5 }]
                }
            }
        });
        let data2 = serde_json::json!({
            "sources": {
                "FRED": {
                    "indicators": [{ "id": "VIXCLS", "value": 22.0 }]
                }
            }
        });

        store.save_sweep(&data1, 100, 1, 0, 1).unwrap();
        store.save_sweep(&data2, 200, 1, 0, 1).unwrap();

        let trends = store.get_metric_trends(&["fred:VIXCLS"], 10).unwrap();
        assert_eq!(trends.len(), 2);

        // Chronological order: first sweep first
        let vix0 = trends[0].metrics.get("fred:VIXCLS").unwrap();
        assert!((vix0 - 18.5).abs() < f64::EPSILON);
        let vix1 = trends[1].metrics.get("fred:VIXCLS").unwrap();
        assert!((vix1 - 22.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_extract_metric_value_dot_path() {
        let data = serde_json::json!({
            "sources": {
                "EIA": {
                    "data": {
                        "wti": { "value": 89.5 }
                    }
                }
            }
        });

        let result = super::extract_metric_value(&data, "sources.EIA.data.wti.value");
        assert!(result.is_some());
        assert!((result.unwrap() - 89.5).abs() < f64::EPSILON);
    }
}
