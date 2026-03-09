use std::ffi::{CStr, CString};
use std::ptr::{null, null_mut};
use std::str::FromStr;
use std::sync::{LazyLock, RwLock};

use evolution_glue::{
    EComposerHeaderTable, e_composer_header_table_get_subject, e_composer_header_table_get_type,
    e_composer_header_table_set_subject, EExtension, EExtensionClass, e_extension_get_extensible,
    e_extension_get_type, EMsgComposer, e_msg_composer_get_header_table, e_msg_composer_get_type,
    e_signal_connect_notify, gint, GObject, GObjectClass, GParamSpec, gpointer, GType, GTypeClass,
    g_type_class_adjust_private_offset, g_type_class_peek_parent, GTypeInfo, GTypeInstance,
    GTypeModule, g_type_module_register_type, G_TYPE_OBJECT,
};
use regex::Regex;

use crate::glib_magic::{class_cast, type_cast};


#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct GTypeClassPtr {
    ptr: *mut GTypeClass,
}
impl GTypeClassPtr {
    pub const fn new(ptr: *mut GTypeClass) -> Self {
        Self { ptr }
    }

    pub const fn null() -> Self {
        Self {
            ptr: null_mut(),
        }
    }
}
unsafe impl Send for GTypeClassPtr {}
unsafe impl Sync for GTypeClassPtr {}
impl From<*mut GTypeClass> for GTypeClassPtr {
    fn from(value: *mut GTypeClass) -> Self { Self { ptr: value } }
}
impl From<GTypeClassPtr> for *mut GTypeClass {
    fn from(value: GTypeClassPtr) -> Self { value.ptr }
}


static RE_RE_RE_COMPOSER_EXTENSION_TYPE_ID: LazyLock<RwLock<GType>> = LazyLock::new(|| RwLock::new(0));
static RE_RE_RE_COMPOSER_EXTENSION_PRIVATE_OFFSET: LazyLock<RwLock<gint>> = LazyLock::new(|| RwLock::new(0));
static RE_RE_RE_COMPOSER_EXTENSION_PARENT_CLASS: LazyLock<RwLock<GTypeClassPtr>> = LazyLock::new(|| RwLock::new(GTypeClassPtr::null()));
static SUBJ_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(concat!(
    "(?P<prefix>",
        "Re|Fwd", // English, Unix tradition
        "|",
        "RE|FW", // English, Outlook
        "|",
        "WG|AW", // German, Outlook
    ")",
    "(?:",
        "\\[",
        "(?P<depth>",
            "[0-9]+",
        ")",
        "\\]",
    ")?",
    ":", // colon
)).unwrap());


struct ReReReComposerExtensionPrivate {
    pub subject_already_altered: bool,
}


#[repr(C)]
pub(crate) struct ReReReComposerExtension {
    parent: EExtension,
    private: *mut ReReReComposerExtensionPrivate,
}
impl ReReReComposerExtension {
    pub fn type_id() -> GType {
        let read_guard = RE_RE_RE_COMPOSER_EXTENSION_TYPE_ID
            .read().expect("type ID lock poisoned?!");
        *read_guard
    }

    pub fn register_type(type_module: *mut GTypeModule) {
        let define_type_info = GTypeInfo {
            class_size: size_of::<ReReReComposerExtensionClass>().try_into().unwrap(),
            base_init: None,
            base_finalize: None,
            class_init: Some(Self::class_intern_init),
            class_finalize: Some(Self::class_finalize),
            class_data: null_mut(),
            instance_size: size_of::<ReReReComposerExtension>().try_into().unwrap(),
            n_preallocs: 0,
            instance_init: Some(Self::init),
            value_table: null(),
        };
        let type_id = unsafe {
            g_type_module_register_type(
                type_module,
                e_extension_get_type(),
                c"ReReReComposerExtension".as_ptr(),
                &raw const define_type_info,
                0,
            )
        };

        {
            let mut write_guard = RE_RE_RE_COMPOSER_EXTENSION_TYPE_ID
                .write().expect("type ID lock poisoned?!");
            *write_guard = type_id;
        }

        {
            let private_offset = size_of::<ReReReComposerExtensionPrivate>().try_into().unwrap();
            let mut write_guard = RE_RE_RE_COMPOSER_EXTENSION_PRIVATE_OFFSET
                .write().expect("private offset lock poisoned?!");
            *write_guard = private_offset;
        }
    }

    extern "C" fn class_intern_init(cls: gpointer, _class_data: gpointer) {
        let parent_class = unsafe {
            g_type_class_peek_parent(cls)
        };
        {
            let mut write_guard= RE_RE_RE_COMPOSER_EXTENSION_PARENT_CLASS
                .write().expect("parent class lock poisoned?!");
            *write_guard = GTypeClassPtr::new(parent_class as *mut GTypeClass);
        }

        // convert private size to private offset
        let mut private_offset = {
            let read_guard = RE_RE_RE_COMPOSER_EXTENSION_PRIVATE_OFFSET
                .read().expect("private offset lock poisoned?!");
            *read_guard
        };
        unsafe {
            g_type_class_adjust_private_offset(cls, &raw mut private_offset)
        };
        {
            let mut write_guard = RE_RE_RE_COMPOSER_EXTENSION_PRIVATE_OFFSET
                .write().expect("private offset lock poisoned?!");
            *write_guard = private_offset;
        }

        Self::class_init(cls as *mut ReReReComposerExtensionClass);
    }

    fn class_init(cls: *mut ReReReComposerExtensionClass) {
        let object_class: *mut GObjectClass = unsafe { class_cast(cls, G_TYPE_OBJECT) };
        unsafe { &mut *object_class }.constructed = Some(Self::constructed);

        let e_extension_type = unsafe { e_extension_get_type() };
        let extension_class: *mut EExtensionClass = unsafe { class_cast(cls, e_extension_type) };
        unsafe { &mut *extension_class }.extensible_type = unsafe { e_msg_composer_get_type() };
    }

    extern "C" fn get_instance_private(myself: *mut ReReReComposerExtension) -> *mut ReReReComposerExtensionPrivate {
        let private_offset = {
            let read_guard = RE_RE_RE_COMPOSER_EXTENSION_PRIVATE_OFFSET
                .read().expect("private offset lock poisoned?!");
            *read_guard
        };
        let myself_u8 = myself as *mut u8;
        let private_u8 = myself_u8.wrapping_offset(private_offset.try_into().unwrap());
        private_u8 as *mut ReReReComposerExtensionPrivate
    }

    extern "C" fn class_finalize(_class: gpointer, _class_data: gpointer) {
    }

    extern "C" fn init(instance: *mut GTypeInstance, _cls: gpointer) {
        let rerere = instance as *mut ReReReComposerExtension;
        unsafe { &mut *rerere }.private = Self::get_instance_private(rerere);
    }

    extern "C" fn constructed(object: *mut GObject) {
        // super.constructed(object)
        let parent_class = {
            let read_guard = RE_RE_RE_COMPOSER_EXTENSION_PARENT_CLASS
                .read().expect("parent class lock poisoned?!");
            read_guard.ptr
        };
        let parent_gobject_class: *mut GObjectClass = unsafe { class_cast(parent_class, G_TYPE_OBJECT) };
        if let Some(constructed_func) = unsafe { &*parent_gobject_class }.constructed {
            unsafe {
                constructed_func(object);
            }
        }

        let extension: *mut EExtension = unsafe { type_cast(object, e_extension_get_type()) };
        let extensible = unsafe { e_extension_get_extensible(extension) };

        let me: *mut ReReReComposerExtension = unsafe { type_cast(object, Self::type_id()) };
        unsafe { &mut *(*me).private }.subject_already_altered = false;
        let composer: *mut EMsgComposer = unsafe { type_cast(extensible, e_msg_composer_get_type()) };
        let header_table = unsafe { e_msg_composer_get_header_table(composer) };

        // react to a change in subject
        let subject_changed_ptr = Self::subject_changed as *mut ();
        let subject_changed_fn: unsafe extern "C" fn() = unsafe { std::mem::transmute(subject_changed_ptr) };
        unsafe {
            e_signal_connect_notify(
                header_table as gpointer,
                c"notify::subject".as_ptr(),
                Some(subject_changed_fn),
                extension as gpointer,
            )
        };
    }

    extern "C" fn subject_changed(instance: gpointer, _param: *mut GParamSpec, user_data: gpointer) {
        let header_table: *mut EComposerHeaderTable = unsafe { type_cast(instance, e_composer_header_table_get_type()) };
        let extension: *mut ReReReComposerExtension = unsafe { type_cast(user_data, Self::type_id()) };

        // guard against multiple calls for the same subject
        let already_altered = unsafe { &*(*extension).private }.subject_already_altered;
        if already_altered {
            return;
        }
        unsafe { &mut *(*extension).private }.subject_already_altered = true;

        // gimme subject
        let subject_ptr = unsafe { e_composer_header_table_get_subject(header_table) };
        let subject_cstr = unsafe { CStr::from_ptr(subject_ptr) };
        let subject = subject_cstr.to_str().expect("subject not valid UTF-8");

        let mut new_prefix = None;
        let mut new_subject = subject.to_owned();
        loop {
            // eat leading spaces, if any
            while new_subject.starts_with(' ') {
                new_subject.remove(0);
            }

            // do we have a Re/Fwd prefix?
            let cap = match SUBJ_REGEX.captures(&new_subject) {
                None => break, // nope; we're done here
                Some(c) => c,
            };

            // is it at the beginning of the string?
            let cap_pos = cap.get_match().start();
            if cap_pos == 0 {
                // yup; analyze it further
                let prefix = cap.name("prefix").unwrap().as_str();

                // do we have a depth counter?
                if let Some(depth_match) = cap.name("depth") {
                    // yes; try incrementing it
                    if let Ok(old_depth) = u128::from_str(depth_match.as_str()) {
                        let new_depth = old_depth.wrapping_add(1);
                        new_prefix = Some(format!("{}[{}]:", prefix, new_depth));
                    }
                }

                if new_prefix.is_none() {
                    // remember the current prefix for later
                    new_prefix = Some(cap.get_match().as_str().to_owned());
                }
            }

            // clear out the prefix
            new_subject.drain(cap.get_match().range());

            // go around
        }

        // assemble the full new subject
        let full_new_subject = if let Some(np) = new_prefix.as_ref() {
            format!("{} {}", np, new_subject)
        } else {
            new_subject
        };
        let full_new_subject_c = CString::from_str(&full_new_subject).unwrap();

        unsafe {
            e_composer_header_table_set_subject(header_table, full_new_subject_c.as_ptr())
        };
    }
}

#[repr(C)]
struct ReReReComposerExtensionClass {
    parent: EExtensionClass,
}

#[unsafe(no_mangle)]
pub extern "C" fn rerere_composer_extension_get_type() -> GType {
    ReReReComposerExtension::type_id()
}
