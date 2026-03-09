mod gir_xml;
mod model;
mod stringing;
mod typing;


use std::path::{Path, PathBuf};

use clap::Parser;
use xot::output::Indentation;
use xot::output::xml::{Declaration, Parameters};

use crate::gir_xml::{everything_lib_to_xml, lib_to_xml};
use crate::model::{Dependency, Document};
use crate::typing::TypeDatabase;


#[derive(Parser)]
struct Opts {
    pub grr_file: PathBuf,
    pub output_dir: PathBuf,
}

fn get_favorite_xml_params() -> Parameters {
    Parameters {
        indentation: Some(Indentation::default()),
        cdata_section_elements: Vec::default(),
        declaration: Some(Declaration::default()),
        doctype: None,
        unescaped_gt: false,
    }
}

fn write_xml_string(output_dir: &Path, library: &Dependency, gir_xml_str: &str) {
    let mut library_path = output_dir.to_owned();
    let library_fn = format!("{}-{}.gir", library.name, library.version);
    library_path.push(&library_fn);
    std::fs::write(&library_path, gir_xml_str)
        .expect("failed to write GIR file");
}

fn main() {
    let opts = Opts::parse();

    // read info
    let doc_string = std::fs::read_to_string(&opts.grr_file)
        .expect("failed to load grr YAML file");
    let mut doc: Document = strict_yaml_rust::serde::from_str(&doc_string)
        .expect("failed to parse grr YAML file");

    crate::typing::realize_parent_and_priv_fields(&mut doc);
    crate::typing::realize_property_getters_and_setters(&mut doc);
    crate::typing::realize_class_and_iface_structs(&mut doc);

    // enrich type information
    let type_database = TypeDatabase::try_from_mut_document(&mut doc)
        .expect("failed to construct type database");
    type_database.enrich_document(&mut doc)
        .expect("failed to enrich document from type database");

    println!("{:#?}", doc);

    for library in &doc.libraries {
        let (lib_xot, lib_doc) = lib_to_xml(library);
        let xml_params = get_favorite_xml_params();
        let lib_xml = lib_xot.serialize_xml_string(xml_params, lib_doc)
            .expect("failed to serialize GIR document");
        write_xml_string(&opts.output_dir, &library.to_dependency(), &lib_xml);
    }

    if let Some(everything_lib) = doc.everything_lib.as_ref() {
        let (el_xot, el_doc) = everything_lib_to_xml(&doc);
        let xml_params = get_favorite_xml_params();
        let el_xml = el_xot.serialize_xml_string(xml_params, el_doc)
            .expect("failed to serialize \"everything\" GIR document");
        write_xml_string(&opts.output_dir, everything_lib, &el_xml);
    }
}
