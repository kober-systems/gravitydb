// Multiple schema-derived enums can coexist in the same module.
use gravitydb_derive::Schema;

#[derive(Schema)]
#[derive(Debug, PartialEq)]
enum NodeSchema {
  Person,
  SchemaType(String),
}

#[derive(Schema)]
#[derive(Debug, PartialEq)]
enum EdgeSchema {
  Knows,
  SchemaType(String),
}

fn main() {
  assert_eq!(NodeSchema::Person.nested(), vec![NodeSchema::SchemaType("Person".to_string())]);
  assert_eq!(EdgeSchema::Knows.nested(), vec![EdgeSchema::SchemaType("Knows".to_string())]);
}
