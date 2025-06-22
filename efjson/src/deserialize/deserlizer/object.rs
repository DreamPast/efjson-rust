use crate::deserialize::{
  create_object_deserializer, DefaultDeserializable, DeserError, ObjectReceiverDeserializer,
  ObjectReceiverTrait,
};
use std::{
  collections::{BTreeMap, HashMap},
  hash::Hash,
};

#[derive(Debug)]
pub struct HashMapReceiver<Key: DefaultDeserializable<Key>, Value: DefaultDeserializable<Value>> {
  map: HashMap<Key, Value>,
}
impl<Key, Value>
  ObjectReceiverTrait<
    Key,
    Value,
    HashMap<Key, Value>,
    <Key as DefaultDeserializable<Key>>::DefaultDeserializer,
    <Value as DefaultDeserializable<Value>>::DefaultDeserializer,
  > for HashMapReceiver<Key, Value>
where
  Key: DefaultDeserializable<Key> + Hash + Eq,
  Value: DefaultDeserializable<Value>,
{
  fn create_key(
    &mut self,
  ) -> Result<<Key as DefaultDeserializable<Key>>::DefaultDeserializer, DeserError> {
    return Ok(Key::default_deserializer());
  }

  fn create_value(
    &mut self,
    _key: &Key,
  ) -> Result<<Value as DefaultDeserializable<Value>>::DefaultDeserializer, DeserError> {
    return Ok(Value::default_deserializer());
  }

  fn set(&mut self, key: Key, value: Value) -> Result<(), DeserError> {
    self.map.insert(key, value);
    Ok(())
  }

  fn end(&mut self) -> Result<HashMap<Key, Value>, DeserError> {
    Ok(std::mem::take(&mut self.map))
  }
}
impl<Key, Value> DefaultDeserializable<HashMap<Key, Value>> for HashMap<Key, Value>
where
  Key: DefaultDeserializable<Key> + Hash + Eq + 'static,
  Value: DefaultDeserializable<Value> + 'static,
{
  type DefaultDeserializer = ObjectReceiverDeserializer<
    Key,
    Value,
    HashMap<Key, Value>,
    HashMapReceiver<Key, Value>,
    <Key as DefaultDeserializable<Key>>::DefaultDeserializer,
    <Value as DefaultDeserializable<Value>>::DefaultDeserializer,
  >;
  fn default_deserializer() -> Self::DefaultDeserializer {
    create_object_deserializer(HashMapReceiver { map: HashMap::new() })
  }
}

#[derive(Debug)]
pub struct BTreeMapReceiver<Key: DefaultDeserializable<Key>, Value: DefaultDeserializable<Value>> {
  map: BTreeMap<Key, Value>,
}
impl<Key, Value>
  ObjectReceiverTrait<
    Key,
    Value,
    BTreeMap<Key, Value>,
    <Key as DefaultDeserializable<Key>>::DefaultDeserializer,
    <Value as DefaultDeserializable<Value>>::DefaultDeserializer,
  > for BTreeMapReceiver<Key, Value>
where
  Key: DefaultDeserializable<Key> + Ord,
  Value: DefaultDeserializable<Value>,
{
  fn create_key(
    &mut self,
  ) -> Result<<Key as DefaultDeserializable<Key>>::DefaultDeserializer, DeserError> {
    return Ok(Key::default_deserializer());
  }

  fn create_value(
    &mut self,
    _key: &Key,
  ) -> Result<<Value as DefaultDeserializable<Value>>::DefaultDeserializer, DeserError> {
    return Ok(Value::default_deserializer());
  }

  fn set(&mut self, key: Key, value: Value) -> Result<(), DeserError> {
    self.map.insert(key, value);
    Ok(())
  }

  fn end(&mut self) -> Result<BTreeMap<Key, Value>, DeserError> {
    Ok(std::mem::take(&mut self.map))
  }
}
impl<Key, Value> DefaultDeserializable<BTreeMap<Key, Value>> for BTreeMap<Key, Value>
where
  Key: DefaultDeserializable<Key> + Ord,
  Value: DefaultDeserializable<Value>,
{
  type DefaultDeserializer = ObjectReceiverDeserializer<
    Key,
    Value,
    BTreeMap<Key, Value>,
    BTreeMapReceiver<Key, Value>,
    <Key as DefaultDeserializable<Key>>::DefaultDeserializer,
    <Value as DefaultDeserializable<Value>>::DefaultDeserializer,
  >;
  fn default_deserializer() -> Self::DefaultDeserializer {
    create_object_deserializer(BTreeMapReceiver { map: BTreeMap::new() })
  }
}
