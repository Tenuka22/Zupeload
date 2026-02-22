use anyhow::Result;
use redb::{AccessGuard, Database, ReadableDatabase, ReadableTable, TableDefinition, TableError};

use crate::core::utils::cosine_similarity;
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

    pub fn find_match(
        &self,
        query_embedding: &[f32],
        threshold: f32,
    ) -> Result<Option<Person>> {
        let read_txn = self.db.begin_read()?;
        let table = match read_txn.open_table(USER_TABLE) {
            Ok(t) => t,
            Err(TableError::TableDoesNotExist(_)) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        for v in table.range::<&str>(..)? {
            let (_, value_access): (_, AccessGuard<Vec<u8>>) = v?;
            let value = value_access.value();
            let person: Person = serde_json::from_slice(&value)?;
            if person
                .embeddings
                .iter()
                .any(|emb| cosine_similarity(emb, query_embedding) > threshold)
            {
                return Ok(Some(person));
            }
        }
        Ok(None)
    }

    pub fn add_embedding(&self, person_id: &uuid::Uuid, embedding: Vec<f32>) -> Result<()> {
        let key_string = person_id.to_string();
        let key = key_string.as_str();

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(USER_TABLE)?;
            
            let person_opt = if let Some(access) = table.get(key)? {
                let val = access.value();
                Some(serde_json::from_slice::<Person>(&val)?)
            } else {
                None
            };

            if let Some(mut person) = person_opt {
                 person.embeddings.push(embedding);
                 let new_val = serde_json::to_vec(&person)?;
                 table.insert(key, new_val)?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}

pub fn load_all_people(store: &PersonStore) -> Result<Vec<Person>> {
    let mut people = Vec::new();
    let read_txn = store.db.begin_read()?;
    let table = match read_txn.open_table(USER_TABLE) {
        Ok(t) => t,
        Err(TableError::TableDoesNotExist(_)) => return Ok(people),
        Err(e) => return Err(e.into()),
    };

    for v in table.range::<&str>(..)? {
        let (_, value_access): (_, AccessGuard<Vec<u8>>) = v?;
        let value = value_access.value();
        let person: Person = serde_json::from_slice(&value)?;
        people.push(person);
    }
    Ok(people)
}
