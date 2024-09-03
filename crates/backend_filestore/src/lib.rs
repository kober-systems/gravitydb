use sha2::Digest;
use gravity::schema::SchemaElement;
use std::collections::BTreeSet;
use std::fs;
use gravity::{GraphBuilder, GraphStore};
use gravity::schema::Property;
use gravity::ql;
pub use gravity::kv_graph_store::Error;
use gravity::kv_graph_store::{NodeData, EdgeData};
use std::path::{Path, PathBuf};
pub mod cli_helpers;

use gravity::{KVStore, BacklinkType};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

type HashId = String;

#[derive(Debug, Clone)]
pub struct GenericProperty(Vec<u8>);

impl SchemaElement<HashId, Error> for GenericProperty
{
  fn get_key(&self) -> HashId {
    format!("{:X}", sha2::Sha256::digest(&self.0))
  }

  fn serialize(&self) -> Result<Vec<u8>, Error> {
    Ok(self.0.clone())
  }

  fn deserialize(data: &[u8]) -> Result<Self, Error>
  where
    Self: Sized,
  {
    Ok(GenericProperty(data.to_vec()))
  }
}

impl Property<String, Error> for GenericProperty {
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

pub struct FsKvStore<T: Property<HashId, Error>> {
  p_marker: std::marker::PhantomData<T>,
  base_path: PathBuf,
}

impl<'a, T: Property<HashId, Error>> KVStore<Error> for FsKvStore<T>
{
  fn create_bucket(&self, key: &[u8]) -> Result<(), Error> {
    Ok(std::fs::create_dir_all(self.key_to_path(key))?)
  }

  fn delete_record(&self, key: &[u8]) -> Result<(), Error> {
    Ok(std::fs::remove_file(self.key_to_path(key))?)
  }

  fn store_record(&self, key: &[u8], value: &[u8]) -> Result<(), Error> {
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

  /// props_hash: the hash_id of the property that holds the index
  /// id:         the id of the node, edge or property that references
  ///             the property and needs a backling
  /// ty:         the type of the element that needs a backlink
  fn create_idx_backlink(&self, props_hash: &str, id: &str, ty: BacklinkType) -> Result<(), Error> {
    let index_path = "indexes/".to_string() + &props_hash.to_string() + "/";
    self.create_bucket(index_path.as_bytes())?;

    let prefix = match ty {
      BacklinkType::Node => "nodes",
      BacklinkType::Edge => "edges",
      BacklinkType::Property => "props",
    };
    let backlink_path = self.key_to_path(index_path.as_bytes()).join(prefix.to_owned() + "_" + id);
    let path = self.base_path.join(prefix).join(id);
    fs::hard_link(path, backlink_path)?;

    Ok(())
  }

  fn delete_property_backlink(&self, props_hash: &str, id: &str, ty: BacklinkType) -> Result<bool, Error> {
    let index_path = "indexes/".to_string() + &props_hash.to_string() + "/";

    let prefix = match ty {
      BacklinkType::Node => "nodes",
      BacklinkType::Edge => "edges",
      BacklinkType::Property => "props",
    };
    let backlink_path = index_path.clone() + prefix + "_" + id;
    self.delete_record(backlink_path.as_bytes())?;

    if self.list_records(index_path.as_bytes())?.is_empty() {
      fs::remove_dir(&index_path)?;

      Ok(true)
    } else {
      Ok(false)
    }
  }
}

impl<T: Property<HashId, Error>> FsKvStore<T> {
  fn key_to_path(&self, key: &[u8]) -> PathBuf {
    let path = Path::new(OsStr::from_bytes(key));
    PathBuf::from(self.base_path.join(path))
  }

  pub fn open(path: &Path) -> Result<Self, Error> {
    if !path.is_dir() {
      return Err(Error::MalformedDB);
    }
    if !&path.join("nodes/").is_dir() ||
      !&path.join("edges/").is_dir() ||
      !&path.join("props/").is_dir() ||
      !&path.join("indexes/").is_dir() {
        return Err(Error::MalformedDB);
    }

    Ok(FsKvStore {
      base_path: path.to_path_buf(),
      p_marker: std::marker::PhantomData,
    })
  }

  pub fn init(path: &Path) -> Result<Self, Error> {
    if !path.is_dir() {
      if path.exists() {
        return Err(Error::MalformedDB);
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

impl mlua::UserData for GenericProperty {}

pub fn to_query(data: &Vec<u8>) -> Result<BasicQuery, Error> {
  // TODO Verschiedene Query Sprachen über zweiten Parameter
  // TODO Internes Schema verwenden um Abfragen zu verbessern
  let query = serde_json::from_slice(data)?;

  Ok(query)
}

