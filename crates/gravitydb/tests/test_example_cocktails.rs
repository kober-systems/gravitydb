use gravitydb::{kv_graph_store, mem_kv_store};

#[test]
fn trivial_queries() -> Result<(), Error> {
  let kv = mem_kv_store::MemoryKvStore::default();
  gravitydb_test_utils::trivial_queries(kv)
}

#[test]
fn alexander_ingredients() -> Result<(), Error> {
  let kv = mem_kv_store::MemoryKvStore::default();
  gravitydb_test_utils::alexander_ingredients(kv)
}

#[test]
fn which_cocktails_include_gin() -> Result<(), Error> {
  let kv = mem_kv_store::MemoryKvStore::default();
  gravitydb_test_utils::which_cocktails_include_gin(kv)
}

#[test]
fn cocktail_statistic() -> Result<(), Error> {
  let kv = mem_kv_store::MemoryKvStore::default();
  gravitydb_test_utils::cocktail_statistic(kv)
}

type Error = kv_graph_store::Error<mem_kv_store::Error>;

