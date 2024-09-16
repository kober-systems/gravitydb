use gravity::*;
use uuid::uuid;

#[test]
fn create_a_node_in_empty_store() -> Result<(), Error> {
  let mut graph = create_empty_graph();

  graph.create_node(uuid!(EXAMPLE_UUID), &PROPERTY_EMPTY.to_vec())?;

  let node_content = format!(
    "{{\"id\":\"{}\",\"properties\":\"{}\",\"incoming\":[],\"outgoing\":[]}}",
      EXAMPLE_UUID,
      PROPERTY_EMPTY_ID
  );
  let mut store = get_kv_store(graph);
  check_string(
    store.remove(&format!("nodes/{}", EXAMPLE_UUID)),
    &node_content
  );
  check_string(
    store.remove(&format!("props/{}", PROPERTY_EMPTY_ID)),
    ""
  );
  check_string(
    store.remove(&format!("indexes/{}/nodes_{}", PROPERTY_EMPTY_ID, EXAMPLE_UUID)),
    &node_content
  );

  Ok(assert_eq!(store.len(), 0))
}

fn check_string(left: Option<Vec<u8>>, right: &str) {
  let left = left.unwrap();
  let formatted = String::from_utf8(left).expect("should be an utf8 string");

  assert_eq!(formatted, right.to_string())
}

const EXAMPLE_UUID : &str = "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8";
const PROPERTY_EMPTY : &[u8] = "".as_bytes();
const PROPERTY_EMPTY_ID: &str = "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855";

fn create_empty_graph() -> GStore {
  let kv = mem_kv_store::MemoryKvStore::default();
  kv_graph_store::KvGraphStore::from_kv(kv)
}

fn get_kv_store(graph: GStore) -> std::collections::BTreeMap<String, Vec<u8>> {
  graph.into_kv().get_inner()
}

type Error = kv_graph_store::Error<mem_kv_store::Error>;
type GStore = kv_graph_store::KvGraphStore::<Vec<u8>, mem_kv_store::MemoryKvStore, mem_kv_store::Error>;

