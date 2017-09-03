//! `Boxed` struct which represents a boxed MTProto data type.

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
            id: inner.get_id(),
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
    fn get_size_hint(&self) -> error::Result<usize> {
        let id_size_hint = self.id.get_size_hint()?;
        let inner_size_hint = self.inner.get_size_hint()?;

        Ok(id_size_hint + inner_size_hint)
    }
}
