//! Wrapper structs for attaching additional data to a type for
//! [de]serialization purposes.
//!
//! ## Data and metadata layout
//!
//! | Wrapper type      | Layout           |
//! |-------------------|------------------|
//! | [`Boxed`]         | (id, data)       |
//! | [`WithSize`]      | (size, data)     |
//! | [`BoxedWithSize`] | (id, size, data) |
//!
//! ## Why the wrappers are not `Identifiable` when the type to be wrapped is one?
//!
//! Wrappers have never been found to be applicable in places where any
//! `Identifiable` type can be used.
//! The current setup enforces this relationship between wrappers and
//! `Identifiable` trait at type level.
//! This is a forward-compatible solution in case if wrappers will
//! really need to implement `Identifiable` for some reason.
//!
//! ## Why does `BoxedWithSize` exist?
//!
//! Since `Boxed::new` requires `T` to be `Identifiable` and `WithSize`
//! is not, `Boxed<WithSize<T>>` cannot be created.
//! `BoxedWithSize` can be used for this purpose.
//!
//! ## `BoxedWithSize` arranges fields as (id, size, data). What if I want (size, id, data) instead?
//!
//! `WithSize<Boxed<T>>` can be used perfectly fine.
//!
//! Note: the size in this case will be the size of `Boxed<T>`, not `T`,
//! i.e. it equals the size of the given instance of `T` + 4.
//!
//! ## `Boxed` vs `WithId`
//!
//! `Boxed` and `BoxedWithSize` types have aliases `WithId` and
//! `WithIdAndSize` respectively to convey different meanings about
//! them:
//!
//! * `Boxed<T>`/`BoxedWithSize<T>` mean "not a bare `T`/`T` with size"
//!   respectively where boxed/bare types distinction is drawn from the
//!   MTProto official documentation about serialization:
//!   <https://core.telegram.org/mtproto/serialize#boxed-and-bare-types>.
//! * `WithId<T>`/`WithIdAndSize<T>` mean "`T` with an id/an id and a
//!   size attached" repectively which explains *how* this type arranges
//!   data and metadata.
//!
//! This crate uses `Boxed*` as the main naming scheme, whereas
//! `WithId*` are type aliases.

use std::fmt;
use std::marker::PhantomData;

#[cfg(feature = "quickcheck")]
use quickcheck::{Arbitrary, Gen};
use serde::de::{Deserialize, DeserializeSeed, Deserializer,
                Error as DeError, MapAccess, SeqAccess, Visitor};

use error::{self, DeErrorKind};
use identifiable::Identifiable;
use sized::MtProtoSized;
use utils::{safe_int_cast, safe_uint_eq};


/// A struct that wraps an [`Identifiable`] type value to serialize and
/// deserialize as a boxed MTProto data type.
///
/// Note: if you want to attach both id and serialized size to the
/// underlying data (in this order), see [`BoxedWithSize`] since
/// [`WithSize`] is not `Identifiable`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Boxed<T> {
    id: u32,
    inner: T,
}

/// An alias for [`Boxed`] that is similar to [`WithSize`].
pub type WithId<T> = Boxed<T>;

impl<T: Identifiable> Boxed<T> {
    /// Wrap a value along with its id.
    pub fn new(inner: T) -> Boxed<T> {
        Boxed {
            id: inner.type_id(),
            inner,
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
            let boxed_value = Boxed::new(value);

            if type_id != boxed_value.id {
                bail!(DeErrorKind::TypeIdMismatch(type_id, boxed_value.id));
            }

            Ok(boxed_value)
        }

        // TODO: Use rvalue static promotion after bumping minimal Rust version to 1.21
        const FIELDS: &[&str] = &["id", "inner"];
        deserializer.deserialize_struct("Boxed", FIELDS, BoxedVisitor(PhantomData))
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


/// A struct that wraps a [`MtProtoSized`] type value to serialize and
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
            inner,
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
        // Here we only implement through a helper struct because fully manual implementation
        // (like what is present for `Boxed` and `BoxedWithSize`) won't provide us eny benefits
        // over this solution - we can obtain a deserialized size beforehand, but we can't apply it
        // since neither Serde deserializable types, nor Serde deserializers in general have any
        // means to limit the amount of raw data to be processed.
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


/// A struct that wraps an [`Identifiable`] and [`MtProtoSized`] type
/// value to serialize and deserialize as a boxed MTProto data type with
/// the size of its serialized value.
///
/// This struct exists because `Boxed<WithSize<T>>` cannot be created
/// due to `WithSize<T>` not being `Identifiable` (for the reasoning
/// behind this fact see the module-level docs).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct BoxedWithSize<T> {
    id: u32,
    size: u32,
    inner: T,
}

/// An alias for [`BoxedWithSize`] that is similar to [`WithId`] and
/// [`WithSize`].
pub type WithIdAndSize<T> = BoxedWithSize<T>;

impl<T: Identifiable + MtProtoSized> BoxedWithSize<T> {
    /// Wrap a value along with its id and serialized size.
    pub fn new(inner: T) -> error::Result<BoxedWithSize<T>> {
        let boxed_with_size = BoxedWithSize {
            id: inner.type_id(),
            size: safe_int_cast(inner.size_hint()?)?,
            inner,
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

                let size = next_seq_element(&mut seq, 1, 3)?;
                let value = next_seq_element(&mut seq, 2, 3)?;
                checked_boxed_with_size_value::<T>(type_id, size, value).map_err(A::Error::custom)
            }

            fn visit_map<A>(self, mut map: A) -> Result<BoxedWithSize<T>, A::Error>
                where A: MapAccess<'de>
            {
                let type_id = next_struct_element(&mut map, "id", 0, 3)?;
                check_type_id::<T>(type_id).map_err(A::Error::custom)?;

                let size = next_struct_element(&mut map, "size", 1, 3)?;
                let value = next_struct_element(&mut map, "inner", 2, 3)?;
                checked_boxed_with_size_value::<T>(type_id, size, value).map_err(A::Error::custom)
            }
        }

        fn checked_boxed_with_size_value<T>(type_id: u32,
                                            size: u32,
                                            value: T)
                                           -> error::Result<BoxedWithSize<T>>
            where T: Identifiable + MtProtoSized
        {
            let boxed_with_size_value = BoxedWithSize::new(value)?;

            // Proritize type id mismatch errors over size mismatch ones since type id being
            // incorrect will likely lead to a wrong size too.
            // Also, without correct type information matching sizes don't mean anything anymore.
            // Data is corrupt. Period.
            if type_id != boxed_with_size_value.id {
                bail!(DeErrorKind::TypeIdMismatch(type_id, boxed_with_size_value.id));
            }

            let boxed_with_size_size_hint = boxed_with_size_value.size_hint()?;
            let inner_size_hint = boxed_with_size_size_hint - type_id.size_hint()? - size.size_hint()?;

            if !safe_uint_eq(size, inner_size_hint) {
                bail!(DeErrorKind::SizeMismatch(size, safe_int_cast(inner_size_hint)?));
            }

            Ok(boxed_with_size_value)
        }

        // TODO: Use rvalue static promotion after bumping minimal Rust version to 1.21
        const FIELDS: &[&str] = &["id", "size", "inner"];
        deserializer.deserialize_struct("BoxedWithSize", FIELDS, BoxedWithSizeVisitor(PhantomData))
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
