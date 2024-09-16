use crate::{BacklinkType, GraphStore, KVStore};
use crate::schema::{Property, SchemaElement};
use crate::ql;
use serde::{Serialize, Deserialize};
use sha2::Digest;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::marker::PhantomData;
use thiserror::Error;

type HashId = String;
type BasicQuery = ql::BasicQuery<uuid::Uuid, HashId, HashId, ql::ShellFilter, ql::ShellFilter>;
type QueryResult = ql::QueryResult<uuid::Uuid, HashId>;
type NodeCtx = HashMap<uuid::Uuid, ql::VertexQueryContext<uuid::Uuid, HashId>>;
type EdgeCtx = HashMap<HashId, ql::EdgeQueryContext<uuid::Uuid, HashId>>;

pub struct KvGraphStore<T, K, E>
where
  T: Property<HashId, SerialisationError>,
  K: KVStore<E>,
  E: Send,
{
  p_marker: PhantomData<T>,
  kv_err_marker: PhantomData<E>,
  kv: K,
}

impl<T, K, E> KvGraphStore<T, K, E>
where
  T: Property<HashId, SerialisationError>,
  K: KVStore<E>,
  E: Send,
{
  pub fn from_kv(kv: K) -> Self {
    KvGraphStore {
      p_marker: PhantomData,
      kv_err_marker: PhantomData,
      kv,
    }
  }

  pub fn into_kv(self) -> K {
    self.kv
  }

  pub fn query(&self, q: BasicQuery) -> Result<QueryResult, Error<E>> {
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
  ) -> Result<NodeCtx, Error<E>> {
    use ql::VertexQuery::*;

    let result = match q {
      All => {
        self.kv.list_records("nodes/".as_bytes())
          .map_err(|e| Error::KV(e))?
          .into_iter()
          .map(|entry| {
            let id = String::from_utf8(entry)
              .or(Err(Error::MalformedDB))?;
            let id = uuid::Uuid::parse_str(&id)?;
            Ok((id, ql::VertexQueryContext::new(id)))
        })
        .collect::<Result<HashMap<_,_>, Error<E>>>()?
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
          for entry in self.kv.list_records(index_path.as_bytes()).map_err(|e| Error::KV(e))? {
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
          .collect::<Result<HashMap<_,_>, Error<E>>>()?
      }
      In(q) => {
        self.query_edges(q)?.into_iter()
            .map(|(edge_id, ctx)| {
            let edge = self.read_edge(&edge_id)?;
            Ok((edge.n1, ctx.into_vertex_ctx(edge.n1)))
          })
          .collect::<Result<HashMap<_,_>, Error<E>>>()?
      }
      Filter(_q, _filter) => unreachable!(),
    };

    Ok(result)
  }

  fn query_edges(
    &self,
    q: ql::EdgeQuery<uuid::Uuid, HashId, HashId, ql::ShellFilter, ql::ShellFilter>,
  ) -> Result<EdgeCtx, Error<E>> {
    use ql::EdgeQuery::*;

    let result = match q {
      All => {
        self.kv.list_records("edges/".as_bytes())
          .map_err(|e| Error::KV(e))?
          .into_iter()
          .map(|entry| {
          let id = String::from_utf8(entry)
            .or(Err(Error::MalformedDB))?;
          let key = id.clone();
          Ok((id, ql::EdgeQueryContext::new(key)))
        })
        .collect::<Result<HashMap<_,_>, Error<E>>>()?
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
          for entry in self.kv.list_records(index_path.as_bytes()).map_err(|e| Error::KV(e))? {
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
  ) -> Result<NodeCtx, Error<E>> {
    let mut result = HashMap::default();

    let properties = self.query_properties(q)?;
    // TODO Wie bei ReferencedProperties properties aber Verweise auf Knoten herausfiltern

    Ok(result)
  }

  fn query_properties(
    &self,
    q: ql::PropertyQuery<HashId>
  ) -> Result<HashSet<HashId>, Error<E>> {
    use ql::PropertyQuery::*;

    let mut result = HashSet::default();

    match q {
      Specific(id) => {
        let path = "props/".to_string() + &id;
        if self.kv.exists(path.as_bytes())
          .map_err(|e| Error::KV(e))?
        {
          result.insert(id);
        }
      }
      ReferencingProperties(q) => {
        for prop_id in self.query_properties(*q)? {
          let index_path = "indexes/".to_string() + &prop_id + "/";
          for entry in self.kv.list_records(index_path.as_bytes()).map_err(|e| Error::KV(e))? {
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
pub enum Error<E: Send> {
  #[error("wrongly formatted database at path TODO")]
  MalformedDB,
  #[error("node {0} allready exists")]
  NodeExists(String),
  #[error("the element existed before")]
  ExistedBefore,
  #[error("uuid parsing error (corrupted db)")]
  Uuid { #[from] source: uuid::Error },
  #[error("problem with kv store")]
  KV(E),
  #[error(transparent)]
  Prop(#[from] SerialisationError),
}

#[derive(Error, Debug)]
pub enum SerialisationError {
  #[error("io error")]
  Io { #[from] source: std::io::Error },
  #[error("node {0} allready exists")]
  NodeExists(String),
  #[error("json error")]
  Json { #[from] source: serde_json::Error },
  #[error("uuid parsing error (corrupted db)")]
  Uuid { #[from] source: uuid::Error },
}

impl<P, K, E> GraphStore<uuid::Uuid, NodeData, HashId, EdgeData, HashId, P, Error<E>> for KvGraphStore<P, K, E>
where
  P: Property<HashId, SerialisationError>,
  K: KVStore<E>,
  E: Send,
{
  fn create_node(&mut self, id: uuid::Uuid, properties: &P) -> Result<(), Error<E>> {
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

    if self.kv.exists(path.as_bytes()).map_err(|e| Error::KV(e))? {
      return Err(Error::NodeExists(path));
    };

    self.kv.store_record(&path.as_bytes(), &node).map_err(|e| Error::KV(e))?;

    self.kv.create_idx_backlink(&props_hash, &id, BacklinkType::Node).map_err(|e| Error::KV(e))?;

    Ok(())
  }

  fn read_node(&self, id: uuid::Uuid) -> Result<NodeData, Error<E>> {
    let path = "nodes/".to_string() + &uuid_to_key(id);

    let data = self.kv.fetch_record(path.as_bytes()).map_err(|e| Error::KV(e))?;
    let node: NodeData = SchemaElement::deserialize(&data)?;
    Ok(node)
  }

  fn update_node(&mut self, id: uuid::Uuid, properties: &P) -> Result<(), Error<E>> {
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
    self.kv.store_record(&path.as_bytes(), &node).map_err(|e| Error::KV(e))?;

    let id = uuid_to_key(id);
    let last_reference = self.kv.delete_property_backlink(&old_properties, &id, BacklinkType::Node).map_err(|e| Error::KV(e))?;
    if last_reference {
      self.delete_property(&old_properties)?;
    }

    self.kv.create_idx_backlink(&props_hash, &id, BacklinkType::Node).map_err(|e| Error::KV(e))?;

    Ok(())
  }

  fn delete_node(&mut self, id: uuid::Uuid) -> Result<(), Error<E>> {
    let NodeData {
      id,
      properties,
      incoming: _,
      outgoing: _,
    } = self.read_node(id)?;

    let id = uuid_to_key(id);
    let path = "nodes/".to_string() + &id;

    let last_reference = self.kv.delete_property_backlink(&properties, &id, BacklinkType::Node).map_err(|e| Error::KV(e))?;
    if last_reference {
      self.delete_property(&properties)?;
    }

    self.kv.delete_record(path.as_bytes()).map_err(|e| Error::KV(e))?;
    Ok(())
  }

  fn create_edge(&mut self, n1: uuid::Uuid, n2: uuid::Uuid, properties: &P) -> Result<HashId, Error<E>> {
    let props_hash = self.create_property(properties)?;
    let edge = EdgeData {
      n1,
      n2,
      properties: props_hash.clone(),
    };

    let hash = edge.get_key();
    let path = "edges/".to_string() + &hash;

    let edge = SchemaElement::serialize(&edge)?;
    self.kv.store_record(&path.as_bytes(), &edge).map_err(|e| Error::KV(e))?;

    self.kv.create_idx_backlink(&props_hash, &hash, BacklinkType::Edge).map_err(|e| Error::KV(e))?;

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
    self.kv.store_record(&path.as_bytes(), &node).map_err(|e| Error::KV(e))?;

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
    self.kv.store_record(&path.as_bytes(), &node).map_err(|e| Error::KV(e))?;

    Ok(hash)
  }

  fn read_edge(&self, id: &HashId) -> Result<EdgeData, Error<E>> {
    let path = "edges/".to_string() + id;

    let data = self.kv.fetch_record(path.as_bytes()).map_err(|e| Error::KV(e))?;
    let edge = SchemaElement::deserialize(&data)?;
    Ok(edge)
  }

  fn delete_edge(&mut self, id: &HashId) -> Result<(), Error<E>> {
    let EdgeData {
      properties: props_hash,
      n1,
      n2,
    } = self.read_edge(id)?;

    let path = "edges/".to_string() + id;

    self.kv.delete_record(&path.as_bytes()).map_err(|e| Error::KV(e))?;

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
    self.kv.store_record(&path.as_bytes(), &node).map_err(|e| Error::KV(e))?;

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
    self.kv.store_record(&path.as_bytes(), &node).map_err(|e| Error::KV(e))?;

    let last_reference = self.kv.delete_property_backlink(&props_hash, &id, BacklinkType::Edge).map_err(|e| Error::KV(e))?;
    if last_reference {
      self.delete_property(&props_hash)?;
    }

    Ok(())
  }

  fn create_property(&mut self, properties: &P) -> Result<HashId, Error<E>> {
    let hash = properties.get_key();
    let path = "props/".to_string() + &hash;

    let data = properties.serialize()?;
    self.kv.store_record(&path.as_bytes(), &data).map_err(|e| Error::KV(e))?;

    properties.nested().iter().try_for_each(|nested| {
      match self.create_property(nested) {
        Ok(nested_hash) => {
          self.kv.create_idx_backlink(&nested_hash, &hash, BacklinkType::Property).map_err(|e| Error::KV(e))?;
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

  fn read_property(&mut self, id: &HashId) -> Result<P, Error<E>> {
    let path = "props/".to_string() + id;

    let data = self.kv.fetch_record(path.as_bytes()).map_err(|e| Error::KV(e))?;
    let property = SchemaElement::deserialize(&data)?;
    Ok(property)
  }

  fn delete_property(&mut self, id: &HashId) -> Result<(), Error<E>> {
    let path = "props/".to_string() + id;

    let data = self.kv.fetch_record(&path.as_bytes()).map_err(|e| Error::KV(e))?;
    let properties: P = SchemaElement::deserialize(&data)?;

    for nested in properties.nested().iter() {
      let nested_hash = nested.get_key();
      let last_reference = self.kv.delete_property_backlink(&nested_hash, id, BacklinkType::Property).map_err(|e| Error::KV(e))?;
      if last_reference {
        self.delete_property(&nested_hash)?;
      }
    }

    self.kv.delete_record(path.as_bytes()).map_err(|e| Error::KV(e))?;
    Ok(())
  }
}

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

impl SchemaElement<String, SerialisationError> for NodeData
{
  fn get_key(&self) -> String {
    uuid_to_key(self.id)
  }

  fn serialize(&self) -> Result<Vec<u8>, SerialisationError> {
    Ok(serde_json::to_vec(self)?)
  }

  fn deserialize(data: &[u8]) -> Result<Self, SerialisationError>
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

impl SchemaElement<HashId, SerialisationError> for EdgeData
{
  fn get_key(&self) -> HashId {
    let data = serde_json::to_vec(self).unwrap();
    format!("{:X}", sha2::Sha256::digest(&data))
  }

  fn serialize(&self) -> Result<Vec<u8>, SerialisationError> {
    Ok(serde_json::to_vec(self)?)
  }

  fn deserialize(data: &[u8]) -> Result<Self, SerialisationError>
  where
    Self: Sized,
  {
    Ok(serde_json::from_slice(data)?)
  }
}

impl<P, K, E> mlua::UserData for KvGraphStore<P, K, E>
where
  P: Property<HashId, SerialisationError> + mlua::UserData + std::clone::Clone + 'static,
  K: KVStore<E>,
  E: Send + Sync + std::fmt::Debug + 'static,
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

fn uuid_to_key(id: uuid::Uuid) -> String {
  id
    .hyphenated()
    .encode_lower(&mut uuid::Uuid::encode_buffer())
    .to_string()
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

