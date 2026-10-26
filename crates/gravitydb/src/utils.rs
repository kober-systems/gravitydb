use std::collections::BTreeMap;

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
}
