use core::hash::Hash;
use mlua::{FromLua, IntoLua, IntoLuaMulti, Lua, LuaSerdeExt, UserData, UserDataMethods};

use crate::kv_graph_store::*;
use crate::{GraphStore, KVStore};
use crate::ql;
use crate::ql::{VertexQuery, EdgeQuery, PropertyQuery, QueryResult};
use crate::schema::Property;

impl UserData for Uuid {
  fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_function("key", |_, id: Self| {
      Ok(id.to_key())
    });
  }
}

impl<P, K, E> UserData for KvGraphStore<P, K, E>
where
  for<'lua> P: Property<HashId, SerialisationError> + UserData + std::clone::Clone + 'lua + FromLua<'lua>,
  K: KVStore<E>,
  E: Send + Sync + std::fmt::Debug,
{
  fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
    use mlua::prelude::LuaError;

    methods.add_method_mut("create_node", |_, db, props: P| {
      let id = Uuid::new();
      match db.create_node(id, &props) {
        Ok(id) => Ok(id),
        Err(e) => Err(LuaError::external(e.to_string()))
      }
    });

    methods.add_method_mut("update_node", |_, db, (id, props): (VertexId, P)| {
      match db.update_node(id, &props) {
        Ok(id) => Ok(id),
        Err(e) => Err(LuaError::external(e.to_string()))
      }
    });

    methods.add_method_mut("delete_node", |_, db, id: VertexId| {
      match db.delete_node(id) {
        Ok(id) => Ok(id),
        Err(e) => Err(LuaError::external(e.to_string()))
      }
    });

    methods.add_method_mut("create_edge", |_, db, (n1, n2, props): (VertexId, VertexId, P)| {
      match db.create_edge(n1, n2, &props) {
        Ok(id) => Ok(id),
        Err(e) => Err(LuaError::external(e.to_string()))
      }
    });

    methods.add_method_mut("delete_edge", |_, db, id: HashId| {
      match db.delete_edge(&id) {
        Ok(_) => Ok(()),
        Err(e) => Err(LuaError::external(e.to_string()))
      }
    });

    methods.add_method_mut("query", |lua, db, query: mlua::AnyUserData| {
      let query: BasicQuery = match query.take::<ql::VertexQuery<_,_,_,_,_>>() {
        Ok(q) => q.into(),
        Err(_) => match query.take::<ql::EdgeQuery<_,_,_,_,_>>() {
          Ok(q) => q.into(),
          Err(_) => query.take::<ql::PropertyQuery<_>>()?.into(),
        }
      };
      match db.query(query) {
        Ok(result) => Ok(lua.to_value(&result)),
        Err(e) => Err(LuaError::external(e.to_string()))
      }
    });
  }
}

pub fn init_lua<VertexId, EdgeId, PropertyId, VFilter, EFilter>(lua: &Lua) -> mlua::Result<()>
where
  for<'lua> VertexId:   Clone + 'lua + FromLua<'lua>,
  for<'lua> EdgeId:     Clone + 'lua + FromLua<'lua>,
  for<'lua> PropertyId: Clone + 'lua + FromLua<'lua>,
  VFilter:    Clone + 'static,
  EFilter:    Clone + 'static,
{
  let globals = lua.globals();
  globals.set("vq_all", lua.create_function(|_, ()| {
    Ok(VertexQuery::<VertexId, EdgeId, PropertyId, VFilter, EFilter>::all())
  })?)?;
  globals.set("vq_from_ids", lua.create_function(|_, ids: Vec<VertexId>| {
    Ok(VertexQuery::<VertexId, EdgeId, PropertyId, VFilter, EFilter>::from_ids(ids))
  })?)?;
  globals.set("vq_from_property", lua.create_function(|_, p: LuaPropertyQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>| {
    Ok(VertexQuery::<VertexId, EdgeId, PropertyId, VFilter, EFilter>::from_property(p.q))
  })?)?;

  globals.set("eq_all", lua.create_function(|_, ()| {
    Ok(EdgeQuery::<VertexId, EdgeId, PropertyId, VFilter, EFilter>::all())
  })?)?;
  globals.set("eq_from_ids", lua.create_function(|_, ids: Vec<EdgeId>| {
    Ok(EdgeQuery::<VertexId, EdgeId, PropertyId, VFilter, EFilter>::from_ids(ids))
  })?)?;
  globals.set("eq_from_property", lua.create_function(|_, p: LuaPropertyQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>| {
    Ok(EdgeQuery::<VertexId, EdgeId, PropertyId, VFilter, EFilter>::from_property(p.q))
  })?)?;
  globals.set("pq_from_id", lua.create_function(|_, id: PropertyId| {
    Ok(LuaPropertyQuery::<VertexId, EdgeId, PropertyId, VFilter, EFilter> {
      q: PropertyQuery::from_id(id),
      marker: std::marker::PhantomData,
    })
  })?)?;

  Ok(())
}

impl<VertexId, EdgeId, PropertyId, VFilter, EFilter> UserData for VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>
where
  for<'lua> VertexId:   Clone + 'lua + FromLua<'lua>,
  for<'lua> EdgeId:     Clone + 'lua + FromLua<'lua>,
  for<'lua> PropertyId: Clone + 'lua + FromLua<'lua>,
  VFilter:    Clone + 'static,
  EFilter:    Clone + 'static,
{
  fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
    //methods.add_method("union", |_, this, q2: VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>| {
    //  Ok(this.clone().union(q2))
    //});

    methods.add_function("outgoing", |lua, q: (Self, Option<mlua::AnyUserData>)| {
      let (q, filter) = q;

      match filter {
        Some(filter) => match filter.take::<ql::VertexQuery<_,_,_,_,_>>() {
          Ok(filter) => q.outgoing().outgoing().intersect(filter).into_lua(lua),
          Err(_) => match filter.take::<ql::EdgeQuery<_,_,_,_,_>>() {
            Ok(filter) => q.outgoing().intersect(filter).into_lua(lua),
            Err(_) => q.outgoing().intersect(filter.take::<ql::PropertyQuery<_>>()?.referencing_edges()).into_lua(lua),
          }
        },
        None => q.outgoing().into_lua(lua)
      }
    });
    methods.add_function("ingoing", |_, q: Self| {
      Ok(q.ingoing())
    });
    methods.add_function("union", |_, queries: (Self, Self)| {
      let (q1, q2) = queries;
      Ok(q1.union(q2))
    });
    methods.add_function("intersect", |_, queries: (Self, Self)| {
      let (q1, q2) = queries;
      Ok(q1.intersect(q2))
    });
    methods.add_function("substract", |_, queries: (Self, Self)| {
      let (q1, q2) = queries;
      Ok(q1.substract(q2))
    });
  }
}

impl<VertexId, EdgeId, PropertyId, VFilter, EFilter> UserData for EdgeQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>
where
  for<'lua> VertexId:   Clone + 'lua + FromLua<'lua>,
  for<'lua> EdgeId:     Clone + 'lua + FromLua<'lua>,
  for<'lua> PropertyId: Clone + 'lua + FromLua<'lua>,
  VFilter:    Clone + 'static,
  EFilter:    Clone + 'static,
{
  fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_function("outgoing", |_, q: Self| {
      Ok(q.outgoing())
    });
    methods.add_function("ingoing", |_, q: Self| {
      Ok(q.ingoing())
    });
    methods.add_function("union", |_, queries: (Self, Self)| {
      let (q1, q2) = queries;
      Ok(q1.union(q2))
    });
    methods.add_function("intersect", |_, queries: (Self, Self)| {
      let (q1, q2) = queries;
      Ok(q1.intersect(q2))
    });
    methods.add_function("substract", |_, queries: (Self, Self)| {
      let (q1, q2) = queries;
      Ok(q1.substract(q2))
    });
  }
}

#[derive(Clone, FromLua)]
pub struct LuaPropertyQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter> {
  q: PropertyQuery<PropertyId>,
  marker: std::marker::PhantomData<VertexQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>>,
}

impl<VertexId, EdgeId, PropertyId, VFilter, EFilter> LuaPropertyQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter> {
  pub fn from_property_query(q: PropertyQuery<PropertyId>) -> Self {
    LuaPropertyQuery {
      q,
      marker: std::marker::PhantomData,
    }
  }
}

impl<VertexId, EdgeId, PropertyId, VFilter, EFilter> UserData for LuaPropertyQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>
where
  for<'lua> VertexId:   Clone + 'lua + FromLua<'lua>,
  for<'lua> EdgeId:     Clone + 'lua + FromLua<'lua>,
  for<'lua> PropertyId: Clone + 'lua + FromLua<'lua>,
  VFilter:    Clone + 'static,
  EFilter:    Clone + 'static,
{
  fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_function("referencing_properties", |_, q: Self| {
      let q = q.q;
      Ok(LuaPropertyQuery::<VertexId, EdgeId, PropertyId, VFilter, EFilter>::from_property_query(q.referencing_properties()))
    });
    methods.add_function("referenced_properties", |_, q: Self| {
      let q = q.q;
      Ok(LuaPropertyQuery::<VertexId, EdgeId, PropertyId, VFilter, EFilter>::from_property_query(q.referenced_properties()))
    });
    methods.add_function("referencing_vertices", |_, q: Self| {
      Ok(q.q.referencing_vertices::<VertexId, EdgeId, VFilter, EFilter>())
    });
    methods.add_function("referencing_edges", |_, q: Self| {
      Ok(q.q.referencing_edges::<VertexId, EdgeId, VFilter, EFilter>())
    });
  }
}

impl<VertexId, EdgeId, PropertyId> UserData for QueryResult<VertexId, EdgeId, PropertyId>
where
  for<'lua> VertexId:   Hash + Eq + Clone + 'lua + FromLua<'lua>,
  for<'lua> EdgeId:     Hash + Eq + Clone + 'lua + FromLua<'lua>,
  for<'lua> PropertyId: Hash + Eq + Clone + 'lua + FromLua<'lua>,
{}

use rustyline::{completion::Completer, Helper, Hinter, Validator, Highlighter};

#[derive(Helper, Hinter, Validator, Highlighter)]
struct LuaCompleter<'a> { lua: &'a Lua }

impl Completer for LuaCompleter<'_> {
  type Candidate = String;
  fn complete(
          &self,
          line: &str,
          pos: usize,
          _ctx: &rustyline::Context<'_>
  ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
    let mut completetions = vec![];

    let line = &line[..pos];
    let bounderies = [' ', '\t', '(', ')', '[', ']', '{', '}'];
    let start = line.rfind(&bounderies).unwrap_or(0);
    let tokens = &line[start..].split(&['.', ':']);
    let level_cnt = tokens.clone().count();

    use mlua::Value::*;

    tokens.clone().fold((1, Table(self.lua.globals())), |(level, v), t| {
      let t = t.trim_start_matches(&bounderies);
      if let Table(ref v) = v {
        if level == level_cnt {
          v.for_each(|k: mlua::Value, _v: mlua::Value| {
              if let Ok(k) = k.to_string() {
                if k.starts_with(t) {
                  completetions.push(k[t.len()..].to_string());
                }
            };

            Ok(())
          }).unwrap_or_default();
        } else {
          return (level + 1, match v.raw_get(t) {
            Ok(v) => {
              v
            }
            Err(_) => {
              Nil
            }
          });
        }
      }

      (level + 1, v)
    });

    Ok((pos, completetions))
  }
}

pub fn lua_repl<T, Kv, E, OutE>(db: KvGraphStore<T, Kv, E>, init_fn: fn(&Lua) -> mlua::Result<()>) -> Result<(), OutE>
where
  for<'lua> T: Property<HashId, SerialisationError> + 'lua + FromLua<'lua> + UserData + Clone,
  Kv: KVStore<E> + 'static,
  E: Send + Sync + std::fmt::Debug + 'static,
  OutE: From<rustyline::error::ReadlineError> + From<mlua::Error>,
{
  use mlua::{Error, MultiValue};
  use rustyline::{Editor, error::ReadlineError};

  let lua = lua_init(db, init_fn)?;
  let mut editor = Editor::<LuaCompleter, rustyline::history::DefaultHistory>::new().expect("Failed to make rustyline editor");
  editor.set_helper(Some(LuaCompleter { lua: &lua }));

  loop {
    let mut prompt = "> ";
    let mut line = String::new();

    loop {
      let input = match editor.readline(prompt) {
        Ok(out) => Ok(out),
        Err(ReadlineError::Eof) => return Ok(()),
        Err(e) => Err(e),
      }?;
      line.push_str(&input);

      match lua.load(&line).eval::<MultiValue>() {
        Ok(values) => {
          editor.add_history_entry(line)?;
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

pub fn lua_run<T, Kv, E, S, S2>(db: KvGraphStore<T, Kv, E>, init_fn: fn(&Lua) -> mlua::Result<()>, code: S, code_name: S2) -> Result<(), mlua::Error>
where
  for<'lua> T: Property<HashId, SerialisationError> + 'lua + FromLua<'lua> + UserData + Clone,
  Kv: KVStore<E> + 'static,
  E: Send + Sync + std::fmt::Debug + 'static,
  S: AsRef<str>,
  S2: AsRef<str>,
{
  lua_init(db, init_fn)?.load(code.as_ref())
    .set_name(code_name.as_ref())
    .eval()
}

fn lua_init<T, Kv, E>(db: KvGraphStore<T, Kv, E>, init_fn: fn(&Lua) -> mlua::Result<()>) -> Result<Lua, mlua::Error>
where
  for<'lua> T: Property<HashId, SerialisationError> + 'lua + FromLua<'lua> + UserData + Clone,
  Kv: KVStore<E> + 'static,
  E: Send + Sync + std::fmt::Debug + 'static,
{
  let lua = Lua::new();
  lua.globals().raw_set("db", db)?;
  init_lua::<String, HashId, HashId, String, String>(&lua)?;
  init_fn(&lua)?;

  Ok(lua)
}

