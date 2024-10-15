use gravity::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[test]
fn trivial_queries() -> Result<(), Error> {
  use CocktailSchema::*;

  let graph = create_cocktail_graph()?;

  // please give me a cup of tea -> we have no tea. this is a cocktail bar
  let teacup = Glass("teacup".to_string());

  let q = teacup.start()
    .referencing_vertices();
  let result = graph.query(q)?;

  let actual = graph.extract_properties(&result)?;
  assert_eq!(actual, vec![]);

  // ok, so please give me a cocktail glass
  let cocktail_glass = Glass("Cocktail glass".to_string());
  let q = cocktail_glass.start()
    .referencing_vertices();
  let result = graph.query(q)?;

  let actual = graph.extract_properties(&result)?;
  assert_eq!(actual, vec![Glass("Cocktail glass".to_string())]);

  Ok(())
}

#[test]
fn alexander_ingredients() -> Result<(), Error> {
  use CocktailSchema::*;

  let graph = create_cocktail_graph()?;

  // so which cocktail would you like to drink?
  // <- alexander
  // -> well there are two variants of it
  let alexander = Cocktail("Alexander".to_string());

  let q = alexander.start()
    .referencing_vertices();
  let result = graph.query(q)?;

  assert_eq!(result.vertices.len(), 2);

  // what is the difference between the variants?
  let mut vertices: Vec<Uuid> = result.vertices.into_iter().collect();
  let variant_1 = vertices.pop().unwrap();
  let variant_2 = vertices.pop().unwrap();

  // well both have in common ...
  let q_ingredients_v1 = ql::VertexQuery::from_ids(vec![variant_1])
    .outgoing()
    .intersect(Includes.start().referencing_edges())
    .outgoing();
  let q_ingredients_v2 = ql::VertexQuery::from_ids(vec![variant_2])
    .outgoing()
    .intersect(Includes.start().referencing_edges())
    .outgoing();
  let q = q_ingredients_v1.clone()
    .intersect(q_ingredients_v2.clone());
  let result = graph.query(q)?;

  let mut actual = graph.extract_properties(&result)?;
  actual.sort_by_key(|v| format!("{:?}",v));
  assert_eq!(actual, vec![
    Garnish("nutmeg".to_string()),
    Ingredient("cream".to_string()),
    Ingredient("crème de cacao".to_string()),
  ]);

  // But the base is different, the original one uses gin
  // and the newer version cognac
  let q = q_ingredients_v1.clone().substract(q_ingredients_v2.clone());
  let result = graph.query(q)?;
  let mut actual_v1 = graph.extract_properties(&result)?;
  actual_v1.sort_by_key(|v| format!("{:?}",v));
  let q = q_ingredients_v2.substract(q_ingredients_v1);
  let result = graph.query(q)?;
  let mut actual_v2 = graph.extract_properties(&result)?;
  actual_v2.sort_by_key(|v| format!("{:?}",v));
  let (alexander_original, alexander) = if actual_v1 == vec![Ingredient("gin".to_string())] {
    (actual_v1, actual_v2)
  } else {
    (actual_v2, actual_v1)
  };
  assert_eq!(alexander_original, vec![Ingredient("gin".to_string())]);
  assert_eq!(alexander, vec![Ingredient("cognac".to_string())]);

  Ok(())
}

#[test]
fn which_cocktails_include_gin() -> Result<(), Error> {
  use CocktailSchema::*;

  let graph = create_cocktail_graph()?;

  // list all cocktails, that have gin as an ingredient
  let gin = Ingredient("gin".to_string());
  let cocktail = SchemaType("Cocktail".to_string());
  let includes = Includes;

  let q = gin.start()
    .referencing_vertices()
    .ingoing()
    .intersect(includes.start().referencing_edges())
    .ingoing()
    .intersect(cocktail.start().referencing_properties().referencing_vertices());
  let result = graph.query(q)?;

  let expected = vec![
    Cocktail("Alexander".to_string()),
    Cocktail("Angel face".to_string()),
    Cocktail("Aviation".to_string()),
    Cocktail("Casino".to_string()),
    Cocktail("Clover Club".to_string()),
    Cocktail("Dry Martini".to_string()),
    Cocktail("Gin fizz".to_string()),
    Cocktail("Golden fizz".to_string()),
    Cocktail("Hanky panky".to_string()),
    Cocktail("John Collins".to_string()),
    Cocktail("Martini".to_string()),
    Cocktail("Royal fizz".to_string()),
    Cocktail("Silver fizz".to_string()),
    Cocktail("maiden's prayer".to_string()),
  ];

  let mut actual = graph.extract_properties(&result)?;
  actual.sort_by_key(|v| format!("{:?}",v));
  assert_eq!(actual, expected);

  // How do you do this? How does you reasoning work?

  // I start from a known starting point and than traverse my way till I
  // find all I search
  let mut paths = graph.extract_path_properties(&result)?;
  paths.sort_by_key(|v| format!("{:?}",v));

  assert_eq!(paths, vec![
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Alexander".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Angel face".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Aviation".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Casino".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Clover Club".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Dry Martini".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Gin fizz".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Golden fizz".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Hanky panky".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("John Collins".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Martini".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Royal fizz".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Silver fizz".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("maiden's prayer".to_string())],
    ]
  );

  // But this is not the only way of reasoning. You can get to the
  // same results from different starting points. For example from the
  // cocktails ...
  let q = cocktail.start()
    .referencing_properties()
    .referencing_vertices()
    .intersect(gin.start()
      .referencing_vertices()
      .ingoing()
      .intersect(includes.start().referencing_edges())
      .ingoing()
    );
  let result = graph.query(q)?;

  let mut actual = graph.extract_properties(&result)?;
  actual.sort_by_key(|v| format!("{:?}",v));
  assert_eq!(actual, expected);

  let q = includes.start()
    .referencing_edges()
    .ingoing()
    .intersect(gin.start()
      .referencing_vertices()
      .ingoing()
      .ingoing()
    )
    .intersect(cocktail.start()
      .referencing_properties()
      .referencing_vertices()
    );
  let result = graph.query(q)?;

  let mut actual = graph.extract_properties(&result)?;
  actual.sort_by_key(|v| format!("{:?}",v));
  assert_eq!(actual, expected);

  // While this leads to the same results the reasoning is different.
  // This can become important when you start optimizing your queries
  // for performance.

  Ok(())
}

#[test]
fn cocktail_statistic() -> Result<(), Error> {
  use CocktailSchema::*;

  let graph = create_cocktail_graph()?;

  // So what is the typical cocktail?
  let cocktail = SchemaType("Cocktail".to_string());
  let includes = Includes;

  let q_all_cocktails = cocktail.start()
    .referencing_properties()
    .referencing_vertices();
  let result = graph.query(q_all_cocktails)?;

  // Let me see...
  let ingredients = result.vertices.into_iter().map(|c| {
    graph.extract_properties(&graph.query(
      ql::VertexQuery::from_ids(vec![c])
        .outgoing()
        .intersect(includes.start().referencing_edges())
        .outgoing()
    )?)
  }).collect::<Result<Vec<_>,_>>()?;
  let statistics = ingredients
    .iter()
    .map(|ingredients| {
      ingredients.into_iter().fold((0, 0, 0), |(i_cnt, g_cnt, other_cnt), ingredient| {
        match ingredient {
          CocktailSchema::Ingredient(_) => (i_cnt + 1, g_cnt, other_cnt),
          CocktailSchema::Garnish(_) => (i_cnt, g_cnt + 1, other_cnt),
          _ => (i_cnt, g_cnt, other_cnt + 1),
        }
      })
    });
  // the cocktails I know have between .. and .. ingredients and between
  // .. and .. garnishes. Other things are never put in a cocktail.
  assert_eq!(statistics.clone().map(|(cnt,_,_)| cnt).min().unwrap(), 1);
  assert_eq!(statistics.clone().map(|(cnt,_,_)| cnt).max().unwrap(), 6);
  assert_eq!(statistics.clone().map(|(_,cnt,_)| cnt).min().unwrap(), 0);
  assert_eq!(statistics.clone().map(|(_,cnt,_)| cnt).max().unwrap(), 2);
  assert_eq!(statistics.clone().map(|(_,_,cnt)| cnt).sum::<u32>(), 0);
  // The normal cocktail has .. ingrediants and .. garnishes.
  let sum = statistics.clone().fold(0, |acc, (cnt,_,_)| { acc + cnt}) as f32;
  let cnt = statistics.clone().count() as f32;
  assert_eq!(format!("{:.3}", sum / cnt), "3.455");
  let sum = statistics.clone().fold(0, |acc, (_,cnt,_)| { acc + cnt}) as f32;
  assert_eq!(format!("{:.3}", sum / cnt), "0.909");

  // The most used ingredients are ...
  use std::collections::HashMap;
  let mut statistics = ingredients
        .into_iter()
        .fold(HashMap::default(), |mut acc, ingredients| -> HashMap<String, u32> {
          for ingredient in ingredients.into_iter() {
            if let CocktailSchema::Ingredient(v)  = ingredient {
              let cnt = acc.remove(&v).unwrap_or(0);
              acc.insert(v, cnt + 1);
            }
          }

          acc
        })
        .into_iter()
        .collect::<Vec<(String, u32)>>();
  statistics.sort_by(|(key1, cnt1), (key2, cnt2)| {
    if cnt1 == cnt2 {
      key1.cmp(key2)
    } else {
      // reverse order as we want most used ingredients
      cnt2.cmp(cnt1)
    }
  });
  assert_eq!(&statistics[..5], vec![
    ("gin".to_string(), 14),
    ("lemon juice".to_string(), 11),
    ("club soda".to_string(), 6),
    ("sugar syrup".to_string(), 6),
    ("campari".to_string(), 3),
  ]);

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
  let cognac = g.create_node(Uuid::new_v4(), &Ingredient("cognac".to_string()))?;
  let cream = g.create_node(Uuid::new_v4(), &Ingredient("cream".to_string()))?;
  let creme_de_cacao = g.create_node(Uuid::new_v4(), &Ingredient("crème de cacao".to_string()))?;
  let creme_de_violette = g.create_node(Uuid::new_v4(), &Ingredient("crème de violette".to_string()))?;
  let curacao = g.create_node(Uuid::new_v4(), &Ingredient("curacao".to_string()))?;
  let egg_white = g.create_node(Uuid::new_v4(), &Ingredient("egg white".to_string()))?;
  let egg_yolk = g.create_node(Uuid::new_v4(), &Ingredient("egg yolk".to_string()))?;
  let fernet_branca = g.create_node(Uuid::new_v4(), &Ingredient("fernet branca".to_string()))?;
  let gin = g.create_node(Uuid::new_v4(), &Ingredient("gin".to_string()))?;
  let green_chartreuse = g.create_node(Uuid::new_v4(), &Ingredient("green chartreuse".to_string()))?;
  let lemon_juice = g.create_node(Uuid::new_v4(), &Ingredient("lemon juice".to_string()))?;
  let lime_juice = g.create_node(Uuid::new_v4(), &Ingredient("lime juice".to_string()))?;
  let maraschino = g.create_node(Uuid::new_v4(), &Ingredient("maraschino".to_string()))?;
  let orange_bitters = g.create_node(Uuid::new_v4(), &Ingredient("orange bitters".to_string()))?;
  let raspberry_syrup = g.create_node(Uuid::new_v4(), &Ingredient("raspberry syrup".to_string()))?;
  let soda = g.create_node(Uuid::new_v4(), &Ingredient("club soda".to_string()))?;
  let sugar_syrup = g.create_node(Uuid::new_v4(), &Ingredient("sugar syrup".to_string()))?;
  let superfine_sugar = g.create_node(Uuid::new_v4(), &Ingredient("superfine sugar".to_string()))?;
  let sweet_vermouth = g.create_node(Uuid::new_v4(), &Ingredient("sweet vermouth".to_string()))?;
  let sweet_red_vermouth = g.create_node(Uuid::new_v4(), &Ingredient("sweet red vermouth".to_string()))?;
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
  let collins_glass = g.create_node(Uuid::new_v4(), &Glass("Collins glass".to_string()))?;

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
  g.create_edge(aviation, maraschino, &Includes)?;
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

  let dry_martini = g.create_node(Uuid::new_v4(), &Cocktail("Dry Martini".to_string()))?;
  g.create_edge(dry_martini, gin, &Includes)?;
  g.create_edge(dry_martini, olive, &Includes)?;
  g.create_edge(dry_martini, cocktail_glass, &ServedIn)?;

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

  let hanky_panky = g.create_node(Uuid::new_v4(), &Cocktail("Hanky panky".to_string()))?;
  g.create_edge(hanky_panky, gin, &Includes)?;
  g.create_edge(hanky_panky, sweet_red_vermouth, &Includes)?;
  g.create_edge(hanky_panky, fernet_branca, &Includes)?;
  g.create_edge(hanky_panky, orange_zest, &Includes)?;
  g.create_edge(hanky_panky, cocktail_glass, &ServedIn)?;

  let john_collins = g.create_node(Uuid::new_v4(), &Cocktail("John Collins".to_string()))?;
  g.create_edge(john_collins, gin, &Includes)?;
  g.create_edge(john_collins, lemon_juice, &Includes)?;
  g.create_edge(john_collins, sugar_syrup, &Includes)?;
  g.create_edge(john_collins, soda, &Includes)?;
  g.create_edge(john_collins, lemon_slice, &Includes)?;
  g.create_edge(john_collins, maraschino_cherry, &Includes)?;
  g.create_edge(john_collins, collins_glass, &ServedIn)?;
  
  let last_word = g.create_node(Uuid::new_v4(), &Cocktail("Last Word".to_string()))?;
  g.create_edge(last_word, gin, &Includes)?;
  g.create_edge(last_word, green_chartreuse, &Includes)?;
  g.create_edge(last_word, maraschino, &Includes)?;
  g.create_edge(last_word, lime_juice, &Includes)?;
  g.create_edge(last_word, cocktail_glass, &ServedIn)?;
  
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

  let vodka_martini = g.create_node(Uuid::new_v4(), &Cocktail("Vodka Martini".to_string()))?;
  g.create_edge(vodka_martini, vodka, &Includes)?;
  g.create_edge(vodka_martini, vermouth, &Includes)?;
  g.create_edge(vodka_martini, olive, &Includes)?;
  g.create_edge(vodka_martini, cocktail_glass, &ServedIn)?;

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

  /// get a starting point for queries
  pub fn start(&self) -> ql::PropertyQuery<String> {
    ql::PropertyQuery::from_id(self.id())
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
    use CocktailSchema::*;

    match self {
      SchemaType(_) => vec![],
      Cocktail(_) => vec![SchemaType("Cocktail".to_string())],
      Ingredient(_) => vec![SchemaType("Ingredient".to_string())],
      Garnish(_) => vec![SchemaType("Garnish".to_string())],
      Glass(_) => vec![SchemaType("Glass".to_string())],
      Includes => vec![SchemaType("Includes".to_string())],
      ServedIn => vec![SchemaType("ServedIn".to_string())],
    }
  }
}
