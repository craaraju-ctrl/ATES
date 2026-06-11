use redb::{Database, ReadableTable, TableDefinition};
use std::path::Path;

const DECISIONS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("decisions");

pub struct MemoryStore {
    db: Database,
}

impl MemoryStore {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, redb::Error> {
        let db = Database::create(path)?;
        // Create table if it doesn't exist
        let write_txn = db.begin_write()?;
        {
            write_txn.open_table(DECISIONS_TABLE)?;
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
        let result = match table.get(key)? {
            Some(value) => Some(value.value().to_string()),
            None => None,
        };
        Ok(result)
    }
}