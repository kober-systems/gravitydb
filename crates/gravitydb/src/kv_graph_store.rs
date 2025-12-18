use std::str::FromStr;
use sha2::Digest;
use crate::schema::SchemaElement;
use serde::{Serialize, Deserialize};
use std::collections::{BTreeSet, HashMap, HashSet};
use crate::{PropertyGraphReader, PropertyFilter};
use crate::GraphStore;
use crate::GraphBuilder;
use crate::schema::Property;
use crate::ql;
use core::hash::Hash;
use crate::KVStore;
use std::marker::PhantomData;
use thiserror::Error;
#[cfg(feature="lua")]
use mlua::FromLua;

pub trait Node<P: Property<HashId, SerialisationError>> {
  fn id(&self) -> VertexId;
  fn properties(&self) -> P;
}

pub type VertexId = Uuid;

#[derive(Hash, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "lua", derive(FromLua))]
pub struct Uuid(pub uuid::Uuid);

impl Uuid {
  pub fn new() -> Self {
    Self(uuid::Uuid::new_v4())
  }

  pub fn from_key(key: &str) -> Result<Self, uuid::Error> {
    Ok(Self(uuid::Uuid::from_str(key)?))
  }

  pub fn to_key(&self) -> String {
    self.0
      .hyphenated()
      .encode_lower(&mut uuid::Uuid::encode_buffer())
      .to_string()
  }
}

pub type HashId = String;

enum BacklinkType {
  Node,
  Edge,
  Property,
}

pub type BasicQuery = ql::BasicQuery<VertexId, HashId, HashId, ql::ShellFilter, ql::ShellFilter>;
type QueryResult = ql::QueryResult<VertexId, HashId, HashId>;

type NodeCtx = HashMap<VertexId, ql::VertexQueryContext<VertexId, HashId>>;
type EdgeCtx = HashMap<HashId, ql::EdgeQueryContext<VertexId, HashId>>;

pub struct KvGraphStore<T, K, E>
where
  T: Property<HashId, SerialisationError>,
  K: KVStore<E>,
  E: Send,
{
  kv: K,
  p_marker: PhantomData<T>,
  kv_err_marker: PhantomData<E>,
}

impl<T, K, E> KvGraphStore<T, K, E>
where
  T: Property<HashId, SerialisationError>,
  K: KVStore<E>,
  E: Send,
{
  pub fn query<Q: Into<BasicQuery>>(&self, q: Q) -> Result<QueryResult, Error<E>> {
    let q = q.into();
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

  pub fn extract_properties(&self, result: &QueryResult) -> Result<Vec<T>, Error<E>> {
    let nodes_iter = result.vertices.iter().map(|(n_id, _prop)| {
      let n = self.read_node(*n_id)?;
      self.read_property(&n.properties)
    });
    let edges_iter = result.edges.iter().map(|(e_id, _prop)| {
      let e = self.read_edge(&e_id)?;
      self.read_property(&e.properties)
    });
    nodes_iter.chain(edges_iter).collect::<Result<Vec<T>,_>>()
  }

  pub fn extract_path_properties(&self, result: &QueryResult) -> Result<Vec<Vec<T>>, Error<E>> {
    result.paths.iter()
      .map(|(start, path, end)| {
        path.into_iter()
          .fold(Ok(vec![]), |path, (v_id, e_id)| {
            let mut path: Vec<_> = path?;
            let n = self.read_node(*v_id)?;
            let prop = self.read_property(&n.properties)?;
            path.push(prop);

            let e = self.read_edge(e_id)?;
            let prop = self.read_property(&e.properties)?;
            path.push(prop);

            if let Some(e_id) = start {
              let e = self.read_edge(e_id)?;
              let prop = self.read_property(&e.properties)?;
              path.insert(0, prop);
            }
            if let Some(v_id) = end {
              let n = self.read_node(*v_id)?;
              let prop = self.read_property(&n.properties)?;
              path.push(prop);
            }

            Ok(path)
          })
      })
      .collect::<Result<Vec<Vec<_>>, _>>()
  }

  fn query_nodes(
    &self,
    q: ql::VertexQuery<VertexId, HashId, HashId, ql::ShellFilter, ql::ShellFilter>
  ) -> Result<NodeCtx, Error<E>> {
    use ql::VertexQuery::*;

    let result = match q {
      All => {
        self.nodes(PropertyFilter::All)?
          .map(|id| Ok((id, ql::VertexQueryContext::new(id))))
          .collect::<Result<HashMap<_,_>, Error<E>>>()?
      }
      Specific(ids) => {
        ids.into_iter()
          .map(|id| (id, ql::VertexQueryContext::new(id)))
          .collect()
      }
      Property(q) => {
        let mut result = HashMap::default();

        for prop_id in self.query_properties(q)? {
          for id in self.nodes(PropertyFilter::Only(prop_id))? {
            result.insert(id, ql::VertexQueryContext::new(id));
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
    q: ql::EdgeQuery<VertexId, HashId, HashId, ql::ShellFilter, ql::ShellFilter>,
  ) -> Result<EdgeCtx, Error<E>> {
    use ql::EdgeQuery::*;

    let result = match q {
      All => {
        self.edges(PropertyFilter::All)?
          .map(|id| {
            let key = id.clone();
            Ok((id, ql::EdgeQueryContext::new(key)))
          })
          .collect::<Result<HashMap<_,_>, Error<E>>>()?
      }
      Specific(ids) => {
        ids.into_iter()
          .map(|id| (id.clone(), ql::EdgeQueryContext::new(id)))
          .collect()
      }
      Property(q) => {
        let mut result = HashMap::default();

        for prop_id in self.query_properties(q)? {
          for id in self.edges(PropertyFilter::Only(prop_id))? {
            let key = id.clone();
            result.insert(id, ql::EdgeQueryContext::new(key));
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
          self.query_edges(*sub2)?,
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
    let result = HashMap::default();

    let _properties = self.query_properties(q)?;
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
          for id in self.properties(PropertyFilter::Only(prop_id))? {
            result.insert(id);
          }
        }
      }
      ReferencedProperties(_q) => {
        // TODO Hier benötigen wir das Schema
      }
    };

    Ok(result)
  }

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

  /// props_hash: the hash_id of the property that holds the index
  /// id:         the id of the node, edge or property that references
  ///             the property and needs a backling
  /// ty:         the type of the element that needs a backlink
  fn create_idx_backlink(&mut self, props_hash: &str, id: &str, ty: BacklinkType) -> Result<(), Error<E>> {
    let index_path = "indexes/".to_string() + props_hash + "/";
    self.kv.create_bucket(index_path.as_bytes()).map_err(|e| Error::KV(e))?;

    let prefix = match ty {
      BacklinkType::Node => "nodes",
      BacklinkType::Edge => "edges",
      BacklinkType::Property => "props",
    };
    let backlink_path = index_path + prefix + "_" + id;
    let path = prefix.to_string() + "/" + id;
    self.kv.store_record(&backlink_path.as_bytes(), &path.as_bytes()).map_err(|e| Error::KV(e))?;

    Ok(())
  }

  fn delete_property_backlink(&mut self, props_hash: &str, id: &str, ty: BacklinkType) -> Result<bool, Error<E>> {
    let index_path = "indexes/".to_string() + props_hash + "/";

    let prefix = match ty {
      BacklinkType::Node => "nodes",
      BacklinkType::Edge => "edges",
      BacklinkType::Property => "props",
    };
    let backlink_path = index_path.clone() + prefix + "_" + id;
    self.kv.delete_record(backlink_path.as_bytes()).map_err(|e| Error::KV(e))?;

    if self.kv.list_records(index_path.as_bytes()).map_err(|e| Error::KV(e))?.is_empty() {
      Ok(true)
    } else {
      Ok(false)
    }
  }

  fn filter_by_property(&self, prefix: &str, filter: PropertyFilter<HashId>) -> Result<impl Iterator<Item=HashId>, Error<E>> {
    use PropertyFilter::*;

    let iter = match &filter {
      Only(prop_id) => {
        self.kv.list_records(format!("indexes/{prop_id}/{prefix}_").as_bytes())
      },
      FromTo(_from, _to) => {
        self.kv.list_records("indexes/".as_bytes())
      },
      All => {
        self.kv.list_records(format!("{prefix}/").as_bytes())
      },
    };
    let iter = iter
      .map_err(|e| Error::KV(e))?
      .into_iter()
      .map(|entry| Ok(String::from_utf8(entry)?));

    let iter = match filter {
      FromTo(from, to) => {
        iter
          .filter_map(|entry: Result<_, Error<E>>| {
            Some(match entry.ok()?.split_once(&format!("/{prefix}_")) {
              Some((prop_id, node_id)) => {
                if *prop_id < *from || *prop_id > *to {
                  return None;
                }
                Ok(node_id.to_string())
              },
              None => { return None; },
            })
          })
          .collect::<Result<Vec<_>, Error<E>>>()?.into_iter()
      }
      All | Only(_) => iter
        .collect::<Result<Vec<_>, Error<E>>>()?.into_iter()
    };

    Ok(iter)
  }
}

#[derive(Error, Debug)]
pub enum Error<E: Send> {
  #[error("wrongly formatted database: {0}")]
  MalformedDB(String),
  #[error("node {0} allready exists")]
  NodeExists(String),
  #[error("the element existed before")]
  ExistedBefore,
  #[error("wrongly formatted input: {0}")]
  MalformedInput(#[from] std::string::FromUtf8Error),
  #[error("uuid parsing error (corrupted db)")]
  Uuid { #[from] source: uuid::Error },
  #[error("problem with kv store")]
  KV(E),
  #[error(transparent)]
  Prop(#[from] SerialisationError),
}

#[derive(Error, Debug)]
pub enum SerialisationError {
  #[error("json error")]
  Json { #[from] source: serde_json::Error },
}

impl<P, K, E> PropertyGraphReader<VertexId, HashId, HashId, P, Error<E>> for KvGraphStore<P, K, E>
where
  P: Property<HashId, SerialisationError>,
  K: KVStore<E>,
  E: Send,
{
  /// List nodes
  fn nodes(&self, filter: PropertyFilter<HashId>) -> Result<impl Iterator<Item=VertexId>, Error<E>> {
    Ok(self.filter_by_property("nodes", filter)?
        .map(|id| Ok(Uuid(uuid::Uuid::parse_str(&id)?)))
        .collect::<Result<Vec<_>, Error<E>>>()?.into_iter())
  }

  /// List edges
  fn edges(&self, filter: PropertyFilter<HashId>) -> Result<impl Iterator<Item=HashId>, Error<E>> {
    self.filter_by_property("edges", filter)
  }

  /// List properties
  fn properties(&self, filter: PropertyFilter<HashId>) -> Result<impl Iterator<Item=HashId>, Error<E>> {
    self.filter_by_property("props", filter)
  }
}

impl<P, K, E> GraphStore<VertexId, NodeData, HashId, EdgeData, HashId, P, Error<E>> for KvGraphStore<P, K, E>
where
  P: Property<HashId, SerialisationError>,
  K: KVStore<E>,
  E: Send,
{
  fn create_node(&mut self, id: VertexId, properties: &P) -> Result<VertexId, Error<E>> {
    let props_hash = self.create_property(properties)?;
    let node = NodeData {
      id,
      properties: props_hash.clone(),
      incoming: BTreeSet::new(),
      outgoing: BTreeSet::new(),
    };
    let key = node.get_key();
    let node = node.serialize()?;

    let path = "nodes/".to_string() + &key;

    if self.kv.exists(path.as_bytes()).map_err(|e| Error::KV(e))? {
      return Err(Error::NodeExists(path));
    };

    self.kv.store_record(&path.as_bytes(), &node).map_err(|e| Error::KV(e))?;

    self.create_idx_backlink(&props_hash, &key, BacklinkType::Node)?;

    Ok(id)
  }

  fn read_node(&self, id: VertexId) -> Result<NodeData, Error<E>> {
    let path = "nodes/".to_string() + &id.to_key();

    let data = self.kv.fetch_record(path.as_bytes()).map_err(|e| Error::KV(e))?;
    let node: NodeData = NodeData::deserialize(&data)?;
    Ok(node)
  }

  fn update_node(&mut self, id: VertexId, properties: &P) -> Result<VertexId, Error<E>> {
    let props_hash = self.create_property(properties)?;
    let path = "nodes/".to_string() + &id.to_key();
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
    let key = id.to_key();
    let node = node.serialize()?;
    self.kv.store_record(&path.as_bytes(), &node).map_err(|e| Error::KV(e))?;

    self.create_idx_backlink(&props_hash, &key, BacklinkType::Node)?;

    let last_reference = self.delete_property_backlink(&old_properties, &key, BacklinkType::Node)?;
    if last_reference {
      self.delete_property(&old_properties)?;
    }

    Ok(id)
  }

  fn delete_node(&mut self, id: VertexId) -> Result<VertexId, Error<E>> {
    let NodeData {
      id,
      properties,
      incoming: _,
      outgoing: _,
    } = self.read_node(id)?;

    let key = id.to_key();
    let path = "nodes/".to_string() + &key;

    let last_reference = self.delete_property_backlink(&properties, &key, BacklinkType::Node)?;
    if last_reference {
      self.delete_property(&properties)?;
    }

    self.kv.delete_record(path.as_bytes()).map_err(|e| Error::KV(e))?;
    Ok(id)
  }

  fn create_edge(&mut self, n1: VertexId, n2: VertexId, properties: &P) -> Result<HashId, Error<E>> {
    let props_hash = self.create_property(properties)?;
    let edge = EdgeData {
      n1,
      n2,
      properties: props_hash.clone(),
    };

    let hash = edge.get_key();
    let path = "edges/".to_string() + &hash;

    let edge = edge.serialize()?;
    self.kv.store_record(&path.as_bytes(), &edge).map_err(|e| Error::KV(e))?;

    self.create_idx_backlink(&props_hash, &hash, BacklinkType::Edge)?;

    let path = "nodes/".to_string() + &n1.to_key();
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
    let node = node.serialize()?;
    self.kv.store_record(&path.as_bytes(), &node).map_err(|e| Error::KV(e))?;

    let path = "nodes/".to_string() + &n2.to_key();
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
    let node = node.serialize()?;
    self.kv.store_record(&path.as_bytes(), &node).map_err(|e| Error::KV(e))?;

    Ok(hash)
  }

  fn read_edge(&self, id: &HashId) -> Result<EdgeData, Error<E>> {
    let path = "edges/".to_string() + id;

    let data = self.kv.fetch_record(path.as_bytes()).map_err(|e| Error::KV(e))?;
    let edge = EdgeData::deserialize(&data)?;
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

    let path = "nodes/".to_string() + &n1.to_key();
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
    let node = node.serialize()?;
    self.kv.store_record(&path.as_bytes(), &node).map_err(|e| Error::KV(e))?;

    let path = "nodes/".to_string() + &n2.to_key();
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
    let node = node.serialize()?;
    self.kv.store_record(&path.as_bytes(), &node).map_err(|e| Error::KV(e))?;

    let last_reference = self.delete_property_backlink(&props_hash, &id, BacklinkType::Edge)?;
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
          self.create_idx_backlink(&nested_hash, &hash, BacklinkType::Property)?;
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

  fn read_property(&self, id: &HashId) -> Result<P, Error<E>> {
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
      let last_reference = self.delete_property_backlink(&nested_hash, id, BacklinkType::Property)?;
      if last_reference {
        self.delete_property(&nested_hash)?;
      }
    }

    self.kv.delete_record(path.as_bytes()).map_err(|e| Error::KV(e))?;
    Ok(())
  }
}

impl<N, P, K, E> GraphBuilder<N, P, Error<E>> for KvGraphStore<P, K, E>
where
  N: Node<P>,
  P: Property<HashId, SerialisationError>,
  K: KVStore<E>,
  E: Send,
{
  fn add_node(&mut self, node: N) -> Result<(), Error<E>> {
    let p = node.properties();
    self.create_node(node.id(), &p)?;
    Ok(())
  }

  fn add_edge(&mut self, n1: &N, n2: &N, p: &P) -> Result<(), Error<E>> {
    self.create_edge(n1.id(), n2.id(), p)?;
    Ok(())
  }

  fn remove_node(&mut self, node: &N) -> Result<(), Error<E>> {
    self.delete_node(node.id())?;
    Ok(())
  }

  fn remove_edge(&mut self, n1: &N, n2: &N, p: &P) -> Result<(), Error<E>> {
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

#[derive(Deserialize, Serialize, Debug)]
pub struct NodeData {
  /// Unique identifier of the node in the graph.
  pub id: VertexId,
  /// The key of the dataset that represents the properties of the node.
  pub properties: HashId,
  /// A set of hashes representing incoming connections (edges) to this
  /// node.
  pub incoming: BTreeSet<HashId>,
  /// A set of hashes representing outgoing connections (edges) from
  /// this node.
  pub outgoing: BTreeSet<HashId>,
}

impl NodeData
{
  fn get_key(&self) -> String {
    self.id.to_key()
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
  pub n1: VertexId,
  pub n2: VertexId,
}

impl EdgeData
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

pub struct Change {
  pub created: ChangeSet,
  pub modified: BTreeSet<NodeChange>,
  pub deleted: ChangeSet,
  pub depends_on: BTreeSet<HashId>,
}

pub struct NodeChange {
  pub id: VertexId,
  pub properties: HashId,
}

pub struct ChangeSet {
  pub nodes: BTreeSet<NodeChange>,
  pub edges: BTreeSet<EdgeData>,
  //pub properties: BTreeSet<Property>,
}

pub fn to_query(data: &Vec<u8>) -> Result<BasicQuery, SerialisationError> {
  // TODO Verschiedene Query Sprachen über zweiten Parameter
  // TODO Internes Schema verwenden um Abfragen zu verbessern
  let query = serde_json::from_slice(data)?;

  Ok(query)
}

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
