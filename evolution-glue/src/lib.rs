#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unnecessary_transmutes)]
#![allow(unsafe_op_in_unsafe_fn)]


include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


macro_rules! g_type_fundamental_shift { () => (2); }
macro_rules! g_type_make_fundamental {
    ($value:expr) => {
        $value << g_type_fundamental_shift!()
    };
}

pub const G_TYPE_OBJECT: GType = g_type_make_fundamental!(20);
