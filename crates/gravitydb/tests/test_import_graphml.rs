use gravitydb::{kv_graph_store, mem_kv_store, ql};
use gravitydb::import::graphml::{create_graphml_importer, PropType, PropertyTypes};
use gravitydb::export::graphml::create_graphml_exporter;
use gravitydb_derive::Schema;
use serde::{Serialize, Deserialize};
use quick_xml::de::from_str;
use pretty_assertions::assert_eq;

#[test]
fn test_import_simple_graphml() {
  let kv = mem_kv_store::MemoryKvStore::default();
  let mut g: kv_graph_store::KvGraphStore<SimpleSchema, _, _> = kv_graph_store::KvGraphStore::from_kv(kv);

  let graphml_data = r#"
      <graph>
          <node id="1"><Label>Node 1</Label></node>
          <node id="2"><Label>Node 2</Label></node>
          <edge source="1" target="2"><Label>Edge from Node 1 to Node 2</Label></edge>
      </graph>
  "#;

  let xml_reader = quick_xml::reader::Reader::from_str(graphml_data);

  let importer = create_graphml_importer().property_mapper(|input: &str| {
    let prop: SimpleSchema = from_str(input).expect("could not export");
    prop
  });
  importer.import(&mut g, xml_reader).expect("could not import");

  let q = ql::VertexQuery::all();
  let result = g.query(q).unwrap();

  let mut actual = g.extract_properties(&result).unwrap();
  actual.sort();
  assert_eq!(actual, vec![SimpleSchema::Label("Node 1".to_string()), SimpleSchema::Label("Node 2".to_string())]);

  let q = ql::EdgeQuery::all();
  let result = g.query(q).unwrap();

  let actual = g.extract_properties(&result).unwrap();
  assert_eq!(actual, vec![SimpleSchema::Label("Edge from Node 1 to Node 2".to_string())]);
}

#[test]
fn test_import_export_simple_roundtrip() {
  // --- Build a graph identical to the one the importer test reads ---
  let kv = mem_kv_store::MemoryKvStore::default();
  let mut g: kv_graph_store::KvGraphStore<SimpleSchema, _, _> =
    kv_graph_store::KvGraphStore::from_kv(kv);

  let graphml_data = r#"
      <graph>
          <node id="1"><Label>Node 1</Label></node>
          <node id="2"><Label>Node 2</Label></node>
          <edge source="1" target="2"><Label>Edge from Node 1 to Node 2</Label></edge>
      </graph>
  "#;

  let xml_reader = quick_xml::reader::Reader::from_str(graphml_data);
  let importer = create_graphml_importer().property_mapper(|input: &str| {
    let prop: SimpleSchema = from_str(input).expect("could not deserialize");
    prop
  });
  importer.import(&mut g, xml_reader).expect("could not import");

  // --- Export it back to GraphML ---
  let exporter = create_graphml_exporter()
    .property_mapper(|prop| quick_xml::se::to_string(prop).expect("could not serialize"));
  let output = exporter.export(&g).expect("could not export");

  // --- Re-import the exported GraphML into a fresh store ---
  let kv = mem_kv_store::MemoryKvStore::default();
  let mut g: kv_graph_store::KvGraphStore<SimpleSchema, _, _> =
    kv_graph_store::KvGraphStore::from_kv(kv);

  let xml_reader = quick_xml::reader::Reader::from_str(&output);
  importer
    .import(&mut g, xml_reader)
    .expect("could not re-import exported graphml");

  // --- Assert nodes round-tripped correctly ---
  let q = ql::VertexQuery::all();
  let result = g.query(q).unwrap();
  let mut actual_nodes = g.extract_properties(&result).unwrap();
  actual_nodes.sort();
  assert_eq!(
    actual_nodes,
    vec![
      SimpleSchema::Label("Node 1".to_string()),
      SimpleSchema::Label("Node 2".to_string()),
    ]
  );

  // --- Assert edges round-tripped correctly ---
  let q = ql::EdgeQuery::all();
  let result = g.query(q).unwrap();
  let actual_edges = g.extract_properties(&result).unwrap();
  assert_eq!(
    actual_edges,
    vec![SimpleSchema::Label("Edge from Node 1 to Node 2".to_string())]
  );

  // TODO test that uuids are preserved from the first graph db

  // compare formatting
  //println!("{output}");
  //assert_eq!(graphml_data, output);
}

#[test]
fn test_import_export_with_attrs_roundtrip() {
  let graphml_data = include_str!("assets/code_deps.graphml");
  let kv = mem_kv_store::MemoryKvStore::default();
  let mut g: kv_graph_store::KvGraphStore<CodeDepsSchema, _, _> =
    kv_graph_store::KvGraphStore::from_kv(kv);

  #[derive(Deserialize)]
  struct Attributes {
    #[serde(rename = "data")]
    data: Vec<Data>,
  }

  #[derive(Deserialize)]
  struct Data {
    #[serde(rename = "@key")]
    key: String,
    #[serde(rename = "$text")]
    value: String,
  }

  let xml_reader = quick_xml::reader::Reader::from_str(graphml_data);
  let importer = create_graphml_importer().property_mapper_with_context(|input: &str, ptypes: &PropertyTypes| {
    let attributes: Attributes = from_str(&format!("<attributes>{input}</attributes>"))
      .expect("could not deserialize GraphML attributes");

    let mut name = None;
    let mut language = None;
    let mut is_stable = None;
    let mut relation = None;
    let mut weight = None;

    for Data { key, value } in attributes.data {
      match (key.as_str(), ptypes.get(&key)) {
        ("name", Some(PropType::String)) => name = Some(value),
        ("language", Some(PropType::String)) => language = Some(value),
        ("stable", Some(PropType::Boolean)) => {
          is_stable = Some(value.parse().expect("stable must be boolean"));
        }
        ("relation", Some(PropType::String)) => relation = Some(value),
        ("weight", Some(PropType::Integer)) => {
          weight = Some(value.parse::<u32>().expect("weight must be an integer"));
        }
        (_, None) => panic!("undeclared GraphML key `{key}`"),
        _ => panic!("unexpected key/type combination for `{key}`"),
      }
    }

    if let Some(name) = name {
      CodeDepsSchema::Module {
        name,
        language: language.expect("module missing language"),
        is_stable: is_stable.expect("module missing stable"),
      }
    } else {
      let weight = weight.expect("edge missing weight");

      match relation.as_deref() {
        Some("depends_on") => CodeDepsSchema::DependsOn { weight },
        Some("tests") => CodeDepsSchema::Tests { weight },
        Some(other) => panic!("unsupported edge relation `{other}`"),
        None => panic!("edge missing relation"),
      }
    }
  });
  importer.import(&mut g, xml_reader).expect("could not import");

  let exporter = create_graphml_exporter()
    .property_mapper_with_context(|prop, ptypes| match prop {
      CodeDepsSchema::Module { name, language, is_stable } => {
        ptypes.insert("name".to_string(), PropType::String);
        ptypes.insert("language".to_string(), PropType::String);
        ptypes.insert("stable".to_string(), PropType::Boolean);
        format!(
          r#"<data key="name">{name}</data><data key="language">{language}</data><data key="stable">{is_stable}</data>"#
        )
      }
      CodeDepsSchema::DependsOn { weight } => {
        ptypes.insert("relation".to_string(), PropType::String);
        ptypes.insert("weight".to_string(), PropType::Integer);
        format!(r#"<data key="relation">depends_on</data><data key="weight">{weight}</data>"#)
      }
      CodeDepsSchema::Tests { weight } => {
        ptypes.insert("relation".to_string(), PropType::String);
        ptypes.insert("weight".to_string(), PropType::Integer);
        format!(r#"<data key="relation">tests</data><data key="weight">{weight}</data>"#)
      }
      CodeDepsSchema::SchemaType(_) => String::new(),
    });
  let output = exporter.export(&g).expect("could not export");

  let kv = mem_kv_store::MemoryKvStore::default();
  let mut g: kv_graph_store::KvGraphStore<CodeDepsSchema, _, _> =
    kv_graph_store::KvGraphStore::from_kv(kv);

  let xml_reader = quick_xml::reader::Reader::from_str(&output);
  importer
    .import(&mut g, xml_reader)
    .expect("could not re-import exported graphml");

  let q = ql::VertexQuery::all();
  let result = g.query(q).unwrap();
  let mut actual_nodes = g.extract_properties(&result).unwrap();
  actual_nodes.sort();
  assert_eq!(
    actual_nodes,
    vec![
      CodeDepsSchema::Module {
        name: "Task App".to_string(),
        language: "Python".to_string(),
        is_stable: true,
      },
      CodeDepsSchema::Module {
        name: "Test Suite".to_string(),
        language: "Python".to_string(),
        is_stable: true,
      },
      CodeDepsSchema::Module {
        name: "Tiny Cache".to_string(),
        language: "Python".to_string(),
        is_stable: false,
      },
      CodeDepsSchema::Module {
        name: "Weather API Client".to_string(),
        language: "Python".to_string(),
        is_stable: true,
      },
    ]
  );

  let q = ql::EdgeQuery::all();
  let result = g.query(q).unwrap();
  let mut actual_edges = g.extract_properties(&result).unwrap();
  actual_edges.sort();
  assert_eq!(
    actual_edges,
    vec![
      CodeDepsSchema::DependsOn {
        weight: 5,
      },
      CodeDepsSchema::DependsOn {
        weight: 10,
      },
      CodeDepsSchema::Tests {
        weight: 10,
      },
    ]
  );
}

#[derive(Schema)]
#[derive(Eq, PartialOrd, Ord)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SimpleSchema {
  Label(String),
  SchemaType(String),
}

impl gravitydb::schema::JsonSchemaProperty for SimpleSchema {}

#[derive(Schema)]
#[derive(Eq, PartialOrd, Ord)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
enum CodeDepsSchema {
  Module {
    name: String,
    language: String,
    is_stable: bool,
  },
  DependsOn {
    weight: u32,
  },
  Tests {
    weight: u32,
  },
  SchemaType(String),
}

impl gravitydb::schema::JsonSchemaProperty for CodeDepsSchema {}

