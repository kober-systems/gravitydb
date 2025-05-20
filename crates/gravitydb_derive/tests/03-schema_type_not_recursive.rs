// Make sure that the schema is not endlesly nested. SchemaType should return an empty
// vec.
use gravitydb_derive::Schema;

#[derive(Schema)]
#[derive(Debug, PartialEq)]
pub enum BasicPimSchema {
  Person{ name: String, surname: String},
  Email(String),
  Organisation(String),
  // edge types
  BelongsTo,
  SchemaType(String),
}

fn main() {
  assert_eq!(
    BasicPimSchema::Email("example@email.com".to_string()).nested(),
    vec![BasicPimSchema::SchemaType("Email".to_string())]
  );
  assert_eq!(
    BasicPimSchema::SchemaType("Person".to_string()).nested(),
    vec![]
  );
}
