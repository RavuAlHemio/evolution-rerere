use xot::{NamespaceId, Node, Xot};

use crate::model::{ClassField, Field, Library, Method, Parameter, Property, ReadWrite, TypeInfo};
use crate::typing::pascal_to_snake_case;


trait XotExt {
    fn set_attribute_value(&mut self, node: Node, name: &str, value: &str);
    fn set_ns_attribute_value(&mut self, node: Node, name: &str, namespace_id: NamespaceId, value: &str);
    fn new_child_element(&mut self, parent: Node, local_name: &str, namespace_id: NamespaceId) -> Node;
}
impl XotExt for Xot {
    fn set_attribute_value(&mut self, node: Node, name: &str, value: &str) {
        let name_id = self.add_name(name);
        self.set_attribute(node, name_id, value);
    }

    fn set_ns_attribute_value(&mut self, node: Node, name: &str, namespace_id: NamespaceId, value: &str) {
        let name_id = self.add_name_ns(name, namespace_id);
        self.set_attribute(node, name_id, value);
    }

    fn new_child_element(&mut self, parent: Node, local_name: &str, namespace_id: NamespaceId) -> Node {
        let name_id = self.add_name_ns(local_name, namespace_id);
        let elem = self.new_element(name_id);
        self.append(parent, elem).unwrap();
        elem
    }
}


#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct NamespacePack {
    core_id: NamespaceId,
    c_id: NamespaceId,
    glib_id: NamespaceId,
}


#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct InterfaceOrClassState<'a> {
    ns: NamespacePack,

    /// Name of the class or interface.
    ioc_name: &'a str,

    /// Name of the class or interface in the C representation.
    ///
    /// "C" refers to the programming language.
    ioc_c_name: &'a str,

    /// The C symbol (e.g. function or global variable name) prefix of the class or interface.
    ///
    /// "C" refers to the programming language.
    ioc_c_sym_prefix: &'a str,

    /// The C identifier (e.g. struct name) prefix of the library.
    ///
    /// "C" refers to the programming language.
    lib_c_id_prefix: &'a str,
}


fn append_type(xot: &mut Xot, parent_elem: Node, ns: NamespacePack, type_info: &TypeInfo, topmost_type: bool) {
    let type_elem = xot.new_child_element(parent_elem, "type", ns.core_id);
    xot.set_attribute_value(type_elem, "name", &type_info.name);
    if let Some(c_type) = type_info.c_type.as_ref() {
        if topmost_type {
            xot.set_ns_attribute_value(type_elem, "type", ns.c_id, c_type);
        }
    }
    for type_param in &type_info.params {
        append_type(xot, type_elem, ns, type_param, false);
    }
}


fn append_field(xot: &mut Xot, parent_elem: Node, state: InterfaceOrClassState, field: &Field) {
    let field_elem = xot.new_child_element(parent_elem, "field", state.ns.core_id);
    xot.set_attribute_value(field_elem, "name", &field.name);

    if field.private {
        xot.set_attribute_value(field_elem, "private", "1");
    }

    match field.rw {
        ReadWrite::Neither => {
            xot.set_attribute_value(field_elem, "readable", "0");
            xot.set_attribute_value(field_elem, "writable", "0");
        },
        ReadWrite::ReadOnly => {
            xot.set_attribute_value(field_elem, "readable", "1");
            xot.set_attribute_value(field_elem, "writable", "0");
        },
        ReadWrite::ReadConstruct => {
            xot.set_attribute_value(field_elem, "readable", "1");
            xot.set_attribute_value(field_elem, "writable", "1");
            xot.set_attribute_value(field_elem, "construct-only", "1");
        },
        ReadWrite::ReadWrite => {
            xot.set_attribute_value(field_elem, "readable", "1");
            xot.set_attribute_value(field_elem, "writable", "1");
        },
        ReadWrite::ReadWriteConstruct => {
            xot.set_attribute_value(field_elem, "readable", "1");
            xot.set_attribute_value(field_elem, "writable", "1");
            xot.set_attribute_value(field_elem, "construct", "1");
        },
        ReadWrite::WriteOnly => {
            xot.set_attribute_value(field_elem, "readable", "0");
            xot.set_attribute_value(field_elem, "writable", "1");
        },
        ReadWrite::WriteConstruct => {
            xot.set_attribute_value(field_elem, "writable", "1");
            xot.set_attribute_value(field_elem, "construct", "1");
        },
    }
    append_type(xot, field_elem, state.ns, &field.type_info, true);
}


fn append_property(xot: &mut Xot, parent_elem: Node, state: InterfaceOrClassState, property: &Property) {
    let prop_elem = xot.new_child_element(parent_elem, "property", state.ns.core_id);
    xot.set_attribute_value(prop_elem, "name", &property.name);

    match property.rw {
        ReadWrite::Neither => {
            xot.set_attribute_value(prop_elem, "readable", "0");
            xot.set_attribute_value(prop_elem, "writable", "0");
        },
        ReadWrite::ReadOnly => {
            xot.set_attribute_value(prop_elem, "readable", "1");
            xot.set_attribute_value(prop_elem, "writable", "0");
        },
        ReadWrite::ReadConstruct => {
            xot.set_attribute_value(prop_elem, "readable", "1");
            xot.set_attribute_value(prop_elem, "writable", "1");
            xot.set_attribute_value(prop_elem, "construct-only", "1");
        },
        ReadWrite::ReadWrite => {
            xot.set_attribute_value(prop_elem, "readable", "1");
            xot.set_attribute_value(prop_elem, "writable", "1");
        },
        ReadWrite::ReadWriteConstruct => {
            xot.set_attribute_value(prop_elem, "readable", "1");
            xot.set_attribute_value(prop_elem, "writable", "1");
            xot.set_attribute_value(prop_elem, "construct", "1");
        },
        ReadWrite::WriteOnly => {
            xot.set_attribute_value(prop_elem, "readable", "0");
            xot.set_attribute_value(prop_elem, "writable", "1");
        },
        ReadWrite::WriteConstruct => {
            xot.set_attribute_value(prop_elem, "writable", "1");
            xot.set_attribute_value(prop_elem, "construct", "1");
        },
    }

    if let Some(getter_name) = property.getter_name.as_ref() {
        xot.set_attribute_value(prop_elem, "getter", getter_name);
    }
    if let Some(setter_name) = property.setter_name.as_ref() {
        xot.set_attribute_value(prop_elem, "setter", setter_name);
    }

    append_type(xot, prop_elem, state.ns, &property.type_info, true);
}


fn append_method(xot: &mut Xot, parent_elem: Node, state: InterfaceOrClassState, method: &Method) {
    let method_elem = xot.new_child_element(parent_elem, "method", state.ns.core_id);
    xot.set_attribute_value(method_elem, "name", &method.name);

    let c_ident = format!("{}{}", state.ioc_c_sym_prefix, method.name);
    xot.set_ns_attribute_value(method_elem, "identifier", state.ns.c_id, &c_ident);

    let params_elem = xot.new_child_element(method_elem, "parameters", state.ns.core_id);

    for param in &method.params {
        match param {
            Parameter::Instance => {
                let name = pascal_to_snake_case(&state.ioc_name);
                let type_name = state.ioc_name;
                let c_type_name = state.ioc_c_name;

                let ipar_elem = xot.new_child_element(params_elem, "instance-parameter", state.ns.core_id);
                xot.set_attribute_value(ipar_elem, "name", &name);
                xot.set_attribute_value(ipar_elem, "transfer-ownership", "none");
                let type_elem = xot.new_child_element(ipar_elem, "type", state.ns.core_id);
                xot.set_attribute_value(type_elem, "name", type_name);
                xot.set_ns_attribute_value(type_elem, "type", state.ns.c_id, &format!("{}*", c_type_name));
            },
            Parameter::Regular(regular_parameter) => {
                let par_elem = xot.new_child_element(params_elem, "parameter", state.ns.core_id);
                xot.set_attribute_value(par_elem, "name", &regular_parameter.name);
                // TODO: transfer-ownership
                append_type(xot, par_elem, state.ns, &regular_parameter.type_info, true);
            },
        }
    }

    let ret_val_elem = xot.new_child_element(method_elem, "return-value", state.ns.core_id);
    append_type(xot, ret_val_elem, state.ns, &method.return_type, true);
}


fn append_interface_or_class(
    xot: &mut Xot,
    state: InterfaceOrClassState<'_>,
    ns_elem: Node,
    elem_name: &str,
    type_struct_suffix: &str,
    fields: &[Field],
    class_fields: &[ClassField],
    properties: &[Property],
    methods: &[Method],
) {
    let ioc_elem = xot.new_child_element(ns_elem, elem_name, state.ns.core_id);
    xot.set_attribute_value(ioc_elem, "name", state.ioc_name);

    xot.set_ns_attribute_value(ioc_elem, "type", state.ns.c_id, state.ioc_c_name);
    xot.set_ns_attribute_value(ioc_elem, "type-name", state.ns.glib_id, state.ioc_c_name);
    xot.set_ns_attribute_value(ioc_elem, "symbol-prefix", state.ns.c_id, state.ioc_c_sym_prefix);

    // Somethingable -> SomethingableInterface/SomethingableClass
    let type_struct = format!("{}{}", state.ioc_name, type_struct_suffix);
    xot.set_ns_attribute_value(ioc_elem, "type-struct", state.ns.glib_id, &type_struct);

    // prefix_, Somethingable -> prefix_somethingable_get_type
    let get_type_func = format!("{}get_type", state.ioc_c_sym_prefix);
    xot.set_ns_attribute_value(ioc_elem, "get-type", state.ns.glib_id, &get_type_func);

    for field in fields {
        append_field(xot, ioc_elem, state, field);
    }

    for property in properties {
        append_property(xot, ioc_elem, state, property);
    }

    for method in methods {
        append_method(xot, ioc_elem, state, method);
    }
}

pub(crate) fn lib_to_xml(library: &Library) -> (Xot, Node) {
    let mut xot = Xot::new();

    let ns: NamespacePack = NamespacePack {
        core_id: xot.add_namespace("http://www.gtk.org/introspection/core/1.0"),
        c_id: xot.add_namespace("http://www.gtk.org/introspection/c/1.0"),
        glib_id: xot.add_namespace("http://www.gtk.org/introspection/glib/1.0"),
    };

    let core_ns_prefix = xot.empty_prefix();
    let c_ns_prefix = xot.add_prefix("c");
    let glib_ns_prefix = xot.add_prefix("glib");

    let core_ns_node = xot.new_namespace_node(core_ns_prefix, ns.core_id);
    let c_ns_node = xot.new_namespace_node(c_ns_prefix, ns.c_id);
    let glib_ns_node = xot.new_namespace_node(glib_ns_prefix, ns.glib_id);

    let repo_name = xot.add_name_ns("repository", ns.core_id);
    let repo_elem = xot.new_element(repo_name);
    let repo_doc = xot.new_document_with_element(repo_elem)
        .expect("failed to create document element");
    xot.append_namespace_node(repo_elem, core_ns_node).unwrap();
    xot.append_namespace_node(repo_elem, c_ns_node).unwrap();
    xot.append_namespace_node(repo_elem, glib_ns_node).unwrap();

    xot.set_attribute_value(repo_elem, "version", "1.2");

    let ns_elem = xot.new_child_element(repo_elem, "namespace", ns.core_id);
    xot.set_attribute_value(ns_elem, "name", &library.name);
    xot.set_attribute_value(ns_elem, "version", &library.version);
    xot.set_ns_attribute_value(ns_elem, "identifier-prefixes", ns.c_id, &library.c_id_prefix);
    xot.set_ns_attribute_value(ns_elem, "symbol-prefixes", ns.c_id, &library.c_sym_prefix);
    if let Some(sl) = library.shared_lib.as_ref() {
        xot.set_attribute_value(ns_elem, "shared-library", sl);
    }

    for interface in &library.interfaces {
        let symbol_prefix = format!("{}{}_", library.c_sym_prefix, pascal_to_snake_case(&interface.name));
        let state = InterfaceOrClassState {
            ns,
            ioc_name: &interface.name,
            ioc_c_name: interface.c_name.as_ref().expect("interface has no C name"),
            ioc_c_sym_prefix: &symbol_prefix,
            lib_c_id_prefix: &library.c_id_prefix,
        };
        append_interface_or_class(
            &mut xot,
            state,
            ns_elem,
            "interface",
            "Interface",
            &[],
            &[],
            &interface.properties,
            &interface.methods,
        );
    }

    for cls in &library.classes {
        let symbol_prefix = format!("{}{}_", library.c_sym_prefix, pascal_to_snake_case(&cls.name));
        let state = InterfaceOrClassState {
            ns,
            ioc_name: &cls.name,
            ioc_c_name: cls.c_name.as_ref().expect("class has no C name"),
            ioc_c_sym_prefix: &symbol_prefix,
            lib_c_id_prefix: &library.c_id_prefix,
        };

        append_interface_or_class(
            &mut xot,
            state,
            ns_elem,
            "class",
            "Class",
            &cls.fields,
            &cls.class_fields,
            &cls.properties,
            &cls.methods,
        );
    }

    (xot, repo_doc)
}
