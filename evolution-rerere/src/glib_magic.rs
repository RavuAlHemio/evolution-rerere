use evolution_glue::{GType, GTypeClass, GTypeInstance, g_type_check_class_cast, g_type_check_instance_cast};


pub unsafe fn class_cast<F, T>(cls: *mut F, to_type: GType) -> *mut T {
    unsafe {
        g_type_check_class_cast(
            cls as *mut GTypeClass,
            to_type,
        ) as *mut T
    }
}

pub unsafe fn type_cast<F, T>(obj: *mut F, to_type: GType) -> *mut T {
    unsafe {
        g_type_check_instance_cast(
            obj as *mut GTypeInstance,
            to_type,
        ) as *mut T
    }
}
