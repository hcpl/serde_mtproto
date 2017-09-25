//! Wrapper structs for attaching additional data to a type for
//! [de]serializatioh purposes.

use error;
use identifiable::Identifiable;
use sized::MtProtoSized;


/// A struct that wraps an `Identifiable` type value to serialize and
/// deserialize as a boxed MTProto data type.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Boxed<T> {
    id: i32,
    inner: T,
}

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

impl<T: MtProtoSized> MtProtoSized for Boxed<T> {
    fn size_hint(&self) -> error::Result<usize> {
        let id_size_hint = self.id.size_hint()?;
        let inner_size_hint = self.inner.size_hint()?;

        Ok(id_size_hint + inner_size_hint)
    }
}


/// A struct that wraps an `Identifiable` and `MtProtoSized` type value
/// to serialize and deserialize as a boxed MTProto data type with the
/// size of its serialized value.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct BoxedWithLength<T> {
    id: i32,
    size: usize,
    inner: T,
}

impl<T: Identifiable + MtProtoSized> BoxedWithLength<T> {
    /// Wrap a value along with its id and serialized length.
    pub fn new(inner: T) -> error::Result<BoxedWithLength<T>> {
        let boxed_with_length = BoxedWithLength {
            id: inner.type_id(),
            size: inner.size_hint()?,
            inner: inner,
        };

        Ok(boxed_with_length)
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

impl<T: MtProtoSized> MtProtoSized for BoxedWithLength<T> {
    fn size_hint(&self) -> error::Result<usize> {
        let id_size_hint = self.id.size_hint()?;
        let size_size_hint = self.size.size_hint()?;
        let inner_size_hint = self.inner.size_hint()?;

        Ok(id_size_hint + size_size_hint + inner_size_hint)
    }
}
