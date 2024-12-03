use gravity::*;
use gravity::kv_graph_store::Uuid;
use pretty_assertions::assert_eq;
use uuid::uuid;

#[test]
fn create_a_node_in_empty_store() -> Result<(), Error> {
  let mut graph = create_empty_graph();

  graph.create_node(Uuid(uuid!(NODE1_UUID)), &PROPERTY_EMPTY.to_vec())?;

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

  graph.create_node(Uuid(uuid!(NODE1_UUID)), &PROPERTY_EMPTY.to_vec())?;

  // can not create an identical node
  match graph.create_node(Uuid(uuid!(NODE1_UUID)), &PROPERTY_EMPTY.to_vec()) {
    Err(Error::NodeExists(msg)) => assert_eq!(msg, "nodes/a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"),
    _ => panic!("should fail because node exists"),
  };

  // can not create a node with the same id but changed content
  match graph.create_node(Uuid(uuid!(NODE1_UUID)), &PROPERTY_SIMPLE.to_vec()) {
    Err(Error::NodeExists(msg)) => assert_eq!(msg, "nodes/a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"),
    _ => panic!("should fail because node exists"),
  };

  Ok(())
}

#[test]
fn nodes_can_be_connected_with_themselfes() -> Result<(), Error> {
  let mut graph = create_empty_graph();

  graph.create_node(Uuid(uuid!(NODE1_UUID)), &PROPERTY_EMPTY.to_vec())?;
  graph.create_edge(Uuid(uuid!(NODE1_UUID)), Uuid(uuid!(NODE1_UUID)), &PROPERTY_EMPTY.to_vec())?;

  let mut store = get_kv_store(graph);
  let node_path = format!("nodes/{}", NODE1_UUID);
  check_string(
    store.remove(&node_path),
    &format!(
      "{{\"id\":\"{}\",\"properties\":\"{}\",\"incoming\":[\"{}\"],\"outgoing\":[\"{}\"]}}",
        NODE1_UUID,
        PROPERTY_EMPTY_ID,
        EDGE_N1_TO_SELF_ID,
        EDGE_N1_TO_SELF_ID,
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

  let edge1_path = format!("edges/{}", EDGE_N1_TO_SELF_ID);
  check_string(
    store.remove(&edge1_path),
    &format!(
      "{{\"properties\":\"{}\",\"n1\":\"{}\",\"n2\":\"{}\"}}",
        PROPERTY_EMPTY_ID,
        NODE1_UUID,
        NODE1_UUID,
    )
  );
  check_string(
    store.remove(&format!("indexes/{}/edges_{}", PROPERTY_EMPTY_ID, EDGE_N1_TO_SELF_ID)),
    &edge1_path
  );

  Ok(assert_eq!(store.len(), 0))
}

#[test]
fn create_two_nodes_with_connection() -> Result<(), Error> {
  let mut graph = create_empty_graph();

  graph.create_node(Uuid(uuid!(NODE1_UUID)), &PROPERTY_EMPTY.to_vec())?;
  graph.create_node(Uuid(uuid!(NODE2_UUID)), &PROPERTY_SIMPLE.to_vec())?;
  graph.create_edge(Uuid(uuid!(NODE1_UUID)), Uuid(uuid!(NODE2_UUID)), &PROPERTY_EMPTY.to_vec())?;

  let mut store = get_kv_store(graph);
  let node1_path = format!("nodes/{}", NODE1_UUID);
  check_string(
    store.remove(&node1_path),
    &format!(
      "{{\"id\":\"{}\",\"properties\":\"{}\",\"incoming\":[],\"outgoing\":[\"{}\"]}}",
        NODE1_UUID,
        PROPERTY_EMPTY_ID,
        EDGE1_ID,
    )
  );
  check_string(
    store.remove(&format!("props/{}", PROPERTY_EMPTY_ID)),
    ""
  );
  check_string(
    store.remove(&format!("indexes/{}/nodes_{}", PROPERTY_EMPTY_ID, NODE1_UUID)),
    &node1_path
  );

  let node2_path = format!("nodes/{}", NODE2_UUID);
  check_string(
    store.remove(&node2_path),
    &format!(
      "{{\"id\":\"{}\",\"properties\":\"{}\",\"incoming\":[\"{}\"],\"outgoing\":[]}}",
        NODE2_UUID,
        PROPERTY_SIMPLE_ID,
        EDGE1_ID,
    )
  );
  check_string(
    store.remove(&format!("props/{}", PROPERTY_SIMPLE_ID)),
    "simple text property"
  );
  check_string(
    store.remove(&format!("indexes/{}/nodes_{}", PROPERTY_SIMPLE_ID, NODE2_UUID)),
    &node2_path
  );

  let edge1_path = format!("edges/{}", EDGE1_ID);
  check_string(
    store.remove(&edge1_path),
    &format!(
      "{{\"properties\":\"{}\",\"n1\":\"{}\",\"n2\":\"{}\"}}",
        PROPERTY_EMPTY_ID,
        NODE1_UUID,
        NODE2_UUID,
    )
  );
  check_string(
    store.remove(&format!("indexes/{}/edges_{}", PROPERTY_EMPTY_ID, EDGE1_ID)),
    &edge1_path
  );

  Ok(assert_eq!(store.len(), 0))
}

fn check_string(left: Option<Vec<u8>>, right: &str) {
  let left = left.unwrap();
  let formatted = String::from_utf8(left).expect("should be an utf8 string");

  assert_eq!(formatted, right.to_string())
}

const NODE1_UUID : &str = "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8";
const NODE2_UUID : &str = "e1e2e3e4-f1f2-a1a2-b1b2-b3b4b5b6b7b8";
const PROPERTY_EMPTY : &[u8] = "".as_bytes();
const PROPERTY_EMPTY_ID: &str = "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855";
const PROPERTY_SIMPLE : &[u8] = "simple text property".as_bytes();
const PROPERTY_SIMPLE_ID: &str = "4637D294486C315FC8D6C2F11742CBA4958CCB3F083656808C2B257D954DE631";
const EDGE1_ID : &str = "0B49457674D1B570400E6EC9E4B78F9C2C9B0721BA7C315BD0811E3059C3BBBA";
const EDGE_N1_TO_SELF_ID : &str = "7622305FED0A357AF8AAE5ACC4110B8CAD7BDF2D67CAEA195BCDA0889A20FB8A";

fn create_empty_graph() -> GStore {
  let kv = mem_kv_store::MemoryKvStore::default();
  kv_graph_store::KvGraphStore::from_kv(kv)
}

fn get_kv_store(graph: GStore) -> std::collections::BTreeMap<String, Vec<u8>> {
  graph.into_kv().get_inner()
}

type Error = kv_graph_store::Error<mem_kv_store::Error>;
type GStore = kv_graph_store::KvGraphStore::<Vec<u8>, mem_kv_store::MemoryKvStore, mem_kv_store::Error>;

