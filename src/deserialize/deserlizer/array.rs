use crate::deserialize::{
  ArrayReceiverDeserializer, ArrayReceiverTrait, DefaultDeserializable, DeserError, Deserializer,
  create_array_deserializer,
};

#[derive(Debug)]
pub struct VecReceiver<Element: DefaultDeserializable<Element>> {
  vec: Vec<Element>,
}
impl<'a, Element> ArrayReceiverTrait<'a, Element, Vec<Element>> for VecReceiver<Element>
where
  Element: DefaultDeserializable<Element> + 'a,
{
  fn create_element(&mut self) -> Result<Box<dyn Deserializer<Element> + 'a>, DeserError> {
    Ok(Box::new(Element::default_deserializer()))
  }
  fn append(&mut self, element: Element) -> Result<(), DeserError> {
    self.vec.push(element);
    Ok(())
  }
  fn end(&mut self) -> Result<Vec<Element>, DeserError> {
    Ok(std::mem::take(&mut self.vec))
  }
}
impl<Element> DefaultDeserializable<Vec<Element>> for Vec<Element>
where
  Element: DefaultDeserializable<Element> + 'static,
{
  type DefaultDeserializer =
    ArrayReceiverDeserializer<'static, Element, Vec<Element>, VecReceiver<Element>>;
  fn default_deserializer() -> Self::DefaultDeserializer {
    create_array_deserializer(VecReceiver { vec: Vec::new() })
  }
}
