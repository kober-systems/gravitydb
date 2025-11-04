use crate::ql;

pub trait SchemaElement<E> {
  fn deserialize(data: &[u8]) -> Result<Self,E> where Self: Sized;
  fn serialize(&self) -> Result<Vec<u8>,E>;
}

pub trait KeyAdressableElement<K: Sized> {
  fn get_key(&self) -> K;

  /// A starting point for the queries
  fn start(&self) -> crate::ql::PropertyQuery<K> {
    crate::ql::PropertyQuery::from_id(self.get_key())
  }
}

pub trait NestableProperty: Sized
{
  fn nested(&self) -> Vec<Self>;
}

pub trait Property<K: Sized, E>: Sized + SchemaElement<E> + KeyAdressableElement<K> + NestableProperty {}
impl<T: Sized + SchemaElement<E> + NestableProperty + KeyAdressableElement<K>, K: Sized, E> Property<K, E> for T {}

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
  VertexSchema: SchemaElement<E> + KeyAdressableElement<VertexId>,
  EdgeId: Sized,
  EdgeSchema: SchemaElement<E> + KeyAdressableElement<EdgeId>,
  PropertyId: Sized,
  PropertySchema: SchemaElement<E> + KeyAdressableElement<PropertyId>,
{
  pub vertex_properties: VertexSchema,
  pub edge_properties: EdgeSchema,
  pub referenced_properties: PropertySchema,
  pub constraints: Vec<SchemaConstraint<VertexId, EdgeId, PropertyId, VFilter, EFilter>>,
  _err_type: std::marker::PhantomData<E>,
}

/// Trait to mark the the automatic implementation should be used
pub trait JsonSchemaProperty {}
use sha2::Digest;

impl<T: JsonSchemaProperty + serde::Serialize> KeyAdressableElement<String> for T {
  fn get_key(&self) -> String {
    let data = serde_json::to_vec(&self).unwrap();
    format!("{:X}", sha2::Sha256::digest(&data))
  }
}

impl<T: JsonSchemaProperty + serde::Serialize + for<'a> serde::Deserialize<'a>, Error: From<serde_json::Error>> SchemaElement<Error> for T {
  fn serialize(&self) -> Result<Vec<u8>, Error> {
    Ok(serde_json::to_vec(self)?)
  }

  fn deserialize(data: &[u8]) -> Result<Self, Error>
  where
    Self: Sized,
  {
    Ok(serde_json::from_slice::<T>(data)?)
  }
}

#[cfg(feature="lua")]
use mlua::{FromLua, UserData};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "lua", derive(FromLua))]
pub struct GenericProperty(Vec<u8>);

impl KeyAdressableElement<String> for GenericProperty
{
  fn get_key(&self) -> String {
    format!("{:X}", sha2::Sha256::digest(&self.0))
  }
}

impl<E> SchemaElement<E> for GenericProperty
{
  fn serialize(&self) -> Result<Vec<u8>, E> {
    Ok(self.0.clone())
  }

  fn deserialize(data: &[u8]) -> Result<Self, E>
  where
    Self: Sized,
  {
    Ok(GenericProperty(data.to_vec()))
  }
}

impl NestableProperty for GenericProperty {
  fn nested(&self) -> Vec<Self> { Vec::new() }
}

#[cfg(feature="lua")]
impl UserData for GenericProperty {}


impl KeyAdressableElement<String> for Vec<u8>
{
  fn get_key(&self) -> String {
    format!("{:X}", sha2::Sha256::digest(&self))
  }
}

impl<E> SchemaElement<E> for Vec<u8>
{
  fn serialize(&self) -> Result<Self, E> {
    Ok(self.clone())
  }

  fn deserialize(data: &[u8]) -> Result<Self, E> {
    Ok(data.to_vec())
  }
}

impl NestableProperty for Vec<u8> {
  fn nested(&self) -> Vec<Self> { Vec::new() }
}
