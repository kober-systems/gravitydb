use crate::{BacklinkType, KVStore};
use crate::schema::Property;
use std::collections::BTreeMap;
use thiserror::Error;

type HashId = String;

pub struct MemoryKvStore {
  data: BTreeMap<HashId, Vec<u8>>,
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

  /// props_hash: the hash_id of the property that holds the index
  /// id:         the id of the node, edge or property that references
  ///             the property and needs a backling
  /// ty:         the type of the element that needs a backlink
  fn create_idx_backlink(&mut self, props_hash: &str, id: &str, ty: BacklinkType) -> Result<(), Error> {
    let index_path = "indexes/".to_string() + &props_hash.to_string() + "/";
    self.create_bucket(index_path.as_bytes())?;

    let prefix = match ty {
      BacklinkType::Node => "nodes",
      BacklinkType::Edge => "edges",
      BacklinkType::Property => "props",
    };
    let backlink_path = index_path + "/" + prefix + "_" + id;
    let path = prefix.to_string() + "/" + id;
    //fs::hard_link(path, backlink_path)?;

    Ok(())
  }

  fn delete_property_backlink(&mut self, props_hash: &str, id: &str, ty: BacklinkType) -> Result<bool, Error> {
    let index_path = "indexes/".to_string() + &props_hash.to_string() + "/";

    let prefix = match ty {
      BacklinkType::Node => "nodes",
      BacklinkType::Edge => "edges",
      BacklinkType::Property => "props",
    };
    let backlink_path = index_path.clone() + prefix + "_" + id;
    self.delete_record(backlink_path.as_bytes())?;

    if self.list_records(index_path.as_bytes())?.is_empty() {
      Ok(true)
    } else {
      Ok(false)
    }
  }
}

#[derive(Error, Debug)]
pub enum Error {
  #[error("wrongly formatted database at path TODO")]
  MalformedDB,
  #[error("the recored {0} could not be found")]
  Missing(String),
}

fn key_to_string(key: &[u8]) -> String {
  std::str::from_utf8(key).unwrap().to_string()
}

