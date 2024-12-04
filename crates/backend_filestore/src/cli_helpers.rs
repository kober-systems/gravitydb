use gravity::schema::{SchemaElement, Property};
use crate::{FileStoreError, FsKvStore};
use anyhow::bail;
use std::io::{self, Write};
use gravity::GraphStore;
use gravity::kv_graph_store::{KvGraphStore, SerialisationError, Uuid};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use std::io::Read;
use anyhow::Result;

type HashId = String;

pub fn read_input(input: Option<PathBuf>) -> Result<Vec<u8>> {
  let data = match input {
    Some(path) => std::fs::read(path)?,
    None => {
      let mut data = Vec::new();
      std::io::stdin().read_to_end(&mut data)?;
      data
    }
  };
  Ok(data)
}

pub fn log_level(level: u64) -> log::Level {
  use log::Level::*;
  match level {
    0 => Warn,
    1 => Info,
    2 => Debug,
    _ => Trace,
  }
}

pub trait Prop: Property<HashId, SerialisationError> + 'static + std::clone::Clone + mlua::UserData {}
impl <T: Property<HashId, SerialisationError> + 'static + std::clone::Clone + mlua::UserData> Prop for T {}

pub fn db_cmds<T>(init_fn: fn(&mlua::Lua) -> mlua::Result<()>) -> Result<()>
where
  for<'lua> T: Prop + 'lua + mlua::FromLua<'lua>,
{
  #[derive(StructOpt)]
  pub struct Opt {
    #[structopt(parse(from_os_str), long)]
    #[structopt(default_value = "./db")]
    db_path: PathBuf,
    #[structopt(parse(from_os_str), long, short)]
    input: Option<PathBuf>,
    #[structopt(parse(from_os_str), long, short)]
    output: Option<PathBuf>,
    #[structopt(parse(from_occurrences = log_level), short)]
    verbosity: log::Level,
    #[structopt(subcommand)]
    cmd: CmdOpts,
  }

  #[derive(StructOpt)]
  pub enum CmdOpts {
    /// create a new node
    CreateNode {
      #[structopt(long)]
      id: Option<uuid::Uuid>,
      #[structopt(long)]
      create_id: bool,
      #[structopt(short, long)]
      update: bool,
      #[structopt(short, long)]
      get_or_create: bool,
    },
    /// delete a node
    DeleteNode {
      #[structopt(long)]
      id: uuid::Uuid,
    },
    /// create a new edge
    CreateEdge {
      #[structopt(long="in")]
      n1: uuid::Uuid,
      #[structopt(long="out")]
      n2: uuid::Uuid,
    },
    /// calculate property id from content
    PropertyId,
    /// create property storage blob from content
    PropertyBlob,
    /// run a query on the database
    QueryDb,
    /// lua repl for the database
    Repl,
    /// get property data for query result
    ResultData,
    /// initialize a new database
    Init,
  }

  let opt = Opt::from_args();
  simple_logger::init_with_level(opt.verbosity)?;

  use CmdOpts::*;
  match opt.cmd {
    CreateNode {id, create_id, update, get_or_create} => {
      if update && id.is_none() {
        bail!("to update a node you need to provide an id");
      }

      if create_id && get_or_create {
        bail!("you can either for creating an id or using an existing one if possible but not both");
      }

      let properties = read_input(opt.input)?;
      let properties: T = SchemaElement::deserialize(&properties)?;
      let id = match id {
        Some(id) => id,
        None => {
          let hash = properties.get_key();
          if opt
            .db_path
            .join("props/")
            .join(&hash)
            .exists()
          {
            if create_id {
              uuid::Uuid::new_v4()
            } else if get_or_create {
              let index_path = opt.db_path.join("indexes/").join(hash + "/");
              let mut nodes: Vec<uuid::Uuid> = std::fs::read_dir(&index_path)?.into_iter()
                .filter(|entry| {
                  match entry {
                    Ok(entry) => {
                      let reference = entry
                        .file_name()
                        .into_string()
                        .unwrap();
                      let (prefix, _reference) = reference
                        .split_once("_")
                        .unwrap();
                      if prefix == "nodes" {
                        true
                      } else {
                        false
                      }
                    }
                    Err(_) => false
                  }
                })
                .take(2)
                .map(|entry| {
                  let entry = entry.unwrap();
                  let reference = entry
                    .file_name()
                    .into_string()
                    .unwrap();
                  let (_prefix, reference) = reference
                    .split_once("_")
                    .unwrap();
                  uuid::Uuid::parse_str(reference).unwrap()
                })
                .collect();
              if nodes.len() == 1 {
                nodes.pop().unwrap()
              } else {
                bail!("There are several nodes with the same properties. Can't deside which one to use. Please use `--id` to specify the exact node");
              }
            } else {
              bail!("node allready exists. Please use `--create-id` to create a node with equal data anyway");
            }
          } else {
            uuid::Uuid::new_v4()
          }
        }
      };

      let mut db = open(&opt.db_path)?;
      if !update {
        db.create_node(Uuid(id), &properties)?;
      } else {
        db.update_node(Uuid(id), &properties)?;
      }

      println!("{}", id); // TODO opt.output, opt.output_fmt
    }
    DeleteNode {id} => {
      let mut db = open::<T>(&opt.db_path)?;
      db.delete_node(Uuid(id))?;
      log::info!("deleted node {}", id);
    }
    CreateEdge { n1, n2 } => {
      let properties = read_input(opt.input)?;
      let properties: T = SchemaElement::deserialize(&properties)?;

      let mut db = open(&opt.db_path)?;
      let id = db.create_edge(Uuid(n1), Uuid(n2), &properties)?;

      println!("{}", id); // TODO opt.output, opt.output_fmt
    }
    PropertyId => {
      let properties = read_input(opt.input)?;
      let properties: T = SchemaElement::deserialize(&properties)?;
      let hash = properties.get_key();

      println!("{}", hash); // TODO opt.output, opt.output_fmt
    }
    PropertyBlob => {
      let properties = read_input(opt.input)?;
      let properties: T = SchemaElement::deserialize(&properties)?;

      io::stdout().write_all(&SchemaElement::serialize(&properties)?)?;
    }
    QueryDb => {
      let query = read_input(opt.input)?;
      let query = to_query(&query)?;

      let db = open::<T>(&opt.db_path)?;
      let result = db.query(query)?;


      // TODO verschiedene output formate
      println!("{}", serde_json::to_string_pretty(&result)?); // TODO wenn kein Terminal sondern eine pipe verwendet wird kann man kompakteres json ausgeben.

      // TODO Umschliessende Huelle? Alle miteinander verbundenen Edges und Vertices?
    }
    Repl => {
      let db = open::<T>(&opt.db_path)?;
      gravity::kv_graph_store::lua_repl::<T, FsKvStore, _, anyhow::Error>(db, init_fn)?;
    }
    ResultData => {
      let data = read_input(opt.input)?;
      //let mut data: crate::ql::QueryResult = serde_json::from_slice(&data)?;

      let db = open::<T>(&opt.db_path)?;
      //TODO Über die db die Variablen im mit den Properties füllen

      // TODO verschiedene output formate
      println!("{}", serde_json::to_string_pretty(&data)?); // TODO wenn kein Terminal sondern eine pipe verwendet wird kann man kompakteres json ausgeben.
    }
    Init => {
      init::<T>(&opt.db_path)?;
    }
  }

  Ok(())
}

fn open<T>(path: &Path) -> Result<KvGraphStore<T, FsKvStore, FileStoreError>, FileStoreError>
where
  T: Prop,
{
  let kv = FsKvStore::open(path)?;
  Ok(KvGraphStore::from_kv(kv))
}

fn init<T>(path: &Path) -> Result<KvGraphStore<T, FsKvStore, FileStoreError>, FileStoreError>
where
  T: Prop,
{
  let kv = FsKvStore::init(path)?;
  Ok(KvGraphStore::from_kv(kv))
}

type BasicQuery = gravity::kv_graph_store::BasicQuery;

fn to_query(data: &Vec<u8>) -> Result<BasicQuery, SerialisationError> {
  // TODO Verschiedene Query Sprachen über zweiten Parameter
  // TODO Internes Schema verwenden um Abfragen zu verbessern
  let query = serde_json::from_slice(data)?;

  Ok(query)
}
