use gravitydb::*;
use gravitydb::kv_graph_store::Uuid;
use pretty_assertions::assert_eq;
use schema::NestableProperty;
use serde::{Serialize, Deserialize};

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
  let mut vertices: Vec<Uuid> = result.vertices.into_iter().map(|(id, _prop_id)| id).collect();
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
    Cocktail("Last Word".to_string()),
    Cocktail("Martini".to_string()),
    Cocktail("Monkey Gland".to_string()),
    Cocktail("Negroni".to_string()),
    Cocktail("Paradise".to_string()),
    Cocktail("Ramos gin fizz".to_string()),
    Cocktail("Royal fizz".to_string()),
    Cocktail("Silver fizz".to_string()),
    Cocktail("Tuxedo".to_string()),
    Cocktail("White lady".to_string()),
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
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Last Word".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Martini".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Monkey Gland".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Negroni".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Paradise".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Ramos gin fizz".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Royal fizz".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Silver fizz".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("Tuxedo".to_string())],
      vec![Ingredient("gin".to_string()), Includes, Cocktail("White lady".to_string())],
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
  let ingredients = result.vertices.into_iter().map(|(c, _prop_id)| {
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
  assert_eq!(statistics.clone().map(|(cnt,_,_)| cnt).max().unwrap(), 9);
  assert_eq!(statistics.clone().map(|(_,cnt,_)| cnt).min().unwrap(), 0);
  assert_eq!(statistics.clone().map(|(_,cnt,_)| cnt).max().unwrap(), 2);
  assert_eq!(statistics.clone().map(|(_,_,cnt)| cnt).sum::<u32>(), 0);
  // The normal cocktail has .. ingrediants and .. garnishes.
  let sum = statistics.clone().fold(0, |acc, (cnt,_,_)| { acc + cnt}) as f32;
  let cnt = statistics.clone().count() as f32;
  assert_eq!(format!("{:.3}", sum / cnt), "3.578");
  let sum = statistics.clone().fold(0, |acc, (_,cnt,_)| { acc + cnt}) as f32;
  assert_eq!(format!("{:.3}", sum / cnt), "0.844");

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
    ("gin".to_string(), 21),
    ("lemon juice".to_string(), 16),
    ("sugar syrup".to_string(), 8),
    ("club soda".to_string(), 7),
    ("maraschino".to_string(), 7),
  ]);

  Ok(())
}

fn create_cocktail_graph() -> Result<GStore, Error> {
  let kv = mem_kv_store::MemoryKvStore::default();
  let mut g = kv_graph_store::KvGraphStore::from_kv(kv);

  use CocktailSchema::*;

  // ingredients
  let absinthe = g.create_node(Uuid::new(), &Ingredient("absinthe".to_string()))?;
  let angostura_bitters = g.create_node(Uuid::new(), &Ingredient("angostura bitters".to_string()))?;
  let apricot_brandy = g.create_node(Uuid::new(), &Ingredient("apricot brandy".to_string()))?;
  let aromatic_bitters = g.create_node(Uuid::new(), &Ingredient("aromatic bitters".to_string()))?;
  let benedictine = g.create_node(Uuid::new(), &Ingredient("benedictine".to_string()))?;
  let black_pepper = g.create_node(Uuid::new(), &Ingredient("black pepper".to_string()))?;
  let brandy = g.create_node(Uuid::new(), &Ingredient("brandy".to_string()))?;
  let cachaca = g.create_node(Uuid::new(), &Ingredient("cachaça".to_string()))?;
  let calvados = g.create_node(Uuid::new(), &Ingredient("calvados".to_string()))?;
  let campari = g.create_node(Uuid::new(), &Ingredient("campari".to_string()))?;
  let celery_salt = g.create_node(Uuid::new(), &Ingredient("celery salt".to_string()))?;
  let coffee_liqueur = g.create_node(Uuid::new(), &Ingredient("coffee liqueur".to_string()))?;
  let cognac = g.create_node(Uuid::new(), &Ingredient("cognac".to_string()))?;
  let cream = g.create_node(Uuid::new(), &Ingredient("cream".to_string()))?;
  let creme_de_cacao = g.create_node(Uuid::new(), &Ingredient("crème de cacao".to_string()))?;
  let creme_de_menthe = g.create_node(Uuid::new(), &Ingredient("crème de cacao".to_string()))?;
  let creme_de_violette = g.create_node(Uuid::new(), &Ingredient("crème de violette".to_string()))?;
  let curacao = g.create_node(Uuid::new(), &Ingredient("curacao".to_string()))?;
  let drambuie = g.create_node(Uuid::new(), &Ingredient("drambuie".to_string()))?;
  let dry_gin = g.create_node(Uuid::new(), &Ingredient("dry gin".to_string()))?;
  let egg_white = g.create_node(Uuid::new(), &Ingredient("egg white".to_string()))?;
  let egg_yolk = g.create_node(Uuid::new(), &Ingredient("egg yolk".to_string()))?;
  let fernet_branca = g.create_node(Uuid::new(), &Ingredient("fernet branca".to_string()))?;
  let gin = g.create_node(Uuid::new(), &Ingredient("gin".to_string()))?;
  let green_chartreuse = g.create_node(Uuid::new(), &Ingredient("green chartreuse".to_string()))?;
  let grenadine = g.create_node(Uuid::new(), &Ingredient("grenadine".to_string()))?;
  let lemon_juice = g.create_node(Uuid::new(), &Ingredient("lemon juice".to_string()))?;
  let lime = g.create_node(Uuid::new(), &Ingredient("lime".to_string()))?;
  let lime_juice = g.create_node(Uuid::new(), &Ingredient("lime juice".to_string()))?;
  let maraschino = g.create_node(Uuid::new(), &Ingredient("maraschino".to_string()))?;
  let orange_bitters = g.create_node(Uuid::new(), &Ingredient("orange bitters".to_string()))?;
  let orange_flower_water = g.create_node(Uuid::new(), &Ingredient("orange flower water".to_string()))?;
  let orange_juice = g.create_node(Uuid::new(), &Ingredient("orange juice".to_string()))?;
  let peach_puree = g.create_node(Uuid::new(), &Ingredient("peach purée".to_string()))?;
  let peychauds_bitters = g.create_node(Uuid::new(), &Ingredient("peychauds bitters".to_string()))?;
  let pineapple_juice = g.create_node(Uuid::new(), &Ingredient("pineapple juice".to_string()))?;
  let port = g.create_node(Uuid::new(), &Ingredient("port".to_string()))?;
  let prosecco = g.create_node(Uuid::new(), &Ingredient("prosecco".to_string()))?;
  let raspberry_syrup = g.create_node(Uuid::new(), &Ingredient("raspberry syrup".to_string()))?;
  let rum = g.create_node(Uuid::new(), &Ingredient("rum".to_string()))?;
  let soda = g.create_node(Uuid::new(), &Ingredient("club soda".to_string()))?;
  let sugar_cane_juice = g.create_node(Uuid::new(), &Ingredient("sugar cane juice".to_string()))?;
  let sugar_cube = g.create_node(Uuid::new(), &Ingredient("sugar cube".to_string()))?;
  let sugar_syrup = g.create_node(Uuid::new(), &Ingredient("sugar syrup".to_string()))?;
  let superfine_sugar = g.create_node(Uuid::new(), &Ingredient("superfine sugar".to_string()))?;
  let sweet_vermouth = g.create_node(Uuid::new(), &Ingredient("sweet vermouth".to_string()))?;
  let sweet_red_vermouth = g.create_node(Uuid::new(), &Ingredient("sweet red vermouth".to_string()))?;
  let tabasco_sauce = g.create_node(Uuid::new(), &Ingredient("tabasco sauce".to_string()))?;
  let tomato_juice = g.create_node(Uuid::new(), &Ingredient("tomato juice".to_string()))?;
  let triple_sec = g.create_node(Uuid::new(), &Ingredient("triple sec".to_string()))?;
  let vanilla_extract = g.create_node(Uuid::new(), &Ingredient("vanilla extract".to_string()))?;
  let vermouth = g.create_node(Uuid::new(), &Ingredient("vermouth".to_string()))?;
  let vodka = g.create_node(Uuid::new(), &Ingredient("vodka".to_string()))?;
  let water = g.create_node(Uuid::new(), &Ingredient("water".to_string()))?;
  let whiskey = g.create_node(Uuid::new(), &Ingredient("whiskey".to_string()))?;
  let white_cane_sugar = g.create_node(Uuid::new(), &Ingredient("white cane sugar".to_string()))?;
  let white_creme_de_menthe = g.create_node(Uuid::new(), &Ingredient("crème de cacao".to_string()))?;
  let white_rum = g.create_node(Uuid::new(), &Ingredient("white rum".to_string()))?;
  let whole_egg = g.create_node(Uuid::new(), &Ingredient("whole egg".to_string()))?;
  let worcestershire_sauce = g.create_node(Uuid::new(), &Ingredient("worcestershire sauce".to_string()))?;

  // garnishes
  let celery = g.create_node(Uuid::new(), &Garnish("celery".to_string()))?;
  let cherry = g.create_node(Uuid::new(), &Garnish("cherry".to_string()))?;
  let lemon_slice = g.create_node(Uuid::new(), &Garnish("lemon slice".to_string()))?;
  let lemon_twist = g.create_node(Uuid::new(), &Garnish("lemon twist".to_string()))?;
  let lemon_zest = g.create_node(Uuid::new(), &Garnish("lemon zest".to_string()))?;
  let maraschino_cherry = g.create_node(Uuid::new(), &Garnish("maraschino cherry".to_string()))?;
  let mint_leave = g.create_node(Uuid::new(), &Garnish("mint leave".to_string()))?;
  let nutmeg = g.create_node(Uuid::new(), &Garnish("nutmeg".to_string()))?;
  let olive = g.create_node(Uuid::new(), &Garnish("olive".to_string()))?;
  let orange_slice = g.create_node(Uuid::new(), &Garnish("orange slice".to_string()))?;
  let orange_twist = g.create_node(Uuid::new(), &Garnish("orange twist".to_string()))?;
  let orange_zest = g.create_node(Uuid::new(), &Garnish("orange zest".to_string()))?;

  // glasses
  let beverage_glass = g.create_node(Uuid::new(), &Glass("Beverage glass".to_string()))?;
  let champagne_flute = g.create_node(Uuid::new(), &Glass("Champagne flute".to_string()))?;
  let cocktail_glass = g.create_node(Uuid::new(), &Glass("Cocktail glass".to_string()))?;
  let collins_glass = g.create_node(Uuid::new(), &Glass("Collins glass".to_string()))?;
  let highball_glass = g.create_node(Uuid::new(), &Glass("Highball glass".to_string()))?;
  let old_fashioned_glass = g.create_node(Uuid::new(), &Glass("Old fashioned glass".to_string()))?;

  // cocktails
  let alexander = g.create_node(Uuid::new(), &Cocktail("Alexander".to_string()))?;
  g.create_edge(alexander, cognac, &Includes)?;
  g.create_edge(alexander, creme_de_cacao, &Includes)?;
  g.create_edge(alexander, cream, &Includes)?;
  g.create_edge(alexander, nutmeg, &Includes)?;
  g.create_edge(alexander, cocktail_glass, &ServedIn)?;

  let alexander_original = g.create_node(Uuid::new(), &Cocktail("Alexander".to_string()))?;
  g.create_edge(alexander_original, gin, &Includes)?;
  g.create_edge(alexander_original, creme_de_cacao, &Includes)?;
  g.create_edge(alexander_original, cream, &Includes)?;
  g.create_edge(alexander_original, nutmeg, &Includes)?;
  g.create_edge(alexander_original, cocktail_glass, &ServedIn)?;

  let americano = g.create_node(Uuid::new(), &Cocktail("Americano".to_string()))?;
  g.create_edge(americano, campari, &Includes)?;
  g.create_edge(americano, sweet_vermouth, &Includes)?;
  g.create_edge(americano, orange_slice, &Includes)?;
  g.create_edge(americano, lemon_twist, &Includes)?;
  g.create_edge(americano, old_fashioned_glass, &ServedIn)?;

  let americano_sparkling = g.create_node(Uuid::new(), &Cocktail("Americano sparkling version".to_string()))?;
  g.create_edge(americano_sparkling, campari, &Includes)?;
  g.create_edge(americano_sparkling, sweet_vermouth, &Includes)?;
  g.create_edge(americano_sparkling, soda, &Includes)?;
  g.create_edge(americano_sparkling, lemon_slice, &Includes)?;
  g.create_edge(americano_sparkling, old_fashioned_glass, &ServedIn)?;

  let angel_face = g.create_node(Uuid::new(), &Cocktail("Angel face".to_string()))?;
  g.create_edge(angel_face, gin, &Includes)?;
  g.create_edge(angel_face, calvados, &Includes)?;
  g.create_edge(angel_face, apricot_brandy, &Includes)?;
  g.create_edge(angel_face, cocktail_glass, &ServedIn)?;

  let aviation = g.create_node(Uuid::new(), &Cocktail("Aviation".to_string()))?;
  g.create_edge(aviation, gin, &Includes)?;
  g.create_edge(aviation, maraschino, &Includes)?;
  g.create_edge(aviation, lemon_juice, &Includes)?;
  g.create_edge(aviation, creme_de_violette, &Includes)?;
  g.create_edge(aviation, maraschino_cherry, &Includes)?;
  g.create_edge(aviation, cocktail_glass, &ServedIn)?;

  let bellini = g.create_node(Uuid::new(), &Cocktail("Bellini".to_string()))?;
  g.create_edge(bellini, prosecco, &Includes)?;
  g.create_edge(bellini, peach_puree, &Includes)?;
  g.create_edge(bellini, champagne_flute, &ServedIn)?;

  let between_the_sheets = g.create_node(Uuid::new(), &Cocktail("Between the sheets".to_string()))?;
  g.create_edge(between_the_sheets, white_rum, &Includes)?;
  g.create_edge(between_the_sheets, cognac, &Includes)?;
  g.create_edge(between_the_sheets, lemon_juice, &Includes)?;
  g.create_edge(between_the_sheets, triple_sec, &Includes)?;
  g.create_edge(between_the_sheets, cocktail_glass, &ServedIn)?;

  let black_russian = g.create_node(Uuid::new(), &Cocktail("Black Russian".to_string()))?;
  g.create_edge(black_russian, vodka, &Includes)?;
  g.create_edge(black_russian, coffee_liqueur, &Includes)?;
  g.create_edge(black_russian, old_fashioned_glass, &ServedIn)?;

  let bloody_mary = g.create_node(Uuid::new(), &Cocktail("Bloody Mary".to_string()))?;
  g.create_edge(bloody_mary, vodka, &Includes)?;
  g.create_edge(bloody_mary, tomato_juice, &Includes)?;
  g.create_edge(bloody_mary, lemon_juice, &Includes)?;
  g.create_edge(bloody_mary, worcestershire_sauce, &Includes)?;
  g.create_edge(bloody_mary, tabasco_sauce, &Includes)?;
  g.create_edge(bloody_mary, celery_salt, &Includes)?;
  g.create_edge(bloody_mary, black_pepper, &Includes)?;
  g.create_edge(bloody_mary, celery, &Includes)?;
  g.create_edge(bloody_mary, highball_glass, &ServedIn)?;

  let boulevardier = g.create_node(Uuid::new(), &Cocktail("Boulevardier".to_string()))?;
  g.create_edge(boulevardier, whiskey, &Includes)?;
  g.create_edge(boulevardier, campari, &Includes)?;
  g.create_edge(boulevardier, vermouth, &Includes)?;
  g.create_edge(boulevardier, orange_zest, &Includes)?;
  g.create_edge(boulevardier, cocktail_glass, &ServedIn)?;

  let brandy_crusta = g.create_node(Uuid::new(), &Cocktail("Brandy crusta".to_string()))?;
  g.create_edge(brandy_crusta, brandy, &Includes)?;
  g.create_edge(brandy_crusta, maraschino, &Includes)?;
  g.create_edge(brandy_crusta, curacao, &Includes)?;
  g.create_edge(brandy_crusta, lemon_juice, &Includes)?;
  g.create_edge(brandy_crusta, sugar_syrup, &Includes)?;
  g.create_edge(brandy_crusta, aromatic_bitters, &Includes)?;
  g.create_edge(brandy_crusta, orange_twist, &Includes)?;
  g.create_edge(brandy_crusta, cocktail_glass, &ServedIn)?;

  let casino = g.create_node(Uuid::new(), &Cocktail("Casino".to_string()))?;
  g.create_edge(casino, gin, &Includes)?;
  g.create_edge(casino, maraschino, &Includes)?;
  g.create_edge(casino, lemon_juice, &Includes)?;
  g.create_edge(casino, orange_bitters, &Includes)?;
  g.create_edge(casino, lemon_twist, &Includes)?;
  g.create_edge(casino, maraschino_cherry, &Includes)?;
  g.create_edge(casino, cocktail_glass, &ServedIn)?;

  let caipirinha = g.create_node(Uuid::new(), &Cocktail("Caipirinha".to_string()))?;
  g.create_edge(caipirinha, cachaca, &Includes)?;
  g.create_edge(caipirinha, lime, &Includes)?;
  g.create_edge(caipirinha, white_cane_sugar, &Includes)?;
  g.create_edge(caipirinha, old_fashioned_glass, &ServedIn)?;

  let clover_club = g.create_node(Uuid::new(), &Cocktail("Clover Club".to_string()))?;
  g.create_edge(clover_club, gin, &Includes)?;
  g.create_edge(clover_club, lemon_juice, &Includes)?;
  g.create_edge(clover_club, raspberry_syrup, &Includes)?;
  g.create_edge(clover_club, egg_white, &Includes)?;
  g.create_edge(clover_club, cocktail_glass, &ServedIn)?;

  let daiquiri = g.create_node(Uuid::new(), &Cocktail("Daiquiri".to_string()))?;
  g.create_edge(daiquiri, white_rum, &Includes)?;
  g.create_edge(daiquiri, lime_juice, &Includes)?;
  g.create_edge(daiquiri, superfine_sugar, &Includes)?;
  g.create_edge(daiquiri, cocktail_glass, &ServedIn)?;

  let dry_martini = g.create_node(Uuid::new(), &Cocktail("Dry Martini".to_string()))?;
  g.create_edge(dry_martini, gin, &Includes)?;
  g.create_edge(dry_martini, olive, &Includes)?;
  g.create_edge(dry_martini, cocktail_glass, &ServedIn)?;

  let gin_fizz = g.create_node(Uuid::new(), &Cocktail("Gin fizz".to_string()))?;
  g.create_edge(gin_fizz, gin, &Includes)?;
  g.create_edge(gin_fizz, lemon_juice, &Includes)?;
  g.create_edge(gin_fizz, sugar_syrup, &Includes)?;
  g.create_edge(gin_fizz, soda, &Includes)?;
  g.create_edge(gin_fizz, lemon_slice, &Includes)?;
  g.create_edge(gin_fizz, old_fashioned_glass, &ServedIn)?;

  let golden_fizz = g.create_node(Uuid::new(), &Cocktail("Golden fizz".to_string()))?;
  g.create_edge(golden_fizz, gin, &Includes)?;
  g.create_edge(golden_fizz, lemon_juice, &Includes)?;
  g.create_edge(golden_fizz, sugar_syrup, &Includes)?;
  g.create_edge(golden_fizz, soda, &Includes)?;
  g.create_edge(golden_fizz, egg_yolk, &Includes)?;
  g.create_edge(golden_fizz, lemon_slice, &Includes)?;
  g.create_edge(golden_fizz, old_fashioned_glass, &ServedIn)?;

  let hanky_panky = g.create_node(Uuid::new(), &Cocktail("Hanky panky".to_string()))?;
  g.create_edge(hanky_panky, gin, &Includes)?;
  g.create_edge(hanky_panky, sweet_red_vermouth, &Includes)?;
  g.create_edge(hanky_panky, fernet_branca, &Includes)?;
  g.create_edge(hanky_panky, orange_zest, &Includes)?;
  g.create_edge(hanky_panky, cocktail_glass, &ServedIn)?;

  let john_collins = g.create_node(Uuid::new(), &Cocktail("John Collins".to_string()))?;
  g.create_edge(john_collins, gin, &Includes)?;
  g.create_edge(john_collins, lemon_juice, &Includes)?;
  g.create_edge(john_collins, sugar_syrup, &Includes)?;
  g.create_edge(john_collins, soda, &Includes)?;
  g.create_edge(john_collins, lemon_slice, &Includes)?;
  g.create_edge(john_collins, maraschino_cherry, &Includes)?;
  g.create_edge(john_collins, collins_glass, &ServedIn)?;

  let last_word = g.create_node(Uuid::new(), &Cocktail("Last Word".to_string()))?;
  g.create_edge(last_word, gin, &Includes)?;
  g.create_edge(last_word, green_chartreuse, &Includes)?;
  g.create_edge(last_word, maraschino, &Includes)?;
  g.create_edge(last_word, lime_juice, &Includes)?;
  g.create_edge(last_word, cocktail_glass, &ServedIn)?;

  let maidens_prayer = g.create_node(Uuid::new(), &Cocktail("maiden's prayer".to_string()))?;
  g.create_edge(maidens_prayer, gin, &Includes)?;
  g.create_edge(maidens_prayer, lemon_juice, &Includes)?;
  g.create_edge(maidens_prayer, triple_sec, &Includes)?;
  g.create_edge(maidens_prayer, cocktail_glass, &ServedIn)?;

  let manhattan = g.create_node(Uuid::new(), &Cocktail("Manhattan".to_string()))?;
  g.create_edge(manhattan, whiskey, &Includes)?;
  g.create_edge(manhattan, sweet_red_vermouth, &Includes)?;
  g.create_edge(manhattan, angostura_bitters, &Includes)?;
  g.create_edge(manhattan, maraschino_cherry, &Includes)?;
  g.create_edge(manhattan, cocktail_glass, &ServedIn)?;

  let martinez = g.create_node(Uuid::new(), &Cocktail("Martinez".to_string()))?;
  g.create_edge(martinez, dry_gin, &Includes)?;
  g.create_edge(martinez, sweet_red_vermouth, &Includes)?;
  g.create_edge(martinez, maraschino, &Includes)?;
  g.create_edge(martinez, orange_bitters, &Includes)?;
  g.create_edge(martinez, lemon_slice, &Includes)?;
  g.create_edge(martinez, cocktail_glass, &ServedIn)?;

  let martini = g.create_node(Uuid::new(), &Cocktail("Martini".to_string()))?;
  g.create_edge(martini, gin, &Includes)?;
  g.create_edge(martini, vermouth, &Includes)?;
  g.create_edge(martini, olive, &Includes)?;
  g.create_edge(martini, cocktail_glass, &ServedIn)?;

  let mary_pickford = g.create_node(Uuid::new(), &Cocktail("Mary Pickford".to_string()))?;
  g.create_edge(mary_pickford, white_rum, &Includes)?;
  g.create_edge(mary_pickford, pineapple_juice, &Includes)?;
  g.create_edge(mary_pickford, grenadine, &Includes)?;
  g.create_edge(mary_pickford, maraschino, &Includes)?;
  g.create_edge(mary_pickford, maraschino_cherry, &Includes)?;
  g.create_edge(mary_pickford, cocktail_glass, &ServedIn)?;

  let monkey_gland = g.create_node(Uuid::new(), &Cocktail("Monkey Gland".to_string()))?;
  g.create_edge(monkey_gland, gin, &Includes)?;
  g.create_edge(monkey_gland, orange_juice, &Includes)?;
  g.create_edge(monkey_gland, absinthe, &Includes)?;
  g.create_edge(monkey_gland, grenadine, &Includes)?;
  g.create_edge(monkey_gland, cocktail_glass, &ServedIn)?;

  let negroni = g.create_node(Uuid::new(), &Cocktail("Negroni".to_string()))?;
  g.create_edge(negroni, gin, &Includes)?;
  g.create_edge(negroni, sweet_red_vermouth, &Includes)?;
  g.create_edge(negroni, campari, &Includes)?;
  g.create_edge(negroni, orange_slice, &Includes)?;
  g.create_edge(negroni, old_fashioned_glass, &ServedIn)?;

  let old_fashioned = g.create_node(Uuid::new(), &Cocktail("Old Fashioned".to_string()))?;
  g.create_edge(old_fashioned, whiskey, &Includes)?;
  g.create_edge(old_fashioned, sugar_cube, &Includes)?;
  g.create_edge(old_fashioned, angostura_bitters, &Includes)?;
  g.create_edge(old_fashioned, water, &Includes)?;
  g.create_edge(old_fashioned, orange_slice, &Includes)?;
  g.create_edge(old_fashioned, maraschino_cherry, &Includes)?;
  g.create_edge(old_fashioned, old_fashioned_glass, &ServedIn)?;

  let paradise = g.create_node(Uuid::new(), &Cocktail("Paradise".to_string()))?;
  g.create_edge(paradise, gin, &Includes)?;
  g.create_edge(paradise, apricot_brandy, &Includes)?;
  g.create_edge(paradise, orange_juice, &Includes)?;
  g.create_edge(paradise, cocktail_glass, &ServedIn)?;

  let planters_punch = g.create_node(Uuid::new(), &Cocktail("Planter's punch".to_string()))?;
  g.create_edge(planters_punch, rum, &Includes)?;
  g.create_edge(planters_punch, lime_juice, &Includes)?;
  g.create_edge(planters_punch, sugar_cane_juice, &Includes)?;
  g.create_edge(planters_punch, orange_zest, &Includes)?;
  g.create_edge(planters_punch, beverage_glass, &ServedIn)?;

  let porto_flip = g.create_node(Uuid::new(), &Cocktail("Porto flip".to_string()))?;
  g.create_edge(porto_flip, brandy, &Includes)?;
  g.create_edge(porto_flip, port, &Includes)?;
  g.create_edge(porto_flip, egg_yolk, &Includes)?;
  g.create_edge(porto_flip, nutmeg, &Includes)?;
  g.create_edge(porto_flip, cocktail_glass, &ServedIn)?;

  let ramos_gin_fizz = g.create_node(Uuid::new(), &Cocktail("Ramos gin fizz".to_string()))?;
  g.create_edge(ramos_gin_fizz, gin, &Includes)?;
  g.create_edge(ramos_gin_fizz, lime_juice, &Includes)?;
  g.create_edge(ramos_gin_fizz, lemon_juice, &Includes)?;
  g.create_edge(ramos_gin_fizz, sugar_syrup, &Includes)?;
  g.create_edge(ramos_gin_fizz, cream, &Includes)?;
  g.create_edge(ramos_gin_fizz, orange_flower_water, &Includes)?;
  g.create_edge(ramos_gin_fizz, egg_white, &Includes)?;
  g.create_edge(ramos_gin_fizz, vanilla_extract, &Includes)?;
  g.create_edge(ramos_gin_fizz, soda, &Includes)?;
  g.create_edge(ramos_gin_fizz, collins_glass, &ServedIn)?;

  let royal_fizz = g.create_node(Uuid::new(), &Cocktail("Royal fizz".to_string()))?;
  g.create_edge(royal_fizz, gin, &Includes)?;
  g.create_edge(royal_fizz, lemon_juice, &Includes)?;
  g.create_edge(royal_fizz, sugar_syrup, &Includes)?;
  g.create_edge(royal_fizz, soda, &Includes)?;
  g.create_edge(royal_fizz, gin, &Includes)?;
  g.create_edge(royal_fizz, whole_egg, &Includes)?;
  g.create_edge(royal_fizz, lemon_slice, &Includes)?;
  g.create_edge(royal_fizz, old_fashioned_glass, &ServedIn)?;

  let rusty_nail = g.create_node(Uuid::new(), &Cocktail("Rusty nail".to_string()))?;
  g.create_edge(rusty_nail, whiskey, &Includes)?;
  g.create_edge(rusty_nail, drambuie, &Includes)?;
  g.create_edge(rusty_nail, orange_zest, &Includes)?;
  g.create_edge(rusty_nail, old_fashioned_glass, &ServedIn)?;

  let sazerac = g.create_node(Uuid::new(), &Cocktail("Sazerac".to_string()))?;
  g.create_edge(sazerac, cognac, &Includes)?;
  g.create_edge(sazerac, absinthe, &Includes)?;
  g.create_edge(sazerac, sugar_cube, &Includes)?;
  g.create_edge(sazerac, peychauds_bitters, &Includes)?;
  g.create_edge(sazerac, lemon_zest, &Includes)?;
  g.create_edge(sazerac, old_fashioned_glass, &ServedIn)?;

  let sidecar = g.create_node(Uuid::new(), &Cocktail("Sidecar".to_string()))?;
  g.create_edge(sidecar, cognac, &Includes)?;
  g.create_edge(sidecar, triple_sec, &Includes)?;
  g.create_edge(sidecar, lemon_juice, &Includes)?;
  g.create_edge(sidecar, cocktail_glass, &ServedIn)?;

  let silver_fizz = g.create_node(Uuid::new(), &Cocktail("Silver fizz".to_string()))?;
  g.create_edge(silver_fizz, gin, &Includes)?;
  g.create_edge(silver_fizz, lemon_juice, &Includes)?;
  g.create_edge(silver_fizz, sugar_syrup, &Includes)?;
  g.create_edge(silver_fizz, soda, &Includes)?;
  g.create_edge(silver_fizz, gin, &Includes)?;
  g.create_edge(silver_fizz, egg_white, &Includes)?;
  g.create_edge(silver_fizz, lemon_slice, &Includes)?;
  g.create_edge(silver_fizz, old_fashioned_glass, &ServedIn)?;

  let stinger = g.create_node(Uuid::new(), &Cocktail("Stinger".to_string()))?;
  g.create_edge(stinger, cognac, &Includes)?;
  g.create_edge(stinger, white_creme_de_menthe, &Includes)?;
  g.create_edge(stinger, mint_leave, &Includes)?;
  g.create_edge(stinger, cocktail_glass, &ServedIn)?;

  let tuxedo = g.create_node(Uuid::new(), &Cocktail("Tuxedo".to_string()))?;
  g.create_edge(tuxedo, gin, &Includes)?;
  g.create_edge(tuxedo, maraschino, &Includes)?;
  g.create_edge(tuxedo, vermouth, &Includes)?;
  g.create_edge(tuxedo, absinthe, &Includes)?;
  g.create_edge(tuxedo, orange_bitters, &Includes)?;
  g.create_edge(tuxedo, lemon_zest, &Includes)?;
  g.create_edge(tuxedo, cherry, &Includes)?;
  g.create_edge(tuxedo, cocktail_glass, &ServedIn)?;

  let vieux_carre = g.create_node(Uuid::new(), &Cocktail("Vieux Carré".to_string()))?;
  g.create_edge(vieux_carre, whiskey, &Includes)?;
  g.create_edge(vieux_carre, cognac, &Includes)?;
  g.create_edge(vieux_carre, sweet_vermouth, &Includes)?;
  g.create_edge(vieux_carre, benedictine, &Includes)?;
  g.create_edge(vieux_carre, peychauds_bitters, &Includes)?;
  g.create_edge(vieux_carre, orange_zest, &Includes)?;
  g.create_edge(vieux_carre, maraschino_cherry, &Includes)?;
  g.create_edge(vieux_carre, cocktail_glass, &ServedIn)?;

  let vodka_martini = g.create_node(Uuid::new(), &Cocktail("Vodka Martini".to_string()))?;
  g.create_edge(vodka_martini, vodka, &Includes)?;
  g.create_edge(vodka_martini, vermouth, &Includes)?;
  g.create_edge(vodka_martini, olive, &Includes)?;
  g.create_edge(vodka_martini, cocktail_glass, &ServedIn)?;

  let whiskey_sour = g.create_node(Uuid::new(), &Cocktail("Whiskey sour".to_string()))?;
  g.create_edge(whiskey_sour, whiskey, &Includes)?;
  g.create_edge(whiskey_sour, lemon_juice, &Includes)?;
  g.create_edge(whiskey_sour, sugar_syrup, &Includes)?;
  g.create_edge(whiskey_sour, maraschino_cherry, &Includes)?;
  g.create_edge(whiskey_sour, orange_slice, &Includes)?;
  g.create_edge(whiskey_sour, old_fashioned_glass, &ServedIn)?;

  let white_lady = g.create_node(Uuid::new(), &Cocktail("White lady".to_string()))?;
  g.create_edge(white_lady, gin, &Includes)?;
  g.create_edge(white_lady, lemon_juice, &Includes)?;
  g.create_edge(white_lady, triple_sec, &Includes)?;
  g.create_edge(white_lady, cocktail_glass, &ServedIn)?;

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

use gravitydb::schema::{JsonSchemaProperty, SchemaElement};
impl JsonSchemaProperty for CocktailSchema {}

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

impl NestableProperty for CocktailSchema {
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
