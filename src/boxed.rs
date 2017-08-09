use identifiable::Identifiable;


#[derive(Serialize, Deserialize)]
pub struct Boxed<T> {
    id: i32,
    inner: T,
}

impl<T: Identifiable> Boxed<T> {
    pub fn new(inner: T) -> Boxed<T> {
        Boxed {
            id: inner.get_id(),
            inner: inner,
        }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}
