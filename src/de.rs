use std::io;

use serde::de::{self, Deserialize, DeserializeOwned, DeserializeSeed, SeqAccess, Visitor};

use error;


pub struct Deserializer<R: io::Read> {
    reader: R,
}

impl<R: io::Read> Deserializer<R> {
    fn with_reader(reader: R) -> Deserializer<R> {
        Deserializer { reader: reader }
    }
}


impl<'de, 'a, R> de::Deserializer<'de> for &'a mut Deserializer<R>
    where R: io::Read
{
    type Error = error::Error;

    fn deserialize_any<V>(mut self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        let value = visitor.visit_seq(Combinator::new(&mut self))?;

        Ok(value)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }
}


struct Combinator<'a, R: 'a + io::Read> {
    de: &'a mut Deserializer<R>,
}

impl<'a, R: io::Read> Combinator<'a, R> {
    fn new(de: &'a mut Deserializer<R>) -> Combinator<'a, R> {
        Combinator { de: de }
    }
}

impl<'de, 'a, R> SeqAccess<'de> for Combinator<'a, R>
    where R: 'a + io::Read
{
    type Error = error::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> error::Result<Option<T::Value>>
        where T: DeserializeSeed<'de>
    {
        seed.deserialize(&mut *self.de).map(Some)
    }
}


pub fn from_slice<'a, T>(slice: &'a [u8]) -> error::Result<T>
    where T: Deserialize<'a>
{
    let mut de = Deserializer::with_reader(slice);
    let value = Deserialize::deserialize(&mut de)?;

    Ok(value)
}

pub fn from_reader<R, T>(reader: R) -> error::Result<T>
    where R: io::Read,
          T: DeserializeOwned,
{
    let mut de = Deserializer::with_reader(reader);
    let value = Deserialize::deserialize(&mut de)?;

    Ok(value)
}
