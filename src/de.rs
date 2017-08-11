//! Deserialize MTProto binary representation to a Rust data structure.

use std::io;

use byteorder::{ReadBytesExt, LittleEndian};
use serde::de::{self, Deserialize, DeserializeOwned, DeserializeSeed, Visitor};

use error::{self, DeErrorKind, DeSerdeType};
use identifiable::{BOOL_FALSE_ID, BOOL_TRUE_ID};
use utils::safe_cast;


/// A structure that deserializes  MTProto binary representation into Rust values.
pub struct Deserializer<R: io::Read> {
    reader: R,
    enum_variant_id: Option<&'static str>,
}

impl<R: io::Read> Deserializer<R> {
    /// Create a MTProto deserializer from an `io::Read` and enum variant hint.
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
        } else { // must be 255
            assert_eq!(first_byte, 255);
            return Err(de::Error::invalid_value(
                de::Unexpected::Unsigned(255),
                &"a byte in [0..254] range"));
        }

        let padding = (4 - rem) % 4;

        Ok((len, padding))
    }

    fn read_string(&mut self) -> error::Result<String> {
        let (len, padding) = self.get_str_info()?;

        // Safe version of
        //     let mut s = String::with_capacity(len);
        //     unsafe {
        //         self.reader.read_exact(s.as_mut_vec())?;
        //     }
        let mut s_bytes = vec![0; len];
        self.reader.read_exact(&mut s_bytes)?;
        let s = String::from_utf8(s_bytes)?;

        let mut p = vec![0; padding];
        self.reader.read_exact(&mut p)?;

        Ok(s)
    }

    fn read_byte_buf(&mut self) -> error::Result<Vec<u8>> {
        let (len, padding) = self.get_str_info()?;

        let mut b = vec![0; len];
        self.reader.read_exact(&mut b)?;

        let mut p = vec![0; padding];
        self.reader.read_exact(&mut p)?;

        Ok(b)
    }
}


macro_rules! impl_deserialize_small_int {
    ($small_type:ty, $small_deserialize:ident, $big_read:ident::<$big_endianness:ident>,
     $small_visit:ident
    ) => {
        fn $small_deserialize<V>(self, visitor: V) -> error::Result<V::Value>
            where V: Visitor<'de>
        {
            let value = self.reader.$big_read::<$big_endianness>()?;
            debug!("Deserialized big int: {:#x}", value);
            let casted = safe_cast(value)?;
            debug!("Casted to {}: {:#x}", stringify!($small_type), casted);

            visitor.$small_visit(casted)
        }
    };
}

macro_rules! impl_deserialize_big_int {
    ($type:ty, $deserialize:ident, $read:ident::<$endianness:ident>, $visit:ident) => {
        fn $deserialize<V>(self, visitor: V) -> error::Result<V::Value>
            where V: Visitor<'de>
        {
            let value = self.reader.$read::<$endianness>()?;
            debug!("Deserialized {}: {:#x}", stringify!($type), value);

            visitor.$visit(value)
        }
    };
}

macro_rules! impl_deserialize_float {
    ($type:ty, $deserialize:ident, $read:ident::<$endianness:ident>, $visit:ident) => {
        fn $deserialize<V>(self, visitor: V) -> error::Result<V::Value>
            where V: Visitor<'de>
        {
            let value = self.reader.$read::<$endianness>()?;
            debug!("Deserialized {}: {}", stringify!($type), value);

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
        bail!(DeErrorKind::UnsupportedSerdeType(DeSerdeType::Any));
    }

    fn deserialize_bool<V>(self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        let id_value = self.reader.read_i32::<LittleEndian>()?;

        let value = match id_value {
            BOOL_FALSE_ID => false,
            BOOL_TRUE_ID => true,
            _ => {
                return Err(de::Error::invalid_value(
                    de::Unexpected::Signed(id_value as i64),
                    &format!("either {} for false or {} for true", BOOL_FALSE_ID, BOOL_TRUE_ID).as_str()));
            }
        };

        debug!("Deserialized bool: {}", value);

        visitor.visit_bool(value)
    }

    impl_deserialize_small_int!(i8,  deserialize_i8,  read_i32::<LittleEndian>, visit_i8);
    impl_deserialize_small_int!(i16, deserialize_i16, read_i32::<LittleEndian>, visit_i16);
    impl_deserialize_big_int!(i32, deserialize_i32, read_i32::<LittleEndian>, visit_i32);
    impl_deserialize_big_int!(i64, deserialize_i64, read_i64::<LittleEndian>, visit_i64);

    impl_deserialize_small_int!(u8,  deserialize_u8,  read_u32::<LittleEndian>, visit_u8);
    impl_deserialize_small_int!(u16, deserialize_u16, read_u32::<LittleEndian>, visit_u16);
    impl_deserialize_big_int!(u32, deserialize_u32, read_u32::<LittleEndian>, visit_u32);
    impl_deserialize_big_int!(u64, deserialize_u64, read_u64::<LittleEndian>, visit_u64);

    impl_deserialize_float!(f32, deserialize_f32, read_f32::<LittleEndian>, visit_f32);
    impl_deserialize_float!(f64, deserialize_f64, read_f64::<LittleEndian>, visit_f64);

    fn deserialize_char<V>(self, _visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        bail!(DeErrorKind::UnsupportedSerdeType(DeSerdeType::Char));
    }

    fn deserialize_str<V>(self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        let s = self.read_string()?;
        debug!("Deserialized str: {:?}", s);
        visitor.visit_str(&s)
    }

    fn deserialize_string<V>(self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        let s = self.read_string()?;
        debug!("Deserialized string: {:?}", s);
        visitor.visit_string(s)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        let b = self.read_byte_buf()?;
        debug!("Deserialized bytes: {:?}", b);
        visitor.visit_bytes(&b)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        let b = self.read_byte_buf()?;
        debug!("Deserialized byte buffer: {:?}", b);
        visitor.visit_byte_buf(b)
    }

    fn deserialize_option<V>(self, _visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        bail!(DeErrorKind::UnsupportedSerdeType(DeSerdeType::Option));
    }

    fn deserialize_unit<V>(self, _visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        bail!(DeErrorKind::UnsupportedSerdeType(DeSerdeType::Unit));
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

    fn deserialize_seq<V>(self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        let len = self.reader.read_u32::<LittleEndian>()?;

        visitor.visit_seq(SeqAccess::new(self, len))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_seq(SeqAccess::new(self, safe_cast(len)?))
    }

    fn deserialize_tuple_struct<V>(self, _name: &'static str, len: usize, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_seq(SeqAccess::new(self, safe_cast(len)?))
    }

    fn deserialize_map<V>(self, visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        let len = self.reader.read_u32::<LittleEndian>()?;

        visitor.visit_map(MapAccess::new(self, len))
    }

    fn deserialize_struct<V>(self, _name: &'static str, fields: &'static [&'static str], visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_seq(SeqAccess::new(self, safe_cast(fields.len())?))
    }

    fn deserialize_enum<V>(self, _name: &'static str, _variants: &'static [&'static str], visitor: V) -> error::Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_enum(EnumVariantAccess::new(self))
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
        bail!(DeErrorKind::UnsupportedSerdeType(DeSerdeType::IgnoredAny));
    }
}


struct SeqAccess<'a, R: 'a + io::Read> {
    de: &'a mut Deserializer<R>,
    len: u32,
    next_index: u32,
}

impl<'a, R: io::Read> SeqAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>, len: u32) -> SeqAccess<'a, R> {
        SeqAccess {
            de: de,
            next_index: 0,
            len: len,
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
        if self.next_index < self.len {
            self.next_index += 1;
        } else {
            return Ok(None);
        }

        seed.deserialize(&mut *self.de).map(Some)
    }
}


struct MapAccess<'a, R: 'a + io::Read> {
    de: &'a mut Deserializer<R>,
    len: u32,
    next_index: u32,
}

impl<'a, R: io::Read> MapAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>, len: u32) -> MapAccess<'a, R> {
        MapAccess {
            de: de,
            next_index: 0,
            len: len,
        }
    }
}

impl<'de, 'a, R> de::MapAccess<'de> for MapAccess<'a, R>
    where R: 'a + io::Read
{
    type Error = error::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> error::Result<Option<K::Value>>
        where K: DeserializeSeed<'de>
    {
        if self.next_index < self.len {
            self.next_index += 1;
        } else {
            return Ok(None);
        }

        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> error::Result<V::Value>
        where V: DeserializeSeed<'de>
    {
        seed.deserialize(&mut *self.de)
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


/// Deserialize an instance of type `T` from bytes of binary MTProto.
pub fn from_bytes<'a, T>(bytes: &'a [u8], enum_variant_id: Option<&'static str>) -> error::Result<T>
    where T: Deserialize<'a>
{
    let mut de = Deserializer::new(bytes, enum_variant_id);
    let value: T = Deserialize::deserialize(&mut de)?;

    Ok(value)
}

/// Deserialize an instance of type `T` from an IO stream of binary MTProto.
pub fn from_reader<R, T>(reader: R, enum_variant_id: Option<&'static str>) -> error::Result<T>
    where R: io::Read,
          T: DeserializeOwned,
{
    let mut de = Deserializer::new(reader, enum_variant_id);
    let value: T = Deserialize::deserialize(&mut de)?;

    Ok(value)
}
