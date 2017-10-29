//! Wrapper structs for attaching additional data to a type for
//! [de]serialization purposes.
//!
//! `Boxed` and `BoxedWithSize` types have aliases `WithId` and
//! `WithIdAndSize` respectively to convey different meanings about
//! them:
//!
//! * `Boxed<T>`/`BoxedWithSize<T>` mean "not a bare `T`/`T` with size"
//!   respectively where boxed/bare types distinction is drawn from the
//!   MTProto official documentation about serialization:
//!   <https://core.telegram.org/mtproto/serialize>.
//! * `WithId<T>`/`WithIdAndSize<T>` mean "`T` with an id/an id and a
//!   size attached" repectively which explains *how* this type is
//!   representing data.
//!
//! This crate uses `Boxed*` family as the default, whereas `WithId*`
//! are type aliases.

use std::fmt;
use std::marker::PhantomData;

#[cfg(feature = "quickcheck")]
use quickcheck::{Arbitrary, Gen};
use serde::de::{Deserialize, Deserializer, Error as DeError, MapAccess, SeqAccess, Visitor};

use error::{self, DeErrorKind};
use identifiable::Identifiable;
use sized::MtProtoSized;
use utils::{safe_int_cast, safe_uint_cmp};


/// A struct that wraps an `Identifiable` type value to serialize and
/// deserialize as a boxed MTProto data type.
///
/// Note: if you want to attach both id and serialized size to the
/// underlying data (in this order), see `BoxedWithSize`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Boxed<T> {
    id: u32,
    inner: T,
}

/// Give `Boxed` an alias that is similar to `WithSize`.
pub type WithId<T> = Boxed<T>;

impl<T: Identifiable> Boxed<T> {
    /// Wrap a value along with its id.
    pub fn new(inner: T) -> Boxed<T> {
        Boxed {
            id: inner.type_id(),
            inner: inner,
        }
    }

    /// Return an immutable reference to the underlying data.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Return a mutable reference to the underlying data.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Unwrap the box and return the wrapped value.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

// Using a custom implementation instead of the derived one because we need to check validity
// of the deserialized type id __before__ deserializing the value.
impl<'de, T> Deserialize<'de> for Boxed<T>
    where T: Deserialize<'de> + Identifiable
{
    fn deserialize<D>(deserializer: D) -> Result<Boxed<T>, D::Error>
        where D: Deserializer<'de>
    {
        use identifiable::Identifiable;

        struct BoxedVisitor<T>(PhantomData<T>);

        fn check_type_id<T: Identifiable>(type_id: u32) -> error::Result<()> {
            let expected_type_ids = T::all_type_ids();
            if expected_type_ids.iter().find(|&id| *id == type_id).is_none() {
                bail!(DeErrorKind::InvalidTypeId(type_id, expected_type_ids));
            }

            Ok(())
        }

        fn checked_boxed_value<T: Identifiable>(type_id: u32, value: T) -> error::Result<Boxed<T>> {
            let boxed_value = Boxed::new(value);

            if type_id != boxed_value.id {
                bail!(DeErrorKind::TypeIdMismatch(type_id, boxed_value.id));
            }

            Ok(boxed_value)
        }

        impl<'de, T> Visitor<'de> for BoxedVisitor<T>
            where T: Deserialize<'de> + Identifiable
        {
            type Value = Boxed<T>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("type id and an `Identifiable` value")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Boxed<T>, A::Error>
                where A: SeqAccess<'de>
            {
                let errconv = |kind: DeErrorKind| A::Error::custom(error::Error::from(kind));

                let type_id = seq.next_element()?
                    .ok_or(errconv(DeErrorKind::NotEnoughElements(0, 2)))?;

                check_type_id::<T>(type_id).map_err(A::Error::custom)?;

                let value = seq.next_element()?
                    .ok_or(errconv(DeErrorKind::NotEnoughElements(1, 2)))?;

                checked_boxed_value::<T>(type_id, value).map_err(A::Error::custom)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Boxed<T>, A::Error>
                where A: MapAccess<'de>
            {
                let errconv = |kind: DeErrorKind| A::Error::custom(error::Error::from(kind));

                let type_id = match map.next_key()?
                    .ok_or(errconv(DeErrorKind::NotEnoughElements(0, 2)))?
                {
                    "id" => map.next_value()?,
                    key => bail!(errconv(DeErrorKind::InvalidMapKey(key.to_owned(), "id"))),
                };

                check_type_id::<T>(type_id).map_err(A::Error::custom)?;

                let value = match map.next_key()?
                    .ok_or(errconv(DeErrorKind::NotEnoughElements(1, 2)))?
                {
                    "inner" => map.next_value()?,
                    key => bail!(errconv(DeErrorKind::InvalidMapKey(key.to_owned(), "inner"))),
                };

                checked_boxed_value::<T>(type_id, value).map_err(A::Error::custom)
            }
        }

        deserializer.deserialize_struct("Boxed", &["id", "inner"], BoxedVisitor(PhantomData))
    }
}

impl<T: Identifiable> Identifiable for Boxed<T> {
    fn all_type_ids() -> &'static [u32] {
        T::all_type_ids()
    }

    fn type_id(&self) -> u32 {
        self.id
    }

    fn enum_variant_id(&self) -> Option<&'static str> {
        None
    }
}

impl<T: MtProtoSized> MtProtoSized for Boxed<T> {
    fn size_hint(&self) -> error::Result<usize> {
        let id_size_hint = self.id.size_hint()?;
        let inner_size_hint = self.inner.size_hint()?;

        Ok(id_size_hint + inner_size_hint)
    }
}

#[cfg(feature = "quickcheck")]
impl<T> Arbitrary for Boxed<T>
    where T: Arbitrary + Identifiable
{
    fn arbitrary<G: Gen>(g: &mut G) -> Boxed<T> {
        Boxed::new(T::arbitrary(g))
    }

    fn shrink(&self) -> Box<Iterator<Item=Boxed<T>>> {
        Box::new(self.inner.shrink().map(Boxed::new))
    }
}


/// A struct that wraps a `MtProtoSized` type value to serialize and
/// deserialize as a MTProto data type with the size of its serialized
/// value.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct WithSize<T> {
    size: u32,
    inner: T,
}

impl<T: MtProtoSized> WithSize<T> {
    /// Wrap a value along with its serialized size.
    pub fn new(inner: T) -> error::Result<WithSize<T>> {
        let with_size = WithSize {
            size: safe_int_cast(inner.size_hint()?)?,
            inner: inner,
        };

        Ok(with_size)
    }

    /// Return an immutable reference to the underlying data.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Return a mutable reference to the underlying data.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Unwrap the box and return the wrapped value.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

// Using a custom implementation instead of the derived one because we need to check validity
// of the deserialized size against the size hint of a deserialized value.
impl<'de, T> Deserialize<'de> for WithSize<T>
    where T: Deserialize<'de> + MtProtoSized
{
    fn deserialize<D>(deserializer: D) -> Result<WithSize<T>, D::Error>
        where D: Deserializer<'de>
    {
        let errconv = |kind: DeErrorKind| D::Error::custom(error::Error::from(kind));

        #[derive(Deserialize)]
        #[serde(rename = "WithSize")]
        struct WithSizeHelper<T> {
            size: u32,
            inner: T,
        }

        let helper = WithSizeHelper::<T>::deserialize(deserializer)?;
        let helper_size_hint = helper.inner.size_hint().map_err(D::Error::custom)?;

        if !safe_uint_cmp(helper.size, helper_size_hint) {
            bail!(errconv(DeErrorKind::SizeMismatch(
                helper.size,
                safe_int_cast(helper_size_hint).map_err(D::Error::custom)?,
            )));
        }

        Ok(WithSize {
            size: helper.size,
            inner: helper.inner,
        })
    }
}

impl<T: MtProtoSized> MtProtoSized for WithSize<T> {
    fn size_hint(&self) -> error::Result<usize> {
        let size_size_hint = self.size.size_hint()?;
        let inner_size_hint = self.inner.size_hint()?;

        Ok(size_size_hint + inner_size_hint)
    }
}

#[cfg(feature = "quickcheck")]
impl<T> Arbitrary for WithSize<T>
    where T: Arbitrary + MtProtoSized
{
    fn arbitrary<G: Gen>(g: &mut G) -> WithSize<T> {
        WithSize::new(T::arbitrary(g))
            .expect("failed to wrap a generated random value using `WithSize`")
    }

    fn shrink(&self) -> Box<Iterator<Item=WithSize<T>>> {
        Box::new(self.inner.shrink().map(|x| WithSize::new(x)
            .expect("failed to wrap a shrinked value using `WithSize`")))
    }
}


/// A struct that wraps an `Identifiable` and `MtProtoSized` type value
/// to serialize and deserialize as a boxed MTProto data type with the
/// size of its serialized value.
///
/// This struct exists because `Boxed<WithSize<T>>` cannot be created
/// due to `WithSize<T>` not being `Identifiable` (this restriction is
/// made on purpose).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct BoxedWithSize<T> {
    id: u32,
    size: u32,
    inner: T,
}

/// Give `BoxedWithSize` an alias that is similar to `WithId` and `WithSize`.
pub type WithIdAndSize<T> = BoxedWithSize<T>;

impl<T: Identifiable + MtProtoSized> BoxedWithSize<T> {
    /// Wrap a value along with its id and serialized size.
    pub fn new(inner: T) -> error::Result<BoxedWithSize<T>> {
        let boxed_with_size = BoxedWithSize {
            id: inner.type_id(),
            size: safe_int_cast(inner.size_hint()?)?,
            inner: inner,
        };

        Ok(boxed_with_size)
    }

    /// Return an immutable reference to the underlying data.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Return a mutable reference to the underlying data.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Unwrap the box and return the wrapped value.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

// Using a custom implementation instead of the derived one because we need to check validity
// of the deserialized type id __before__ deserializing the value and also of the deserialized size
// against the size hint of a deserialized value.
impl<'de, T> Deserialize<'de> for BoxedWithSize<T>
    where T: Deserialize<'de> + Identifiable + MtProtoSized
{
    fn deserialize<D>(deserializer: D) -> Result<BoxedWithSize<T>, D::Error>
        where D: Deserializer<'de>
    {
        use identifiable::Identifiable;
        use sized::MtProtoSized;

        struct BoxedWithSizeVisitor<T>(PhantomData<T>);

        fn check_type_id<T: Identifiable>(type_id: u32) -> error::Result<()> {
            let expected_type_ids = T::all_type_ids();
            if expected_type_ids.iter().find(|&id| *id == type_id).is_none() {
                bail!(DeErrorKind::InvalidTypeId(type_id, expected_type_ids));
            }

            Ok(())
        }

        fn checked_boxed_with_size_value<T>(type_id: u32,
                                            size: u32,
                                            value: T)
                                           -> error::Result<BoxedWithSize<T>>
            where T: Identifiable + MtProtoSized
        {
            let boxed_with_size_value = BoxedWithSize::new(value)?;

            if type_id != boxed_with_size_value.id {
                bail!(DeErrorKind::TypeIdMismatch(type_id, boxed_with_size_value.id));
            }

            let boxed_with_size_size_hint = boxed_with_size_value.size_hint()?;
            if !safe_uint_cmp(size, boxed_with_size_size_hint) {
                bail!(DeErrorKind::SizeMismatch(size, safe_int_cast(boxed_with_size_size_hint)?));
            }

            Ok(boxed_with_size_value)
        }

        fn next_seq_element<'de, T, A>(seq: &mut A,
                                       deserialized_count: u32,
                                       expected_count: u32)
                                      -> Result<T, A::Error>
            where T: Deserialize<'de>,
                  A: SeqAccess<'de>,
        {
            let errconv = |kind: DeErrorKind| A::Error::custom(error::Error::from(kind));

            seq.next_element()?
                .ok_or(errconv(DeErrorKind::NotEnoughElements(deserialized_count, expected_count)))
        }

        fn next_map_element<'de, T, A>(map: &mut A,
                                       expected_key: &'static str,
                                       deserialized_count: u32,
                                       expected_count: u32)
                                      -> Result<T, A::Error>
            where T: Deserialize<'de>,
                  A: MapAccess<'de>,
        {
            let errconv = |kind: DeErrorKind| A::Error::custom(error::Error::from(kind));

            let next_key: String = map.next_key()?
                .ok_or(errconv(DeErrorKind::NotEnoughElements(deserialized_count, expected_count)))?;

            if &next_key != expected_key {
                bail!(errconv(DeErrorKind::InvalidMapKey(next_key, expected_key)));
            }

            map.next_value()
        }

        impl<'de, T> Visitor<'de> for BoxedWithSizeVisitor<T>
            where T: Deserialize<'de> + Identifiable + MtProtoSized
        {
            type Value = BoxedWithSize<T>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("type id and an `Identifiable` value")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<BoxedWithSize<T>, A::Error>
                where A: SeqAccess<'de>
            {
                let type_id = next_seq_element(&mut seq, 0, 3)?;
                check_type_id::<T>(type_id).map_err(A::Error::custom)?;

                let value = next_seq_element(&mut seq, 1, 3)?;
                let size = next_seq_element(&mut seq, 2, 3)?;
                checked_boxed_with_size_value::<T>(type_id, size, value).map_err(A::Error::custom)
            }

            fn visit_map<A>(self, mut map: A) -> Result<BoxedWithSize<T>, A::Error>
                where A: MapAccess<'de>
            {
                let type_id = next_map_element(&mut map, "id", 0, 3)?;
                check_type_id::<T>(type_id).map_err(A::Error::custom)?;

                let size = next_map_element(&mut map, "size", 0, 3)?;
                let value = next_map_element(&mut map, "inner", 0, 3)?;
                checked_boxed_with_size_value::<T>(type_id, size, value).map_err(A::Error::custom)
            }
        }

        deserializer.deserialize_struct(
            "BoxedWithSize", &["id", "size", "inner"], BoxedWithSizeVisitor(PhantomData))
    }
}

impl<T: MtProtoSized> MtProtoSized for BoxedWithSize<T> {
    fn size_hint(&self) -> error::Result<usize> {
        let id_size_hint = self.id.size_hint()?;
        let size_size_hint = self.size.size_hint()?;
        let inner_size_hint = self.inner.size_hint()?;

        Ok(id_size_hint + size_size_hint + inner_size_hint)
    }
}

#[cfg(feature = "quickcheck")]
impl<T> Arbitrary for BoxedWithSize<T>
    where T: Arbitrary + Identifiable + MtProtoSized
{
    fn arbitrary<G: Gen>(g: &mut G) -> BoxedWithSize<T> {
        BoxedWithSize::new(T::arbitrary(g))
            .expect("failed to wrap a generated random value using `BoxedWithSize`")
    }

    fn shrink(&self) -> Box<Iterator<Item=BoxedWithSize<T>>> {
        Box::new(self.inner.shrink().map(|x| BoxedWithSize::new(x)
            .expect("failed to wrap a shrinked value using `BoxedWithSize`")))
    }
}
