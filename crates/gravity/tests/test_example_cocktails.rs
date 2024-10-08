use gravity::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[test]
fn which_cocktails_include_gin() -> Result<(), Error> {
  use CocktailSchema::*;

  let graph = create_cocktail_graph()?;

  // list all cocktails, that have gin as an ingredient
  let gin = Ingredient("gin".to_string());
  let cocktail = SchemaType("Cocktail".to_string());
  let includes = Includes;

  let q = ql::PropertyQuery::from_id(gin.id())
    .referencing_vertices()
    .ingoing()
    .intersect(ql::PropertyQuery::from_id(includes.id()).referencing_edges())
    .ingoing()
    .intersect(ql::PropertyQuery::from_id(cocktail.id()).referencing_properties().referencing_vertices());
  let result = graph.query(ql::BasicQuery::V(q))?;

  let expected = vec![
    Cocktail("Alexander".to_string()),
    Cocktail("Alexander".to_string()), // original
    Cocktail("Angel face".to_string()),
    Cocktail("Aviation".to_string()),
    Cocktail("Casino".to_string()),
    Cocktail("Clover Club".to_string()),
    Cocktail("Gin fizz".to_string()),
    Cocktail("Golden fizz".to_string()),
    Cocktail("maiden's prayer".to_string()),
    Cocktail("Martini".to_string()),
    Cocktail("Royal fizz".to_string()),
    Cocktail("Silver fizz".to_string()),
  ];

  let mut actual = result.vertices.into_iter().map(|n_id| {
    let n = graph.read_node(n_id)?;
    graph.read_property(&n.properties)
  }).collect::<Result<Vec<CocktailSchema>,_>>()?;
  actual.sort_by_key(|v| match v {
    Cocktail(value) => value.clone(),
    _ => "zzz not a cocktail!!".to_string(),
  });
  assert_eq!(actual, expected);

  Ok(())
}

fn create_cocktail_graph() -> Result<GStore, Error> {
  let kv = mem_kv_store::MemoryKvStore::default();
  let mut g = kv_graph_store::KvGraphStore::from_kv(kv);

  use CocktailSchema::*;

  // ingredients
  let apricot_brandy = g.create_node(Uuid::new_v4(), &Ingredient("apricot brandy".to_string()))?;
  let aromatic_bitters = g.create_node(Uuid::new_v4(), &Ingredient("aromatic bitters".to_string()))?;
  let brandy = g.create_node(Uuid::new_v4(), &Ingredient("brandy".to_string()))?;
  let calvados = g.create_node(Uuid::new_v4(), &Ingredient("calvados".to_string()))?;
  let campari = g.create_node(Uuid::new_v4(), &Ingredient("campari".to_string()))?;
  let soda = g.create_node(Uuid::new_v4(), &Ingredient("club soda".to_string()))?;
  let cognac = g.create_node(Uuid::new_v4(), &Ingredient("cognac".to_string()))?;
  let cream = g.create_node(Uuid::new_v4(), &Ingredient("cream".to_string()))?;
  let creme_de_cacao = g.create_node(Uuid::new_v4(), &Ingredient("crème de cacao".to_string()))?;
  let creme_de_violette = g.create_node(Uuid::new_v4(), &Ingredient("crème de violette".to_string()))?;
  let curacao = g.create_node(Uuid::new_v4(), &Ingredient("curacao".to_string()))?;
  let egg_white = g.create_node(Uuid::new_v4(), &Ingredient("egg white".to_string()))?;
  let egg_yolk = g.create_node(Uuid::new_v4(), &Ingredient("egg yolk".to_string()))?;
  let gin = g.create_node(Uuid::new_v4(), &Ingredient("gin".to_string()))?;
  let lemon_juice = g.create_node(Uuid::new_v4(), &Ingredient("lemon juice".to_string()))?;
  let lime_juice = g.create_node(Uuid::new_v4(), &Ingredient("lime juice".to_string()))?;
  let maraschino = g.create_node(Uuid::new_v4(), &Ingredient("maraschino".to_string()))?;
  let orange_bitters = g.create_node(Uuid::new_v4(), &Ingredient("orange bitters".to_string()))?;
  let raspberry_syrup = g.create_node(Uuid::new_v4(), &Ingredient("raspberry syrup".to_string()))?;
  let sugar_syrup = g.create_node(Uuid::new_v4(), &Ingredient("sugar syrup".to_string()))?;
  let superfine_sugar = g.create_node(Uuid::new_v4(), &Ingredient("superfine sugar".to_string()))?;
  let sweet_vermouth = g.create_node(Uuid::new_v4(), &Ingredient("sweet vermouth".to_string()))?;
  let triple_sec = g.create_node(Uuid::new_v4(), &Ingredient("triple sec".to_string()))?;
  let vermouth = g.create_node(Uuid::new_v4(), &Ingredient("vermouth".to_string()))?;
  let vodka = g.create_node(Uuid::new_v4(), &Ingredient("vodka".to_string()))?;
  let whiskey = g.create_node(Uuid::new_v4(), &Ingredient("whiskey".to_string()))?;
  let white_rum = g.create_node(Uuid::new_v4(), &Ingredient("white rum".to_string()))?;
  let whole_egg = g.create_node(Uuid::new_v4(), &Ingredient("whole egg".to_string()))?;

  // garnishes
  let lemon_slice = g.create_node(Uuid::new_v4(), &Garnish("lemon slice".to_string()))?;
  let lemon_twist = g.create_node(Uuid::new_v4(), &Garnish("lemon twist".to_string()))?;
  let maraschino_cherry = g.create_node(Uuid::new_v4(), &Garnish("maraschino cherry".to_string()))?;
  let nutmeg = g.create_node(Uuid::new_v4(), &Garnish("nutmeg".to_string()))?;
  let olive = g.create_node(Uuid::new_v4(), &Garnish("olive".to_string()))?;
  let orange_slice = g.create_node(Uuid::new_v4(), &Garnish("orange slice".to_string()))?;
  let orange_twist = g.create_node(Uuid::new_v4(), &Garnish("orange twist".to_string()))?;
  let orange_zest = g.create_node(Uuid::new_v4(), &Garnish("orange zest".to_string()))?;

  // glasses
  let cocktail_glass = g.create_node(Uuid::new_v4(), &Glass("Cocktail glass".to_string()))?;
  let old_fashioned_glass = g.create_node(Uuid::new_v4(), &Glass("Old fashioned glass".to_string()))?;

  // cocktails
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
  g.create_edge(americano_sparkling, soda, &Includes)?;
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
  g.create_edge(between_the_sheets, cocktail_glass, &ServedIn)?;

  let boulevardier = g.create_node(Uuid::new_v4(), &Cocktail("Boulevardier".to_string()))?;
  g.create_edge(boulevardier, whiskey, &Includes)?;
  g.create_edge(boulevardier, campari, &Includes)?;
  g.create_edge(boulevardier, vermouth, &Includes)?;
  g.create_edge(boulevardier, orange_zest, &Includes)?;
  g.create_edge(boulevardier, cocktail_glass, &ServedIn)?;

  let brandy_crusta = g.create_node(Uuid::new_v4(), &Cocktail("Brandy crusta".to_string()))?;
  g.create_edge(brandy_crusta, brandy, &Includes)?;
  g.create_edge(brandy_crusta, maraschino, &Includes)?;
  g.create_edge(brandy_crusta, curacao, &Includes)?;
  g.create_edge(brandy_crusta, lemon_juice, &Includes)?;
  g.create_edge(brandy_crusta, sugar_syrup, &Includes)?;
  g.create_edge(brandy_crusta, aromatic_bitters, &Includes)?;
  g.create_edge(brandy_crusta, orange_twist, &Includes)?;
  g.create_edge(brandy_crusta, cocktail_glass, &ServedIn)?;

  let casino = g.create_node(Uuid::new_v4(), &Cocktail("Casino".to_string()))?;
  g.create_edge(casino, gin, &Includes)?;
  g.create_edge(casino, maraschino, &Includes)?;
  g.create_edge(casino, lemon_juice, &Includes)?;
  g.create_edge(casino, orange_bitters, &Includes)?;
  g.create_edge(casino, lemon_twist, &Includes)?;
  g.create_edge(casino, maraschino_cherry, &Includes)?;
  g.create_edge(casino, cocktail_glass, &ServedIn)?;

  let clover_club = g.create_node(Uuid::new_v4(), &Cocktail("Clover Club".to_string()))?;
  g.create_edge(clover_club, gin, &Includes)?;
  g.create_edge(clover_club, lemon_juice, &Includes)?;
  g.create_edge(clover_club, raspberry_syrup, &Includes)?;
  g.create_edge(clover_club, egg_white, &Includes)?;
  g.create_edge(clover_club, cocktail_glass, &ServedIn)?;

  let daiquiri = g.create_node(Uuid::new_v4(), &Cocktail("Daiquiri".to_string()))?;
  g.create_edge(daiquiri, white_rum, &Includes)?;
  g.create_edge(daiquiri, lime_juice, &Includes)?;
  g.create_edge(daiquiri, superfine_sugar, &Includes)?;
  g.create_edge(daiquiri, cocktail_glass, &ServedIn)?;

  let gin_fizz = g.create_node(Uuid::new_v4(), &Cocktail("Gin fizz".to_string()))?;
  g.create_edge(gin_fizz, gin, &Includes)?;
  g.create_edge(gin_fizz, lemon_juice, &Includes)?;
  g.create_edge(gin_fizz, sugar_syrup, &Includes)?;
  g.create_edge(gin_fizz, soda, &Includes)?;
  g.create_edge(gin_fizz, lemon_slice, &Includes)?;
  g.create_edge(gin_fizz, old_fashioned_glass, &ServedIn)?;

  let golden_fizz = g.create_node(Uuid::new_v4(), &Cocktail("Golden fizz".to_string()))?;
  g.create_edge(golden_fizz, gin, &Includes)?;
  g.create_edge(golden_fizz, lemon_juice, &Includes)?;
  g.create_edge(golden_fizz, sugar_syrup, &Includes)?;
  g.create_edge(golden_fizz, soda, &Includes)?;
  g.create_edge(golden_fizz, gin, &Includes)?;
  g.create_edge(golden_fizz, egg_yolk, &Includes)?;
  g.create_edge(golden_fizz, lemon_slice, &Includes)?;
  g.create_edge(golden_fizz, old_fashioned_glass, &ServedIn)?;

  let maidens_prayer = g.create_node(Uuid::new_v4(), &Cocktail("maiden's prayer".to_string()))?;
  g.create_edge(maidens_prayer, gin, &Includes)?;
  g.create_edge(maidens_prayer, lemon_juice, &Includes)?;
  g.create_edge(maidens_prayer, triple_sec, &Includes)?;
  g.create_edge(maidens_prayer, cocktail_glass, &ServedIn)?;

  let martini = g.create_node(Uuid::new_v4(), &Cocktail("Martini".to_string()))?;
  g.create_edge(martini, gin, &Includes)?;
  g.create_edge(martini, vermouth, &Includes)?;
  g.create_edge(martini, olive, &Includes)?;
  g.create_edge(martini, cocktail_glass, &ServedIn)?;

  let royal_fizz = g.create_node(Uuid::new_v4(), &Cocktail("Royal fizz".to_string()))?;
  g.create_edge(royal_fizz, gin, &Includes)?;
  g.create_edge(royal_fizz, lemon_juice, &Includes)?;
  g.create_edge(royal_fizz, sugar_syrup, &Includes)?;
  g.create_edge(royal_fizz, soda, &Includes)?;
  g.create_edge(royal_fizz, gin, &Includes)?;
  g.create_edge(royal_fizz, whole_egg, &Includes)?;
  g.create_edge(royal_fizz, lemon_slice, &Includes)?;
  g.create_edge(royal_fizz, old_fashioned_glass, &ServedIn)?;

  let silver_fizz = g.create_node(Uuid::new_v4(), &Cocktail("Silver fizz".to_string()))?;
  g.create_edge(silver_fizz, gin, &Includes)?;
  g.create_edge(silver_fizz, lemon_juice, &Includes)?;
  g.create_edge(silver_fizz, sugar_syrup, &Includes)?;
  g.create_edge(silver_fizz, soda, &Includes)?;
  g.create_edge(silver_fizz, gin, &Includes)?;
  g.create_edge(silver_fizz, egg_white, &Includes)?;
  g.create_edge(silver_fizz, lemon_slice, &Includes)?;
  g.create_edge(silver_fizz, old_fashioned_glass, &ServedIn)?;


  Ok(g)
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CocktailSchema {
  Cocktail(String),
  Ingredient(String),
  Garnish(String),
  Glass(String),
  // edge types
  Includes, // TODO how much in l,%,grammes,etc
  ServedIn,
  // Meta type to describe the lables of the schema itself
  SchemaType(String),
}

impl CocktailSchema {
  pub fn id(&self) -> String {
    SchemaElement::<String, serde_json::Error>::get_key(self)
  }
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
