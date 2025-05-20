// Should be able to add more types via attribute
use gravitydb_derive::Schema;

#[derive(Schema)]
#[derive(Debug, PartialEq)]
pub enum BasicPimSchema {
  Person{ name: String, surname: String},
  #[schema(additional_types = ("Person", "Vertex"))]
  Manager,
  Organisation(String),
  // edge types
  #[schema(additional_types = "Connection")]
  BelongsTo,
  SchemaType(String),
}

fn main() {
  assert_eq!(
    BasicPimSchema::Manager.nested(),
    vec![
      BasicPimSchema::SchemaType("Manager".to_string()),
      BasicPimSchema::SchemaType("Person".to_string()),
      BasicPimSchema::SchemaType("Vertex".to_string()),
    ]
  );
  assert_eq!(
    BasicPimSchema::BelongsTo.nested(),
    vec![
      BasicPimSchema::SchemaType("BelongsTo".to_string()),
      BasicPimSchema::SchemaType("Connection".to_string()),
    ]
  );
}
