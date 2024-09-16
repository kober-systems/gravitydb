use sha2::Digest;
use gravity::schema::SchemaElement;
use std::collections::BTreeSet;
use std::fs;
use gravity::schema::Property;
use gravity::ql;
pub use gravity::kv_graph_store::SerialisationError;
use gravity::kv_graph_store::EdgeData;
use std::path::{Path, PathBuf};
use std::io::Error;
use thiserror::Error;
pub mod cli_helpers;

use gravity::{KVStore, BacklinkType};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

type HashId = String;

#[derive(Debug, Clone)]
pub struct GenericProperty(Vec<u8>);

impl SchemaElement<HashId, SerialisationError> for GenericProperty
{
  fn get_key(&self) -> HashId {
    format!("{:X}", sha2::Sha256::digest(&self.0))
  }

  fn serialize(&self) -> Result<Vec<u8>, SerialisationError> {
    Ok(self.0.clone())
  }

  fn deserialize(data: &[u8]) -> Result<Self, SerialisationError>
  where
    Self: Sized,
  {
    Ok(GenericProperty(data.to_vec()))
  }
}

impl Property<String, SerialisationError> for GenericProperty {
  fn nested(&self) -> Vec<Self> { Vec::new() }
}

pub struct Change {
  pub created: ChangeSet,
  pub modified: BTreeSet<NodeChange>,
  pub deleted: ChangeSet,
  pub depends_on: BTreeSet<HashId>,
}

pub struct NodeChange {
  pub id: uuid::Uuid,
  pub properties: HashId,
}

pub struct ChangeSet {
  pub nodes: BTreeSet<NodeChange>,
  pub edges: BTreeSet<EdgeData>,
  //pub properties: BTreeSet<Property>,
}

type BasicQuery = ql::BasicQuery<uuid::Uuid, HashId, HashId, ql::ShellFilter, ql::ShellFilter>;

pub struct FsKvStore<T: Property<HashId, SerialisationError>> {
  p_marker: std::marker::PhantomData<T>,
  base_path: PathBuf,
}

impl<'a, T: Property<HashId, SerialisationError>> KVStore<Error> for FsKvStore<T>
{
  fn create_bucket(&mut self, key: &[u8]) -> Result<(), Error> {
    Ok(std::fs::create_dir_all(self.key_to_path(key))?)
  }

  fn delete_record(&mut self, key: &[u8]) -> Result<(), Error> {
    Ok(std::fs::remove_file(self.key_to_path(key))?)
  }

  fn store_record(&mut self, key: &[u8], value: &[u8]) -> Result<(), Error> {
    Ok(std::fs::write(self.key_to_path(key), value)?)
  }

  fn fetch_record(&self, key: &[u8]) -> Result<Vec<u8>, Error> {
    Ok(std::fs::read(self.key_to_path(key))?)
  }

  fn list_records(&self, key: &[u8]) -> Result<Vec<Vec<u8>>, Error> {
    let iter: Vec<Vec<u8>> = fs::read_dir(self.key_to_path(key))?.into_iter().filter_map(|entry| {
      match entry {
        Ok(entry) => Some(entry.file_name().into_encoded_bytes()),
        Err(_) => None,
      }
    }).collect();
    Ok(iter)
  }

  fn exists(&self, key: &[u8]) -> Result<bool, Error> {
    Ok(self.key_to_path(key).exists())
  }
}

impl<T: Property<HashId, SerialisationError>> FsKvStore<T> {
  fn key_to_path(&self, key: &[u8]) -> PathBuf {
    let path = Path::new(OsStr::from_bytes(key));
    PathBuf::from(self.base_path.join(path))
  }

  pub fn open(path: &Path) -> Result<Self, FileStoreError> {
    if !path.is_dir() {
      return Err(FileStoreError::MalformedDB);
    }
    if !&path.join("nodes/").is_dir() ||
      !&path.join("edges/").is_dir() ||
      !&path.join("props/").is_dir() ||
      !&path.join("indexes/").is_dir() {
        return Err(FileStoreError::MalformedDB);
    }

    Ok(FsKvStore {
      base_path: path.to_path_buf(),
      p_marker: std::marker::PhantomData,
    })
  }

  pub fn init(path: &Path) -> Result<Self, FileStoreError> {
    if !path.is_dir() {
      if path.exists() {
        return Err(FileStoreError::MalformedDB);
      } else {
        fs::create_dir_all(&path)?;
      }
    }

    fs::create_dir_all(&path.join("nodes/"))?;
    fs::create_dir_all(&path.join("edges/"))?;
    fs::create_dir_all(&path.join("props/"))?;
    fs::create_dir_all(&path.join("indexes/"))?;

    Ok(FsKvStore {
      base_path: path.to_path_buf(),
      p_marker: std::marker::PhantomData,
    })
  }
}

#[derive(Error, Debug)]
pub enum FileStoreError {
  #[error("wrongly formatted database at path TODO")]
  MalformedDB,
  #[error("io error")]
  Io { #[from] source: Error },
}

impl mlua::UserData for GenericProperty {}

pub fn to_query(data: &Vec<u8>) -> Result<BasicQuery, SerialisationError> {
  // TODO Verschiedene Query Sprachen über zweiten Parameter
  // TODO Internes Schema verwenden um Abfragen zu verbessern
  let query = serde_json::from_slice(data)?;

  Ok(query)
}

