use crate::deserialize::{
  ArrayReceiverTrait, DefaultDeserializable, DeserError, Deserializer, create_array_deserializer,
};

struct VecReceiver<Element: DefaultDeserializable<Element>> {
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
  fn default_deserializer() -> impl Deserializer<Vec<Element>> {
    create_array_deserializer(VecReceiver { vec: Vec::new() })
  }
}
