use byteorder::{WriteBytesExt, LittleEndian};
use serde::ser::{self, Serialize};

use error;


pub struct Serializer {
    // This vector starts empty and bytes are appended as values are serialized.
    output: Vec<u8>,
}

impl Serializer {
    fn new() -> Serializer {
        Serializer { output: Vec::new() }
    }
}


macro_rules! impl_serialize {
    ($type:ty, $method:ident, $write:path) => {
        fn $method(self, value: $type) -> error::Result<()> {
            $write(&mut self.output, value)?;
            Ok(())
        }
    };
}

const TRUE_ID: i32 = -1720552011;
const FALSE_ID: i32 = -1132882121;

impl<'a> ser::Serializer for &'a mut Serializer {
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
        self.output.write_i32::<LittleEndian>(if value { TRUE_ID } else { FALSE_ID })?;
        Ok(())
    }

    fn serialize_i8(self, value: i8) -> error::Result<()> {
        self.output.push(value as u8);
        Ok(())
    }

    fn serialize_u8(self, value: u8) -> error::Result<()> {
        self.output.push(value);
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

    fn serialize_char(self, value: char) -> error::Result<()> {
        unimplemented!()
    }

    fn serialize_str(self, value: &str) -> error::Result<()> {
        // TODO: unicode length?
        let len = value.len();

        if len <= 253 {
            // If L <= 253, the serialization contains one byte with the value of L,
            // then L bytes of the string followed by 0 to 3 characters containing 0,
            // such that the overall length of the value be divisible by 4,
            // whereupon all of this is interpreted as a sequence
            // of int(L/4)+1 32-bit little-endian integers.

            self.output.push(len as u8);
        } else {
            // If L >= 254, the serialization contains byte 254, followed by 3
            // bytes with the string length L in little-endian order, followed by L
            // bytes of the string, further followed by 0 to 3 null padding bytes.

            self.output.push(254);
            self.output.write_uint::<LittleEndian>(len as u64, 3)?;
        }

        // Write each character in the string
        self.output.extend(value.as_bytes());

        // [...] string followed by 0 to 3 characters containing 0,
        // such that the overall length of the value be divisible by 4 [...]
        let rem = len % 4;
        if rem > 0 {
            for _ in 0..(4 - rem) {
                self.output.push(0);
            }
        }

        Ok(())
    }

    fn serialize_bytes(self, value: &[u8]) -> error::Result<()> {
        let len = value.len();

        if len <= 253 {
            // If L <= 253, the serialization contains one byte with the value of L,
            // then L bytes of the string followed by 0 to 3 characters containing 0,
            // such that the overall length of the value be divisible by 4,
            // whereupon all of this is interpreted as a sequence
            // of int(L/4)+1 32-bit little-endian integers.

            self.output.push(len as u8);
        } else {
            // If L >= 254, the serialization contains byte 254, followed by 3
            // bytes with the string length L in little-endian order, followed by L
            // bytes of the string, further followed by 0 to 3 null padding bytes.

            self.output.push(254);
            self.output.write_uint::<LittleEndian>(len as u64, 3)?;
        }

        // Write each character in the string
        self.output.extend(value);

        // [...] string followed by 0 to 3 characters containing 0,
        // such that the overall length of the value be divisible by 4 [...]
        let rem = len % 4;
        if rem > 0 {
            for _ in 0..(4 - rem) {
                self.output.push(0);
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
        unimplemented!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> error::Result<()> {
        Ok(())
    }

    fn serialize_unit_variant(self,
                              _name: &'static str,
                              _variant_index: u32,
                              variant: &'static str)
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
                                    variant: &'static str,
                                    value: &T)
                                   -> error::Result<()>
        where T: ?Sized + Serialize
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> error::Result<Self> {
        unimplemented!()
    }

    fn serialize_tuple(self, len: usize) -> error::Result<Self> {
        unimplemented!()
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> error::Result<Self> {
        unimplemented!()
    }

    fn serialize_tuple_variant(self,
                               _name: &'static str,
                               _variant_index: u32,
                               variant: &'static str,
                               _len: usize)
                              -> error::Result<Self> {
        unimplemented!()
    }

    fn serialize_map(self, _len: Option<usize>) -> error::Result<Self> {
        unimplemented!()
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> error::Result<Self> {
        Ok(self)
    }

    fn serialize_struct_variant(self,
                                _name: &'static str,
                                _variant_index: u32,
                                variant: &'static str,
                                _len: usize)
                               -> error::Result<Self> {
        unimplemented!()
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = error::Error;

    fn serialize_element<T>(&mut self, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        unimplemented!()
    }

    fn end(self) -> error::Result<()> {
        unimplemented!()
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = error::Error;

    fn serialize_element<T>(&mut self, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        unimplemented!()
    }

    fn end(self) -> error::Result<()> {
        unimplemented!()
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = error::Error;

    fn serialize_field<T>(&mut self, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        unimplemented!()
    }

    fn end(self) -> error::Result<()> {
        unimplemented!()
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = error::Error;

    fn serialize_field<T>(&mut self, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        unimplemented!()
    }

    fn end(self) -> error::Result<()> {
        unimplemented!()
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = error::Error;

    fn serialize_key<T>(&mut self, key: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        unimplemented!()
    }

    fn serialize_value<T>(&mut self, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        unimplemented!()
    }

    fn end(self) -> error::Result<()> {
        unimplemented!()
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = error::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        value.serialize(self)
    }

    fn end(self) -> error::Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = error::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        value.serialize(self)
    }

    fn end(self) -> error::Result<()> {
        Ok(())
    }
}


pub fn to_vec<T>(value: &T) -> error::Result<Vec<u8>>
    where T: Serialize
{
    let mut ser = Serializer { output: Vec::new() };
    value.serialize(&mut ser)?;
    Ok(ser.output)
}
