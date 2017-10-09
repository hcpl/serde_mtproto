//! Serialize a Rust data structure into its MTProto binary representation.

use std::io;

use byteorder::{WriteBytesExt, LittleEndian};
use serde::ser::{self, Serialize};

use error::{self, SerErrorKind, SerSerdeType};
use identifiable::Identifiable;
use utils::safe_int_cast;


/// A structure for serializing Rust values into MTProto binary representation.
pub struct Serializer<W: io::Write> {
    writer: W,
}

impl<W: io::Write> Serializer<W> {
    /// Create a MTProto serializer from an `io::Write`.
    pub fn new(writer: W) -> Serializer<W> {
        Serializer { writer: writer }
    }

    /// Unwraps the `Serializer` and returns the underlying `io::Write`.
    pub fn into_writer(self) -> W {
        self.writer
    }

    fn impl_serialize_bytes(&mut self, value: &[u8]) -> error::Result<()> {
        let len = value.len();
        let rem;

        if len <= 253 {
            // If L <= 253, the serialization contains one byte with the value of L,
            // then L bytes of the string followed by 0 to 3 characters containing 0,
            // such that the overall length of the value be divisible by 4,
            // whereupon all of this is interpreted as a sequence
            // of int(L/4)+1 32-bit little-endian integers.

            self.writer.write_u8(len as u8)?; // `as` is safe: [0..253] \subseteq [0..255]

            rem = (len + 1) % 4;
        } else if len <= 0xff_ff_ff {
            // If L >= 254, the serialization contains byte 254, followed by 3
            // bytes with the string length L in little-endian order, followed by L
            // bytes of the string, further followed by 0 to 3 null padding bytes.

            self.writer.write_u8(254)?;
            self.writer.write_u24::<LittleEndian>(len as u32)?; // `as` is safe: [0..0xff_ff_ff] \subseteq [0..0xff_ff_ff_ff]

            rem = len % 4;
        } else {
            bail!(SerErrorKind::StringTooLong(len));
        }

        // Write each character in the string
        self.writer.write_all(value)?;

        // [...] string followed by 0 to 3 characters containing 0,
        // such that the overall length of the value be divisible by 4 [...]
        if rem > 0 {
            assert!(rem < 4);
            let padding = 4 - rem;
            self.writer.write_uint::<LittleEndian>(0, padding)?;
        }

        Ok(())
    }
}


macro_rules! impl_serialize_small_int {
    ($small_type:ty, $small_method:ident, $big_type:ident, $big_method:ident) => {
        fn $small_method(self, value: $small_type) -> error::Result<()> {
            self.$big_method($big_type::from(value))?;
            debug!("Serialized {} as {}: {:#x}", stringify!($small_type), stringify!($big_type), value);
            Ok(())
        }
    }
}

macro_rules! impl_serialize_big_int {
    ($type:ty, $method:ident, $write:path) => {
        fn $method(self, value: $type) -> error::Result<()> {
            $write(&mut self.writer, value)?;
            debug!("Serialized {}: {:#x}", stringify!($type), value);
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
    type SerializeMap = SerializeFixedLengthMap<'a, W>;
    type SerializeStruct = SerializeFixedLengthSeq<'a, W>;
    type SerializeStructVariant = SerializeFixedLengthSeq<'a, W>;


    fn serialize_bool(self, value: bool) -> error::Result<()> {
        self.writer.write_i32::<LittleEndian>(value.type_id())?;
        debug!("Serialized bool: {} => {:#x}", value, value.type_id());
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

    fn serialize_f32(self, value: f32) -> error::Result<()> {
        // There is only one floating-point type, and it's double precision
        WriteBytesExt::write_f64::<LittleEndian>(&mut self.writer, f64::from(value))?;
        debug!("Serialized f32 as f64: {}", value);
        Ok(())
    }

    fn serialize_f64(self, value: f64) -> error::Result<()> {
        WriteBytesExt::write_f64::<LittleEndian>(&mut self.writer, value)?;
        debug!("Serialized f64: {}", value);
        Ok(())
    }

    fn serialize_char(self, _value: char) -> error::Result<()> {
        bail!(SerErrorKind::UnsupportedSerdeType(SerSerdeType::Char));
    }

    fn serialize_str(self, value: &str) -> error::Result<()> {
        self.impl_serialize_bytes(value.as_bytes())?;
        debug!("Serialized str: {:?}", value);
        Ok(())
    }

    fn serialize_bytes(self, value: &[u8]) -> error::Result<()> {
        self.impl_serialize_bytes(value)?;
        debug!("Serialized bytes: {:?}", value);
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
        debug!("Serialized unit struct");
        Ok(())
    }

    fn serialize_unit_variant(self,
                              _name: &'static str,
                              _variant_index: u32,
                              _variant: &'static str)
                             -> error::Result<()> {
        debug!("Serialized unit variant");
        Ok(())
    }

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        debug!("Serializing newtype variant {}", name);
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(self,
                                    name: &'static str,
                                    variant_index: u32,
                                    variant: &'static str,
                                    value: &T)
                                   -> error::Result<()>
        where T: ?Sized + Serialize
    {
        debug!("Serializing newtype variant {}::{} (variant index {})", name, variant, variant_index);
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> error::Result<Self::SerializeSeq> {
        if let Some(len) = len {
            debug!("Serializing seq of len {}", len);
            SerializeFixedLengthSeq::with_serialize_len(self, safe_int_cast(len)?)
        } else {
            bail!(SerErrorKind::SeqsWithUnknownLengthUnsupported);
        }
    }

    fn serialize_tuple(self, len: usize) -> error::Result<Self::SerializeTuple> {
        debug!("Serializing tuple of len {}", len);
        Ok(SerializeFixedLengthSeq::new(self, safe_int_cast(len)?))
    }

    fn serialize_tuple_struct(self,
                              name: &'static str,
                              len: usize)
                             -> error::Result<Self::SerializeTupleStruct> {
        debug!("Serializing tuple struct {} of len {}", name, len);
        Ok(SerializeFixedLengthSeq::new(self, safe_int_cast(len)?))
    }

    fn serialize_tuple_variant(self,
                               name: &'static str,
                               variant_index: u32,
                               variant: &'static str,
                               len: usize)
                              -> error::Result<Self::SerializeTupleVariant> {
        debug!("Serializing tuple variant {}::{} (variant index {}) of len {}",
            name, variant, variant_index, len);
        Ok(SerializeFixedLengthSeq::new(self, safe_int_cast(len)?))
    }

    fn serialize_map(self, len: Option<usize>) -> error::Result<Self::SerializeMap> {
        if let Some(len) = len {
            debug!("Serializing map of len {}", len);
            SerializeFixedLengthMap::with_serialize_len(self, safe_int_cast(len)?)
        } else {
            bail!(SerErrorKind::MapsWithUnknownLengthUnsupported);
        }
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> error::Result<Self::SerializeStruct> {
        debug!("Serializing struct {} of len {}", name, len);
        Ok(SerializeFixedLengthSeq::new(self, safe_int_cast(len)?))
    }

    fn serialize_struct_variant(self,
                                name: &'static str,
                                variant_index: u32,
                                variant: &'static str,
                                len: usize)
                               -> error::Result<Self::SerializeStructVariant> {
        debug!("Serializing struct variant {}::{} (variant index {}) of len {}",
            name, variant, variant_index, len);
        Ok(SerializeFixedLengthSeq::new(self, safe_int_cast(len)?))
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

    fn impl_serialize_seq_value<T>(&mut self,
                                   key: Option<&'static str>,
                                   value: &T,
                                   serializer_type: &'static str)
                                  -> error::Result<()>
        where T: ?Sized + Serialize
    {
        if self.next_index < self.len {
            self.next_index += 1;
        } else {
            debug!("{}::serialize_element() is called when no elements is left to serialize",
                serializer_type);

            bail!(SerErrorKind::ExcessElements(self.len));
        }

        if let Some(key) = key {
            debug!("Serializing field {}", key);
        } else {
            debug!("Serializing element");
        }

        value.serialize(&mut *self.ser)
    }

    fn impl_serialize_end(self, data_type: &'static str) -> error::Result<()> {
        if self.next_index < self.len {
            bail!(SerErrorKind::NotEnoughElements(self.next_index, self.len))
        }

        // `self.index > self.len` here is a programming error
        assert_eq!(self.next_index, self.len);

        debug!("Finished serializing {}", data_type);

        Ok(())
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
        self.impl_serialize_seq_value(None, value, "SerializeSeq")
    }

    fn end(self) -> error::Result<()> {
        self.impl_serialize_end("seq")
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
        self.impl_serialize_seq_value(None, value, "SerializeTuple")
    }

    fn end(self) -> error::Result<()> {
        self.impl_serialize_end("tuple")
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
        self.impl_serialize_seq_value(None, value, "SerializeTupleStruct")
    }

    fn end(self) -> error::Result<()> {
        self.impl_serialize_end("tuple struct")
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
        self.impl_serialize_seq_value(None, value, "SerializeTupleVariant")
    }

    fn end(self) -> error::Result<()> {
        self.impl_serialize_end("tuple variant")
    }
}

impl<'a, W> ser::SerializeStruct for SerializeFixedLengthSeq<'a, W>
    where W: io::Write
{
    type Ok = ();
    type Error = error::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        self.impl_serialize_seq_value(Some(key), value, "SerializeStruct")
    }

    fn end(self) -> error::Result<()> {
        self.impl_serialize_end("struct")
    }
}

impl<'a, W> ser::SerializeStructVariant for SerializeFixedLengthSeq<'a, W>
    where W: io::Write
{
    type Ok = ();
    type Error = error::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        self.impl_serialize_seq_value(Some(key), value, "SerializeStructVariant")
    }

    fn end(self) -> error::Result<()> {
        self.impl_serialize_end("struct variant")
    }
}


/// Helper structure for serializing maps.
pub struct SerializeFixedLengthMap<'a, W: 'a + io::Write> {
    ser: &'a mut Serializer<W>,
    len: u32,
    next_index: u32,
}

impl<'a, W: io::Write> SerializeFixedLengthMap<'a, W> {
    fn with_serialize_len(ser: &'a mut Serializer<W>,
                          len: u32)
                         -> error::Result<SerializeFixedLengthMap<'a, W>> {
        ser::Serializer::serialize_u32(&mut *ser, len)?;

        Ok(SerializeFixedLengthMap {
            ser: ser,
            len: len,
            next_index: 0,
        })
    }
}

impl<'a, W> ser::SerializeMap for SerializeFixedLengthMap<'a, W>
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
            debug!("SerializeMap::serialize_key() is called when no elements is left to serialize");

            bail!(SerErrorKind::ExcessElements(self.len));
        }

        debug!("Serializing key");
        key.serialize(&mut *self.ser)
    }

    fn serialize_value<T>(&mut self, value: &T) -> error::Result<()>
        where T: ?Sized + Serialize
    {
        debug!("Serializing value");
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> error::Result<()> {
        debug!("Finished serializing map");
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

/// Serialize bytes with padding to 16 bytes as a byte vector of binary MTProto.
pub fn unsized_bytes_pad_to_bytes(value: &[u8]) -> error::Result<Vec<u8>> {
    let padding = (16 - value.len() % 16) % 16;
    let mut result = Vec::with_capacity(value.len() + padding);

    result.extend(value);
    for _ in 0..padding {
        result.push(0);
    }

    Ok(result)
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

/// Serialize bytes with padding to 16 bytes into the IO stream.
pub fn unsized_bytes_pad_to_writer<W>(mut writer: W, value: &[u8]) -> error::Result<()>
    where W: io::Write
{
    let padding = (16 - value.len() % 16) % 16;

    writer.write_all(value)?;
    for _ in 0..padding {
        writer.write_u8(0)?;
    }

    Ok(())
}
