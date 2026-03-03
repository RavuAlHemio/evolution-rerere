use core::marker::{PhantomData, PhantomPinned};

use glib::gobject_ffi::{GObject, GObjectClass, GTypeInterface, GTypeModule};


macro_rules! opaque_struct {
    ($name:ident) => {
        pub(crate) struct $name {
            // opaque
            _data: [u8; 0],
            _marker: PhantomData<(*mut u8, PhantomPinned)>,
        }
    }
}


pub type GType = libc::size_t;


opaque_struct!(EExtensionPrivate);
opaque_struct!(EExtensible);

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct EExtensibleInterface {
    parent_interface: GTypeInterface,
}


#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct EExtension {
    pub parent_instance: GObject,
    pub private: *mut EExtensionPrivate,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct EExtensionClass {
    pub parent_class: GObjectClass,
    pub extensible_type: GType,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct ReReReExtension {
    pub parent_instance: EExtension,
    pub already_modified: bool,
}


unsafe extern "C" {
    pub(crate) unsafe fn e_extension_get_type() -> GType;
}


pub extern "C" fn e_module_load(type_module: *mut GTypeModule) {

}

pub extern "C" fn e_module_unload(_type_module: *mut GTypeModule) {

}
