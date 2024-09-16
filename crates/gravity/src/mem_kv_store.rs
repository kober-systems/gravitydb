use crate::KVStore;
use crate::schema::Property;
use std::collections::BTreeMap;
use thiserror::Error;

type HashId = String;

#[derive(Debug, Default)]
pub struct MemoryKvStore {
  data: BTreeMap<HashId, Vec<u8>>,
}

impl MemoryKvStore {
  pub fn get_inner(self) -> BTreeMap<HashId, Vec<u8>> {
    self.data
  }
}

impl KVStore<Error> for MemoryKvStore
{
  fn create_bucket(&mut self, _key: &[u8]) -> Result<(), Error> {
    Ok(())
  }

  fn delete_record(&mut self, key: &[u8]) -> Result<(), Error> {
    self.data.remove(std::str::from_utf8(key).unwrap());
    Ok(())
  }

  fn store_record(&mut self, key: &[u8], value: &[u8]) -> Result<(), Error> {
    self.data.insert(key_to_string(key), value.to_vec());
    Ok(())
  }

  fn fetch_record(&self, key: &[u8]) -> Result<Vec<u8>, Error> {
    let key = key_to_string(key);
    let out = self.data.get(&key).ok_or(Error::Missing(key))?;
    Ok(out.to_vec())
  }

  fn list_records(&self, key: &[u8]) -> Result<Vec<Vec<u8>>, Error> {
    let iter: Vec<Vec<u8>> = self.data.iter()
      .into_iter()
      .filter_map(|(k, v)| {
        if k.starts_with(&key_to_string(key)) {
          Some(v.to_vec())
        } else {
          None
        }
      }).collect();
    Ok(iter)
  }

  fn exists(&self, key: &[u8]) -> Result<bool, Error> {
    Ok(self.data.contains_key(&key_to_string(key)))
  }
}

#[derive(Error, Debug)]
pub enum Error {
  #[error("the record {0} could not be found")]
  Missing(String),
}

fn key_to_string(key: &[u8]) -> String {
  std::str::from_utf8(key).unwrap().to_string()
}

