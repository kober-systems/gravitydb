use gravity::schema::{SchemaElement, Property};
use crate::{Error, FsKvStore, HashId};
use anyhow::bail;
use std::io::{self, Write};
use gravity::{ql, GraphStore};
use gravity::kv_graph_store::KvGraphStore;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use std::io::Read;
use anyhow::Result;

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

pub fn db_cmds<T>() -> Result<()>
where
  T: Property<HashId, Error> + 'static + std::clone::Clone + mlua::UserData,
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
        db.create_node(id, &properties)?;
      } else {
        db.update_node(id, &properties)?;
      }

      println!("{}", id); // TODO opt.output, opt.output_fmt
    }
    DeleteNode {id} => {
      let mut db = open::<T>(&opt.db_path)?;
      db.delete_node(id)?;
      log::info!("deleted node {}", id);
    }
    CreateEdge { n1, n2 } => {
      let properties = read_input(opt.input)?;
      let properties: T = SchemaElement::deserialize(&properties)?;

      let mut db = open(&opt.db_path)?;
      let id = db.create_edge(n1, n2, &properties)?;

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
      let query = crate::to_query(&query)?;

      let db = open::<T>(&opt.db_path)?;
      let result = db.query(query)?;


      // TODO verschiedene output formate
      println!("{}", serde_json::to_string_pretty(&result)?); // TODO wenn kein Terminal sondern eine pipe verwendet wird kann man kompakteres json ausgeben.

      // TODO Umschliessende Huelle? Alle miteinander verbundenen Edges und Vertices?
    }
    Repl => {
      use mlua::{Error, Lua, MultiValue};
      use rustyline::Editor;

      let lua = Lua::new();
      let mut editor = Editor::<()>::new().expect("Failed to make rustyline editor");

      let globals = lua.globals();
      let db_open = lua.create_function(|_, path: String| {
        use mlua::prelude::LuaError;
        use std::sync::Arc;

        let path = crate::Path::new(&path);
        match open::<T>(&path) {
          Ok(db) => Ok(db),
          Err(e) => Err(LuaError::ExternalError(Arc::new(e))),
        }
      })?;
      globals.set("db_open", db_open)?;
      ql::init_lua::<String, HashId, HashId, String, String>(&lua)?;

      loop {
        let mut prompt = "> ";
        let mut line = String::new();

        loop {
          let input = editor.readline(prompt)?;
          line.push_str(&input);

          match lua.load(&line).eval::<MultiValue>() {
            Ok(values) => {
              editor.add_history_entry(line);
              println!(
                "{}",
                values
                  .iter()
                  .map(|value| format!("{:?}", value))
                  .collect::<Vec<_>>()
                  .join("\t")
              );
              break;
            }
            Err(Error::SyntaxError {
              incomplete_input: true,
              ..
            }) => {
              // continue reading input and append it to `line`
              line.push_str("\n"); // separate input lines
              prompt = ">> ";
            }
            Err(e) => {
              eprintln!("error: {}", e);
              break;
            }
          }
        }
      }
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

fn open<T>(path: &Path) -> Result<KvGraphStore<T, FsKvStore<T>>, Error>
where
  T: Property<HashId, Error> + 'static + std::clone::Clone + mlua::UserData,
{
  let kv = FsKvStore::<T>::open(path)?;
  Ok(KvGraphStore::from_kv(kv))
}

fn init<T>(path: &Path) -> Result<KvGraphStore<T, FsKvStore<T>>, Error>
where
  T: Property<HashId, Error> + 'static + std::clone::Clone + mlua::UserData,
{
  let kv = FsKvStore::<T>::init(path)?;
  Ok(KvGraphStore::from_kv(kv))
}
