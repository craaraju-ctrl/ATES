use redb::{Database, ReadableTable, TableDefinition};
use std::path::Path;

const DECISIONS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("decisions");
const STATE_TABLE: TableDefinition<&str, &str> = TableDefinition::new("state");
const EPISODES_TABLE: TableDefinition<&str, &str> = TableDefinition::new("episodes");

pub struct MemoryStore {
    db: Database,
}

impl MemoryStore {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, redb::Error> {
        let db = Database::create(path)?;
        // Create tables if they don't exist
        let write_txn = db.begin_write()?;
        {
            write_txn.open_table(DECISIONS_TABLE)?;
            write_txn.open_table(STATE_TABLE)?;
            write_txn.open_table(EPISODES_TABLE)?;
        }
        write_txn.commit()?;
        Ok(Self { db })
    }

    pub fn store_decision(&self, key: &str, value: &str) -> Result<(), redb::Error> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(DECISIONS_TABLE)?;
            table.insert(key, value)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_decision(&self, key: &str) -> Result<Option<String>, redb::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(DECISIONS_TABLE)?;
        let result = table.get(key)?.map(|value| value.value().to_string());
        Ok(result)
    }

    /// Persist structured state (portfolio, goals, tasks) as JSON by key.
    pub fn store_state(&self, key: &str, value: &str) -> Result<(), redb::Error> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(STATE_TABLE)?;
            table.insert(key, value)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Load structured state by key.
    pub fn load_state(&self, key: &str) -> Result<Option<String>, redb::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(STATE_TABLE)?;
        let result = table.get(key)?.map(|value| value.value().to_string());
        Ok(result)
    }

    // ── Episode Storage ────────────────────────────────────────────────────

    /// Store a full trading episode as JSON, keyed by episode_id.
    pub fn store_episode(&self, episode_id: &str, json: &str) -> Result<(), redb::Error> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(EPISODES_TABLE)?;
            table.insert(episode_id, json)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Load a single episode by ID.
    pub fn load_episode(&self, episode_id: &str) -> Result<Option<String>, redb::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(EPISODES_TABLE)?;
        let result = table.get(episode_id)?.map(|value| value.value().to_string());
        Ok(result)
    }

    /// Load all episode IDs stored since a given Unix timestamp.
    pub fn list_episode_ids_since(&self, since_ts: i64) -> Result<Vec<String>, redb::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(EPISODES_TABLE)?;
        let prefix = "ep/";
        let mut ids = Vec::new();
        for result in table.iter()? {
            let (key, _) = result?;
            let key_str = key.value().to_string();
            // Extract timestamp from key: "ep/{symbol}/{unix_ts}"
            if let Some(ts_str) = key_str.rsplit('/').next() {
                if let Ok(ts) = ts_str.parse::<i64>() {
                    if ts >= since_ts && key_str.starts_with(prefix) {
                        ids.push(key_str);
                    }
                }
            }
        }
        // Sort by time ascending
        ids.sort();
        Ok(ids)
    }

    /// Load all episodes since a given timestamp, returning parsed JSON strings.
    pub fn load_episodes_since(&self, since_ts: i64) -> Result<Vec<(String, String)>, redb::Error> {
        let ids = self.list_episode_ids_since(since_ts)?;
        let mut episodes = Vec::with_capacity(ids.len());
        for id in &ids {
            if let Some(json) = self.load_episode(id)? {
                episodes.push((id.clone(), json));
            }
        }
        Ok(episodes)
    }
}