//! Wrapper structs for attaching additional data to a type for
//! [de]serialization purposes.
//!
//! ## Data and metadata layout
//!
//! | Wrapper type | Layout       |
//! |--------------|--------------|
//! | [`Boxed`]    | (id, data)   |
//! | [`WithSize`] | (size, data) |
//!
//! ## How does `Boxed<WithSize<T>>` differ from `WithSize<Boxed<T>>`?
//!
//! The first is laid out as (id, size, data) while the second — as
//! (size, id, data).
//! While the `id` value in both cases represent the type id of `data`
//! the `size` value in two layouts above are not the same thing: in the
//! first one it equals `data.size_hint()?`, but in the second one it
//! equals `data.size_hint()? + 4` because it also includes the size of
//! the `id` value.
//!
//! ## `Boxed` vs `WithId`
//!
//! `Boxed` type has an alias `WithId` to convey different meanings
//! about it:
//!
//! * `Boxed<T>` means "not a bare `T`" where boxed/bare types
//!   distinction is drawn from the MTProto official documentation about
//!   serialization:
//!   <https://core.telegram.org/mtproto/serialize#boxed-and-bare-types>.
//! * `WithId<T>` means "`T` with an id" which explains *how* this type
//!   arranges data and metadata.
//!
//! This crate uses `Boxed` as the main naming scheme, whereas `WithId`
//! is a type alias.

use std::fmt;
use std::marker::PhantomData;

use error_chain::bail;
#[cfg(feature = "quickcheck")]
use quickcheck::{Arbitrary, Gen};
use serde::de::{Deserialize, DeserializeSeed, Deserializer,
                Error as DeError, MapAccess, SeqAccess, Visitor};
use serde::ser::{Error as SerError, Serialize, Serializer, SerializeStruct};
use serde_derive::Deserialize;

use crate::error::{self, DeErrorKind};
use crate::identifiable::Identifiable;
use crate::sized::MtProtoSized;
use crate::utils::{safe_uint_cast, safe_uint_eq};


/// A struct that wraps an [`Identifiable`] type value to serialize and
/// deserialize as a boxed MTProto data type.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Boxed<T> {
    inner: T,
}

/// An alias for [`Boxed`] that is similar to [`WithSize`].
pub type WithId<T> = Boxed<T>;

impl<T: Identifiable> Boxed<T> {
    /// Wrap a value along with its id.
    pub fn new(inner: T) -> Boxed<T> {
        Boxed { inner }
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

impl<T> Serialize for Boxed<T>
    where T: Serialize + Identifiable
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
    {
        let mut ser = serializer.serialize_struct("Boxed", 2)?;
        ser.serialize_field("id", &self.inner.type_id())?;
        ser.serialize_field("inner", &self.inner)?;
        ser.end()
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
        struct BoxedVisitor<T>(PhantomData<T>);

        impl<'de, T> Visitor<'de> for BoxedVisitor<T>
            where T: Deserialize<'de> + Identifiable
        {
            type Value = Boxed<T>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("type id and an `Identifiable` value")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Boxed<T>, A::Error>
                where A: SeqAccess<'de>
            {
                let type_id = next_seq_element(&mut seq, 0, 2)?;
                check_type_id::<T>(type_id).map_err(A::Error::custom)?;

                let value = next_seq_element(&mut seq, 1, 2)?;
                checked_boxed_value::<T>(type_id, value).map_err(A::Error::custom)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Boxed<T>, A::Error>
                where A: MapAccess<'de>
            {
                let type_id = next_struct_element(&mut map, "id", 0, 2)?;
                check_type_id::<T>(type_id).map_err(A::Error::custom)?;

                let value = next_struct_element(&mut map, "inner", 1, 2)?;
                checked_boxed_value::<T>(type_id, value).map_err(A::Error::custom)
            }
        }

        fn checked_boxed_value<T: Identifiable>(type_id: u32, value: T) -> error::Result<Boxed<T>> {
            if type_id != value.type_id() {
                bail!(DeErrorKind::TypeIdMismatch(type_id, value.type_id()));
            }

            Ok(Boxed::new(value))
        }

        deserializer.deserialize_struct("Boxed", &["id", "inner"], BoxedVisitor(PhantomData))
    }
}

impl<T: Identifiable> Identifiable for Boxed<T> {
    fn all_type_ids() -> &'static [u32] {
        T::all_type_ids()
    }

    fn all_enum_variant_names() -> Option<&'static [&'static str]> {
        T::all_enum_variant_names()
    }

    fn type_id(&self) -> u32 {
        T::type_id(&self.inner)
    }

    fn enum_variant_id(&self) -> Option<&'static str> {
        T::enum_variant_id(&self.inner)
    }
}

impl<T: MtProtoSized> MtProtoSized for Boxed<T> {
    fn size_hint(&self) -> error::Result<usize> {
        // Just an u32 value to use for `<u32 as MtProtoSized>::size_hint`
        let id_size_hint = 0_u32.size_hint()?;
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

    fn shrink(&self) -> Box<dyn Iterator<Item=Boxed<T>>> {
        Box::new(self.inner.shrink().map(Boxed::new))
    }
}


/// A struct that wraps a [`MtProtoSized`] type value to serialize and
/// deserialize as a MTProto data type with the size of its serialized
/// value.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WithSize<T> {
    inner: T,
}

impl<T: MtProtoSized> WithSize<T> {
    /// Wrap a value along with its serialized size.
    pub fn new(inner: T) -> error::Result<WithSize<T>> {
        Ok(WithSize { inner })
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

impl<T> Serialize for WithSize<T>
    where T: Serialize + MtProtoSized
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
    {
        let size_usize = self.inner.size_hint().map_err(S::Error::custom)?;
        let size_u32 = safe_uint_cast::<usize, u32>(size_usize).map_err(S::Error::custom)?;

        let mut ser = serializer.serialize_struct("WithSize", 2)?;
        ser.serialize_field("size", &size_u32)?;
        ser.serialize_field("inner", &self.inner)?;
        ser.end()
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
        // Here we only implement through a helper struct because fully manual implementation
        // (like what is present for `Boxed`) won't provide us eny benefits over this solution - we
        // can obtain a deserialized size beforehand, but we can't apply it since neither Serde
        // deserializable types, nor Serde deserializers in general have any means to limit the
        // amount of raw data to be processed.
        #[derive(Deserialize)]
        #[serde(rename = "WithSize")]
        struct WithSizeHelper<T> {
            size: u32,
            inner: T,
        }

        let helper = WithSizeHelper::<T>::deserialize(deserializer)?;
        let helper_size_hint = helper.inner.size_hint().map_err(D::Error::custom)?;

        if !safe_uint_eq(helper.size, helper_size_hint) {
            bail!(errconv::<D::Error>(DeErrorKind::SizeMismatch(
                helper.size,
                safe_uint_cast(helper_size_hint).map_err(D::Error::custom)?,
            )));
        }

        Ok(WithSize {
            inner: helper.inner,
        })
    }
}

impl<T: Identifiable> Identifiable for WithSize<T> {
    fn all_type_ids() -> &'static [u32] {
        T::all_type_ids()
    }

    fn all_enum_variant_names() -> Option<&'static [&'static str]> {
        T::all_enum_variant_names()
    }

    fn type_id(&self) -> u32 {
        T::type_id(&self.inner)
    }

    fn enum_variant_id(&self) -> Option<&'static str> {
        T::enum_variant_id(&self.inner)
    }
}

impl<T: MtProtoSized> MtProtoSized for WithSize<T> {
    fn size_hint(&self) -> error::Result<usize> {
        // Just an u32 value to use for `<u32 as MtProtoSized>::size_hint`
        let size_size_hint = 0_u32.size_hint()?;
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

    fn shrink(&self) -> Box<dyn Iterator<Item=WithSize<T>>> {
        Box::new(self.inner.shrink().map(|x| WithSize::new(x)
            .expect("failed to wrap a shrinked value using `WithSize`")))
    }
}


// ========== UTILS ========== //

fn check_type_id<T: Identifiable>(type_id: u32) -> error::Result<()> {
    let expected_type_ids = T::all_type_ids();
    if expected_type_ids.iter().find(|&id| *id == type_id).is_none() {
        bail!(DeErrorKind::InvalidTypeId(type_id, expected_type_ids));
    }

    Ok(())
}


fn next_seq_element<'de, T, A>(seq: &mut A,
                               deserialized_count: u32,
                               expected_count: u32)
                              -> Result<T, A::Error>
    where T: Deserialize<'de>,
          A: SeqAccess<'de>,
{
    next_seq_element_seed(seq, PhantomData, deserialized_count, expected_count)
}

fn next_seq_element_seed<'de, S, A>(seq: &mut A,
                                    seed: S,
                                    deserialized_count: u32,
                                    expected_count: u32)
                                   -> Result<S::Value, A::Error>
    where S: DeserializeSeed<'de>,
          A: SeqAccess<'de>,
{
    seq.next_element_seed(seed)?
        .ok_or_else(|| errconv(DeErrorKind::NotEnoughElements(deserialized_count, expected_count)))
}


fn next_struct_element<'de, T, A>(map: &mut A,
                                  expected_key: &'static str,
                                  deserialized_count: u32,
                                  expected_count: u32)
                                 -> Result<T, A::Error>
    where T: Deserialize<'de>,
          A: MapAccess<'de>,
{
    next_struct_element_seed(
        map, PhantomData, PhantomData, expected_key, deserialized_count, expected_count)
}

fn next_struct_element_seed<'de, K, V, A>(map: &mut A,
                                          string_key_seed: K,
                                          value_seed: V,
                                          expected_key: &'static str,
                                          deserialized_count: u32,
                                          expected_count: u32)
                                         -> Result<V::Value, A::Error>
    where K: DeserializeSeed<'de, Value=String>,
          V: DeserializeSeed<'de>,
          A: MapAccess<'de>,
{
    let next_key = map.next_key_seed(string_key_seed)?
        .ok_or_else(|| errconv(DeErrorKind::NotEnoughElements(deserialized_count, expected_count)))?;

    // Don't even try to deserialize value if keys don't match
    // (the reason behind not using `.next_entry_seed()`)
    if next_key != expected_key {
        bail!(errconv::<A::Error>(DeErrorKind::InvalidMapKey(next_key, expected_key)));
    }

    map.next_value_seed(value_seed)
}


fn errconv<E>(kind: DeErrorKind) -> E
    where E: DeError
{
    E::custom(error::Error::from(kind))
}
