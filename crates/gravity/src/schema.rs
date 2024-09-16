use crate::ql;

pub trait SchemaElement<K: Sized, E> {
  fn deserialize(data: &[u8]) -> Result<Self,E> where Self: Sized;
  fn serialize(&self) -> Result<Vec<u8>,E>;
  fn get_key(&self) -> K;
}

pub trait Property<K: Sized, E>: Sized + SchemaElement<K, E>
{
  fn nested(&self) -> Vec<Self>;
}

pub enum SchemaConstraint<VertexId, EdgeId, PropertyId, VFilter, EFilter> {
  Requiered(ql::BasicQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>),
  Prohibited(ql::BasicQuery<VertexId, EdgeId, PropertyId, VFilter, EFilter>),
}

pub struct Schema<
  VertexId,
  EdgeId,
  PropertyId,
  VFilter,
  EFilter,
  VertexSchema,
  EdgeSchema,
  PropertySchema,
  E,
>
where
  VertexId: Sized,
  VertexSchema: SchemaElement<VertexId, E>,
  EdgeId: Sized,
  EdgeSchema: SchemaElement<EdgeId, E>,
  PropertyId: Sized,
  PropertySchema: SchemaElement<PropertyId, E>,
{
  pub vertex_properties: VertexSchema,
  pub edge_properties: EdgeSchema,
  pub referenced_properties: PropertySchema,
  pub constraints: Vec<SchemaConstraint<VertexId, EdgeId, PropertyId, VFilter, EFilter>>,
  _err_type: std::marker::PhantomData<E>,
}

use sha2::Digest;

impl<E> SchemaElement<String, E> for Vec<u8>
{
  fn get_key(&self) -> String {
    format!("{:X}", sha2::Sha256::digest(&self))
  }

  fn serialize(&self) -> Result<Self, E> {
    Ok(self.clone())
  }

  fn deserialize(data: &[u8]) -> Result<Self, E> {
    Ok(data.to_vec())
  }
}

impl<E> Property<String, E> for Vec<u8> {
  fn nested(&self) -> Vec<Self> { Vec::new() }
}
