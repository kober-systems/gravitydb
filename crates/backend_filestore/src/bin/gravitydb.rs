use gravitydb_filestore::cli_helpers;
use anyhow::Result;

fn main() -> Result<()> {
  cli_helpers::db_cmds::<gravitydb::schema::GenericProperty>(init_nothing)
}

fn init_nothing(_ : &mlua::Lua) -> mlua::Result<()> {
  Ok(())
}
