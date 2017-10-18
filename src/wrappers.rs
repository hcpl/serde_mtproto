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
//!   https://core.telegram.org/mtproto/serialize.
//! * `WithId<T>`/`WithIdAndSize<T>` mean "`T` with an id/an id and a
//!   size attached" repectively which explains *how* this type is
//!   representing data.
//!
//! This crate uses `Boxed*` family as the default, whereas `WithId*`
//! are type aliases.

use error;
use identifiable::Identifiable;
use sized::MtProtoSized;
use utils::safe_int_cast;


/// A struct that wraps an `Identifiable` type value to serialize and
/// deserialize as a boxed MTProto data type.
///
/// Note: if you want to attach both id and serialized size to the
/// underlying data (in this order), see `BoxedWithSize`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
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

impl<T: Identifiable> Identifiable for Boxed<T> {
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


/// A struct that wraps a `MtProtoSized` type value to serialize and
/// deserialize as a MTProto data type with the size of its serialized
/// value.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
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

impl<T: MtProtoSized> MtProtoSized for WithSize<T> {
    fn size_hint(&self) -> error::Result<usize> {
        let size_size_hint = self.size.size_hint()?;
        let inner_size_hint = self.inner.size_hint()?;

        Ok(size_size_hint + inner_size_hint)
    }
}


/// A struct that wraps an `Identifiable` and `MtProtoSized` type value
/// to serialize and deserialize as a boxed MTProto data type with the
/// size of its serialized value.
///
/// This struct exists because `Boxed<WithSize<T>>` cannot be created
/// due to `WithSize<T>` not being `Identifiable` (this restriction is
/// made on purpose).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
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

impl<T: MtProtoSized> MtProtoSized for BoxedWithSize<T> {
    fn size_hint(&self) -> error::Result<usize> {
        let id_size_hint = self.id.size_hint()?;
        let size_size_hint = self.size.size_hint()?;
        let inner_size_hint = self.inner.size_hint()?;

        Ok(id_size_hint + size_size_hint + inner_size_hint)
    }
}
