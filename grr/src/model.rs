use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error;


#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Document {
    pub libraries: Vec<Library>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Library {
    pub name: String,
    pub version: String,
    pub c_id_prefix: String,
    pub c_sym_prefix: String,
    #[serde(default)] pub shared_lib: Option<String>,
    #[serde(default)] pub classes: Vec<Class>,
    #[serde(default)] pub interfaces: Vec<Interface>,
    #[serde(default)] pub opaque_records: Vec<OpaqueRecord>,
    #[serde(default)] pub records: Vec<Record>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Class {
    pub name: String,
    pub c_name: String,
    #[serde(default)] pub abstr: bool,
    #[serde(default)] pub parent_and_priv_fields: bool,
    #[serde(default)] pub properties: Vec<Property>,
    #[serde(default)] pub fields: Vec<Field>,
    #[serde(default)] pub class_fields: Vec<ClassField>,
    #[serde(default)] pub methods: Vec<Method>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Interface {
    pub name: String,
    pub c_name: String,
    #[serde(default)] pub properties: Vec<Property>,
    #[serde(default)] pub methods: Vec<Method>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct OpaqueRecord {
    pub name: String,
    pub c_name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Record {
    pub name: String,
    pub c_name: String,
    #[serde(default)] pub fields: Vec<Field>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Property {
    pub name: String,
    pub rw: ReadWrite,
    #[serde(rename = "type")] pub type_info: TypeInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Field {
    pub name: String,
    #[serde(default = "Field::default_private")] pub private: bool,
    #[serde(default = "Field::default_read_write")] pub rw: ReadWrite,
    #[serde(rename = "type")] pub type_info: TypeInfo,
}
impl Field {
    pub fn default_private() -> bool { true }
    pub fn default_read_write() -> ReadWrite { ReadWrite::Neither }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ClassField {
    pub name: String,
    #[serde(rename = "type")] pub type_info: TypeInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Method {
    pub name: String,
    #[serde(default)] pub c_name: Option<String>,
    #[serde(default)] pub params: Vec<Parameter>,
    #[serde(rename = "return")] pub return_type: TypeInfo,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ReadWrite {
    /// The value is neither readable nor writable.
    #[serde(rename = "n")]
    Neither,

    /// The value can only be read, not written.
    #[serde(rename = "ro")]
    ReadOnly,

    /// The value is provided during construction and can then only be read, not written.
    #[serde(rename = "rco")]
    ReadConstruct,

    /// The value can be read and written.
    #[default]
    #[serde(rename = "rw")]
    ReadWrite,

    /// The value is provided during construction and can be read and written.
    #[serde(rename = "rwc")]
    ReadWriteConstruct,

    /// The value can only be written, not read.
    #[serde(rename = "wo")]
    WriteOnly,

    /// The value is provided during construction and can only be written, not read.
    #[serde(rename = "wco")]
    WriteConstruct,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeInfo {
    pub name: String,
    pub params: Vec<String>,
}
impl<'de> Deserialize<'de> for TypeInfo {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let json_value = serde_json::Value::deserialize(deserializer)?;
        if let Some(jv) = json_value.as_str() {
            Ok(Self {
                name: jv.to_owned(),
                params: Vec::with_capacity(0),
            })
        } else if let Some(jv) = json_value.as_object() {
            let name = jv.get("name")
                .ok_or_else(|| D::Error::custom("TypeInfo object missing \"name\" entry"))?
                .as_str()
                .ok_or_else(|| D::Error::custom("TypeInfo object \"name\" entry not a string"))?;
            let params_val = jv.get("params")
                .ok_or_else(|| D::Error::custom("TypeInfo object missing \"params\" entry"))?;
            let params: Vec<String> = serde_json::from_value(params_val.clone())
                .map_err(|_| D::Error::custom("TypeInfo object \"params\" entry not an array of strings"))?;
            Ok(Self {
                name: name.to_owned(),
                params,
            })
        } else {
            Err(D::Error::custom("TypeInfo must be either a string (name) or an object with keys [name, params]"))
        }
    }
}
impl Serialize for TypeInfo {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.params.len() == 0 {
            self.name.serialize(serializer)
        } else {
            let value = serde_json::json!({
                "name": self.name,
                "params": self.params,
            });
            value.serialize(serializer)
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Parameter {
    Instance,
    Regular(RegularParameter),
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RegularParameter {
    pub name: String,
    #[serde(rename = "type")] pub type_info: TypeInfo,
}
