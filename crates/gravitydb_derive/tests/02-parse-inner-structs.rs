// This test looks for a derive macro with the right name to exist.
use gravitydb_derive::Schema;
use gravitydb::schema::SchemaElement;

#[derive(Schema)]
pub enum BasicPimSchema {
  Person{ name: String, surname: String},
  Email(String),
  Organisation(String),
  // edge types
  BelongsTo,
  SchemaType(String),
}

impl<Error> SchemaElement<String, Error> for BasicPimSchema {
  fn get_key(&self) -> String {
    format!("empty stub")
  }

  fn serialize(&self) -> Result<Vec<u8>, Error> {
    Ok(vec![])
  }

  fn deserialize(_data: &[u8]) -> Result<Self, Error>
  where
    Self: Sized,
  {
    Ok(BasicPimSchema::BelongsTo)
  }
}

fn main() {}
