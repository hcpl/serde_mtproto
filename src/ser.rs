use std::io;

use byteorder::{WriteBytesExt, LittleEndian};
use serde::ser::{self, Serialize};

use common::{FALSE_ID, TRUE_ID};
use error;


pub struct Serializer<W: io::Write> {
    writer: W,
}

impl<W: io::Write> Serializer<W> {
    fn with_writer(writer: W) -> Serializer<W> {
        Serializer { writer: writer }
    }
}


macro_rules! impl_serialize {
    ($type:ty, $method:ident, $write:path) => {
        fn $method(self, value: $type) -> error::Result<()> {
            $write(&mut self.writer, value)?;
            Ok(())
        }
    };
}

impl<'a, W> ser::Serializer for &'a mut Serializer<W>
    where W: io::Write
{
    type Ok = ();
    type Error = error::Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;


    fn serialize_bool(self, value: bool) -> error::Result<()> {
        self.writer.write_i32::<LittleEndian>(if value { TRUE_ID } else { FALSE_ID })?;
        Ok(())
    }

    fn serialize_i8(self, value: i8) -> error::Result<()> {
        self.writer.write_all(&[value as u8])?;
        Ok(())
    }

    fn serialize_u8(self, value: u8) -> error::Result<()> {
        self.writer.write_all(&[value])?;
        Ok(())
    }

    impl_serialize!(i16, serialize_i16, WriteBytesExt::write_i16<LittleEndian>);
    impl_serialize!(i32, serialize_i32, WriteBytesExt::write_i32<LittleEndian>);
    impl_serialize!(i64, serialize_i64, WriteBytesExt::write_i64<LittleEndian>);

    impl_serialize!(u16, serialize_u16, WriteBytesExt::write_u16<LittleEndian>);
    impl_serialize!(u32, serialize_u32, WriteBytesExt::write_u32<LittleEndian>);
    impl_serialize!(u64, serialize_u64, WriteBytesExt::write_u64<LittleEndian>);

    impl_serialize!(f32, serialize_f32, WriteBytesExt::write_f32<LittleEndian>);
    impl_serialize!(f64, serialize_f64, WriteBytesExt::write_f64<LittleEndian>);

    fn serialize_char(self, _value: char) -> error::Result<()> {
        unreachable!("this method shouldn't be called")
    }

    fn serialize_str(self, value: &str) -> error::Result<()> {
        // TODO: unicode length?
        let len = value.len();
        let rem;

        if len <= 253 {
            // If L <= 253, the serialization contains one byte with the value of L,
            // then L bytes of the string followed by 0 to 3 characters containing 0,
            // such that the overall length of the value be divisible by 4,
            // whereupon all of this is interpreted as a sequence
            // of int(L/4)+1 32-bit little-endian integers.

            self.writer.write_all(&[len as u8])?;

            rem = (len + 1) % 4;
        } else {
            // If L >= 254, the serialization contains byte 254, followed by 3
            // bytes with the string length L in little-endian order, followed by L
            // bytes of the string, further followed by 0 to 3 null padding bytes.

            self.writer.write_all(&[254])?;
            self.writer.write_uint::<LittleEndian>(len as u64, 3)?;

            rem = len % 4;
        }

        // Write each character in the string
        self.writer.write_all(value.as_bytes())?;

        // [...] string followed by 0 to 3 characters containing 0,
        // such that the overall length of the value be divisible by 4 [...]
        if rem > 0 {
            for _ in 0..(4 - rem) {
                self.writer.write_all(&[0])?;
            }
        }

        Ok(())
    }

    fn serialize_bytes(self, value: &[u8]) -> error::Result<()> {
        let len = value.len();
        let rem;

        if len <= 253 {
            // If L <= 253, the serialization contains one byte with the value of L,
            // then L bytes of the string followed by 0 to 3 characters containing 0,
            // such that the overall length of the value be divisible by 4,
            // whereupon all of this is interpreted as a sequence
            // of int(L/4)+1 32-bit little-endian integers.

            self.writer.write_all(&[len as u8])?;

            rem = (len + 1) % 4;
        } else {
            // If L >= 254, the serialization contains byte 254, followed by 3
            // bytes with the string length L in little-endian order, followed by L
            // bytes of the string, further followed by 0 to 3 null padding bytes.

            self.writer.write_all(&[254])?;
            self.writer.write_uint::<LittleEndian>(len as u64, 3)?;

            rem = len % 4;
        }

        // Write each character in the string
        self.writer.write_all(value)?;

        // [...] string followed by 0 to 3 characters containing 0,
        // such that the overall length of the value be divisible by 4 [...]
        if rem > 0 {
            for _ in 0..(4 - rem) {
                self.writer.write_all(&[0])?;
            }
        }

        Ok(())
    }

    fn serialize_none(self) -> error::Result<()> {
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> error::Result<()> {
        unreachable!("this method shouldn't be called")
    }

    fn serialize_unit_struct(self, _name: &'static str) -> error::Result<()> {
        Ok(())
    }

    fn serialize_unit_variant(self,
                              _name: &'static str,
                              _variant_index: u32,
                              _variant: &'static str)
                             -> error::Result<()> {
        Ok(())
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(self,
                                    _name: &'static str,
                                    _variant_index: u32,
                                    _variant: &'static str,
                                    value: &T)
                                   -> error::Result<()>
        where T: ?Sized + Serialize
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> error::Result<Self> {
        unreachable!("this method shouldn't be called")
    }

    fn serialize_tuple(self, _len: usize) -> error::Result<Self> {
        Ok(self)
    }

    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> error::Result<Self> {
        unreachable!("this method shouldn't be called")
    }

    fn serialize_tuple_variant(self,
                               _name: &'static str,
                               _variant_index: u32,
                               _variant: &'static str,
                               _len: usize)
                              -> error::Result<Self> {
        unreachable!("this method shouldn't be called")
    }

    fn serialize_map(self, _len: Option<usize>) -> error::Result<Self> {
        unreachable!("this method shouldn't be called")
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> error::Result<Self> {
        Ok(self)
    }

    fn serialize_struct_variant(self,
                                _name: &'static str,
                                _variant_index: u32,
                                _variant: &'static str,
                                _len: usize)
                               -> error::Result<Self> {
        Ok(self)
    }
}

impl<'a, W> ser::SerializeSeq for &'a mut Serializer<W>
    where W: io::Write
{
    type Ok = ();
    type Error = error::Error;

    fn serialize_element<T>(&mut self, _value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        unreachable!("this method shouldn't be called")
    }

    fn end(self) -> error::Result<()> {
        unreachable!("this method shouldn't be called")
    }
}

impl<'a, W> ser::SerializeTuple for &'a mut Serializer<W>
    where W: io::Write
{
    type Ok = ();
    type Error = error::Error;

    fn serialize_element<T>(&mut self, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> error::Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleStruct for &'a mut Serializer<W>
    where W: io::Write
{
    type Ok = ();
    type Error = error::Error;

    fn serialize_field<T>(&mut self, _value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        unreachable!("this method shouldn't be called")
    }

    fn end(self) -> error::Result<()> {
        unreachable!("this method shouldn't be called")
    }
}

impl<'a, W> ser::SerializeTupleVariant for &'a mut Serializer<W>
    where W: io::Write
{
    type Ok = ();
    type Error = error::Error;

    fn serialize_field<T>(&mut self, _value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        unreachable!("this method shouldn't be called")
    }

    fn end(self) -> error::Result<()> {
        unreachable!("this method shouldn't be called")
    }
}

impl<'a, W> ser::SerializeMap for &'a mut Serializer<W>
    where W: io::Write
{
    type Ok = ();
    type Error = error::Error;

    fn serialize_key<T>(&mut self, _key: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        unreachable!("this method shouldn't be called")
    }

    fn serialize_value<T>(&mut self, _value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        unreachable!("this method shouldn't be called")
    }

    fn end(self) -> error::Result<()> {
        unreachable!("this method shouldn't be called")
    }
}

impl<'a, W> ser::SerializeStruct for &'a mut Serializer<W>
    where W: io::Write
{
    type Ok = ();
    type Error = error::Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> error::Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeStructVariant for &'a mut Serializer<W>
    where W: io::Write
{
    type Ok = ();
    type Error = error::Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> error::Result<()> {
        Ok(())
    }
}


pub fn to_vec<T>(value: &T) -> error::Result<Vec<u8>>
    where T: Serialize
{
    let mut ser = Serializer::with_writer(Vec::new());
    value.serialize(&mut ser)?;
    Ok(ser.writer)
}

pub fn to_writer<W, T>(writer: W, value: &T) -> error::Result<()>
    where W: io::Write,
          T: Serialize,
{
    let mut ser = Serializer::with_writer(writer);
    value.serialize(&mut ser)?;
    Ok(())
}
