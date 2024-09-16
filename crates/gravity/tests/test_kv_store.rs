use gravity::*;
use uuid::uuid;

#[test]
fn create_a_node_in_empty_store() -> Result<(), Error> {
  let kv = mem_kv_store::MemoryKvStore::default();
  let mut graph = kv_graph_store::KvGraphStore::<Vec<u8>, mem_kv_store::MemoryKvStore, mem_kv_store::Error>::from_kv(kv);

  graph.create_node(uuid!(EXAMPLE_UUID), &"".as_bytes().to_vec())?;

  let mut store = graph.into_kv().get_inner();
  check_string(
    store.remove(&format!("nodes/{}", EXAMPLE_UUID)),
    &format!("{{\"id\":\"{}\",\"properties\":\"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855\",\"incoming\":[],\"outgoing\":[]}}", EXAMPLE_UUID)
  );
  check_string(
    store.remove("props/E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
    ""
  );

  Ok(assert_eq!(store.len(), 0))
}

fn check_string(left: Option<Vec<u8>>, right: &str) {
  let left = left.unwrap();
  let formatted = String::from_utf8(left).expect("should be an utf8 string");

  assert_eq!(formatted, right.to_string())
}

const EXAMPLE_UUID : &str = "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8";

type Error = kv_graph_store::Error<mem_kv_store::Error>;

