use gravity::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[ignore]
#[test]
fn whick_cocktails_include_gin() -> Result<(), Error> {
  let graph = create_cocktail_graph()?;

  // list all cocktails, that have gin as an ingredient
  todo!();

  Ok(())
}

fn create_cocktail_graph() -> Result<GStore, Error> {
  let kv = mem_kv_store::MemoryKvStore::default();
  let mut g = kv_graph_store::KvGraphStore::from_kv(kv);

  use CocktailSchema::*;

  // ingredients
  let gin = g.create_node(Uuid::new_v4(), &Ingredient("Gin".to_string()))?;
  let wermuth = g.create_node(Uuid::new_v4(), &Ingredient("Wermuth".to_string()))?;
  let olive = g.create_node(Uuid::new_v4(), &Ingredient("Olive".to_string()))?;
  let lemmon = g.create_node(Uuid::new_v4(), &Ingredient("Lemmon".to_string()))?;
  let vodka = g.create_node(Uuid::new_v4(), &Ingredient("Vodka".to_string()))?;

  // glasses
  let cocktail_glass = g.create_node(Uuid::new_v4(), &Glass("Cocktail glass".to_string()))?;

  let martini = g.create_node(Uuid::new_v4(), &Cocktail("Martini".to_string()))?;
  g.create_edge(martini, gin, &Includes)?;
  g.create_edge(martini, wermuth, &Includes)?;
  g.create_edge(martini, cocktail_glass, &ServedIn)?;

  Ok(g)
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum CocktailSchema {
  Cocktail(String),
  Ingredient(String),
  Glass(String),
  // edge types
  Includes, // TODO how much in l,%,grammes,etc
  ServedIn,
}

type Error = kv_graph_store::Error<mem_kv_store::Error>;
type GStore = kv_graph_store::KvGraphStore::<CocktailSchema, mem_kv_store::MemoryKvStore, mem_kv_store::Error>;

use gravity::schema::{SchemaElement, Property};
use sha2::Digest;

impl<Error: From<serde_json::Error>> SchemaElement<String, Error> for CocktailSchema {
  fn get_key(&self) -> String {
    let data = serde_json::to_vec(&self).unwrap();
    format!("{:X}", sha2::Sha256::digest(&data))
  }

  fn serialize(&self) -> Result<Vec<u8>, Error> {
    Ok(serde_json::to_vec(self)?)
  }

  fn deserialize(data: &[u8]) -> Result<Self, Error>
  where
    Self: Sized,
  {
    Ok(serde_json::from_slice::<CocktailSchema>(data)?)
  }
}

impl<Error: From<serde_json::Error>> Property<String, Error> for CocktailSchema {
  fn nested(&self) -> Vec<Self> {

    match self {
      //<<get_nested_properties>>
      _ => Vec::new(),
    }
  }
}
