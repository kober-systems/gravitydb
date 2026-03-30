use std::{collections::HashMap, marker::PhantomData};
use thiserror::Error;
use quick_xml::{encoding::EncodingError, events::Event};
use crate::{GraphStore, kv_graph_store::Uuid};

/// Function to import GraphML with closures for mapping node keys and properties
pub fn import_graphml<'a, NodeKey, EdgeKey, PropKey, Prop, FNodeKey, FProp, GStore, E>(
  graph: &mut GStore,
  mut xml_reader: quick_xml::reader::Reader<&'a [u8]>,
  node_key_mapper: FNodeKey,
  prop_mapper: FProp,
) -> Result<HashMap<String, NodeKey>, XMLError<E>>
where
  FNodeKey: Fn(&str, &mut HashMap<String, NodeKey>) -> NodeKey,
  FProp: Fn(&str) -> Prop,
  GStore: GraphStore<NodeKey, EdgeKey, PropKey, Prop, E>,
{
  let mut mappings = HashMap::default();

  loop {
    let start_evt = match xml_reader.read_event() {
      Ok(Event::Start(ref e)) => {
        match e.name().local_name().as_ref() {
          b"node"|b"edge" => Some(e.clone().into_owned()),
          b"graph" => continue,
          other => Err(XMLError::UnexpectedElement(other.to_owned()))?,
        }
      },
      Ok(Event::Text(e)) => {
        let text = e.decode()?;
        if text.trim() != "" {
          println!("unexpected props {}", text);
        }
        None
      },
      Ok(Event::End(ref e)) => {
        match e.name().local_name().as_ref() {
          b"graph" => None,
          other => Err(XMLError::UnexpectedElement(other.to_owned()))?,
        }
      }
      Ok(Event::Eof) => break,
      Ok(evt) => todo!("{:?}", evt),
      Err(e) => Err(e)?,
    };

    // read_to_end
    if let Some(start_evt) = start_evt {
      let end_evt = start_evt.to_end().into_owned();
      let text = xml_reader.read_text(end_evt.name())?;

      match start_evt.name().local_name().as_ref() {
        b"node" => {
          let node_id = start_evt.attributes()
            .filter_map(Result::ok)
            .find(|a| a.key.as_ref() == b"id")
            .map(|a| a.unescape_value())
            .ok_or(XMLError::MissingAttr("id"))?;
          let node_key = node_key_mapper(&node_id?, &mut mappings);
          let properties = prop_mapper(&text);
          graph.create_node(node_key, &properties).map_err(|e| XMLError::GraphDB(e))?;
        }
        b"edge" => {
          // Read source and target attributes
          let source = start_evt.attributes()
            .filter_map(Result::ok)
            .find(|a| a.key.as_ref() == b"source")
            .map(|a| a.unescape_value())
            .ok_or(XMLError::MissingAttr("source"))?;

          let target = start_evt.attributes()
            .filter_map(Result::ok)
            .find(|a| a.key.as_ref() == b"target")
            .map(|a| a.unescape_value())
            .ok_or(XMLError::MissingAttr("target"))?;

          let source_key = node_key_mapper(&source?, &mut mappings);
          let target_key = node_key_mapper(&target?, &mut mappings);
          let properties = prop_mapper(&text);
          graph.create_edge(source_key, target_key, &properties).map_err(|e| XMLError::GraphDB(e))?;
        }
        other => Err(XMLError::UnexpectedElement(other.to_owned()))?,
      };
    }
  }

  Ok(mappings)
}

pub fn uuid_mapper<'a>(id: &'a str, id_store: &mut HashMap<String, Uuid>) -> Uuid {
  let uuid = match id_store.get(id) {
    Some(id) => return *id,
    None => Uuid::new(),
  };
  id_store.insert(id.to_string(), uuid.clone());
  uuid
}

pub fn string_prop_mapper(prop: &str) -> String {
  prop.to_string()
}

/// GraphML importer struct with builder API
pub struct GraphML<NodeKey, Prop, FNodeKey, FProp>
where
  FNodeKey: Fn(&str, &mut HashMap<String, NodeKey>) -> NodeKey,
  FProp: Fn(&str) -> Prop,
{
  node_key_mapper: FNodeKey,
  prop_mapper: FProp,
  node_type: std::marker::PhantomData<NodeKey>,
}

impl<NodeKey, Prop, FNodeKey, FProp> GraphML<NodeKey, Prop, FNodeKey, FProp>
where
  FNodeKey: Fn(&str, &mut HashMap<String, NodeKey>) -> NodeKey,
  FProp: Fn(&str) -> Prop,
{
  pub fn node_id_mapper<NewNodeKey, FNewNodeKey>(self, mapper: FNewNodeKey) -> GraphML<NewNodeKey, Prop, FNewNodeKey, FProp>
  where
    FNewNodeKey: Fn(&str, &mut HashMap<String, NewNodeKey>) -> NewNodeKey,
  {
    let GraphML {
      node_key_mapper: _,
      prop_mapper,
      node_type: _,
    } = self;
    GraphML {
      node_key_mapper: mapper,
      prop_mapper,
      node_type: PhantomData::<NewNodeKey>
    }
  }

  pub fn property_mapper<NewProp, FNewProp>(self, mapper: FNewProp) -> GraphML<NodeKey, NewProp, FNodeKey, FNewProp>
  where
    FNewProp: Fn(&str) -> NewProp,
  {
    let GraphML {
      node_key_mapper,
      prop_mapper: _,
      node_type,
    } = self;
    GraphML { node_key_mapper, prop_mapper: mapper, node_type }
  }

  pub fn import<'a, PropKey, EdgeKey, GStore, E>(
    &self,
    graph: &mut GStore,
    xml_reader: quick_xml::reader::Reader<&'a [u8]>,
  ) -> Result<HashMap<String, NodeKey>, XMLError<E>>
  where
    GStore: GraphStore<NodeKey, EdgeKey, PropKey, Prop, E>,
  {
    import_graphml(graph, xml_reader, &self.node_key_mapper, &self.prop_mapper)
  }
}

pub fn create_graphml_importer() -> GraphML<Uuid, String, fn(&str, &mut HashMap<String, Uuid>) -> Uuid, fn(&str) -> String> {
  GraphML {
    node_key_mapper: uuid_mapper,
    prop_mapper: string_prop_mapper,
    node_type: PhantomData::<Uuid>,
  }
}

#[derive(Error, Debug)]
pub enum XMLError<E>
{
  #[error(transparent)]
  QuickXML(#[from] quick_xml::Error),
  #[error(transparent)]
  Parsing(#[from] quick_xml::DeError),
  #[error(transparent)]
  Decode(#[from] EncodingError),
  #[error(transparent)]
  GraphDB(E),
  MissingAttr(&'static str),
  UnexpectedElement(Vec<u8>),
}

