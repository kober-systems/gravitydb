use gravitydb_filestore::cli_helpers;
use anyhow::Result;

fn main() -> Result<()> {
  cli_helpers::db_cmds::<gravitydb_filestore::GenericProperty>()
}
