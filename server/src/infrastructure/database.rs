use anyhow::Result;
use redb::{Database, TableDefinition};

use crate::domain::models::Person;

pub const USER_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("user_table");

pub struct PersonStore {
    pub db: Database,
}

impl PersonStore {
    pub fn new(path: &str) -> Result<Self> {
        let db = Database::create(path)?;
        Ok(PersonStore { db })
    }

    pub fn save(&self, person: &Person) -> Result<()> {
        let key_string = person.id.to_string();
        let key = key_string.as_str();
        let val = serde_json::to_vec(person)?;

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(USER_TABLE)?;
            table.insert(key, val)?;
        }
        write_txn.commit()?;
        Ok(())
    }
}
