use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error;


#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Document {
    #[serde(rename = "libs")] pub libraries: Vec<Library>,
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
    #[serde(default)] pub c_name: Option<String>,
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
    #[serde(default)] pub c_name: Option<String>,
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
    pub c_type: Option<String>,
    pub params: Vec<TypeInfo>,
}
impl<'de> Deserialize<'de> for TypeInfo {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let json_value = serde_json::Value::deserialize(deserializer)?;
        if let Some(jv) = json_value.as_str() {
            Ok(Self {
                name: jv.to_owned(),
                c_type: None,
                params: Vec::with_capacity(0),
            })
        } else if let Some(jv) = json_value.as_object() {
            let name = jv.get("name")
                .ok_or_else(|| D::Error::custom("TypeInfo object missing \"name\" entry"))?
                .as_str()
                .ok_or_else(|| D::Error::custom("TypeInfo object \"name\" entry not a string"))?;
            let c_type_val_opt = jv.get("c_type");
            let c_type = if let Some(c_type_val) = c_type_val_opt {
                if !c_type_val.is_null() {
                    let c_type_str = c_type_val
                        .as_str()
                        .ok_or_else(|| D::Error::custom("TypeInfo object \"c_type\" entry not a string"))?;
                    Some(c_type_str.to_owned())
                } else {
                    None
                }
            } else {
                None
            };
            let empty_array = serde_json::Value::Array(Vec::with_capacity(0));
            let params_val = jv.get("params")
                .unwrap_or_else(|| &empty_array);
            let params: Vec<TypeInfo> = serde_json::from_value(params_val.clone())
                .map_err(|_| D::Error::custom("TypeInfo object \"params\" entry not an array of strings or type information"))?;
            Ok(Self {
                name: name.to_owned(),
                c_type,
                params,
            })
        } else {
            Err(D::Error::custom("TypeInfo must be either a string (name) or an object with keys [name, params]"))
        }
    }
}
impl Serialize for TypeInfo {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.params.len() == 0 && self.c_type.is_none() {
            self.name.serialize(serializer)
        } else {
            let value = serde_json::json!({
                "name": self.name,
                "c_type": self.c_type,
                "params": self.params,
            });
            value.serialize(serializer)
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Parameter {
    Instance,
    Regular(RegularParameter),
}
impl<'de> Deserialize<'de> for Parameter {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut json_dict: BTreeMap<String, serde_json::Value> = Deserialize::deserialize(deserializer)?;

        let instance_val_opt = json_dict.get("instance");
        if let Some(instance_val) = instance_val_opt {
            let instance_str = instance_val.as_str()
                .ok_or_else(|| D::Error::custom("Parameter object \"instance\" entry not a string"))?;
            if instance_str == "true" {
                return if json_dict.len() > 1 {
                    Err(D::Error::custom("Parameter object with \"instance\" true has additional parameters"))
                } else {
                    Ok(Parameter::Instance)
                };
            } else if instance_str != "false" {
                return Err(D::Error::custom(format!("Parameter object has invalid \"instance\" value {:?}", instance_str)));
            }

            // keep processing as a regular parameter
        }

        json_dict.remove("instance");

        let regular_param_val: serde_json::Value = serde_json::to_value(&json_dict)
            .expect("failed to serialize Parameter object back to JSON value");
        let regular_param: RegularParameter = serde_json::from_value(regular_param_val)
            .map_err(|e| D::Error::custom(format!("failed to process Parameter with \"instance\" false: {}", e)))?;
        Ok(Self::Regular(regular_param))
    }
}
impl Serialize for Parameter {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Parameter::Instance => {
                serde_json::json!({
                    "instance": "true",
                }).serialize(serializer)
            },
            Parameter::Regular(regular_parameter) => {
                regular_parameter.serialize(serializer)
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RegularParameter {
    pub name: String,
    #[serde(rename = "type")] pub type_info: TypeInfo,
}
