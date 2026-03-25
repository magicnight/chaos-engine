use serde_json::Value;

use crate::store::Store;

use super::{compute_delta, DeltaResult};

/// Manages sweep memory via SQLite, providing previous-sweep lookups
/// and delta computation. Replaces the Node.js hot/cold JSON file storage.
#[allow(dead_code)]
pub struct MemoryManager<'a> {
    store: &'a Store,
}

#[allow(dead_code)]
impl<'a> MemoryManager<'a> {
    pub fn new(store: &'a Store) -> Self {
        MemoryManager { store }
    }

    /// Get the most recent sweep data from the database.
    pub fn get_previous_sweep(&self) -> Option<Value> {
        self.store
            .get_latest_sweep()
            .ok()
            .flatten()
            .map(|(_, data)| data)
    }

    /// Compute a delta between the current sweep data and the latest stored sweep.
    /// Returns `None` if there is no previous sweep or if the data is null.
    pub fn compute_and_store_delta(&self, current: &Value) -> Option<DeltaResult> {
        let previous = self.get_previous_sweep()?;
        compute_delta(current, &previous)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn temp_store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mem_test.db");
        let store = Store::open(path.to_str().unwrap()).unwrap();
        (dir, store)
    }

    #[test]
    fn test_no_previous_returns_none() {
        let (_dir, store) = temp_store();
        let mm = MemoryManager::new(&store);
        let current = json!({"chaos": {"timestamp": "t1"}, "sources": {}});
        assert!(mm.compute_and_store_delta(&current).is_none());
    }

    #[test]
    fn test_delta_with_previous() {
        let (_dir, store) = temp_store();

        // Save a previous sweep
        let prev = json!({
            "chaos": { "timestamp": "2025-01-01T00:00:00Z", "sourcesOk": 10 },
            "sources": {
                "FRED": { "indicators": [{ "id": "VIXCLS", "value": 20.0 }] }
            }
        });
        store.save_sweep(&prev, 100, 10, 0, 10).unwrap();

        let mm = MemoryManager::new(&store);

        // Current sweep with VIX change above threshold
        let current = json!({
            "chaos": { "timestamp": "2025-01-01T01:00:00Z", "sourcesOk": 10 },
            "sources": {
                "FRED": { "indicators": [{ "id": "VIXCLS", "value": 23.0 }] }
            }
        });

        let delta = mm.compute_and_store_delta(&current);
        assert!(delta.is_some());
        let delta = delta.unwrap();
        assert!(delta.summary.total_changes > 0);
    }

    #[test]
    fn test_get_previous_sweep() {
        let (_dir, store) = temp_store();
        let mm = MemoryManager::new(&store);

        assert!(mm.get_previous_sweep().is_none());

        let data = json!({"test": "value"});
        store.save_sweep(&data, 50, 1, 0, 1).unwrap();

        let prev = mm.get_previous_sweep();
        assert!(prev.is_some());
        assert_eq!(prev.unwrap()["test"], "value");
    }
}
