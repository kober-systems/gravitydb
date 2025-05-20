// This test looks for a derive macro with the right name to exist.
use gravitydb_derive::Schema;
use gravitydb::schema::SchemaElement;

#[derive(Schema)]
pub enum CocktailSchema {
  Cocktail(String),
  Ingredient(String),
  Garnish(String),
  Glass(String),
  // edge types
  Includes,
  ServedIn,
  SchemaType(String),
}

impl<Error> SchemaElement<String, Error> for CocktailSchema {
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
    Ok(CocktailSchema::Includes)
  }
}

fn main() {}
