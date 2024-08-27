pub mod ql;
pub mod schema;

trait GraphFilter<GIN, GOUT>
{
  fn filter(&mut self, graph: GIN) -> GOUT;
}

pub trait Graph<'a, N: 'a, E> {
  type NodeIterator: Iterator<Item=&'a N>;
  type NeighborIterator: Iterator<Item=&'a N>;
  type EdgeIterator: Iterator<Item=(&'a N, &'a N)>;

  /// Returns true if there are no nodes, or false otherwise.
  fn is_empty(&self) -> bool;

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
  fn delete_node(&mut self, node: &N) -> Result<(), E>;
  fn delete_edge(&mut self, n1: &N, n2: &N, p: &P) -> Result<(), E>;
}

pub enum BacklinkType {
  Node,
  Edge,
  Property,
}

pub trait KVStore<E> {
  type Error;
  //type KeyIterator: Iterator<Item = AsRef<[u8]>>;

  /// creates a new bucket
  fn create_bucket(&self, key: &[u8]) -> Result<(), Self::Error>;
  /// delete a data record (could also be a bucket)
  fn delete_record(&self, key: &[u8]) -> Result<(), Self::Error>;
  /// list all records and buckets inside a bucket
  //fn list_records(&self, key: Option<&[u8]>) -> Result<Self::KeyIterator, Self::Error>;
  /// store a data record
  fn store_record(&self, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;
  /// fetch a data record
  fn fetch_record(&self, key: &[u8]) -> Result<Vec<u8>, Self::Error>;
  fn exists(&self, key: &[u8]) -> Result<bool, Self::Error>;
}

pub trait GraphStore<E> {
  /// helper functions used to implement the default api
  fn create_idx_backlink(&self, props_hash: &str, id: &str, ty: BacklinkType) -> Result<(), E>;
  fn delete_property_backlink(&self, props_hash: &str, id: &str, ty: BacklinkType) -> Result<bool, E>;

  // CRUD functions
  //      TODO these functions should have a default implementation
  //fn create_node(&mut self, id: uuid::Uuid, properties: &T) -> Result<(), E>;
  //fn read_node(&self, id: uuid::Uuid) -> Result<NodeData, E>;
  //fn update_node(&mut self, id: uuid::Uuid, properties: &T) -> Result<(), E>;
  //fn delete_node(&mut self, id: uuid::Uuid) -> Result<(), E>;
  //fn create_edge(&mut self, n1: uuid::Uuid, n2: uuid::Uuid, properties: &T) -> Result<HashId, E>;
  //fn read_edge(&self, id: &HashId) -> Result<EdgeData, E>;
  //fn delete_edge(&mut self, id: &HashId) -> Result<(), E>;
  //fn create_property(&mut self, properties: &T) -> Result<HashId, E>;
  //fn read_property(&mut self, id: &HashId) -> Result<T, E>;
  //fn delete_property(&mut self, id: &HashId) -> Result<(), E>;

  // Query functions
  //       TODO these functions should have a default implementation
  //fn query(&self, q: BasicQuery) -> Result<QueryResult, E>;
}
