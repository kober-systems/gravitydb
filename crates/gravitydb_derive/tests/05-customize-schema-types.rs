// Should be able to add more types via attribute
use gravitydb_derive::Schema;

#[derive(Schema)]
#[derive(Debug, PartialEq)]
pub enum BasicPimSchema {
  #[schema(custom = get_person_schema_type)]
  Person{ name: String, surname: String, is_male: bool },
  Organisation(String),
  // edge types
  BelongsTo,
  SchemaType(String),
}

fn get_person_schema_type(_name: &String, _surname: &String, is_male: &bool) -> Vec<BasicPimSchema> {
  vec![BasicPimSchema::SchemaType(if *is_male {
      "Male"
    } else {
      "Female"
    }.to_string()
  )]
}

fn main() {
  assert_eq!(
    BasicPimSchema::Person {
      name: "John".to_string(),
      surname: "Doe".to_string(),
      is_male: true }.nested(),
    vec![
      BasicPimSchema::SchemaType("Person".to_string()),
      BasicPimSchema::SchemaType("Male".to_string()),
    ]
  );
  assert_eq!(
    BasicPimSchema::Person {
      name: "Jane".to_string(),
      surname: "Doe".to_string(),
      is_male: false }.nested(),
    vec![
      BasicPimSchema::SchemaType("Person".to_string()),
      BasicPimSchema::SchemaType("Female".to_string()),
    ]
  );
}
