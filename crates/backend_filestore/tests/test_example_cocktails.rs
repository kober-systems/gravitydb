use gravitydb_filestore::{FsKvStore, FileStoreError};

#[test]
fn trivial_queries() -> Result<(), Error> {
  let kv = FsKvStore::from_memory().expect("Could not create kv store");
  gravitydb_test_utils::trivial_queries(kv)
}

#[test]
fn alexander_ingredients() -> Result<(), Error> {
  let kv = FsKvStore::from_memory().expect("Could not create kv store");
  gravitydb_test_utils::alexander_ingredients(kv)
}

#[test]
fn which_cocktails_include_gin() -> Result<(), Error> {
  let kv = FsKvStore::from_memory().expect("Could not create kv store");
  gravitydb_test_utils::which_cocktails_include_gin(kv)
}

#[test]
fn cocktail_statistic() -> Result<(), Error> {
  let kv = FsKvStore::from_memory().expect("Could not create kv store");
  gravitydb_test_utils::cocktail_statistic(kv)
}

type Error = gravitydb::kv_graph_store::Error<FileStoreError>;

