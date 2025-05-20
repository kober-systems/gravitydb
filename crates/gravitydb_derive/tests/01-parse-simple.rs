// This test looks for a derive macro with the right name to exist.
use gravitydb_derive::Schema;

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

fn main() {}
