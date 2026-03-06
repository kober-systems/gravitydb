use gravitydb::*;
use gravitydb::kv_graph_store::Uuid;
use gravitydb::schema::SchemaElement;
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

  impl gravitydb::schema::JsonSchemaProperty for OpenWorkShopsSchema {}

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

#[test]
fn t02_optimized_schema() {
  #[derive(Schema)]
  #[derive(Debug, PartialEq)]
  #[derive(Serialize, Deserialize)]
  pub enum OpenWorkShopsSchema {
    Person{ name: String, surname: String},
    Workshop{ name: String },
    Tool(String),
    Location{ address: String },
    Lattitude(u32),
    Longitude(u32),

    // connections
    #[schema(additional_types = Connection)]
    BelongsTo,
    #[schema(additional_types = Connection)]
    IsStaffMember,
    #[schema(additional_types = Connection)]
    LocatedAt,

    // A special type used later for queries
    SchemaType(String),
  }

  impl KeyAdressableElement<String> for OpenWorkShopsSchema {
    fn get_key(&self) -> String {
      match self {
        Lattitude(value) => {
          format!("la_{value}")
        }
        Longitude(value) => {
          format!("lo_{value}")
        }
        _ => {
          let data = serde_json::to_vec(&self).unwrap();
          format!("{:X}", sha2::Sha256::digest(&data))
        }
      }
    }
  }

  impl<Error: From<serde_json::Error>> SchemaElement<Error> for OpenWorkShopsSchema {
    fn serialize(&self) -> Result<Vec<u8>, Error> {
      Ok(serde_json::to_vec(self)?)
    }

    fn deserialize(data: &[u8]) -> Result<Self, Error>
    where
      Self: Sized,
    {
      Ok(serde_json::from_slice::<OpenWorkShopsSchema>(data)?)
    }
  }

  use sha2::Digest;

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
  let pos_la = db.create_node(Uuid::new(), &Lattitude(42)).unwrap();
  db.create_edge(wshop, pos_la, &LocatedAt).unwrap();
  let pos_lo = db.create_node(Uuid::new(), &Longitude(42)).unwrap();
  db.create_edge(wshop, pos_lo, &LocatedAt).unwrap();

  let result = db.query(Tool("Table Saw".to_string()).start()
    .referencing_vertices()
    .outgoing()
    .intersect(BelongsTo.start().referencing_edges())
    .outgoing()).unwrap();
  let result = db.extract_properties(&result).unwrap();
  assert_eq!(result, [Workshop { name: "Nick's Workspace".to_string() }]);

  let result = db.query(Lattitude(30).from_to(&Lattitude(50))
    .referencing_vertices()
    .ingoing()
    .intersect(LocatedAt.start().referencing_edges())
    .ingoing()
  ).unwrap();
  let result = db.extract_properties(&result).unwrap();
  assert_eq!(result, [Workshop { name: "Nick's Workspace".to_string() }]);
}
