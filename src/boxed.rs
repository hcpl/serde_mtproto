//! `Boxed` struct which represents a boxed MTProto data type.

use identifiable::Identifiable;


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

    /// Unwrap the box and return the wrapped value.
    pub fn into_inner(self) -> T {
        self.inner
    }
}
