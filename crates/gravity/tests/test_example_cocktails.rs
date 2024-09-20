use gravity::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[ignore]
#[test]
fn which_cocktails_include_gin() -> Result<(), Error> {
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
  let apricot_brandy = g.create_node(Uuid::new_v4(), &Ingredient("apricot brandy".to_string()))?;
  let calvados = g.create_node(Uuid::new_v4(), &Ingredient("calvados".to_string()))?;
  let campari = g.create_node(Uuid::new_v4(), &Ingredient("campari".to_string()))?;
  let club_soda = g.create_node(Uuid::new_v4(), &Ingredient("club sod".to_string()))?;
  let cognac = g.create_node(Uuid::new_v4(), &Ingredient("cognac".to_string()))?;
  let cream = g.create_node(Uuid::new_v4(), &Ingredient("cream".to_string()))?;
  let creme_de_cacao = g.create_node(Uuid::new_v4(), &Ingredient("crème de cacao".to_string()))?;
  let creme_de_violette = g.create_node(Uuid::new_v4(), &Ingredient("crème de violette".to_string()))?;
  let gin = g.create_node(Uuid::new_v4(), &Ingredient("gin".to_string()))?;
  let lemon_juice = g.create_node(Uuid::new_v4(), &Ingredient("lemon juice".to_string()))?;
  let sweet_vermouth = g.create_node(Uuid::new_v4(), &Ingredient("sweet vermouth".to_string()))?;
  let triple_sec = g.create_node(Uuid::new_v4(), &Ingredient("triple sec".to_string()))?;
  let vermouth = g.create_node(Uuid::new_v4(), &Ingredient("vermouth".to_string()))?;
  let vodka = g.create_node(Uuid::new_v4(), &Ingredient("vodka".to_string()))?;
  let whiskey = g.create_node(Uuid::new_v4(), &Ingredient("whiskey".to_string()))?;
  let white_rum = g.create_node(Uuid::new_v4(), &Ingredient("white rum".to_string()))?;

  // garnishes
  let olive = g.create_node(Uuid::new_v4(), &Garnish("olive".to_string()))?;
  let lemon_twist = g.create_node(Uuid::new_v4(), &Garnish("lemon twist".to_string()))?;
  let lemon_slice = g.create_node(Uuid::new_v4(), &Garnish("lemon slice".to_string()))?;
  let orange_slice = g.create_node(Uuid::new_v4(), &Garnish("orange slice".to_string()))?;
  let nutmeg = g.create_node(Uuid::new_v4(), &Garnish("nutmeg".to_string()))?;
  let maraschino_cherry = g.create_node(Uuid::new_v4(), &Garnish("maraschino cherry".to_string()))?;

  // glasses
  let cocktail_glass = g.create_node(Uuid::new_v4(), &Glass("Cocktail glass".to_string()))?;
  let old_fashioned_glass = g.create_node(Uuid::new_v4(), &Glass("Old fashioned glass".to_string()))?;

  // coctails
  let martini = g.create_node(Uuid::new_v4(), &Cocktail("Martini".to_string()))?;
  g.create_edge(martini, gin, &Includes)?;
  g.create_edge(martini, vermouth, &Includes)?;
  g.create_edge(martini, olive, &Includes)?;
  g.create_edge(martini, cocktail_glass, &ServedIn)?;

  let alexander = g.create_node(Uuid::new_v4(), &Cocktail("Alexander".to_string()))?;
  g.create_edge(alexander, cognac, &Includes)?;
  g.create_edge(alexander, creme_de_cacao, &Includes)?;
  g.create_edge(alexander, cream, &Includes)?;
  g.create_edge(alexander, nutmeg, &Includes)?;
  g.create_edge(alexander, cocktail_glass, &ServedIn)?;

  let alexander_original = g.create_node(Uuid::new_v4(), &Cocktail("Alexander".to_string()))?;
  g.create_edge(alexander_original, gin, &Includes)?;
  g.create_edge(alexander_original, creme_de_cacao, &Includes)?;
  g.create_edge(alexander_original, cream, &Includes)?;
  g.create_edge(alexander_original, nutmeg, &Includes)?;
  g.create_edge(alexander_original, cocktail_glass, &ServedIn)?;

  let americano = g.create_node(Uuid::new_v4(), &Cocktail("Americano".to_string()))?;
  g.create_edge(americano, campari, &Includes)?;
  g.create_edge(americano, sweet_vermouth, &Includes)?;
  g.create_edge(americano, orange_slice, &Includes)?;
  g.create_edge(americano, lemon_twist, &Includes)?;
  g.create_edge(americano, old_fashioned_glass, &ServedIn)?;

let americano_sparkling = g.create_node(Uuid::new_v4(), &Cocktail("Americano sparkling version".to_string()))?;
  g.create_edge(americano_sparkling, campari, &Includes)?;
  g.create_edge(americano_sparkling, sweet_vermouth, &Includes)?;
  g.create_edge(americano_sparkling, club_soda, &Includes)?;
  g.create_edge(americano_sparkling, lemon_slice, &Includes)?;
  g.create_edge(americano_sparkling, old_fashioned_glass, &ServedIn)?;

  let angel_face = g.create_node(Uuid::new_v4(), &Cocktail("Angel face".to_string()))?;
  g.create_edge(angel_face, gin, &Includes)?;
  g.create_edge(angel_face, calvados, &Includes)?;
  g.create_edge(angel_face, apricot_brandy, &Includes)?;
  g.create_edge(angel_face, cocktail_glass, &ServedIn)?;

  let aviation = g.create_node(Uuid::new_v4(), &Cocktail("Aviation".to_string()))?;
  g.create_edge(aviation, gin, &Includes)?;
  g.create_edge(aviation, maraschino_cherry, &Includes)?;
  g.create_edge(aviation, lemon_juice, &Includes)?;
  g.create_edge(aviation, creme_de_violette, &Includes)?;
  g.create_edge(aviation, maraschino_cherry, &Includes)?;
  g.create_edge(aviation, cocktail_glass, &ServedIn)?;

  let between_the_sheets = g.create_node(Uuid::new_v4(), &Cocktail("Between the sheets".to_string()))?;
  g.create_edge(between_the_sheets, white_rum, &Includes)?;
  g.create_edge(between_the_sheets, cognac, &Includes)?;
  g.create_edge(between_the_sheets, lemon_juice, &Includes)?;
  g.create_edge(between_the_sheets, triple_sec, &Includes)?;
  g.create_edge(aviation, cocktail_glass, &ServedIn)?;


  Ok(g)
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum CocktailSchema {
  Cocktail(String),
  Ingredient(String),
  Garnish(String),
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
