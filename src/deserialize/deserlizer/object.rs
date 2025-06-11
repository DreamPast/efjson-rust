use crate::deserialize::{
  DefaultDeserializable, DeserError, Deserializer, ObjectReceiverTrait, create_object_deserializer,
};
use std::{
  collections::{BTreeMap, HashMap},
  hash::Hash,
};

struct HashMapReceiver<Key: DefaultDeserializable<Key>, Value: DefaultDeserializable<Value>> {
  map: HashMap<Key, Value>,
}
impl<'a, Key, Value> ObjectReceiverTrait<'a, Key, Value, HashMap<Key, Value>>
  for HashMapReceiver<Key, Value>
where
  Key: DefaultDeserializable<Key> + Hash + Eq + 'a,
  Value: DefaultDeserializable<Value> + 'a,
{
  fn start(&mut self) -> Result<(), DeserError> {
    Ok(())
  }

  fn create_key(&mut self) -> Result<Box<dyn Deserializer<Key> + 'a>, DeserError> {
    return Ok(Box::new(Key::default_deserializer()));
  }

  fn create_value(&mut self, _key: &Key) -> Result<Box<dyn Deserializer<Value> + 'a>, DeserError> {
    return Ok(Box::new(Value::default_deserializer()));
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
  fn default_deserializer() -> impl Deserializer<HashMap<Key, Value>> {
    create_object_deserializer(HashMapReceiver { map: HashMap::new() })
  }
}

struct BTreeMapReceiver<Key: DefaultDeserializable<Key>, Value: DefaultDeserializable<Value>> {
  map: BTreeMap<Key, Value>,
}
impl<'a, Key, Value> ObjectReceiverTrait<'a, Key, Value, BTreeMap<Key, Value>>
  for BTreeMapReceiver<Key, Value>
where
  Key: DefaultDeserializable<Key> + Ord + 'a,
  Value: DefaultDeserializable<Value> + 'a,
{
  fn start(&mut self) -> Result<(), DeserError> {
    Ok(())
  }

  fn create_key(&mut self) -> Result<Box<dyn Deserializer<Key> + 'a>, DeserError> {
    return Ok(Box::new(Key::default_deserializer()));
  }

  fn create_value(&mut self, _key: &Key) -> Result<Box<dyn Deserializer<Value> + 'a>, DeserError> {
    return Ok(Box::new(Value::default_deserializer()));
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
  Key: DefaultDeserializable<Key> + Ord + 'static,
  Value: DefaultDeserializable<Value> + 'static,
{
  fn default_deserializer() -> impl Deserializer<BTreeMap<Key, Value>> {
    create_object_deserializer(BTreeMapReceiver { map: BTreeMap::new() })
  }
}
