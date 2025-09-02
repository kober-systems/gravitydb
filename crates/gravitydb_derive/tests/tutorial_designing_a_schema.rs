use gravitydb::*;
use gravitydb::kv_graph_store::Uuid;
use gravitydb_derive::Schema;
use pretty_assertions::assert_eq;
use serde::{Serialize, Deserialize};

#[test]
fn t01_basic_schema() {
  #[derive(Schema)]
  #[derive(Debug, PartialEq)]
  #[derive(Serialize, Deserialize)]
  pub enum OpenWorkShopsSchema {
    Person{ name: String, surname: String},
    Workshop{ name: String },
    Tool(String),

    // connections
    #[schema(additional_types = Connection)]
    BelongsTo,
    #[schema(additional_types = Connection)]
    IsStaffMember,

    // A special type used later for queries
    SchemaType(String),
  }

  use gravitydb::schema::{JsonSchemaProperty, SchemaElement};
  impl JsonSchemaProperty for OpenWorkShopsSchema {}

  impl OpenWorkShopsSchema {
    pub fn id(&self) -> String {
      SchemaElement::<String, serde_json::Error>::get_key(self)
    }

    /// get a starting point for queries
    pub fn start(&self) -> ql::PropertyQuery<String> {
      ql::PropertyQuery::from_id(self.id())
    }
  }

  let kv = mem_kv_store::MemoryKvStore::default();
  let mut db = kv_graph_store::KvGraphStore::from_kv(kv);

  use OpenWorkShopsSchema::*;

  let nick = db.create_node(Uuid::new(), &Person{ name: "Nick".to_string(), surname: "Nice".to_string() }).unwrap();
  let wshop = db.create_node(Uuid::new(), &Workshop{ name: "Nick's Workspace".to_string() }).unwrap();
  db.create_edge(wshop, nick, &BelongsTo).unwrap();

  let t1 = db.create_node(Uuid::new(), &Tool("Table Saw".to_string())).unwrap();
  db.create_edge(t1, wshop, &BelongsTo).unwrap();
  let t2 = db.create_node(Uuid::new(), &Tool("Miter Saw".to_string())).unwrap();
  db.create_edge(t2, wshop, &BelongsTo).unwrap();

  let result = db.query(Tool("Table Saw".to_string()).start()
    .referencing_vertices()
    .outgoing()
    .intersect(BelongsTo.start().referencing_edges())
    .outgoing()).unwrap();
  let result = db.extract_properties(&result).unwrap();
  assert_eq!(result, [Workshop { name: "Nick's Workspace".to_string() }])
}
