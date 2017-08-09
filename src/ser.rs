//! Serialize a Rust data structure into its MTProto binary representation.

use std::io;

use byteorder::{WriteBytesExt, LittleEndian};
use serde::ser::{self, Serialize};

use error::{self, SerErrorKind, SerSerdeType};
use identifiable::Identifiable;
use utils::safe_cast;


/// A structure for serializing Rust values into MTProto binary representation.
pub struct Serializer<W: io::Write> {
    writer: W,
}

impl<W: io::Write> Serializer<W> {
    /// Create a MTProto serializer from an `io::Write`.
    pub fn new(writer: W) -> Serializer<W> {
        Serializer { writer: writer }
    }
}


macro_rules! impl_serialize_small_int {
    ($small_type:ty, $small_method:ident, $big_type:ty, $big_method:ident) => {
        fn $small_method(self, value: $small_type) -> error::Result<()> {
            self.$big_method(value as $big_type)    // safe to cast from small to big
        }
    }
}

macro_rules! impl_serialize_big_int {
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

    type SerializeSeq = SerializeFixedLengthSeq<'a, W>;
    type SerializeTuple = SerializeFixedLengthSeq<'a, W>;
    type SerializeTupleStruct = SerializeFixedLengthSeq<'a, W>;
    type SerializeTupleVariant = SerializeFixedLengthSeq<'a, W>;
    type SerializeMap = SerializeMap<'a, W>;
    type SerializeStruct = SerializeFixedLengthSeq<'a, W>;
    type SerializeStructVariant = SerializeFixedLengthSeq<'a, W>;


    fn serialize_bool(self, value: bool) -> error::Result<()> {
        self.writer.write_i32::<LittleEndian>(value.get_id())?;
        Ok(())
    }

    impl_serialize_small_int!(i8,  serialize_i8,  i32, serialize_i32);
    impl_serialize_small_int!(i16, serialize_i16, i32, serialize_i32);
    impl_serialize_big_int!(i32, serialize_i32, WriteBytesExt::write_i32<LittleEndian>);
    impl_serialize_big_int!(i64, serialize_i64, WriteBytesExt::write_i64<LittleEndian>);

    impl_serialize_small_int!(u8,  serialize_u8,  u32, serialize_u32);
    impl_serialize_small_int!(u16, serialize_u16, u32, serialize_u32);
    impl_serialize_big_int!(u32, serialize_u32, WriteBytesExt::write_u32<LittleEndian>);
    impl_serialize_big_int!(u64, serialize_u64, WriteBytesExt::write_u64<LittleEndian>);

    impl_serialize_big_int!(f32, serialize_f32, WriteBytesExt::write_f32<LittleEndian>);
    impl_serialize_big_int!(f64, serialize_f64, WriteBytesExt::write_f64<LittleEndian>);

    fn serialize_char(self, _value: char) -> error::Result<()> {
        bail!(SerErrorKind::UnsupportedSerdeType(SerSerdeType::Char));
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

            self.writer.write_u8(len as u8)?;

            rem = (len + 1) % 4;
        } else if len <= 0xff_ff_ff {
            // If L >= 254, the serialization contains byte 254, followed by 3
            // bytes with the string length L in little-endian order, followed by L
            // bytes of the string, further followed by 0 to 3 null padding bytes.

            self.writer.write_u8(254)?;
            self.writer.write_uint::<LittleEndian>(len as u64, 3)?;

            rem = len % 4;
        } else {
            bail!(SerErrorKind::StringTooLong(len));
        }

        // Write each character in the string
        self.writer.write_all(value.as_bytes())?;

        // [...] string followed by 0 to 3 characters containing 0,
        // such that the overall length of the value be divisible by 4 [...]
        let padding = (4 - rem) % 4;
        self.writer.write_uint::<LittleEndian>(0, padding)?;

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

            self.writer.write_u8(len as u8)?;

            rem = (len + 1) % 4;
        } else if len <= 0xff_ff_ff {
            // If L >= 254, the serialization contains byte 254, followed by 3
            // bytes with the string length L in little-endian order, followed by L
            // bytes of the string, further followed by 0 to 3 null padding bytes.

            self.writer.write_u8(254)?;
            self.writer.write_uint::<LittleEndian>(len as u64, 3)?;

            rem = len % 4;
        } else {
            bail!(SerErrorKind::StringTooLong(len));
        }

        // Write each character in the string
        self.writer.write_all(value)?;

        // [...] string followed by 0 to 3 characters containing 0,
        // such that the overall length of the value be divisible by 4 [...]
        let padding = (4 - rem) % 4;
        self.writer.write_uint::<LittleEndian>(0, padding)?;

        Ok(())
    }

    fn serialize_none(self) -> error::Result<()> {
        bail!(SerErrorKind::UnsupportedSerdeType(SerSerdeType::None));
    }

    fn serialize_some<T>(self, _value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        bail!(SerErrorKind::UnsupportedSerdeType(SerSerdeType::Some));
    }

    fn serialize_unit(self) -> error::Result<()> {
        bail!(SerErrorKind::UnsupportedSerdeType(SerSerdeType::Unit));
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

    fn serialize_seq(self, len: Option<usize>) -> error::Result<Self::SerializeSeq> {
        if let Some(len) = len {
            SerializeFixedLengthSeq::with_serialize_len(self, safe_cast(len)?)
        } else {
            bail!(SerErrorKind::SeqsWithUnknownLengthUnsupported);
        }
    }

    fn serialize_tuple(self, len: usize) -> error::Result<Self::SerializeTuple> {
        Ok(SerializeFixedLengthSeq::new(self, safe_cast(len)?))
    }

    fn serialize_tuple_struct(self,
                              _name: &'static str,
                              len: usize)
                             -> error::Result<Self::SerializeTupleStruct> {
        Ok(SerializeFixedLengthSeq::new(self, safe_cast(len)?))
    }

    fn serialize_tuple_variant(self,
                               _name: &'static str,
                               _variant_index: u32,
                               _variant: &'static str,
                               len: usize)
                              -> error::Result<Self::SerializeTupleVariant> {
        Ok(SerializeFixedLengthSeq::new(self, safe_cast(len)?))
    }

    fn serialize_map(self, len: Option<usize>) -> error::Result<Self::SerializeMap> {
        if let Some(len) = len {
            SerializeMap::with_serialize_len(self, safe_cast(len)?)
        } else {
            bail!(SerErrorKind::MapsWithUnknownLengthUnsupported);
        }
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> error::Result<Self::SerializeStruct> {
        Ok(SerializeFixedLengthSeq::new(self, safe_cast(len)?))
    }

    fn serialize_struct_variant(self,
                                _name: &'static str,
                                _variant_index: u32,
                                _variant: &'static str,
                                len: usize)
                               -> error::Result<Self::SerializeStructVariant> {
        Ok(SerializeFixedLengthSeq::new(self, safe_cast(len)?))
    }
}


/// Helper structure for serializing fixed-length sequences.
pub struct SerializeFixedLengthSeq<'a, W: 'a + io::Write> {
    ser: &'a mut Serializer<W>,
    len: u32,
    next_index: u32,
}

impl<'a, W: io::Write> SerializeFixedLengthSeq<'a, W> {
    fn new(ser: &'a mut Serializer<W>, len: u32) -> SerializeFixedLengthSeq<'a, W> {
        SerializeFixedLengthSeq {
            ser: ser,
            len: len,
            next_index: 0,
        }
    }

    fn with_serialize_len(ser: &'a mut Serializer<W>, len: u32) -> error::Result<SerializeFixedLengthSeq<'a, W>> {
        ser::Serializer::serialize_u32(&mut *ser, len)?;

        Ok(SerializeFixedLengthSeq::new(ser, len))
    }

    fn impl_serialize_seq_value<T>(&mut self, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        if self.next_index < self.len {
            self.next_index += 1;
        } else {
            bail!(SerErrorKind::ExcessElements(self.len));
        }

        value.serialize(&mut *self.ser)
    }
}

impl<'a, W> ser::SerializeSeq for SerializeFixedLengthSeq<'a, W>
    where W: 'a + io::Write
{
    type Ok = ();
    type Error = error::Error;

    fn serialize_element<T>(&mut self, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        self.impl_serialize_seq_value(value)
    }

    fn end(self) -> error::Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTuple for SerializeFixedLengthSeq<'a, W>
    where W: 'a + io::Write
{
    type Ok = ();
    type Error = error::Error;

    fn serialize_element<T>(&mut self, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        self.impl_serialize_seq_value(value)
    }

    fn end(self) -> error::Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleStruct for SerializeFixedLengthSeq<'a, W>
    where W: io::Write
{
    type Ok = ();
    type Error = error::Error;

    fn serialize_field<T>(&mut self, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        self.impl_serialize_seq_value(value)
    }

    fn end(self) -> error::Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleVariant for SerializeFixedLengthSeq<'a, W>
    where W: io::Write
{
    type Ok = ();
    type Error = error::Error;

    fn serialize_field<T>(&mut self, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        self.impl_serialize_seq_value(value)
    }

    fn end(self) -> error::Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeStruct for SerializeFixedLengthSeq<'a, W>
    where W: io::Write
{
    type Ok = ();
    type Error = error::Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        self.impl_serialize_seq_value(value)
    }

    fn end(self) -> error::Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeStructVariant for SerializeFixedLengthSeq<'a, W>
    where W: io::Write
{
    type Ok = ();
    type Error = error::Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        self.impl_serialize_seq_value(value)
    }

    fn end(self) -> error::Result<()> {
        Ok(())
    }
}


/// Helper structure for serializing maps.
pub struct SerializeMap<'a, W: 'a + io::Write> {
    ser: &'a mut Serializer<W>,
    len: u32,
    next_index: u32,
}

impl<'a, W: io::Write> SerializeMap<'a, W> {
    fn with_serialize_len(ser: &'a mut Serializer<W>, len: u32) -> error::Result<SerializeMap<'a, W>> {
        ser::Serializer::serialize_u32(&mut *ser, len)?;

        Ok(SerializeMap {
            ser: ser,
            len: len,
            next_index: 0,
        })
    }
}

impl<'a, W> ser::SerializeMap for SerializeMap<'a, W>
    where W: io::Write
{
    type Ok = ();
    type Error = error::Error;

    fn serialize_key<T>(&mut self, key: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        if self.next_index < self.len {
            self.next_index += 1;
        } else {
            bail!(SerErrorKind::ExcessElements(self.len));
        }

        key.serialize(&mut *self.ser)
    }

    fn serialize_value<T>(&mut self, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> error::Result<()> {
        Ok(())
    }
}


/// Serialize the given data structure as a byte vector of binary MTProto.
pub fn to_bytes<T>(value: &T) -> error::Result<Vec<u8>>
    where T: Serialize
{
    let mut ser = Serializer::new(Vec::new());
    value.serialize(&mut ser)?;

    Ok(ser.writer)
}

/// Serialize the given data structure as binary MTProto into the IO stream.
pub fn to_writer<W, T>(writer: W, value: &T) -> error::Result<()>
    where W: io::Write,
          T: Serialize,
{
    let mut ser = Serializer::new(writer);
    value.serialize(&mut ser)?;

    Ok(())
}
