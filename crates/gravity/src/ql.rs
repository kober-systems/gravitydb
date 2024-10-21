use core::hash::Hash;
use std::collections::{HashMap, HashSet};
use std::convert::From;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(mlua::FromLua)]
pub enum VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>
{
  /// Query over all vertices in the database
  All,
  /// Query specific vertices
  Specific(Vec<VertexId>),
  /// Query all vertices that have a specific property
  Property(PropertyQuery<PropertyId>),
  /// Select all vertices on the outgoing side of
  /// edges.
  Out(EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>),
  /// Select all vertices on the incoming side of
  /// edges.
  In(EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>),
  //PropertyFilter(Box<VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>, PropertyQuery<PropertyId>),
  /// Create a union with all vertices in the query
  /// context (side effect).
  Union(Box<VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>, Box<VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>),
  /// Create an intersection with all vertices in
  /// the query context (side effect).
  Intersect(Box<VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>, Box<VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>),
  /// Remove all vertices in the current query from
  /// the query context (side effect).
  Substract(Box<VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>, Box<VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>),
  /// Store all vertices in the query context which
  /// are either in the current selection or in the
  /// query context but not in both (side effect).
  DisjunctiveUnion(Box<VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>, Box<VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>),
  /// Filter some vertices by function
  Filter(Box<VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>, VFilter),
  /// Store the current selected vertices in the
  /// query context (side effect).
  ///
  /// If there is allready a selection of vertices
  /// the old selection will be lost.
  Store(Box<VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>),
  //  /// Execute some arbitrary function to modify
  //  /// the query context (side effect).
  //  SideEffect(Box<VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>, Fn(VertexId, QueryContext<VertexId, EdgeId>) -> QueryContext<VertexId, EdgeId>),
}

impl<VertexId, EdgeId, PropertyId, VFilter, EFilter> VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter> {
  pub fn all() -> Self {
    VertexQuery::All
  }

  pub fn from_ids(ids: Vec<VertexId>) -> Self {
    VertexQuery::Specific(ids)
  }

  pub fn from_property(p: PropertyQuery<PropertyId>) -> Self {
    VertexQuery::Property(p)
  }

  pub fn union(self, q: VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>) -> Self {
    VertexQuery::Union(Box::new(self), Box::new(q))
  }

  pub fn intersect(self, q: VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>) -> Self {
    VertexQuery::Intersect(Box::new(self), Box::new(q))
  }

  pub fn substract(self, q: VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>) -> Self {
    VertexQuery::Substract(Box::new(self), Box::new(q))
  }

  pub fn outgoing(self) -> EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter> {
    EdgeQuery::Out(Box::new(self))
  }

  pub fn ingoing(self) -> EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter> {
    EdgeQuery::In(Box::new(self))
  }

  pub fn filter(self, filter: VFilter) -> Self {
    VertexQuery::Filter(Box::new(self), filter)
  }

  pub fn store(self) -> Self {
    VertexQuery::Store(Box::new(self))
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(mlua::FromLua)]
pub enum EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>
{
  /// Query over all edges in the database
  All,
  /// Query specific edges
  Specific(Vec<EdgeId>),
  /// Query all Edges that have a specific property
  Property(PropertyQuery<PropertyId>),
  /// Select all edges on the outgoing side of
  /// vertices.
  Out(Box<VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>),
  /// Select all edges on the incoming side of
  /// vertices.
  In(Box<VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>),
  //PropertyFilter(Box<EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>, PropertyQuery<PropertyId>),
  /// Create a union with all edges in the query
  /// context (side effect).
  Union(Box<EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>, Box<EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>),
  /// Create an intersection with all edges in
  /// the query context (side effect).
  Intersect(Box<EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>, Box<EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>),
  /// Remove all edges in the current query from
  /// the query context (side effect).
  Substract(Box<EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>, Box<EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>),
  /// Store all edges in the query context which
  /// are either in the current selection or in the
  /// query context but not in both (side effect).
  DisjunctiveUnion(Box<EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>, Box<EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>),
  /// Filter some edges by function
  Filter(Box<EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>, EFilter),
  /// Store the current selected edges in the
  /// query context (side effect).
  ///
  /// If there is allready a selection of edges
  /// the old selection will be lost.
  Store(Box<EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>),
  //  /// Execute some arbitrary function to modify
  //  /// the query context (side effect).
  //  SideEffect(Box<EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>, Fn(EdgeId, QueryContext<VertexId, EdgeId>) -> QueryContext<VertexId, EdgeId>),
}

impl<VertexId, EdgeId, PropertyId, VFilter, EFilter> EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter> {
  pub fn all() -> Self {
    EdgeQuery::All
  }

  pub fn from_ids(ids: Vec<EdgeId>) -> Self {
    EdgeQuery::Specific(ids)
  }

  pub fn from_property(p: PropertyQuery<PropertyId>) -> Self {
    EdgeQuery::Property(p)
  }

  pub fn union(self, q: EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>) -> Self {
    EdgeQuery::Union(Box::new(self), Box::new(q))
  }

  pub fn intersect(self, q: EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>) -> Self {
    EdgeQuery::Intersect(Box::new(self), Box::new(q))
  }

  pub fn substract(self, q: EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>) -> Self {
    EdgeQuery::Substract(Box::new(self), Box::new(q))
  }

  pub fn outgoing(self) -> VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter> {
    VertexQuery::Out(self)
  }

  pub fn ingoing(self) -> VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter> {
    VertexQuery::In(self)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyQuery<PropertyId> {
  /// Query a specific property
  Specific(PropertyId),
  /// All properties that use this property
  ReferencingProperties(Box<PropertyQuery<PropertyId>>),
  /// All properties that are used by this property
  ReferencedProperties(Box<PropertyQuery<PropertyId>>),
}

impl<PropertyId> PropertyQuery<PropertyId> {
  pub fn from_id(id: PropertyId) -> Self {
    PropertyQuery::Specific(id)
  }

  /// Properties, die diese Property verwenden
  pub fn referencing_properties(self) -> Self {
    PropertyQuery::ReferencingProperties(Box::new(self))
  }

  /// Properties, auf die diese Property verweist
  pub fn referenced_properties(self) -> Self {
    PropertyQuery::ReferencedProperties(Box::new(self))
  }

  pub fn referencing_vertices<
    VertexId,
    EdgeId,
    VFilter,
    EFilter,
  >(self,
  ) -> VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter> {
    VertexQuery::Property(self)
  }

  pub fn referencing_edges<
    VertexId,
    EdgeId,
    VFilter,
    EFilter,
  >(
    self,
  ) -> EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter> {
    EdgeQuery::Property(self)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BasicQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter> {
  V(VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>),
  E(EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>),
  P(PropertyQuery<PropertyId>),
}

impl<VertexId, EdgeId, PropertyId, VFilter, EFilter> From<VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>> for BasicQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter> {
  fn from(value: VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>) -> Self {
    Self::V(value)
  }
}

impl<VertexId, EdgeId, PropertyId, VFilter, EFilter> From<EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>> for BasicQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter> {
  fn from(value: EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>) -> Self {
    Self::E(value)
  }
}

impl<VertexId, EdgeId, PropertyId, VFilter, EFilter> From<PropertyQuery<PropertyId>> for BasicQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter> {
  fn from(value: PropertyQuery<PropertyId>) -> Self {
    Self::P(value)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexQueryContext<VertexId: Hash + Eq, EdgeId: Hash + Eq + Clone> {
  /// The current vertex
  pub id: VertexId,
  /// The path that led till here
  pub path: Vec<(VertexId, EdgeId)>,
  /// If the path started by an edge it
  /// set here
  pub start: Option<EdgeId>,
  /// Variables that were set in side effects
  pub variables: HashMap<String, serde_json::Value>,
  /// Vertexes stored with the store action
  pub v_store: HashSet<VertexId>,
  /// Edges stored with the store action
  pub e_store: HashSet<EdgeId>,
}

impl<VertexId: Hash + Eq, EdgeId: Hash + Eq + Clone> VertexQueryContext<VertexId, EdgeId> {
  pub fn new(id: VertexId) -> Self {
    VertexQueryContext {
      id,
      path: Vec::new(),
      start: None,
      variables: HashMap::default(),
      v_store: HashSet::default(),
      e_store: HashSet::default(),
    }
  }

  pub fn into_edge_ctx(self, id: EdgeId) -> EdgeQueryContext<VertexId, EdgeId> {
    let VertexQueryContext {
      id: vid,
      mut path,
      start,
      variables,
      v_store,
      e_store,
    } = self;

    path.push((vid, id.clone()));

    EdgeQueryContext {
      id,
      path,
      start,
      variables,
      v_store,
      e_store,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeQueryContext<VertexId: Hash + Eq, EdgeId: Hash + Eq + Clone> {
  /// The current vertex
  pub id: EdgeId,
  /// The path that led till here
  pub path: Vec<(VertexId, EdgeId)>,
  /// If the path started by an edge it
  /// set here
  pub start: Option<EdgeId>,
  /// Variables that were set in side effects
  pub variables: HashMap<String, serde_json::Value>,
  /// Vertexes stored with the store action
  pub v_store: HashSet<VertexId>,
  /// Edges stored with the store action
  pub e_store: HashSet<EdgeId>,
}

impl<VertexId: Hash + Eq, EdgeId: Hash + Eq + Clone> EdgeQueryContext<VertexId, EdgeId> {
  pub fn new(id: EdgeId) -> Self {
    EdgeQueryContext {
      id: id.clone(),
      path: Vec::new(),
      start: Some(id),
      variables: HashMap::default(),
      v_store: HashSet::default(),
      e_store: HashSet::default(),
    }
  }

  pub fn into_vertex_ctx(self, id: VertexId) -> VertexQueryContext<VertexId, EdgeId> {
    let EdgeQueryContext {
      id: _,
      path,
      start,
      variables,
      v_store,
      e_store,
    } = self;

    VertexQueryContext {
      id,
      path,
      start,
      variables,
      v_store,
      e_store,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellFilter {
  pub interpreter: String,
  pub script: String,
}

#[cfg(feature="lua")]
pub fn init_lua<VertexId, EdgeId, PropertyId, VFilter, EFilter>(lua: &mlua::Lua) -> mlua::Result<()>
where
  for<'lua> VertexId:   Clone + 'lua + mlua::FromLua<'lua>,
  for<'lua> EdgeId:     Clone + 'lua + mlua::FromLua<'lua>,
  for<'lua> PropertyId: Clone + 'lua + mlua::FromLua<'lua>,
  VFilter:    Clone + 'static,
  EFilter:    Clone + 'static,
{
  let globals = lua.globals();
  globals.set("vq_all", lua.create_function(|_, ()| {
    Ok(VertexQuery::<VertexId, EdgeId, PropertyId, VFilter, EFilter>::all())
  })?)?;
  globals.set("vq_from_ids", lua.create_function(|_, ids: Vec<VertexId>| {
    Ok(VertexQuery::<VertexId, EdgeId, PropertyId, VFilter, EFilter>::from_ids(ids))
  })?)?;
  globals.set("vq_from_property", lua.create_function(|_, p: LuaPropertyQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>| {
    Ok(VertexQuery::<VertexId, EdgeId, PropertyId, VFilter, EFilter>::from_property(p.q))
  })?)?;

  globals.set("eq_all", lua.create_function(|_, ()| {
    Ok(EdgeQuery::<VertexId, EdgeId, PropertyId, VFilter, EFilter>::all())
  })?)?;
  globals.set("eq_from_ids", lua.create_function(|_, ids: Vec<EdgeId>| {
    Ok(EdgeQuery::<VertexId, EdgeId, PropertyId, VFilter, EFilter>::from_ids(ids))
  })?)?;
  globals.set("eq_from_property", lua.create_function(|_, p: LuaPropertyQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>| {
    Ok(EdgeQuery::<VertexId, EdgeId, PropertyId, VFilter, EFilter>::from_property(p.q))
  })?)?;
  globals.set("pq_from_id", lua.create_function(|_, id: PropertyId| {
    Ok(LuaPropertyQuery::<VertexId, EdgeId, PropertyId, VFilter, EFilter> {
      q: PropertyQuery::from_id(id),
      marker: std::marker::PhantomData,
    })
  })?)?;

  Ok(())
}

#[cfg(feature="lua")]
impl<VertexId, EdgeId, PropertyId, VFilter, EFilter> mlua::UserData for VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>
where
  VertexId:   Clone + 'static,
  EdgeId:     Clone + 'static,
  PropertyId: Clone + 'static,
  VFilter:    Clone + 'static,
  EFilter:    Clone + 'static,
{
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    //methods.add_method("union", |_, this, q2: VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>| {
    //  Ok(this.clone().union(q2))
    //});

    methods.add_function("union", |_, queries: (Self, Self)| {
      let (q1, q2) = queries;
      Ok(q1.union(q2))
    });
    methods.add_function("intersect", |_, queries: (Self, Self)| {
      let (q1, q2) = queries;
      Ok(q1.intersect(q2))
    });
    methods.add_function("substract", |_, queries: (Self, Self)| {
      let (q1, q2) = queries;
      Ok(q1.substract(q2))
    });
  }
}

#[cfg(feature="lua")]
impl<VertexId, EdgeId, PropertyId, VFilter, EFilter> mlua::UserData for EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>
where
  for<'lua> VertexId:   Clone + 'lua + mlua::FromLua<'lua>,
  for<'lua> EdgeId:     Clone + 'lua + mlua::FromLua<'lua>,
  for<'lua> PropertyId: Clone + 'lua + mlua::FromLua<'lua>,
  VFilter:    Clone + 'static,
  EFilter:    Clone + 'static,
{
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_function("union", |_, queries: (Self, Self)| {
      let (q1, q2) = queries;
      Ok(q1.union(q2))
    });
    methods.add_function("intersect", |_, queries: (Self, Self)| {
      let (q1, q2) = queries;
      Ok(q1.intersect(q2))
    });
    methods.add_function("substract", |_, queries: (Self, Self)| {
      let (q1, q2) = queries;
      Ok(q1.substract(q2))
    });
  }
}

#[cfg(feature="lua")]
#[derive(Clone, mlua::FromLua)]
struct LuaPropertyQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter> {
  q: PropertyQuery<PropertyId>,
  marker: std::marker::PhantomData<VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>,
}

#[cfg(feature="lua")]
impl<VertexId, EdgeId, PropertyId, VFilter, EFilter> LuaPropertyQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter> {
  fn from_property_query(q: PropertyQuery<PropertyId>) -> Self {
    LuaPropertyQuery {
      q,
      marker: std::marker::PhantomData,
    }
  }
}

#[cfg(feature="lua")]
impl<VertexId, EdgeId, PropertyId, VFilter, EFilter> mlua::UserData for LuaPropertyQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>
where
  for<'lua> VertexId:   Clone + 'lua + mlua::FromLua<'lua>,
  for<'lua> EdgeId:     Clone + 'lua + mlua::FromLua<'lua>,
  for<'lua> PropertyId: Clone + 'lua + mlua::FromLua<'lua>,
  VFilter:    Clone + 'static,
  EFilter:    Clone + 'static,
{
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_function("referencing_properties", |_, q: Self| {
      let q = q.q;
      Ok(LuaPropertyQuery::<VertexId, EdgeId, PropertyId, VFilter, EFilter>::from_property_query(q.referencing_properties()))
    });
    methods.add_function("referenced_properties", |_, q: Self| {
      let q = q.q;
      Ok(LuaPropertyQuery::<VertexId, EdgeId, PropertyId, VFilter, EFilter>::from_property_query(q.referenced_properties()))
    });
    methods.add_function("referencing_vertices", |_, q: Self| {
      Ok(q.q.referencing_vertices::<VertexId, EdgeId, VFilter, EFilter>())
    });
    methods.add_function("referencing_edges", |_, q: Self| {
      Ok(q.q.referencing_edges::<VertexId, EdgeId, VFilter, EFilter>())
    });
  }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct QueryResult<VertexId: Hash + Eq, EdgeId: Hash + Eq + Clone> {
  /// All vertices matched by the query
  pub vertices: HashSet<VertexId>,
  /// All edges matched by the query
  pub edges: HashSet<EdgeId>,
  /// All Paths matched by the query
  pub paths: Vec<(Option<EdgeId>, Vec<(VertexId, EdgeId)>, Option<VertexId>)>,
  pub variables: HashMap<String, serde_json::Value>,
}

impl<VertexId: Hash + Eq, EdgeId: Hash + Eq + Clone> QueryResult<VertexId, EdgeId> {
  pub fn new() -> Self {
    QueryResult {
      vertices: HashSet::default(),
      edges: HashSet::default(),
      paths: Vec::new(),
      variables: HashMap::default(),
    }
  }
}

impl<VertexId: Hash + Eq, EdgeId: Hash + Eq + Clone> From<HashMap<VertexId, VertexQueryContext<VertexId, EdgeId>>> for QueryResult<VertexId, EdgeId> {
  fn from(mut item: HashMap<VertexId, VertexQueryContext<VertexId, EdgeId>>) -> Self {
    let QueryResult {
      mut vertices,
      mut edges,
      mut paths,
      mut variables,
    } = QueryResult::new();

    for (id,ctx) in item.drain() {
      vertices.insert(id);

      let VertexQueryContext {
        id,
        path,
        start,
        variables: ctx_vars,
        v_store,
        e_store,
      } = ctx;

      vertices.extend(v_store);
      edges.extend(e_store);
      paths.push((start, path, Some(id)));
      variables.extend(ctx_vars.into_iter());
    }

    QueryResult {
      vertices,
      edges,
      paths,
      variables,
    }
  }
}

impl<VertexId: Hash + Eq, EdgeId: Hash + Eq + Clone> From<HashMap<EdgeId, EdgeQueryContext<VertexId, EdgeId>>> for QueryResult<VertexId, EdgeId> {
  fn from(mut item: HashMap<EdgeId, EdgeQueryContext<VertexId, EdgeId>>) -> Self {
    let QueryResult {
      mut vertices,
      mut edges,
      mut paths,
      mut variables,
    } = QueryResult::new();

    for (id,ctx) in item.drain() {
      edges.insert(id);

      let EdgeQueryContext {
        id: _,
        path,
        start,
        variables: ctx_vars,
        v_store,
        e_store,
      } = ctx;

      vertices.extend(v_store);
      edges.extend(e_store);
      paths.push((start, path, None));
      variables.extend(ctx_vars.into_iter());
    }

    QueryResult {
      vertices,
      edges,
      paths,
      variables,
    }
  }
}
