use std::fmt::Write;

use xot::{NamespaceId, Node, Xot};

use crate::model::{Library, Method, Parameter, TypeInfo};


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


fn pascal_to_snake_case(pascal: &str) -> String {
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


#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct NamespacePack {
    core_id: NamespaceId,
    c_id: NamespaceId,
    glib_id: NamespaceId,
}


#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct MethodsState<'a> {
    ns: NamespacePack,
    symbol_prefix: &'a str,

    /// Name of the class or interface containing this method.
    parent_name: &'a str,

    /// Name of the class or interface containing this method in the C representation.
    ///
    /// "C" refers to the programming language.
    c_parent_name: &'a str,
}


fn append_type(xot: &mut Xot, parent_elem: Node, ns: NamespacePack, type_info: &TypeInfo, topmost_type: bool) {
    let type_elem = xot.new_child_element(parent_elem, "type", ns.core_id);
    let (type_name, c_type) = match &type_info.name {
        _ => todo!(),
    };
    xot.set_attribute_value(type_elem, "name", type_name);
    if topmost_type {
        xot.set_ns_attribute_value(type_elem, "type", ns.c_id, c_type);
    }
    for type_param in &type_info.params {
        append_type(xot, type_elem, ns, type_param, false);
    }
}


fn append_methods(xot: &mut Xot, parent_elem: Node, state: MethodsState, methods: &[Method]) {
    for method in methods {
        let method_elem = xot.new_child_element(parent_elem, "method", state.ns.core_id);
        xot.set_attribute_value(method_elem, "name", &method.name);

        let c_ident = format!("{}{}", state.symbol_prefix, method.name);
        xot.set_ns_attribute_value(method_elem, "identifier", state.ns.c_id, &c_ident);

        let params_elem = xot.new_child_element(method_elem, "parameters", state.ns.core_id);

        for param in &method.params {
            match param {
                Parameter::Instance => {
                    let name = pascal_to_snake_case(&state.parent_name);
                    let type_name = state.parent_name;
                    let c_type_name = state.c_parent_name;

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

    //let known_records = BTreeSet::new();

    for interface in &library.interfaces {
        let iface_elem = xot.new_child_element(ns_elem, "interface", ns.core_id);
        xot.set_attribute_value(iface_elem, "name", &interface.name);

        // Somethingable -> PrefixSomethingable
        let c_name_storage;
        let c_name = if let Some(cn) = &interface.c_name {
            cn.as_str()
        } else {
            c_name_storage = format!("{}{}", library.c_id_prefix, library.name);
            c_name_storage.as_str()
        };
        xot.set_ns_attribute_value(iface_elem, "type", ns.c_id, c_name);
        xot.set_ns_attribute_value(iface_elem, "type-name", ns.glib_id, c_name);

        // Somethingable -> SomethingableInterface
        let type_struct = format!("{}Interface", interface.name);
        xot.set_ns_attribute_value(iface_elem, "type-struct", ns.glib_id, &type_struct);

        // prefix_, Somethingable -> prefix_somethingable_, prefix_somethingable_get_type
        let symbol_prefix = format!("{}{}_", library.c_sym_prefix, pascal_to_snake_case(&interface.name));
        xot.set_ns_attribute_value(iface_elem, "symbol-prefix", ns.c_id, &symbol_prefix);
        let get_type_func = format!("{}get_type", symbol_prefix);
        xot.set_ns_attribute_value(iface_elem, "get-type", ns.glib_id, &get_type_func);

        let methods_state = MethodsState {
            ns,
            symbol_prefix: &symbol_prefix,
            parent_name: &interface.name,
            c_parent_name: c_name,
        };
        append_methods(&mut xot, iface_elem, methods_state, &interface.methods);
    }

    (xot, repo_doc)
}
