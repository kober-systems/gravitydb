use gravity::*;
use uuid::uuid;

#[test]
fn create_a_node_in_empty_store() -> Result<(), Error> {
  let mut graph = create_empty_graph();

  graph.create_node(uuid!(NODE1_UUID), &PROPERTY_EMPTY.to_vec())?;

  let mut store = get_kv_store(graph);
  let node_path = format!("nodes/{}", NODE1_UUID);
  check_string(
    store.remove(&node_path),
    &format!(
      "{{\"id\":\"{}\",\"properties\":\"{}\",\"incoming\":[],\"outgoing\":[]}}",
        NODE1_UUID,
        PROPERTY_EMPTY_ID
    )
  );
  check_string(
    store.remove(&format!("props/{}", PROPERTY_EMPTY_ID)),
    ""
  );
  check_string(
    store.remove(&format!("indexes/{}/nodes_{}", PROPERTY_EMPTY_ID, NODE1_UUID)),
    &node_path
  );

  Ok(assert_eq!(store.len(), 0))
}

#[test]
fn cannot_create_a_node_twice() -> Result<(), Error> {
  let mut graph = create_empty_graph();

  graph.create_node(uuid!(NODE1_UUID), &PROPERTY_EMPTY.to_vec())?;

  // can not create an identical node
  match graph.create_node(uuid!(NODE1_UUID), &PROPERTY_EMPTY.to_vec()) {
    Err(Error::NodeExists(msg)) => assert_eq!(msg, "nodes/a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"),
    _ => panic!("should fail because node exists"),
  };

  // can not create a node with the same id but changed content
  match graph.create_node(uuid!(NODE1_UUID), &PROPERTY_SIMPLE.to_vec()) {
    Err(Error::NodeExists(msg)) => assert_eq!(msg, "nodes/a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"),
    _ => panic!("should fail because node exists"),
  };

  Ok(())
}

fn check_string(left: Option<Vec<u8>>, right: &str) {
  let left = left.unwrap();
  let formatted = String::from_utf8(left).expect("should be an utf8 string");

  assert_eq!(formatted, right.to_string())
}

const NODE1_UUID : &str = "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8";
const PROPERTY_EMPTY : &[u8] = "".as_bytes();
const PROPERTY_EMPTY_ID: &str = "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855";
const PROPERTY_SIMPLE : &[u8] = "simple text property".as_bytes();

fn create_empty_graph() -> GStore {
  let kv = mem_kv_store::MemoryKvStore::default();
  kv_graph_store::KvGraphStore::from_kv(kv)
}

fn get_kv_store(graph: GStore) -> std::collections::BTreeMap<String, Vec<u8>> {
  graph.into_kv().get_inner()
}

type Error = kv_graph_store::Error<mem_kv_store::Error>;
type GStore = kv_graph_store::KvGraphStore::<Vec<u8>, mem_kv_store::MemoryKvStore, mem_kv_store::Error>;

