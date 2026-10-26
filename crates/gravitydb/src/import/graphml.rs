use std::{borrow::Cow, collections::HashMap, marker::PhantomData};
use thiserror::Error;
use quick_xml::{events::{BytesStart, Event}, XmlVersion};
use crate::{GraphStore, kv_graph_store::Uuid};
pub use crate::utils::{PropType, PropertyTypes};

/// Function to import GraphML with closures for mapping node keys and properties
pub fn import_graphml<'a, NodeKey, EdgeKey, PropKey, Prop, FNodeKey, FProp, GStore, E>(
  graph: &mut GStore,
  mut xml_reader: quick_xml::reader::Reader<&'a [u8]>,
  node_key_mapper: FNodeKey,
  prop_mapper: FProp,
) -> Result<HashMap<String, NodeKey>, ImportError<E>>
where
  FNodeKey: Fn(&str, &mut HashMap<String, NodeKey>) -> NodeKey,
  FProp: Fn(&str, &PropertyTypes) -> Prop,
  GStore: GraphStore<NodeKey, EdgeKey, PropKey, Prop, E>,
{
  let mut mappings = HashMap::default();
  let mut prop_types = PropertyTypes::default();

  loop {
    let start_evt = match xml_reader.read_event() {
      Ok(Event::Start(ref e)) => {
        match e.name().local_name().as_ref() {
          "node" | "edge" => Some(e.clone().into_owned()),
          "graph" | "graphml" => None,
          other => Err(ImportError::UnexpectedElement(other.to_string()))?,
        }
      },
      Ok(Event::Empty(e)) => match e.name().local_name().as_ref() {
        "key" => {
          prop_types = insert_property_type(prop_types, &e)?;
          None
        },
        other => Err(ImportError::UnexpectedElement(other.to_string()))?,
      },
      Ok(Event::Text(e)) => {
        let text = e.into_inner();
        if text.trim() != "" {
          println!("unexpected props {}", text);
        }
        None
      },
      Ok(Event::End(ref e)) => {
        match e.name().local_name().as_ref() {
          "graph" | "graphml" => None,
          other => Err(ImportError::UnexpectedElement(other.to_string()))?,
        }
      }
      Ok(Event::Eof) => break,
      Ok(Event::Decl(_)) => None,
      Ok(evt) => todo!("{:?}", evt),
      Err(e) => Err(e)?,
    };

    // read_to_end
    if let Some(start_evt) = start_evt {
      let end_evt = start_evt.to_end().into_owned();
      let text = xml_reader.read_text(end_evt.name())?;

      match start_evt.name().local_name().as_ref() {
        "node" => {
          let node_id = attribute_value(&start_evt, "id")?
            .ok_or(ImportError::MissingAttr("id"))?;
          let node_key = node_key_mapper(&node_id, &mut mappings);
          let properties = prop_mapper(&text, &prop_types);
          graph.create_node(node_key, &properties).map_err(|e| ImportError::GraphDB(e))?;
        }
        "edge" => {
          let source = attribute_value(&start_evt, "source")?
            .ok_or(ImportError::MissingAttr("source"))?;
          let target = attribute_value(&start_evt, "target")?
            .ok_or(ImportError::MissingAttr("target"))?;

          let source_key = node_key_mapper(&source, &mut mappings);
          let target_key = node_key_mapper(&target, &mut mappings);
          let properties = prop_mapper(&text, &prop_types);
          graph.create_edge(source_key, target_key, &properties).map_err(|e| ImportError::GraphDB(e))?;
        }
        other => Err(ImportError::UnexpectedElement(other.to_string()))?,
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

fn attribute_value<'a, E>(
  element: &'a BytesStart<'a>,
  name: &str,
) -> Result<Option<Cow<'a, str>>, ImportError<E>> {
  for attribute in element.attributes() {
    let attribute = attribute.map_err(quick_xml::Error::from)?;
    if attribute.key.as_ref() == name {
      return Ok(Some(attribute.normalized_value(XmlVersion::Implicit1_0)?));
    }
  }
  Ok(None)
}

/// GraphML importer struct with builder API
pub struct GraphML<NodeKey, Prop, FNodeKey, FProp>
where
  FNodeKey: Fn(&str, &mut HashMap<String, NodeKey>) -> NodeKey,
  FProp: Fn(&str, &PropertyTypes) -> Prop,
{
  node_key_mapper: FNodeKey,
  prop_mapper: FProp,
  node_type: PhantomData<NodeKey>,
}

impl<NodeKey, Prop, FNodeKey, FProp> GraphML<NodeKey, Prop, FNodeKey, FProp>
where
  FNodeKey: Fn(&str, &mut HashMap<String, NodeKey>) -> NodeKey,
  FProp: Fn(&str, &PropertyTypes) -> Prop,
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

  pub fn property_mapper<NewProp, FNewProp>(self, mapper: FNewProp) -> GraphML<NodeKey, NewProp, FNodeKey, impl Fn(&str, &PropertyTypes) -> NewProp>
  where
    FNewProp: Fn(&str) -> NewProp,
  {
    let GraphML {
      node_key_mapper,
      prop_mapper: _,
      node_type,
    } = self;
    GraphML {
      node_key_mapper,
      prop_mapper: move |prop: &str, _: &PropertyTypes| mapper(prop),
      node_type
    }
  }

  pub fn property_mapper_with_context<NewProp, FNewProp>(self, mapper: FNewProp) -> GraphML<NodeKey, NewProp, FNodeKey, FNewProp>
  where
    FNewProp: Fn(&str, &PropertyTypes) -> NewProp,
  {
    let GraphML {
      node_key_mapper,
      prop_mapper: _,
      node_type,
    } = self;
    GraphML {
      node_key_mapper,
      prop_mapper: mapper,
      node_type
    }
  }

  pub fn import<'a, PropKey, EdgeKey, GStore, E>(
    &self,
    graph: &mut GStore,
    xml_reader: quick_xml::reader::Reader<&'a [u8]>,
  ) -> Result<HashMap<String, NodeKey>, ImportError<E>>
  where
    GStore: GraphStore<NodeKey, EdgeKey, PropKey, Prop, E>,
  {
    import_graphml(graph, xml_reader, &self.node_key_mapper, &self.prop_mapper)
  }
}

pub fn create_graphml_importer() -> GraphML<Uuid, String, fn(&str, &mut HashMap<String, Uuid>) -> Uuid, fn(&str, &PropertyTypes) -> String> {
  GraphML {
    node_key_mapper: uuid_mapper,
    prop_mapper: |prop: &str, _: &PropertyTypes| string_prop_mapper(prop),
    node_type: PhantomData::<Uuid>,
  }
}

fn insert_property_type<E>(
  mut property_types: PropertyTypes,
  key: &BytesStart<'_>,
) -> Result<PropertyTypes, ImportError<E>> {
  let mut id = None;
  let mut declared_type = None;

  for attribute in key.attributes() {
    let attribute = attribute.map_err(quick_xml::Error::from)?;
    let value = attribute.normalized_value(XmlVersion::Implicit1_0)?;

    match attribute.key.as_ref() {
      "id" => id = Some(value.into_owned()),
      "attr.type" => declared_type = Some(value.into_owned()),
      _ => {}
    }
  }

  let id = id.ok_or(ImportError::MissingAttr("id"))?;

  let prop_type = match declared_type.as_deref() {
    Some("string") => PropType::String,
    Some("int" | "long" | "integer") => PropType::Integer,
    Some("float" | "double") => PropType::Double,
    Some("boolean") => PropType::Boolean,
    _ => PropType::Custom(format!("<{}/>", <BytesStart as AsRef<str>>::as_ref(key))),
  };

  property_types.insert(id, prop_type);
  Ok(property_types)
}

#[derive(Error, Debug)]
pub enum ImportError<E>
{
  #[error(transparent)]
  QuickXML(#[from] quick_xml::Error),
  #[error(transparent)]
  GraphDB(E),
  MissingAttr(&'static str),
  UnexpectedElement(String),
}

#[cfg(test)]
mod tests {
  use super::*;
  use quick_xml::{events::Event, reader::Reader};

  #[test]
  fn key_declarations_are_mapped_by_id_and_preserve_unknown_types() {
    let mut reader = Reader::from_str(
      r#"<graphml>
        <key id="title" attr.name="name" attr.type="string"/>
        <key id="count" attr.type="int"/>
        <key id="metadata" attr.type="json"/>
      </graphml>"#,
    );
    let mut property_types = PropertyTypes::new();

    loop {
      match reader.read_event().unwrap() {
        Event::Empty(key) if key.name().local_name().as_ref() == "key" => {
          property_types = insert_property_type::<()>(property_types, &key).unwrap();
        }
        Event::Eof => break,
        _ => {}
      }
    }

    assert!(matches!(property_types.get("title"), Some(PropType::String)));
    assert!(matches!(property_types.get("count"), Some(PropType::Integer)));
    assert!(matches!(
      property_types.get("metadata"),
      Some(PropType::Custom(xml)) if xml == r#"<key id="metadata" attr.type="json"/>"#
    ));
  }

  #[test]
  fn malformed_key_attributes_are_rejected() {
    let mut reader = Reader::from_str(r#"<key id="title" attr.type/>"#);
    let Event::Empty(key) = reader.read_event().unwrap() else {
      panic!("expected an empty key element");
    };

    assert!(matches!(
      insert_property_type::<()>(PropertyTypes::new(), &key),
      Err(ImportError::QuickXML(_))
    ));
  }

  #[test]
  fn key_declaration_without_an_id_is_rejected() {
    let mut reader = Reader::from_str(r#"<key attr.type="string"/>"#);
    let Event::Empty(key) = reader.read_event().unwrap() else {
      panic!("expected an empty key element");
    };

    assert!(matches!(
      insert_property_type::<()>(PropertyTypes::new(), &key),
      Err(ImportError::MissingAttr("id"))
    ));
  }
}

