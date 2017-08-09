pub trait Identifiable {
    fn get_id(&self) -> i32;
    fn get_enum_variant_id(&self) -> Option<&'static str>;
}

impl<'a, T: Identifiable> Identifiable for &'a T {
    fn get_id(&self) -> i32 {
        (*self).get_id()
    }

    fn get_enum_variant_id(&self) -> Option<&'static str> {
        (*self).get_enum_variant_id()
    }
}
