// This test looks for a derive macro with the right name to exist.
use gravitydb_derive::Schema;

#[derive(Schema)]
pub enum BasicPimSchema {
  Person{ name: String, surname: String},
  Email(String),
  Organisation(String),
  // edge types
  BelongsTo,
  SchemaType(String),
}

fn main() {}
