pub trait Identifiable {
    fn get_id(&self) -> i32;
}

impl<'a, T: Identifiable> Identifiable for &'a T {
    fn get_id(&self) -> i32 {
        (*self).get_id()
    }
}


#[derive(Serialize, Deserialize)]
pub struct Wrapper<T> {
    id: i32,
    data: T,
}

impl<T: Identifiable> Wrapper<T> {
    pub fn new(data: T) -> Wrapper<T> {
        Wrapper {
            id: data.get_id(),
            data: data,
        }
    }

    pub fn take_data(self) -> T {
        self.data
    }
}
