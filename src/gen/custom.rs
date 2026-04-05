use super::oracle::{AsCodeType, DartCodeOracle};
use super::render::{Renderable, TypeHelperRenderer};
use super::CodeType;
use genco::prelude::*;
use uniffi_bindgen::interface::AsType;
use uniffi_bindgen::interface::Type;

#[derive(Debug)]
pub struct CustomCodeType {
    name: String,
    module_path: String,
    builtin: Box<Type>,
}

impl CustomCodeType {
    pub fn new(name: String, module_path: String, builtin: Box<Type>) -> Self {
        CustomCodeType {
            name,
            module_path,
            builtin,
        }
    }
}

impl CodeType for CustomCodeType {
    fn type_label(&self) -> String {
        DartCodeOracle::class_name(&self.name)
    }
    
    fn add_additional_type_helpers(&self, type_helper: &dyn TypeHelperRenderer) {
        type_helper.include_once_check(&self.builtin.as_codetype().canonical_name(), &self.builtin);
    }
}

impl AsType for CustomCodeType {
    fn as_type(&self) -> Type {
        Type::Custom {
            name: self.name.clone(),
            module_path: self.module_path.clone(),
            builtin: self.builtin.clone(),
        }
    }
}

impl Renderable for CustomCodeType {
    fn render_type_helper(&self, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
        type_helper.include_once_check(&self.name, &self.as_type());

        let ffi_converter_name = &self.ffi_converter_name();
        let type_name = &self.type_label();
        let builtin_ffi_converter_name = &(*self.builtin).as_codetype().ffi_converter_name();
        let builtin_name = DartCodeOracle::dart_type_label(Some(&*self.builtin));

        quote! {
            typedef $(type_name) = $(builtin_name);
            typedef $(ffi_converter_name) = $(builtin_ffi_converter_name);
        }
    }
}
