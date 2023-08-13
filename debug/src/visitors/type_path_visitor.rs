use std::collections::HashMap;
use syn::visit::{self, Visit};

pub(crate) struct TypePahtVisitor {
    pub(crate) generic_type_names: Vec<String>,
    pub(crate) associated_types: HashMap<String, Vec<syn::TypePath>>,
}

impl<'ast> Visit<'ast> for TypePahtVisitor {
    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        if node.path.segments.len() >= 2 {
            let generic_type_name = node.path.segments[0].ident.to_string();
            if self.generic_type_names.contains(&generic_type_name) {
                self.associated_types.entry(generic_type_name).or_insert(vec![]).push(node.clone())
            }
        }
        visit::visit_type_path(self, node);
    }
}
