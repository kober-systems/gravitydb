use sha2::Digest;
use std::collections::BTreeMap;
use std::fmt::Write;

pub(crate) fn sha256_key(data: &[u8]) -> String {
  let digest = sha2::Sha256::digest(data);
  let mut key = String::with_capacity(digest.len() * 2);
  for byte in digest {
    write!(&mut key, "{byte:02X}").expect("writing to a String cannot fail");
  }
  key
}

/// GraphML's declared type for a property key.
pub enum PropType {
  String,
  Integer,
  Double,
  Boolean,
  Custom(String),
}

impl<'a> From<&'a PropType> for &'a str {
  fn from(prop_type: &'a PropType) -> Self {
    match prop_type {
      PropType::String => "string",
      PropType::Integer => "int",
      PropType::Double => "double",
      PropType::Boolean => "boolean",
      PropType::Custom(name) => name,
    }
  }
}

/// GraphML property-key IDs mapped to their declared types.
pub type PropertyTypes = BTreeMap<String, PropType>;

#[cfg(test)]
mod tests {
  use super::PropType;

  #[test]
  fn property_type_converts_to_its_graphml_name() {
    let prop_type = PropType::Custom("json".to_string());
    let name: &str = (&prop_type).into();
    assert_eq!(name, "json");
  }

  #[test]
  fn sha256_key_is_uppercase_hex() {
    assert_eq!(
      super::sha256_key(b"abc"),
      "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD"
    );
  }
}
