pub const BOOL_TRUE_ID: i32 = -1720552011;
pub const BOOL_FALSE_ID: i32 = -1132882121;
pub const INT_ID: i32 = -1471112230;
pub const LONG_ID: i32 = 570911930;
pub const DOUBLE_ID: i32 = 571523412;
pub const STRING_ID: i32 = -1255641564;
pub const VECTOR_ID: i32 = 481674261;


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

impl Identifiable for bool {
    fn get_id(&self) -> i32 {
        match *self {
            false => BOOL_FALSE_ID,
            true => BOOL_TRUE_ID,
        }
    }

    // Doesn't really serve any purpose here, but implement anyway for completeness
    fn get_enum_variant_id(&self) -> Option<&'static str> {
        match *self {
            false => Some("false"),
            true  => Some("true"),
        }
    }
}


macro_rules! impl_identifiable_for_primitives {
    () => {};

    ($type:ty => $id_value:expr, $($rest:tt)*) => {
        impl Identifiable for $type {
            fn get_id(&self) -> i32 {
                $id_value
            }

            fn get_enum_variant_id(&self) -> Option<&'static str> {
                None
            }
        }

        impl_identifiable_for_primitives! { $($rest)* }
    };
}

impl_identifiable_for_primitives! {
    i8  => INT_ID,
    i16 => INT_ID,
    i32 => INT_ID,
    i64 => LONG_ID,

    u8  => INT_ID,
    u16 => INT_ID,
    u32 => INT_ID,
    u64 => LONG_ID,

    f32 => DOUBLE_ID,
    f64 => DOUBLE_ID,

    String => STRING_ID,
}

impl<'a> Identifiable for &'a str {
    fn get_id(&self) -> i32 {
        STRING_ID
    }

    fn get_enum_variant_id(&self) -> Option<&'static str> {
        None
    }
}

impl<T> Identifiable for Vec<T> {
    fn get_id(&self) -> i32 {
        VECTOR_ID
    }

    fn get_enum_variant_id(&self) -> Option<&'static str> {
        None
    }
}
