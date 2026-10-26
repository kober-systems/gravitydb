use std::marker::PhantomData;
use std::io;

use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use thiserror::Error;

use crate::{PropertyFilter, PropertyGraphReader};
use crate::utils::PropertyTypes;
use crate::kv_graph_store::{EdgeData, NodeData, Uuid};

pub type PropKey = String;

/// Export a property graph as GraphML.
///
/// `prop_to_graphml` returns the inner XML for one node or edge (normally
/// `<data>` elements) and registers every used GraphML key in `property_types`.
pub fn export_graphml<NodeKey, EdgeKey, Prop, E, R, FN, FP>(
  reader: &R,
  node_id_fn: FN,
  prop_to_graphml: FP,
) -> Result<String, ExportError<E>>
where
  R: PropertyGraphReader<NodeKey, NodeData, EdgeKey, EdgeData<NodeKey>, PropKey, Prop, E>,
  FN: Fn(&NodeKey) -> String,
  FP: Fn(&Prop, &mut PropertyTypes) -> String,
{
  // GraphML requires key declarations before graph content. Convert and cache
  // properties first, so the mapper can build the shared declaration context.
  let mut property_types = PropertyTypes::default();
  let node_keys: Vec<NodeKey> = reader
    .nodes(PropertyFilter::All)
    .map_err(ExportError::Graph)?
    .collect();
  let mut nodes = Vec::with_capacity(node_keys.len());
  for nk in node_keys {
    let node_id = node_id_fn(&nk);
    let node = reader.read_node(nk).map_err(ExportError::Graph)?;
    let prop = reader.read_property(&node.properties).map_err(ExportError::Graph)?;
    nodes.push((node_id, prop_to_graphml(&prop, &mut property_types)));
  }

  let edge_keys: Vec<EdgeKey> = reader
    .edges(PropertyFilter::All)
    .map_err(ExportError::Graph)?
    .collect();
  let mut edges = Vec::with_capacity(edge_keys.len());
  for (i, ek) in edge_keys.iter().enumerate() {
    let edge = reader.read_edge(ek).map_err(ExportError::Graph)?;
    let prop = reader.read_property(&edge.properties).map_err(ExportError::Graph)?;
    edges.push((
      format!("e{i}"),
      node_id_fn(&edge.n1),
      node_id_fn(&edge.n2),
      prop_to_graphml(&prop, &mut property_types),
    ));
  }

  let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
  writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

  let mut graphml_start = BytesStart::new("graphml");
  graphml_start.push_attribute(("xmlns", "http://graphml.graphdrawing.org/graphml"));
  writer.write_event(Event::Start(graphml_start))?;

  for (id, prop_type) in &property_types {
    let mut key = BytesStart::new("key");
    key.push_attribute(("id", id.as_str()));
    key.push_attribute(("for", "all"));
    key.push_attribute(("attr.name", id.as_str()));
    key.push_attribute(("attr.type", <&str>::from(prop_type)));
    writer.write_event(Event::Empty(key))?;
  }

  let mut graph_start = BytesStart::new("graph");
  graph_start.push_attribute(("id", "G"));
  graph_start.push_attribute(("edgedefault", "directed"));
  writer.write_event(Event::Start(graph_start))?;

  for (node_id, properties) in nodes {
    let mut node = BytesStart::new("node");
    node.push_attribute(("id", node_id.as_str()));
    writer.write_event(Event::Start(node))?;
    writer.write_event(Event::Text(BytesText::from_escaped(&properties)))?;
    writer.write_event(Event::End(BytesEnd::new("node")))?;
  }

  for (edge_id, source, target, properties) in edges {
    let mut edge = BytesStart::new("edge");
    edge.push_attribute(("id", edge_id.as_str()));
    edge.push_attribute(("source", source.as_str()));
    edge.push_attribute(("target", target.as_str()));
    writer.write_event(Event::Start(edge))?;
    writer.write_event(Event::Text(BytesText::from_escaped(&properties)))?;
    writer.write_event(Event::End(BytesEnd::new("edge")))?;
  }

  writer.write_event(Event::End(BytesEnd::new("graph")))?;
  writer.write_event(Event::End(BytesEnd::new("graphml")))?;

  String::from_utf8(writer.into_inner()).map_err(ExportError::Utf8)
}

/// GraphML exporter struct with builder API.
pub struct GraphML<NodeKey, Prop, FNodeKey, FProp>
where
  FNodeKey: Fn(&NodeKey) -> String,
  FProp: Fn(&Prop, &mut PropertyTypes) -> String,
{
  node_key_mapper: FNodeKey,
  prop_mapper: FProp,
  node_type: PhantomData<NodeKey>,
  prop_type: PhantomData<Prop>,
}

impl<NodeKey, Prop, FNodeKey, FProp> GraphML<NodeKey, Prop, FNodeKey, FProp>
where
  FNodeKey: Fn(&NodeKey) -> String,
  FProp: Fn(&Prop, &mut PropertyTypes) -> String,
{
  /// Configure how to serialize a node key into a string.
  pub fn node_key_mapper<NewNodeKey, FNewNodeKey>(self, mapper: FNewNodeKey) -> GraphML<NewNodeKey, Prop, FNewNodeKey, FProp>
  where
    FNewNodeKey: Fn(&NewNodeKey) -> String,
  {
    let GraphML { prop_mapper, prop_type, .. } = self;
    GraphML {
      node_key_mapper: mapper,
      prop_mapper,
      node_type: PhantomData,
      prop_type,
    }
  }

  /// Configure a simple property mapper that does not declare GraphML keys.
  pub fn property_mapper<NewProp, FNewProp>(self, mapper: FNewProp) -> GraphML<NodeKey, NewProp, FNodeKey, impl Fn(&NewProp, &mut PropertyTypes) -> String>
  where
    FNewProp: Fn(&NewProp) -> String,
  {
    let GraphML { node_key_mapper, node_type, .. } = self;
    GraphML {
      node_key_mapper,
      prop_mapper: move |prop, _| mapper(prop),
      node_type,
      prop_type: PhantomData,
    }
  }

  /// Configure a mapper that emits GraphML property XML and declares its keys.
  pub fn property_mapper_with_context<NewProp, FNewProp>(self, mapper: FNewProp) -> GraphML<NodeKey, NewProp, FNodeKey, FNewProp>
  where
    FNewProp: Fn(&NewProp, &mut PropertyTypes) -> String,
  {
    let GraphML { node_key_mapper, node_type, .. } = self;
    GraphML {
      node_key_mapper,
      prop_mapper: mapper,
      node_type,
      prop_type: PhantomData,
    }
  }

  pub fn export<EdgeKey, E, R>(&self, reader: &R) -> Result<String, ExportError<E>>
  where
    R: PropertyGraphReader<NodeKey, NodeData, EdgeKey, EdgeData<NodeKey>, PropKey, Prop, E>,
  {
    export_graphml(reader, &self.node_key_mapper, &self.prop_mapper)
  }
}

pub fn uuid_mapper(id: &Uuid) -> String {
  id.to_key()
}

fn empty_prop_mapper(_: &(), _: &mut PropertyTypes) -> String {
  String::new()
}

pub fn create_graphml_exporter() -> GraphML<Uuid, (), fn(&Uuid) -> String, fn(&(), &mut PropertyTypes) -> String> {
  GraphML {
    node_key_mapper: uuid_mapper,
    prop_mapper: empty_prop_mapper,
    node_type: PhantomData,
    prop_type: PhantomData,
  }
}

#[derive(Error, Debug)]
pub enum ExportError<E> {
  #[error("graph error")]
  Graph(E),
  #[error(transparent)]
  Utf8(std::string::FromUtf8Error),
  #[error(transparent)]
  Io(#[from] io::Error),
}
