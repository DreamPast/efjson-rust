use crate::deserialize::{
  ArrayReceiverDeserializer, ArrayReceiverTrait, DefaultDeserializable, DeserError,
  create_array_deserializer,
};

#[derive(Debug)]
pub struct VecReceiver<Element: DefaultDeserializable<Element>> {
  vec: Vec<Element>,
}

impl<Element> ArrayReceiverTrait<Element, Vec<Element>, Element::DefaultDeserializer>
  for VecReceiver<Element>
where
  Element: DefaultDeserializable<Element>,
{
  fn create_element(&mut self) -> Result<Element::DefaultDeserializer, DeserError> {
    Ok(Element::default_deserializer())
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
  Element: DefaultDeserializable<Element>,
{
  type DefaultDeserializer = ArrayReceiverDeserializer<
    Element,
    Vec<Element>,
    VecReceiver<Element>,
    Element::DefaultDeserializer,
  >;
  fn default_deserializer() -> Self::DefaultDeserializer {
    create_array_deserializer(VecReceiver { vec: Vec::new() })
  }
}

impl<Element, const N: usize>
  ArrayReceiverTrait<Element, [Element; N], Element::DefaultDeserializer> for VecReceiver<Element>
where
  Element: DefaultDeserializable<Element>,
{
  fn create_element(&mut self) -> Result<Element::DefaultDeserializer, DeserError> {
    Ok(Element::default_deserializer())
  }
  fn append(&mut self, element: Element) -> Result<(), DeserError> {
    if self.vec.len() == N {
      return Err("the length of the array is too long".into());
    }
    self.vec.push(element);
    Ok(())
  }
  fn end(&mut self) -> Result<[Element; N], DeserError> {
    std::mem::take(&mut self.vec)
      .try_into()
      .map_err(|v: Vec<Element>| format!("expected array of length {}, got {}", N, v.len()).into())
  }
}
impl<Element, const N: usize> DefaultDeserializable<[Element; N]> for [Element; N]
where
  Element: DefaultDeserializable<Element>,
{
  type DefaultDeserializer = ArrayReceiverDeserializer<
    Element,
    [Element; N],
    VecReceiver<Element>,
    Element::DefaultDeserializer,
  >;
  fn default_deserializer() -> Self::DefaultDeserializer {
    create_array_deserializer(VecReceiver { vec: Vec::with_capacity(N) })
  }
}
