use sha2::Digest;
use serde::{Serialize, Deserialize};
use gravity::schema::SchemaElement;
use std::collections::BTreeSet;
use std::fs;
use gravity::{GraphBuilder, GraphStore};
use gravity::schema::Property;
use gravity::ql;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use thiserror::Error;
pub mod cli_helpers;

pub trait Node<P: Property<HashId, Error>> {
  fn id(&self) -> uuid::Uuid;
  fn properties(&self) -> P;
}

use gravity::{KVStore, BacklinkType};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

type HashId = String;

#[derive(Deserialize, Serialize, Debug)]
pub struct NodeData {
  pub id: uuid::Uuid,
  // Schlüssel des Datensatzes, welcher die Eigenschaften
  // des Knotens enthält
  pub properties: HashId,
  // Hashes der eingehenden Verbindungen (Edges)
  pub incoming: BTreeSet<HashId>,
  // Hashes der ausgehenden Verbindungen (Edges)
  pub outgoing: BTreeSet<HashId>,
}

impl SchemaElement<String, Error> for NodeData
{
  fn get_key(&self) -> String {
    uuid_to_key(self.id)
  }

  fn serialize(&self) -> Result<Vec<u8>, Error> {
    Ok(serde_json::to_vec(self)?)
  }

  fn deserialize(data: &[u8]) -> Result<Self, Error>
  where
    Self: Sized,
  {
    Ok(serde_json::from_slice(data)?)
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EdgeData {
  pub properties: HashId,
  pub n1: uuid::Uuid,
  pub n2: uuid::Uuid,
}

impl SchemaElement<HashId, Error> for EdgeData
{
  fn get_key(&self) -> HashId {
    let data = serde_json::to_vec(self).unwrap();
    format!("{:X}", sha2::Sha256::digest(&data))
  }

  fn serialize(&self) -> Result<Vec<u8>, Error> {
    Ok(serde_json::to_vec(self)?)
  }

  fn deserialize(data: &[u8]) -> Result<Self, Error>
  where
    Self: Sized,
  {
    Ok(serde_json::from_slice(data)?)
  }
}

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
type QueryResult = ql::QueryResult<uuid::Uuid, HashId>;

type NodeCtx = HashMap<uuid::Uuid, ql::VertexQueryContext<uuid::Uuid, HashId>>;
type EdgeCtx = HashMap<HashId, ql::EdgeQueryContext<uuid::Uuid, HashId>>;

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

pub struct FsStore<T, K>
where
  T: Property<HashId, Error>,
  K: KVStore<Error>,
{
  p_marker: std::marker::PhantomData<T>,
  kv: K,
}

impl<T, K> FsStore<T, K>
where
  T: Property<HashId, Error>,
  K: KVStore<Error>,
{
  pub fn from_kv(kv: K) -> Self {
    FsStore {
      p_marker: std::marker::PhantomData,
      kv,
    }
  }

  pub fn query(&self, q: BasicQuery) -> Result<QueryResult, Error> {
    let context = match q {
      BasicQuery::V(q) => {
        self.query_nodes(q)?.into()
      }
      BasicQuery::E(q) => {
        self.query_edges(q)?.into()
      }
      BasicQuery::P(q) => {
        self.query_property_nodes(q)?.into()
      }
    };

    Ok(context)
  }

  fn query_nodes(
    &self,
    q: ql::VertexQuery<uuid::Uuid, HashId, HashId, ql::ShellFilter, ql::ShellFilter>
  ) -> Result<NodeCtx, Error> {
    use ql::VertexQuery::*;

    let result = match q {
      All => {
        self.kv.list_records("nodes/".as_bytes())?
          .into_iter()
          .map(|entry| {
            let id = String::from_utf8(entry)
              .or(Err(Error::MalformedDB))?;
            let id = uuid::Uuid::parse_str(&id)?;
            Ok((id, ql::VertexQueryContext::new(id)))
        })
        .collect::<Result<HashMap<_,_>, Error>>()?
      }
      Specific(ids) => {
        ids.into_iter()
          .map(|id| (id, ql::VertexQueryContext::new(id)))
          .collect()
      }
      Property(q) => {
        let mut result = HashMap::default();

        for prop_id in self.query_properties(q)?.into_iter() {
          let index_path = "indexes/".to_string() + &prop_id + "/";
          for entry in self.kv.list_records(index_path.as_bytes())? {
            let reference = String::from_utf8(entry)
              .or(Err(Error::MalformedDB))?;
            let (prefix, reference) = reference
              .split_once("_")
              .ok_or(Error::MalformedDB)?;
            if prefix == "nodes" {
              let id = uuid::Uuid::parse_str(reference)?;
              result.insert(id, ql::VertexQueryContext::new(id));
            }
          }
        }

        result
      }
      Union(sub1, sub2) => {
        union(
          self.query_nodes(*sub1)?,
          self.query_nodes(*sub2)?
        )
      }
      Intersect(sub1, sub2) => {
        intersection(
          self.query_nodes(*sub1)?,
          self.query_nodes(*sub2)?,
        )
      }
      Substract(sub1, sub2) => {
        substraction(
          self.query_nodes(*sub1)?,
          self.query_nodes(*sub2)?
        )
      }
      DisjunctiveUnion(sub1, sub2) => {
        disjunction(
          self.query_nodes(*sub1)?,
          self.query_nodes(*sub2)?
        )
      }
      Store(_q) => unreachable!(),
      Out(q) => {
        self.query_edges(q)?.into_iter()
          .map(|(edge_id, ctx)| {
            let edge = self.read_edge(&edge_id)?;
            Ok((edge.n2, ctx.into_vertex_ctx(edge.n2)))
          })
          .collect::<Result<HashMap<_,_>, Error>>()?
      }
      In(q) => {
        self.query_edges(q)?.into_iter()
            .map(|(edge_id, ctx)| {
            let edge = self.read_edge(&edge_id)?;
            Ok((edge.n1, ctx.into_vertex_ctx(edge.n1)))
          })
          .collect::<Result<HashMap<_,_>, Error>>()?
      }
      Filter(_q, _filter) => unreachable!(),
    };

    Ok(result)
  }

  fn query_edges(
    &self,
    q: ql::EdgeQuery<uuid::Uuid, HashId, HashId, ql::ShellFilter, ql::ShellFilter>,
  ) -> Result<EdgeCtx, Error> {
    use ql::EdgeQuery::*;

    let result = match q {
      All => {
        self.kv.list_records("edges/".as_bytes())?
          .into_iter()
          .map(|entry| {
          let id = String::from_utf8(entry)
            .or(Err(Error::MalformedDB))?;
          let key = id.clone();
          Ok((id, ql::EdgeQueryContext::new(key)))
        })
        .collect::<Result<HashMap<_,_>, Error>>()?
      }
      Specific(ids) => {
        ids.into_iter()
          .map(|id| {
            let key = id.clone();
            (id, ql::EdgeQueryContext::new(key))
          })
          .collect()
      }
      Property(q) => {
        let mut result = HashMap::default();

        for prop_id in self.query_properties(q)? {
          let index_path = "indexes/".to_string() + &prop_id + "/";
          for entry in self.kv.list_records(index_path.as_bytes())? {
            let reference = String::from_utf8(entry)
              .or(Err(Error::MalformedDB))?;
            let (prefix, reference) = reference
              .split_once("_")
              .ok_or(Error::MalformedDB)?;
            if prefix == "edges" {
              let id = reference.to_string();
              let key = id.clone();
              result.insert(id, ql::EdgeQueryContext::new(key));
            }
          }
        }

        result
      }
      Union(sub1, sub2) => {
        union(
          self.query_edges(*sub1)?,
          self.query_edges(*sub2)?
        )
      }
      Intersect(sub1, sub2) => {
        intersection(
          self.query_edges(*sub1)?,
          self.query_edges(*sub2)?
        )
      }
      Substract(sub1, sub2) => {
        substraction(
          self.query_edges(*sub1)?,
          self.query_edges(*sub2)?
        )
      }
      DisjunctiveUnion(sub1, sub2) => {
        disjunction(
          self.query_edges(*sub1)?,
          self.query_edges(*sub2)?
        )
      }
      Store(_q) => unreachable!(),
      Out(q) => {
        let context = self.query_nodes(*q)?;

        let mut result = HashMap::default();

        for (node_id, ctx) in context.into_iter() {
          let node = self.read_node(node_id)?;
          for edge_id in node.outgoing.into_iter() {
            let key = edge_id.clone();
            result.insert(edge_id, ctx.clone().into_edge_ctx(key));
          }
        }

        result
      }
      In(q) => {
        let context = self.query_nodes(*q)?;

        let mut result = HashMap::default();

        for (node_id, ctx) in context.into_iter() {
          let node = self.read_node(node_id)?;
          for edge_id in node.incoming.into_iter() {
            let key = edge_id.clone();
            result.insert(edge_id, ctx.clone().into_edge_ctx(key));
          }
        }

        result
      }
      Filter(_q, _filter) => unreachable!(),
    };

    Ok(result)
  }

  fn query_property_nodes(
    &self,
    q: ql::PropertyQuery<HashId>
  ) -> Result<NodeCtx, Error> {
    let mut result = HashMap::default();

    let properties = self.query_properties(q)?;
    // TODO Wie bei ReferencedProperties properties aber Verweise auf Knoten herausfiltern

    Ok(result)
  }

  fn query_properties(
    &self,
    q: ql::PropertyQuery<HashId>
  ) -> Result<HashSet<HashId>, Error> {
    use ql::PropertyQuery::*;

    let mut result = HashSet::default();

    match q {
      Specific(id) => {
        let path = "props/".to_string() + &id;
        if self.kv.exists(path.as_bytes())?
        {
          result.insert(id);
        }
      }
      ReferencingProperties(q) => {
        for prop_id in self.query_properties(*q)? {
          let index_path = "indexes/".to_string() + &prop_id + "/";
          for entry in self.kv.list_records(index_path.as_bytes())? {
            let reference = String::from_utf8(entry)
              .or(Err(Error::MalformedDB))?;
            let (prefix, reference) = reference
              .split_once("_")
              .ok_or(Error::MalformedDB)?;
            if prefix == "props" {
              result.insert(reference.to_string());
            }
          }
        }
      }
      ReferencedProperties(q) => {
        // TODO Hier benötigen wir das Schema
      }
    };

    Ok(result)
  }

}

#[derive(Error, Debug)]
pub enum Error {
  #[error("wrongly formatted database at path TODO")]
  MalformedDB,
  #[error("io error")]
  Io { #[from] source: std::io::Error },
  #[error("node allready exists")]
  NodeExists,
  #[error("json error")]
  Json { #[from] source: serde_json::Error },
  #[error("the element existed before")]
  ExistedBefore,
  #[error("uuid parsing error (corrupted db)")]
  Uuid { #[from] source: uuid::Error },
}

impl<N, P, K> GraphBuilder<N, P, Error> for FsStore<P, K>
where
  N: Node<P>,
  P: Property<HashId, Error>,
  K: KVStore<Error>,
{
  fn add_node(&mut self, node: N) -> Result<(), Error> {
    let p = node.properties();
    self.create_node(node.id(), &p)
  }

  fn add_edge(&mut self, n1: &N, n2: &N, p: &P) -> Result<(), Error> {
    self.create_edge(n1.id(), n2.id(), p)?;
    Ok(())
  }

  fn remove_node(&mut self, node: &N) -> Result<(), Error> {
    self.delete_node(node.id())?;
    Ok(())
  }

  fn remove_edge(&mut self, n1: &N, n2: &N, p: &P) -> Result<(), Error> {
    let props_hash = p.get_key();
    let edge = EdgeData {
      n1: n1.id(),
      n2: n2.id(),
      properties: props_hash,
    };

    self.delete_edge(&edge.get_key())?;
    Ok(())
  }
}

impl<P, K> GraphStore<uuid::Uuid, NodeData, HashId, EdgeData, HashId, P, Error> for FsStore<P, K>
where
  P: Property<HashId, Error>,
  K: KVStore<Error>,
{
  fn create_node(&mut self, id: uuid::Uuid, properties: &P) -> Result<(), Error> {
    let props_hash = self.create_property(properties)?;
    let node = NodeData {
      id,
      properties: props_hash.clone(),
      incoming: BTreeSet::new(),
      outgoing: BTreeSet::new(),
    };
    let id = node.get_key();
    let node = SchemaElement::serialize(&node)?;

    let path = "nodes/".to_string() + &id;

    if self.kv.exists(path.as_bytes())? {
      log::error!("node {:?} allready exists", path);
      return Err(Error::NodeExists);
    };

    log::debug!("creating node file {:?} with content {}", path, String::from_utf8_lossy(&node));
    self.kv.store_record(&path.as_bytes(), &node)?;

    self.kv.create_idx_backlink(&props_hash, &id, BacklinkType::Node)?;

    Ok(())
  }

  fn read_node(&self, id: uuid::Uuid) -> Result<NodeData, Error> {
    let path = "nodes/".to_string() + &uuid_to_key(id);

    let data = self.kv.fetch_record(path.as_bytes())?;
    let node: NodeData = SchemaElement::deserialize(&data)?;
    Ok(node)
  }

  fn update_node(&mut self, id: uuid::Uuid, properties: &P) -> Result<(), Error> {
    let props_hash = self.create_property(properties)?;
    let path = "nodes/".to_string() + &uuid_to_key(id);
    let NodeData {
      id,
      properties: old_properties,
      incoming,
      outgoing,
    } = self.read_node(id)?;
    let node = NodeData {
      id,
      properties: props_hash.clone(),
      incoming,
      outgoing,
    };
    let node = SchemaElement::serialize(&node)?;
    self.kv.store_record(&path.as_bytes(), &node)?;

    let id = uuid_to_key(id);
    let last_reference = self.kv.delete_property_backlink(&old_properties, &id, BacklinkType::Node)?;
    if last_reference {
      self.delete_property(&old_properties)?;
    }

    self.kv.create_idx_backlink(&props_hash, &id, BacklinkType::Node)?;

    Ok(())
  }

  fn delete_node(&mut self, id: uuid::Uuid) -> Result<(), Error> {
    let NodeData {
      id,
      properties,
      incoming: _,
      outgoing: _,
    } = self.read_node(id)?;

    let id = uuid_to_key(id);
    let path = "nodes/".to_string() + &id;

    let last_reference = self.kv.delete_property_backlink(&properties, &id, BacklinkType::Node)?;
    if last_reference {
      self.delete_property(&properties)?;
    }

    self.kv.delete_record(path.as_bytes())?;
    Ok(())
  }

  fn create_edge(&mut self, n1: uuid::Uuid, n2: uuid::Uuid, properties: &P) -> Result<HashId, Error> {
    let props_hash = self.create_property(properties)?;
    let edge = EdgeData {
      n1,
      n2,
      properties: props_hash.clone(),
    };

    let hash = edge.get_key();
    let path = "edges/".to_string() + &hash;

    let edge = SchemaElement::serialize(&edge)?;
    self.kv.store_record(&path.as_bytes(), &edge)?;

    self.kv.create_idx_backlink(&props_hash, &hash, BacklinkType::Edge)?;

    let path = "nodes/".to_string() + &uuid_to_key(n1);
    let NodeData {
      id,
      properties,
      incoming,
      mut outgoing,
    } = self.read_node(n1)?;
    outgoing.insert(hash.clone());
    let node = NodeData {
      id,
      properties,
      incoming,
      outgoing,
    };
    let node = SchemaElement::serialize(&node)?;
    self.kv.store_record(&path.as_bytes(), &node)?;

    let path = "nodes/".to_string() + &uuid_to_key(n2);
    let NodeData {
      id,
      properties,
      mut incoming,
      outgoing,
    } = self.read_node(n2)?;
    incoming.insert(hash.clone());
    let node = NodeData {
      id,
      properties,
      incoming,
      outgoing,
    };
    let node = SchemaElement::serialize(&node)?;
    self.kv.store_record(&path.as_bytes(), &node)?;

    Ok(hash)
  }

  fn read_edge(&self, id: &HashId) -> Result<EdgeData, Error> {
    let path = "edges/".to_string() + id;

    let data = self.kv.fetch_record(path.as_bytes())?;
    let edge = SchemaElement::deserialize(&data)?;
    Ok(edge)
  }

  fn delete_edge(&mut self, id: &HashId) -> Result<(), Error> {
    let EdgeData {
      properties: props_hash,
      n1,
      n2,
    } = self.read_edge(id)?;

    let path = "edges/".to_string() + id;

    self.kv.delete_record(&path.as_bytes())?;

    let path = "nodes/".to_string() + &uuid_to_key(n1);
    let NodeData {
      id: _id,
      properties,
      incoming,
      mut outgoing,
    } = self.read_node(n1)?;
    outgoing.remove(id);
    let node = NodeData {
      id: n1,
      properties,
      incoming,
      outgoing,
    };
    let node = SchemaElement::serialize(&node)?;
    self.kv.store_record(&path.as_bytes(), &node)?;

    let path = "nodes/".to_string() + &uuid_to_key(n2);
    let NodeData {
      id: _id,
      properties,
      mut incoming,
      outgoing,
    } = self.read_node(n2)?;
    incoming.remove(id);
    let node = NodeData {
      id: n2,
      properties,
      incoming,
      outgoing,
    };
    let node = SchemaElement::serialize(&node)?;
    self.kv.store_record(&path.as_bytes(), &node)?;

    let last_reference = self.kv.delete_property_backlink(&props_hash, &id, BacklinkType::Edge)?;
    if last_reference {
      self.delete_property(&props_hash)?;
    }

    Ok(())
  }

  fn create_property(&mut self, properties: &P) -> Result<HashId, Error> {
    let hash = properties.get_key();
    let path = "props/".to_string() + &hash;

    let data = properties.serialize()?;
    log::debug!("creating property file {:?} with content {}", path, String::from_utf8_lossy(&data));
    self.kv.store_record(&path.as_bytes(), &data)?;

    properties.nested().iter().try_for_each(|nested| {
      match self.create_property(nested) {
        Ok(nested_hash) => {
          self.kv.create_idx_backlink(&nested_hash, &hash, BacklinkType::Property)?;
          Ok(())
        }
        Err(e) => {
          use Error::*;
          match e {
            ExistedBefore => Ok(()),
            _ => Err(e),
          }
        }
      }
    })?;

    Ok(hash)
  }

  fn read_property(&mut self, id: &HashId) -> Result<P, Error> {
    let path = "props/".to_string() + id;

    let data = self.kv.fetch_record(path.as_bytes())?;
    let property = SchemaElement::deserialize(&data)?;
    Ok(property)
  }

  fn delete_property(&mut self, id: &HashId) -> Result<(), Error> {
    let path = "props/".to_string() + id;

    let data = self.kv.fetch_record(&path.as_bytes())?;
    let properties: P = SchemaElement::deserialize(&data)?;

    for nested in properties.nested().iter() {
      let nested_hash = nested.get_key();
      let last_reference = self.kv.delete_property_backlink(&nested_hash, id, BacklinkType::Property)?;
      if last_reference {
        self.delete_property(&nested_hash)?;
      }
    }

    self.kv.delete_record(path.as_bytes())?;
    Ok(())
  }
}

impl<P, K> mlua::UserData for FsStore<P, K>
where
  P: Property<HashId, Error> + mlua::UserData + std::clone::Clone + 'static,
  K: KVStore<Error>,
{
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    use mlua::prelude::LuaError;

    methods.add_method_mut("create_node", |_, db, props: P| {
      let id = uuid::Uuid::new_v4();
      match db.create_node(id, &props) {
        Ok(_) => Ok(()),
        Err(e) => Err(LuaError::external(e))
      }
    });
  }
}

impl mlua::UserData for GenericProperty {}

fn uuid_to_key(id: uuid::Uuid) -> String {
  id
    .to_hyphenated()
    .encode_lower(&mut uuid::Uuid::encode_buffer())
    .to_string()
}

pub fn to_query(data: &Vec<u8>) -> Result<BasicQuery, Error> {
  // TODO Verschiedene Query Sprachen über zweiten Parameter
  // TODO Internes Schema verwenden um Abfragen zu verbessern
  let query = serde_json::from_slice(data)?;

  Ok(query)
}

use core::hash::Hash;

fn union<K, V>(
  c1: HashMap<K, V>,
  c2: HashMap<K, V>
) ->
  HashMap<K, V>
where
  K: Eq + Hash,
{
  let mut result = c1;

  result.extend(c2.into_iter());
  result
}

fn intersection<K, V>(
  c1: HashMap<K, V>,
  c2: HashMap<K, V>
) ->
  HashMap<K, V>
where
  K: Eq + Hash,
{
  let mut result = c1;
  let mut c2 = c2;

  c2.retain(|k, _v| result.contains_key(k));
  result.retain(|k, _v| c2.contains_key(k));
  result
}

fn substraction<K, V>(
  c1: HashMap<K, V>,
  c2: HashMap<K, V>
) ->
  HashMap<K, V>
where
  K: Eq + Hash,
{
  let mut result = c1;

  result
    .retain(|k, _v| !c2.contains_key(k));

  result
}

fn disjunction<K, V>(
  c1: HashMap<K, V>,
  c2: HashMap<K, V>
) ->
  HashMap<K, V>
where
  K: Eq + Hash + Clone,
  V: Clone,
{
  let mut result = HashMap::default();

  result.extend(c1.clone().into_iter().filter(|(k, _)| c2.contains_key(k)));
  result.extend(c2.into_iter().filter(|(k, _)| c1.contains_key(k)));

  result
}

