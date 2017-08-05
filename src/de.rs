use std::io;

use byteorder::{ReadBytesExt, LittleEndian};
use serde::de::{self, Deserialize, DeserializeOwned, DeserializeSeed, SeqAccess, Visitor};

use common::{FALSE_ID, TRUE_ID};
use error;
use identifiable::{Identifiable, Wrapper};


pub struct Deserializer<R: io::Read> {
    reader: R,
}

impl<R: io::Read> Deserializer<R> {
    fn with_reader(reader: R) -> Deserializer<R> {
        Deserializer { reader: reader }
    }

    fn get_str_info(&mut self) -> error::Result<(usize, usize)> {
        let first_byte = self.reader.read_u8()?;
        let len;
        let rem;

        if first_byte <= 253 {
            len = first_byte as usize;
            rem = (len + 1) % 4;
        } else if first_byte == 254 {
            len = self.reader.read_uint::<LittleEndian>(3)? as usize;
            rem = len % 4;
        } else { // 255
            unreachable!();
        }

        Ok((len, if rem > 0 { 4 - rem } else { 0 }))
    }

    fn read_string(&mut self) -> error::Result<String> {
        let (len, rem) = self.get_str_info()?;

        // Safe version of
        //     let mut s = String::with_capacity(len);
        //     unsafe {
        //         self.reader.read_exact(s.as_mut_vec())?;
        //     }
        let mut s_bytes = vec![0; len];
        self.reader.read_exact(&mut s_bytes)?;
        let s = String::from_utf8(s_bytes)?;

        let mut padding = vec![0; rem];
        self.reader.read_exact(&mut padding)?;

        Ok(s)
    }

    fn read_byte_buf(&mut self) -> error::Result<Vec<u8>> {
        let (len, rem) = self.get_str_info()?;

        let mut b = vec![0; len];
        self.reader.read_exact(&mut b)?;

        let mut padding = vec![0; rem];
        self.reader.read_exact(&mut padding)?;

        Ok(b)
    }
}


macro_rules! impl_deserialize {
    ($deserialize:ident, $read:ident, $visit:ident) => {
        fn $deserialize<V>(self, visitor: V) -> error::Result<V::Value>
            where V: Visitor<'de>
        {
            let value = self.reader.$read()?;

            visitor.$visit(value)
        }
    };

    ($deserialize:ident, $read:ident::<$endianness:ident>, $visit:ident) => {
        fn $deserialize<V>(self, visitor: V) -> error::Result<V::Value>
            where V: Visitor<'de>
        {
            let value = self.reader.$read::<$endianness>()?;

            visitor.$visit(value)
        }
    };
}

impl<'de, 'a, R> de::Deserializer<'de> for &'a mut Deserializer<R>
    where R: io::Read
{
    type Error = error::Error;

    fn deserialize_any<V>(self, _visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        Err("Non self-described format".into())
    }

    fn deserialize_bool<V>(self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        let id_value = self.reader.read_i32::<LittleEndian>()?;

        let value = match id_value {
            TRUE_ID => true,
            FALSE_ID => false,
            _ => return Err("Expected a bool".into())
        };

        visitor.visit_bool(value)
    }

    impl_deserialize!(deserialize_i8, read_i8, visit_i8);
    impl_deserialize!(deserialize_i16, read_i16::<LittleEndian>, visit_i16);
    impl_deserialize!(deserialize_i32, read_i32::<LittleEndian>, visit_i32);
    impl_deserialize!(deserialize_i64, read_i64::<LittleEndian>, visit_i64);

    impl_deserialize!(deserialize_u8, read_u8, visit_u8);
    impl_deserialize!(deserialize_u16, read_u16::<LittleEndian>, visit_u16);
    impl_deserialize!(deserialize_u32, read_u32::<LittleEndian>, visit_u32);
    impl_deserialize!(deserialize_u64, read_u64::<LittleEndian>, visit_u64);

    impl_deserialize!(deserialize_f32, read_f32::<LittleEndian>, visit_f32);
    impl_deserialize!(deserialize_f64, read_f64::<LittleEndian>, visit_f64);

    fn deserialize_char<V>(self, _visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        unreachable!("this method shouldn't be called")
    }

    fn deserialize_str<V>(self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        let s = self.read_string()?;
        visitor.visit_str(&s)
    }

    fn deserialize_string<V>(self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        let s = self.read_string()?;
        visitor.visit_string(s)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        let b = self.read_byte_buf()?;
        visitor.visit_bytes(&b)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        let b = self.read_byte_buf()?;
        visitor.visit_byte_buf(b)
    }

    fn deserialize_option<V>(self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        unreachable!("this method shouldn't be called")
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_seq(Combinator::without_count(&mut self))
    }

    fn deserialize_tuple<V>(mut self, len: usize, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_seq(Combinator::with_count(&mut self, len))
    }

    fn deserialize_tuple_struct<V>(self, name: &'static str, len: usize, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!()
    }

    fn deserialize_map<V>(self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!()
    }

    fn deserialize_struct<V>(mut self, name: &'static str, fields: &'static [&'static str], visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_seq(Combinator::with_count(&mut self, fields.len()))
    }

    fn deserialize_enum<V>(self, name: &'static str, variants: &'static [&'static str], visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!()
    }
}


struct Combinator<'a, R: 'a + io::Read> {
    de: &'a mut Deserializer<R>,
    next_index: usize,
    count: Option<usize>,
}

impl<'a, R: io::Read> Combinator<'a, R> {
    fn without_count(de: &'a mut Deserializer<R>) -> Combinator<'a, R> {
        Combinator {
            de: de,
            next_index: 0,
            count: None,
        }
    }

    fn with_count(de: &'a mut Deserializer<R>, count: usize) -> Combinator<'a, R> {
        Combinator {
            de: de,
            next_index: 0,
            count: Some(count),
        }
    }
}

impl<'de, 'a, R> SeqAccess<'de> for Combinator<'a, R>
    where R: 'a + io::Read
{
    type Error = error::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> error::Result<Option<T::Value>>
        where T: DeserializeSeed<'de>
    {
        if let Some(count) = self.count {
            if self.next_index < count {
                self.next_index += 1;
            } else {
                return Ok(None);
            }
        }

        seed.deserialize(&mut *self.de).map(Some)
    }
}


pub fn from_slice<'a, T>(slice: &'a [u8]) -> error::Result<T>
    where T: Deserialize<'a> + Identifiable
{
    let mut de = Deserializer::with_reader(slice);
    let wrapper: Wrapper<T> = Deserialize::deserialize(&mut de)?;

    Ok(wrapper.take_data())
}

pub fn from_reader<R, T>(reader: R) -> error::Result<T>
    where R: io::Read,
          T: DeserializeOwned + Identifiable,
{
    let mut de = Deserializer::with_reader(reader);
    let wrapper: Wrapper<T> = Deserialize::deserialize(&mut de)?;

    Ok(wrapper.take_data())
}
