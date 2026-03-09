mod ext;
mod glib_magic;


use evolution_glue::GTypeModule;

use crate::ext::ReReReComposerExtension;


#[unsafe(no_mangle)]
pub extern "C" fn e_module_load(type_module: *mut GTypeModule) {
    ReReReComposerExtension::register_type(type_module);
}

#[unsafe(no_mangle)]
pub extern "C" fn e_module_unload(_type_module: *mut GTypeModule) {
}
