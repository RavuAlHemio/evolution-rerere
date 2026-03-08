use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::fmt;

use crate::model::ReadWrite::{self, WriteConstruct};
use crate::model::{Document, Field, Method, OpaqueRecord, Parameter, Property, RegularParameter, TypeInfo};


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TypeDatabaseError {
    DuplicateType { library: String, type_name: String, one_c_type: String, other_c_type: String },
    DuplicateCType { one_library: String, one_type: String, other_library: String, other_type: String, c_type: String },
}
impl fmt::Display for TypeDatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateType { library, type_name, one_c_type, other_c_type }
                => write!(
                    f, "duplicate type {:?}.{:?}, maps to C types {:?} and {:?}",
                    library, type_name, one_c_type, other_c_type,
                ),
            Self::DuplicateCType { one_library, one_type, other_library, other_type, c_type }
                => write!(
                    f, "duplicate C type {:?}, maps to types {:?}.{:?} and {:?}.{:?}",
                    c_type, one_library, one_type, other_library, other_type,
                ),
        }
    }
}
impl std::error::Error for TypeDatabaseError {
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EnrichError {
    NameTooManyDots { name: String, dot_count: usize },
    UnknownLocalUniversalType { name: String },
    UnknownGlobalType { name: String },
}
impl fmt::Display for EnrichError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NameTooManyDots { name, dot_count }
                => write!(f, "name {:?} has too many dots ({})", name, dot_count),
            Self::UnknownLocalUniversalType { name }
                => write!(f, "failed to resolve local/universal type {:?}", name),
            Self::UnknownGlobalType { name }
                => write!(f, "failed to resolve global type {:?}", name),
        }
    }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeDatabase {
    library_to_type_to_c: BTreeMap<String, BTreeMap<String, String>>,
    c_to_library_and_type: BTreeMap<String, (String, String)>,
}
impl TypeDatabase {
    pub fn new() -> Self {
        Self {
            library_to_type_to_c: BTreeMap::new(),
            c_to_library_and_type: BTreeMap::new(),
        }
    }

    pub fn insert<L: Into<String>, T: Into<String>, C: Into<String>>(&mut self, library: L, type_name: T, c_type: C) -> Result<(), TypeDatabaseError> {
        let library_string = library.into();
        let type_string = type_name.into();
        let c_type_string = c_type.into();

        let type_to_c = self.library_to_type_to_c
            .entry(library_string.clone())
            .or_insert_with(|| BTreeMap::new());
        match type_to_c.entry(type_string.clone()) {
            Entry::Vacant(e) => {
                e.insert(c_type_string.clone());
            },
            Entry::Occupied(e) => {
                return Err(TypeDatabaseError::DuplicateType {
                    library: library_string,
                    type_name: type_string,
                    one_c_type: e.get().clone(),
                    other_c_type: c_type_string,
                });
            },
        }

        match self.c_to_library_and_type.entry(c_type_string.clone()) {
            Entry::Vacant(e) => {
                e.insert((library_string.clone(), type_string.clone()));
            },
            Entry::Occupied(e) => {
                let (existing_library, existing_type) = e.get();
                return Err(TypeDatabaseError::DuplicateCType {
                    one_library: existing_library.clone(),
                    one_type: existing_type.clone(),
                    other_library: library_string,
                    other_type: type_string,
                    c_type: c_type_string,
                });
            },
        }

        Ok(())
    }

    pub fn library_to_type_to_c(&self) -> &BTreeMap<String, BTreeMap<String, String>> {
        &self.library_to_type_to_c
    }

    pub fn c_to_library_and_type(&self) -> &BTreeMap<String, (String, String)> {
        &self.c_to_library_and_type
    }

    /// Collects the type definitions in the document, generating C names where no explicit one is
    /// given.
    pub fn try_from_mut_document(document: &mut Document) -> Result<Self, TypeDatabaseError> {
        let mut type_database = TypeDatabase::default();

        // collect all data for the type database
        // (constructing C names where not explicitly provided)
        for library in &mut document.libraries {
            for interface in &mut library.interfaces {
                if interface.c_name.is_none() {
                    interface.c_name = Some(format!("{}{}", library.c_id_prefix, interface.name));
                }
                type_database.insert(
                    &library.name,
                    &interface.name,
                    interface.c_name.as_ref().unwrap(),
                )?;
            }
            for cls in &mut library.classes {
                if cls.c_name.is_none() {
                    cls.c_name = Some(format!("{}{}", library.c_id_prefix, cls.name));
                }
                type_database.insert(
                    &library.name,
                    &cls.name,
                    cls.c_name.as_ref().unwrap(),
                )?;
            }
            for rec in &library.records {
                type_database.insert(
                    &library.name,
                    &rec.name,
                    &rec.c_name,
                )?;
            }
            for rec in &mut library.opaque_records {
                if rec.c_name.is_none() {
                    rec.c_name = Some(format!("{}{}", library.c_id_prefix, rec.name));
                }
                type_database.insert(
                    &library.name,
                    &rec.name,
                    rec.c_name.as_ref().unwrap(),
                )?;
            }
        }

        Ok(type_database)
    }

    /// Enriches field, property, argument and return types with C type information gleaned from the
    /// other types in the document.
    pub fn enrich_document(&self, document: &mut Document) -> Result<(), EnrichError> {
        let empty_library = BTreeMap::new();
        for library in &mut document.libraries {
            let library_types = self.library_to_type_to_c
                .get(&library.name)
                .unwrap_or(&empty_library);
            for interface in &mut library.interfaces {
                for prop in &mut interface.properties {
                    self.enrich_type_info(&mut prop.type_info, library_types)?;
                }
                for method in &mut interface.methods {
                    self.enrich_method(method, library_types)?;
                }
            }
            for cls in &mut library.classes {
                for class_field in &mut cls.class_fields {
                    self.enrich_type_info(&mut class_field.type_info, library_types)?;
                }
                for field in &mut cls.fields {
                    self.enrich_type_info(&mut field.type_info, library_types)?;
                }
                for prop in &mut cls.properties {
                    self.enrich_type_info(&mut prop.type_info, library_types)?;
                }
                for method in &mut cls.methods {
                    self.enrich_method(method, library_types)?;
                }
            }
            for record in &mut library.records {
                for field in &mut record.fields {
                    self.enrich_type_info(&mut field.type_info, library_types)?;
                }
            }
            // opaque records do not have concrete types
        }
        Ok(())
    }

    fn enrich_method(&self, method: &mut Method, library_types: &BTreeMap<String, String>) -> Result<(), EnrichError> {
        for param in &mut method.params {
            if let Parameter::Regular(p) = param {
                self.enrich_type_info(&mut p.type_info, library_types)?;
            }
        }
        self.enrich_type_info(&mut method.return_type, library_types)
    }

    fn enrich_type_info(&self, type_info: &mut TypeInfo, library_types: &BTreeMap<String, String>) -> Result<(), EnrichError> {
        if type_info.c_type.is_some() {
            return Ok(());
        }

        let dot_count = type_info.name.chars().filter(|c| *c == '.').count();
        let c_type: Cow<str> = match dot_count {
            0 => {
                // local or universal type
                match type_info.name.as_str() {
                    "GType" => Cow::Borrowed(type_info.name.as_str()),
                    "void" => {
                        // special case: we also change the regular name
                        type_info.name = "none".to_owned();
                        Cow::Borrowed("void")
                    },
                    "utf8" => Cow::Borrowed(if type_info.is_const { "const gchar*" } else { "gchar*" }),
                    _ => if let Some(t) = library_types.get(&type_info.name) {
                        let star = if type_info.is_contained { "" } else { "*" };
                        Cow::Owned(format!("{}{}", t, star))
                    } else {
                        return Err(EnrichError::UnknownLocalUniversalType { name: type_info.name.clone() });
                    },
                }
            },
            1 => {
                // global type
                let (type_lib, type_name) = type_info.name.split_once('.').unwrap();
                match type_lib {
                    "GLib" => {
                        match type_name {
                            "List" => if type_info.is_contained { Cow::Borrowed("GList") } else { Cow::Borrowed("GList*") },
                            _ => return Err(EnrichError::UnknownGlobalType { name: type_info.name.clone() }),
                        }
                    },
                    "GObject" => {
                        match type_name {
                            "Object" => if type_info.is_contained { Cow::Borrowed("GObject") } else { Cow::Borrowed("GObject*") },
                            _ => return Err(EnrichError::UnknownGlobalType { name: type_info.name.clone() }),
                        }
                    },
                    _ => {
                        let type_opt = self.library_to_type_to_c
                            .get(type_lib)
                            .and_then(|tl| tl.get(type_name));
                        match type_opt {
                            Some(t) => {
                                let cst = if type_info.is_const { "const " } else { "" };
                                let star = if type_info.is_contained { "" } else { "*" };
                                Cow::Owned(format!("{}{}{}", cst, t, star))
                            },
                            None => return Err(EnrichError::UnknownGlobalType { name: type_info.name.clone() }),
                        }
                    }
                }
            },
            _ => return Err(EnrichError::NameTooManyDots {
                name: type_info.name.clone(),
                dot_count,
            }),
        };
        type_info.c_type = Some(c_type.into_owned());

        for type_param in &mut type_info.params {
            self.enrich_type_info(type_param, library_types)?;
        }

        Ok(())
    }
}

pub fn realize_parent_and_priv_fields(doc: &mut Document) {
    for library in &mut doc.libraries {
        for cls in &mut library.classes {
            if !cls.parent_and_priv_fields {
                continue;
            }

            let priv_name = format!("{}Private", cls.name);

            let parent_field = Field {
                name: "parent".to_owned(),
                private: true,
                rw: WriteConstruct,
                type_info: TypeInfo {
                    name: cls.parent.clone(),
                    c_type: None,
                    is_const: false,
                    is_contained: true,
                    params: Vec::with_capacity(0),
                },
            };
            let priv_field = Field {
                name: "priv".to_owned(),
                private: true,
                rw: WriteConstruct,
                type_info: TypeInfo {
                    name: format!("{}Private", cls.name),
                    c_type: None,
                    is_const: false,
                    is_contained: false,
                    params: Vec::with_capacity(0),
                },
            };

            cls.fields.splice(0..0, [parent_field, priv_field]);
            cls.parent_and_priv_fields = false;

            // also add the opaque record
            library.opaque_records.push(OpaqueRecord {
                name: priv_name,
                c_name: None,
            });
        }
    }
}

fn realize_property_getter_setter(parent_c_prefix: &str, property: &mut Property, methods: &mut Vec<Method>) {
    let (has_getter, has_setter) = match property.rw {
        ReadWrite::Neither => (false, false),
        ReadWrite::ReadOnly => (true, false),
        ReadWrite::ReadConstruct => (true, false),
        ReadWrite::ReadWrite => (true, true),
        ReadWrite::ReadWriteConstruct => (true, true),
        ReadWrite::WriteOnly => (false, true),
        ReadWrite::WriteConstruct => (false, true),
    };

    if has_getter {
        let name = format!("get_{}", property.name);
        let c_name = format!("{}{}", parent_c_prefix, name);
        let getter_method = Method {
            name,
            c_name: Some(c_name),
            params: vec![Parameter::Instance],
            return_type: property.type_info.clone(),
            getter_for_property: Some(property.name.clone()),
            setter_for_property: None,
        };
        methods.push(getter_method);
    }

    if has_setter {
        let name = format!("set_{}", property.name);
        let c_name = format!("{}{}", parent_c_prefix, name);
        let setter_method = Method {
            name,
            c_name: Some(c_name),
            params: vec![
                Parameter::Instance,
                Parameter::Regular(RegularParameter {
                    name: property.name.clone(),
                    type_info: property.type_info.clone(),
                }),
            ],
            return_type: TypeInfo::make_void(),
            getter_for_property: Some(property.name.clone()),
            setter_for_property: None,
        };
        methods.push(setter_method);
    }
}

pub fn realize_property_getters_and_setters(doc: &mut Document) {
    for library in &mut doc.libraries {
        for iface in &mut library.interfaces {
            for prop in &mut iface.properties {
                realize_property_getter_setter(
                    &format!(
                        "{}{}",
                        library.c_sym_prefix,
                        pascal_to_snake_case(&iface.name),
                    ),
                    prop,
                    &mut iface.methods,
                );
            }
        }

        for cls in &mut library.classes {
            for prop in &mut cls.properties {
                realize_property_getter_setter(
                    &format!(
                        "{}{}",
                        library.c_sym_prefix,
                        pascal_to_snake_case(&cls.name),
                    ),
                    prop,
                    &mut cls.methods,
                );
            }
        }
    }
}

pub(crate) fn pascal_to_snake_case(pascal: &str) -> String {
    use std::fmt::Write;

    let mut ret = String::with_capacity(pascal.len() * 2);
    for c in pascal.chars() {
        if c.is_uppercase() {
            let c_lower = c.to_lowercase();
            if ret.len() > 0 {
                ret.push('_');
            }
            write!(ret, "{}", c_lower).unwrap();
        } else {
            ret.push(c);
        }
    }
    ret
}
