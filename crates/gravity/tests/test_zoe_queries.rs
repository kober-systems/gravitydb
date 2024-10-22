use gravity::*;
use pretty_assertions::assert_eq;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[ignore]
#[test]
fn union_query() -> Result<(), Error> {
  let graph = create_simple_beatles_graph()?;

  // list all albums at which ringo was singing
  todo!();

  Ok(())
}

fn create_simple_beatles_graph() -> Result<GStore, Error> {
  let kv = mem_kv_store::MemoryKvStore::default();
  let mut g = kv_graph_store::KvGraphStore::from_kv(kv);

  use BeatlesSchema::*;

  let john = g.create_node(Uuid::new_v4(), &Person("John Lennon".to_string()))?;
  let paul = g.create_node(Uuid::new_v4(), &Person("Paul Mccartney".to_string()))?;
  let george = g.create_node(Uuid::new_v4(), &Person("George Harrison".to_string()))?;
  let ringo = g.create_node(Uuid::new_v4(), &Person("Ringo Starr".to_string()))?;

  let acoustic_guitar = g.create_node(Uuid::new_v4(), &Instrument("Acoustic Guitar".to_string()))?;
  let drums = g.create_node(Uuid::new_v4(), &Instrument("Drums".to_string()))?;

  let album = g.create_node(Uuid::new_v4(), &Album("Please Please Me".to_string()))?;
  let i_saw_her_standing_there = g.create_node(Uuid::new_v4(), &Song("I Saw Her Standing There".to_string()))?;
  g.create_edge(album, i_saw_her_standing_there, &Features)?;
  let misery = g.create_node(Uuid::new_v4(), &Song("Misery".to_string()))?;
  g.create_edge(album, misery, &Features)?;
  let anna_go_to_him = g.create_node(Uuid::new_v4(), &Song("Anna (Go To Him)".to_string()))?;
  g.create_edge(album, anna_go_to_him, &Features)?;
  let chains = g.create_node(Uuid::new_v4(), &Song("Chains".to_string()))?;
  g.create_edge(album, chains, &Features)?;
  let boys = g.create_node(Uuid::new_v4(), &Song("Boys".to_string()))?;
  g.create_edge(album, boys, &Features)?;
  let ask_me_why = g.create_node(Uuid::new_v4(), &Song("Ask Me Why".to_string()))?;
  g.create_edge(album, ask_me_why, &Features)?;
  let please_please_me = g.create_node(Uuid::new_v4(), &Song("Please Please Me".to_string()))?;
  g.create_edge(album, please_please_me, &Features)?;
  let love_me_do = g.create_node(Uuid::new_v4(), &Song("Love Me Do".to_string()))?;
  g.create_edge(album, love_me_do, &Features)?;
  let ps_i_love_you = g.create_node(Uuid::new_v4(), &Song("PS I Love You".to_string()))?;
  g.create_edge(album, ps_i_love_you, &Features)?;
  let baby_its_you = g.create_node(Uuid::new_v4(), &Song("Baby It’s You".to_string()))?;
  g.create_edge(album, baby_its_you, &Features)?;
  let do_you_want_to_know_a_secret = g.create_node(Uuid::new_v4(), &Song("Do You Want To Know A Secret".to_string()))?;
  g.create_edge(album, do_you_want_to_know_a_secret, &Features)?;
  let a_taste_of_honey = g.create_node(Uuid::new_v4(), &Song("A Taste Of Honey".to_string()))?;
  g.create_edge(album, a_taste_of_honey, &Features)?;
  let theres_a_place = g.create_node(Uuid::new_v4(), &Song("There’s A Place".to_string()))?;
  g.create_edge(album, theres_a_place, &Features)?;
  let twist_and_shout = g.create_node(Uuid::new_v4(), &Song("Twist And Shout".to_string()))?;
  g.create_edge(album, twist_and_shout, &Features)?;

  let abbey_road = g.create_node(Uuid::new_v4(), &Album("Abbey Road".to_string()))?;
  let yesterday = g.create_node(Uuid::new_v4(), &Song("Yesterday".to_string()))?;
  g.create_edge(john, yesterday, &Wrote)?;
  g.create_edge(paul, yesterday, &Wrote)?;
  g.create_edge(abbey_road, yesterday, &Features)?;
  let hey_jude = g.create_node(Uuid::new_v4(), &Song("Hey Jude".to_string()))?;
  g.create_edge(paul, hey_jude, &Wrote)?;
  g.create_edge(abbey_road, hey_jude, &Features)?;
  let let_it_be = g.create_node(Uuid::new_v4(), &Song("Let It Be".to_string()))?;
  g.create_edge(paul, let_it_be, &Wrote)?;
  g.create_edge(john, let_it_be, &Sang)?;

  Ok(g)
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum BeatlesSchema {
  Person(String),
  Album(String),
  Song(String),
  Instrument(String),
  // edge types
  Wrote,
  Played,
  Sang,
  Features,
}

type Error = kv_graph_store::Error<mem_kv_store::Error>;
type GStore = kv_graph_store::KvGraphStore::<BeatlesSchema, mem_kv_store::MemoryKvStore, mem_kv_store::Error>;

use gravity::schema::{SchemaElement, Property};
use sha2::Digest;

impl<Error: From<serde_json::Error>> SchemaElement<String, Error> for BeatlesSchema {
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
    Ok(serde_json::from_slice::<BeatlesSchema>(data)?)
  }
}

impl<Error: From<serde_json::Error>> Property<String, Error> for BeatlesSchema {
  fn nested(&self) -> Vec<Self> {

    match self {
      _ => Vec::new(),
    }
  }
}
