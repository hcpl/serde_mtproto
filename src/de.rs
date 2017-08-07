use std::io;

use byteorder::{ReadBytesExt, LittleEndian};
use num_traits::ToPrimitive;
use serde::de::{self, Deserialize, DeserializeOwned, DeserializeSeed, Visitor};

use common::{FALSE_ID, TRUE_ID};
use error::{self, DeErrorKind};
use identifiable::{Identifiable, Wrapper};


pub struct Deserializer<R: io::Read> {
    reader: R,
    enum_variant_id: Option<&'static str>,
}

impl<R: io::Read> Deserializer<R> {
    pub fn new(reader: R, enum_variant_id: Option<&'static str>) -> Deserializer<R> {
        Deserializer {
            reader: reader,
            enum_variant_id: enum_variant_id,
        }
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


macro_rules! impl_deserialize_small_int {
    ($small_deserialize:ident, $small_visit:ident, $cast_to_small:ident,
     $big_read:ident::<$big_endianness:ident>
    ) => {
        fn $small_deserialize<V>(self, visitor: V) -> error::Result<V::Value>
            where V: Visitor<'de>
        {
            let value = self.reader.$big_read::<$big_endianness>()?;
            let casted = value.$cast_to_small()
                .ok_or(error::Error::from(DeErrorKind::IntegerOverflowingCast))?;

            visitor.$small_visit(casted)
        }
    };
}

macro_rules! impl_deserialize_big_int {
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

    impl_deserialize_small_int!(deserialize_i8,  visit_i8,  to_i8,  read_i32::<LittleEndian>);
    impl_deserialize_small_int!(deserialize_i16, visit_i16, to_i16, read_i32::<LittleEndian>);
    impl_deserialize_big_int!(deserialize_i32, read_i32::<LittleEndian>, visit_i32);
    impl_deserialize_big_int!(deserialize_i64, read_i64::<LittleEndian>, visit_i64);

    impl_deserialize_small_int!(deserialize_u8,  visit_u8,  to_u8,  read_u32::<LittleEndian>);
    impl_deserialize_small_int!(deserialize_u16, visit_u16, to_u16, read_u32::<LittleEndian>);
    impl_deserialize_big_int!(deserialize_u32, read_u32::<LittleEndian>, visit_u32);
    impl_deserialize_big_int!(deserialize_u64, read_u64::<LittleEndian>, visit_u64);

    impl_deserialize_big_int!(deserialize_f32, read_f32::<LittleEndian>, visit_f32);
    impl_deserialize_big_int!(deserialize_f64, read_f64::<LittleEndian>, visit_f64);

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

    fn deserialize_option<V>(self, _visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!()
    }

    fn deserialize_unit<V>(self, _visitor: V) -> error::Result<V::Value>
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
        let len = self.reader.read_u32::<LittleEndian>()?;

        visitor.visit_seq(SeqAccess::new(&mut self, len))
    }

    fn deserialize_tuple<V>(mut self, len: usize, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        let len_u32 = len.to_u32().ok_or(DeErrorKind::IntegerOverflowingCast)?;

        visitor.visit_seq(SeqAccess::new(&mut self, len_u32))
    }

    fn deserialize_tuple_struct<V>(self, _name: &'static str, _len: usize, _visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!()
    }

    fn deserialize_map<V>(self, _visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!()
    }

    fn deserialize_struct<V>(mut self, _name: &'static str, fields: &'static [&'static str], visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        let len_u32 = fields.len().to_u32().ok_or(DeErrorKind::IntegerOverflowingCast)?;

        visitor.visit_seq(SeqAccess::new(&mut self, len_u32))
    }

    fn deserialize_enum<V>(mut self, _name: &'static str, _variants: &'static [&'static str], visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_enum(EnumVariantAccess::new(&mut self))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        let variant_id = self.enum_variant_id.unwrap();

        visitor.visit_str(variant_id)
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!()
    }
}


struct SeqAccess<'a, R: 'a + io::Read> {
    de: &'a mut Deserializer<R>,
    next_index: u32,
    count: u32,
}

impl<'a, R: io::Read> SeqAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>, count: u32) -> SeqAccess<'a, R> {
        SeqAccess {
            de: de,
            next_index: 0,
            count: count,
        }
    }
}

impl<'de, 'a, R> de::SeqAccess<'de> for SeqAccess<'a, R>
    where R: 'a + io::Read
{
    type Error = error::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> error::Result<Option<T::Value>>
        where T: DeserializeSeed<'de>
    {
        if self.next_index < self.count {
            self.next_index += 1;
        } else {
            return Ok(None);
        }

        seed.deserialize(&mut *self.de).map(Some)
    }
}


struct EnumVariantAccess<'a, R: 'a + io::Read> {
    de: &'a mut Deserializer<R>,
}

impl<'a, R: io::Read> EnumVariantAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>) -> EnumVariantAccess<'a, R> {
        EnumVariantAccess { de: de }
    }
}

impl<'de, 'a, R> de::EnumAccess<'de> for EnumVariantAccess<'a, R>
    where R: 'a + io::Read
{
    type Error = error::Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> error::Result<(V::Value, Self::Variant)>
        where V: DeserializeSeed<'de>
    {
        let value = seed.deserialize(&mut *self.de)?;

        Ok((value, self))
    }
}

impl<'de, 'a, R> de::VariantAccess<'de> for EnumVariantAccess<'a, R>
    where R: 'a + io::Read
{
    type Error = error::Error;

    fn unit_variant(self) -> error::Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> error::Result<T::Value>
        where T: DeserializeSeed<'de>
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        de::Deserializer::deserialize_tuple_struct(self.de, "", len, visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        de::Deserializer::deserialize_struct(self.de, "", fields, visitor)
    }
}


pub fn from_bytes<'a, T>(bytes: &'a [u8], enum_variant_id: Option<&'static str>) -> error::Result<T>
    where T: Deserialize<'a>
{
    let mut de = Deserializer::new(bytes, enum_variant_id);
    let value: T = Deserialize::deserialize(&mut de)?;

    Ok(value)
}

pub fn from_bytes_identifiable<'a, T>(bytes: &'a [u8], enum_variant_id: Option<&'static str>) -> error::Result<T>
    where T: Deserialize<'a> + Identifiable
{
    let wrapper: Wrapper<T> = from_bytes(bytes, enum_variant_id)?;

    Ok(wrapper.take_data())
}

pub fn from_reader<R, T>(reader: R, enum_variant_id: Option<&'static str>) -> error::Result<T>
    where R: io::Read,
          T: DeserializeOwned,
{
    let mut de = Deserializer::new(reader, enum_variant_id);
    let value: T = Deserialize::deserialize(&mut de)?;

    Ok(value)
}

pub fn from_reader_identifiable<R, T>(reader: R, enum_variant_id: Option<&'static str>) -> error::Result<T>
    where R: io::Read,
          T: DeserializeOwned + Identifiable,
{
    let wrapper: Wrapper<T> = from_reader(reader, enum_variant_id)?;

    Ok(wrapper.take_data())
}
