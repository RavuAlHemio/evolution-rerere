use glib::wrapper;

use crate::ffi::{EExtension, EExtensionClass, e_extension_get_type};


wrapper! {
    pub(crate) struct Extensible (Interface<EExtensible, EExtensibleInterface>);

    match fn {
        type_ => || e_extensible_get_type(),
    }
}


wrapper! {
    pub(crate) struct Extension (Object<EExtension, EExtensionClass>);
    match fn {
        type_ => || e_extension_get_type(),
    }
}

wrapper! {
    pub(crate) struct ReReReExtension (ObjectSubclass<crate::imp::ReReReExtension>)
        @extends Extension;
}
