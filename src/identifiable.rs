//! `Identifiable` trait for any Rust data structure that can have an id.

#![cfg_attr(feature = "cargo-clippy", allow(unreadable_literal))]  // To match the look & feel from TL schema


/// Type id of the bool true value.
pub const BOOL_TRUE_ID: u32 = 0x997275b5;
/// Type id of the bool false value.
pub const BOOL_FALSE_ID: u32 = 0xbc799737;
/// Type id of the int type.
pub const INT_ID: u32 = 0xa8509bda;
/// Type id of the long type.
pub const LONG_ID: u32 = 0x22076cba;
/// Type id of the double type.
pub const DOUBLE_ID: u32 = 0x2210c154;
/// Type id of the string type.
pub const STRING_ID: u32 = 0xb5286e24;
/// Type id of the vector type.
pub const VECTOR_ID: u32 = 0x1cb5c415;


const BOOL_IDS: &[u32] = &[BOOL_TRUE_ID, BOOL_FALSE_ID];
const INT_IDS: &[u32] = &[INT_ID];
const LONG_IDS: &[u32] = &[LONG_ID];
const DOUBLE_IDS: &[u32] = &[DOUBLE_ID];
const STRING_IDS: &[u32] = &[STRING_ID];
const VECTOR_IDS: &[u32] = &[VECTOR_ID];

const BOOL_VARIANT_NAMES: &[&str] = &["false", "true"];


/// A trait for a Rust data structure that can have an id.
pub trait Identifiable {
    /// Get all possible ids (known at compile time) of an identifiable type.
    ///
    /// This is most useful for enums where each variant has its own id.
    ///
    /// # Implementation note
    ///
    /// This method **should** return a slice of **non-duplicate** values, i.e.
    /// the slice must effectively be a set with contents known at compile-time.
    /// Currently, there is no way to enforce this restriction using the
    /// language itself.
    ///
    /// This restriction is marked as **should** because it potentially only
    /// alters the behavior of counting all ids a type in question has, but not
    /// checking if an arbitrary id is in this set - in this case, a reliable
    /// counting routine must traverse the whole slice to eliminate duplicates.
    ///
    /// Unfortunately, this worsens the time complexity from O(*1*) to O(*n*),
    /// but for everyday use-case this is fine since product types we usually
    /// use are relatively small to make this a big concern.
    ///
    /// # Compatibility note
    ///
    /// Will probably be replaced by an associated constant
    /// after bumping minimum supported Rust version to 1.20.
    ///
    /// On the other hand, making static methods associated constants will
    /// prevent this trait from usage with dynamic dispatch.
    fn all_type_ids() -> &'static [u32]
        where Self: Sized;

    /// Get all enum variant names of an identifiable type.
    ///
    /// For structs this method must return `None` and for enums it must return
    /// `Some` with stringified variant names in the same order as the variants
    /// themselves.
    ///
    /// # Compatibility note
    ///
    /// Will probably be replaced by an associated constant
    /// after bumping minimum supported Rust version to 1.20.
    ///
    /// On the other hand, making static methods associated constants will
    /// prevent this trait from usage with dynamic dispatch.
    fn all_enum_variant_names() -> Option<&'static [&'static str]>
        where Self: Sized;

    /// Get id of a value of an identifiable type.
    ///
    /// Its signature is made `(&self) -> i32`, not `() -> i32` because of enum
    /// types where different enum variants can have different ids.
    ///
    /// # Implementation note
    ///
    /// This method **should** return a value contained in the slice returned by
    /// `all_type_ids()` method.
    /// Currently, there is no way to enforce this restriction using the
    /// language itself.
    fn type_id(&self) -> u32;

    /// Get enum variant_hint for a value of an identifiable type.
    ///
    /// This method is purely for assisting `de::Deserializer` to deserialize
    /// enum types because `Deserialize` implementations generated by
    /// `#[derive(Deserialize)]` call `Deserializer::deserialize_identifier()`
    /// to identify an enum variant.
    fn enum_variant_id(&self) -> Option<&'static str>;
}


impl<'a, T: Identifiable> Identifiable for &'a T {
    fn all_type_ids() -> &'static [u32] {
        T::all_type_ids()
    }

    fn all_enum_variant_names() -> Option<&'static [&'static str]> {
        T::all_enum_variant_names()
    }

    fn type_id(&self) -> u32 {
        (*self).type_id()
    }

    fn enum_variant_id(&self) -> Option<&'static str> {
        (*self).enum_variant_id()
    }
}

impl<T: Identifiable> Identifiable for Box<T> {
    fn all_type_ids() -> &'static [u32] {
        T::all_type_ids()
    }

    fn all_enum_variant_names() -> Option<&'static [&'static str]> {
        T::all_enum_variant_names()
    }

    fn type_id(&self) -> u32 {
        (**self).type_id()
    }

    fn enum_variant_id(&self) -> Option<&'static str> {
        (**self).enum_variant_id()
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(match_bool))]  // match looks better here
impl Identifiable for bool {
    fn all_type_ids() -> &'static [u32] {
        BOOL_IDS
    }

    fn all_enum_variant_names() -> Option<&'static [&'static str]> {
        Some(BOOL_VARIANT_NAMES)
    }

    fn type_id(&self) -> u32 {
        match *self {
            false => BOOL_FALSE_ID,
            true => BOOL_TRUE_ID,
        }
    }

    // Doesn't really serve any purpose here, but implement anyway for completeness
    fn enum_variant_id(&self) -> Option<&'static str> {
        match *self {
            false => Some("false"),
            true  => Some("true"),
        }
    }
}


macro_rules! impl_identifiable_for_simple_types {
    ($($type:ty => ($all_ids:expr, $id_of_value:expr),)*) => {
        $(
            impl Identifiable for $type {
                fn all_type_ids() -> &'static [u32] {
                    $all_ids
                }

                fn all_enum_variant_names() -> Option<&'static [&'static str]> {
                    None
                }

                fn type_id(&self) -> u32 {
                    $id_of_value
                }

                fn enum_variant_id(&self) -> Option<&'static str> {
                    None
                }
            }
        )*
    };
}

// Not implemented for `usize` and `isize` because of their machine-dependent nature:
// on 32-bit machine they would have int id, but on 64-bit - long id.
impl_identifiable_for_simple_types! {
    i8  => (INT_IDS,  INT_ID),
    i16 => (INT_IDS,  INT_ID),
    i32 => (INT_IDS,  INT_ID),
    i64 => (LONG_IDS, LONG_ID),

    u8  => (INT_IDS,  INT_ID),
    u16 => (INT_IDS,  INT_ID),
    u32 => (INT_IDS,  INT_ID),
    u64 => (LONG_IDS, LONG_ID),

    f32 => (DOUBLE_IDS, DOUBLE_ID),
    f64 => (DOUBLE_IDS, DOUBLE_ID),

    String => (STRING_IDS, STRING_ID),
}

impl<'a> Identifiable for &'a str {
    fn all_type_ids() -> &'static [u32] {
        STRING_IDS
    }

    fn all_enum_variant_names() -> Option<&'static [&'static str]> {
        None
    }

    fn type_id(&self) -> u32 {
        STRING_ID
    }

    fn enum_variant_id(&self) -> Option<&'static str> {
        None
    }
}

impl<T> Identifiable for Vec<T> {
    fn all_type_ids() -> &'static [u32] {
        VECTOR_IDS
    }

    fn all_enum_variant_names() -> Option<&'static [&'static str]> {
        None
    }

    fn type_id(&self) -> u32 {
        VECTOR_ID
    }

    fn enum_variant_id(&self) -> Option<&'static str> {
        None
    }
}
