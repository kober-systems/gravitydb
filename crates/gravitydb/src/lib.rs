pub mod ql;
pub mod schema;
pub mod kv_graph_store;
pub mod mem_kv_store;
#[cfg(feature="lua")]
pub mod lua;
#[cfg(feature="derive")]
pub use gravitydb_derive::Schema;

trait GraphFilter<GIN, GOUT>
{
  fn filter(&mut self, graph: GIN) -> GOUT;
}

pub trait Graph<'a, N: 'a, E> {
  type NodeIterator: Iterator<Item=&'a N>;
  type NeighborIterator: Iterator<Item=&'a N>;
  type EdgeIterator: Iterator<Item=(&'a N, &'a N)>;

  /// Returns true if there are no nodes, or false otherwise.
  fn is_empty(&self) -> bool {
    self.order() == 0
  }

  /// Returns the number of nodes in this graph.
  fn order(&self) -> usize;

  /// Returns the number of edges in this graph.
  fn size(&self) -> usize;

  /// Iterates the nodes of this graph
  fn nodes(&'a self) -> Self::NodeIterator;

  /// Returns true if node is a member, or false otherwise.
  fn has_node(&self, node: &N) -> bool;

  /// Iterates the neighbors of node.
  fn neighbors(&'a self, node: &N) -> Result<Self::NeighborIterator, E>;

  /// Returns the number of neighbors connected to node.
  fn degree(&self, node: &N) -> Result<usize, E>;

  /// Iterates the edges of this graph.
  fn edges(&'a self) -> Self::EdgeIterator;

  /// Returns true if an edge exists between source and target.
  fn has_edge(&self, source: &N, target: &N) -> Result<bool, E>;
}

pub trait DirectedGraph<'a, N: 'a, E>: Graph<'a, N, E> {
  type OutIterator: Iterator<Item = &'a N>;
  type InIterator: Iterator<Item = &'a N>;

  /// Iterates the outgoing neighbors of node.
  fn outgoing(&'a self, node: &N) -> Result<Self::OutIterator, E>;

  /// Iterates the incoming neighbors of node.
  fn incoming(&'a self, node: &N) -> Result<Self::InIterator, E>;
}

pub trait WeightedGraph<'a, N:'a, P, E> : Graph<'a, N, E> {
  /// Returns the weight between source and target.
  fn weight(&self, source: &'a N, target: &'a N) -> Result<Option<&P>, E>;
}

pub trait GraphBuilder<N, P, E> {
  /// Add a new node to the graph
  fn add_node(&mut self, node: N) -> Result<(), E>;
  /// Add an edge to the graph
  ///
  /// Edges are expected to have properties. If an Implementation
  /// does not have them it should use ().
  fn add_edge(&mut self, n1: &N, n2: &N, p: &P) -> Result<(), E>;
  fn remove_node(&mut self, node: &N) -> Result<(), E>;
  fn remove_edge(&mut self, n1: &N, n2: &N, p: &P) -> Result<(), E>;
}

/// Filter an iterator depending on the connected properties
pub enum PropertyFilter<PropKey> {
  /// Get only the elements connected with this property
  Only(PropKey),
  /// Get all the elements connected with any property in between (and
  /// including) the start and end properties
  FromTo(PropKey, PropKey),
  /// Get all elements (don't filter by property)
  All,
}

/// Low level reading and traversing interface for a graph
pub trait PropertyGraphReader<NodeKey, Node, EdgeKey, Edge, PropKey, Prop, E> {
  /// List nodes
  fn nodes(&self, filter: PropertyFilter<PropKey>) -> Result<impl Iterator<Item=NodeKey>, E>;
  /// List edges
  fn edges(&self, filter: PropertyFilter<PropKey>) -> Result<impl Iterator<Item=EdgeKey>, E>;
  /// List properties
  fn properties(&self, filter: PropertyFilter<PropKey>) -> Result<impl Iterator<Item=PropKey>, E>;

  /// Get a node by its key
  fn read_node(&self, id: NodeKey) -> Result<Node, E>;
  /// Get an edge by its key
  fn read_edge(&self, id: &EdgeKey) -> Result<Edge, E>;
  /// Get a property by its key
  fn read_property(&self, id: &PropKey) -> Result<Prop, E>;

  // Query functions
  //       TODO these functions should have a default implementation
  //fn query(&self, q: BasicQuery) -> Result<QueryResult, E>;
}

/// The Interface for a Graph DB
pub trait GraphStore<NodeKey, EdgeKey, PropKey, Prop, E> {
  // CRUD functions
  fn create_node(&mut self, id: NodeKey, properties: &Prop) -> Result<NodeKey, E>;
  fn update_node(&mut self, id: NodeKey, properties: &Prop) -> Result<NodeKey, E>;
  fn delete_node(&mut self, id: NodeKey) -> Result<NodeKey, E>;
  fn create_edge(&mut self, n1: NodeKey, n2: NodeKey, properties: &Prop) -> Result<EdgeKey, E>;
  fn delete_edge(&mut self, id: &EdgeKey) -> Result<(), E>;
  fn create_property(&mut self, properties: &Prop) -> Result<PropKey, E>;
  fn delete_property(&mut self, id: &PropKey) -> Result<(), E>;
}

/// A Key-Value Store Backend Interface.
///
/// Any Type that implements this interface can be used to run a graph
/// database.
pub trait KVStore<E> {
  /// creates a new bucket
  fn create_bucket(&mut self, key: &[u8]) -> Result<(), E>;
  /// delete a data record (could also be a bucket)
  fn delete_record(&mut self, key: &[u8]) -> Result<(), E>;
  /// list all records in between `from` and `to`
  ///
  /// if `to` is empty it searches everything till the first change of `from`
  fn list_records(&self, from: &[u8], to: &[u8]) -> Result<Vec<Vec<u8>>, E>;
  /// store a data record
  fn store_record(&mut self, key: &[u8], value: &[u8]) -> Result<(), E>;
  /// fetch a data record
  fn fetch_record(&self, key: &[u8]) -> Result<Vec<u8>, E>;
  /// check if an entry exists in the database
  fn exists(&self, key: &[u8]) -> Result<bool, E>;
}
