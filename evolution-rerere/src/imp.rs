use glib::object_subclass;
use glib::subclass::object::ObjectImpl;
use glib::subclass::types::ObjectSubclass;


#[derive(Default)]
pub struct ReReReExtension {
    subject_already_set: bool,
}

impl ObjectImpl for ReReReExtension {
}

#[object_subclass]
#[object_subclass_dynamic(lazy_registration = true)]
impl ObjectSubclass for ReReReExtension {
    const NAME: &'static str = "ReReReExtension";
    type Type = crate::obj::ReReReExtension;
    type ParentType = crate::obj::Extension;

    fn class_init(cls: &mut Self::Class) {

    }

    fn new() -> Self {
        Self {
            subject_already_set: false,
        }
    }
}
