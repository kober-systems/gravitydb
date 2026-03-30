use gravitydb::{kv_graph_store, mem_kv_store, ql};
use gravitydb::import::graphml::create_graphml_importer;
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

#[derive(Schema)]
#[derive(Eq, PartialOrd, Ord)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SimpleSchema {
  Label(String),
  SchemaType(String),
}

impl gravitydb::schema::JsonSchemaProperty for SimpleSchema {}

