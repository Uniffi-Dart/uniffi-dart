use std::fmt::Debug;

use uniffi_bindgen::interface::Literal;

/// A trait tor the implementation.
pub trait CodeType: Debug {
    /// The language specific label used to reference this type. This will be used in
    /// method signatures and property declarations.
    fn type_label(&self) -> String;

    /// A representation of this type label that can be used as part of another
    /// identifier. e.g. `read_foo()`, or `FooInternals`.
    ///
    /// This is especially useful when creating specialized objects or methods to deal
    /// with this type only.
    fn canonical_name(&self) -> String {
        self.type_label()
    }

    fn literal(&self, _literal: &Literal) -> String {
        unimplemented!("Unimplemented for {}", self.type_label())
    }

    /// Name of the FfiConverter
    ///
    /// This is the object that contains the lower, write, lift, and read methods for this type.
    fn ffi_converter_name(&self) -> String {
        format!("FfiConverter{}", self.canonical_name())
    }

    /// Function to run at startup
    fn initialization_fn(&self) -> Option<String> {
        None
    }

    // The following must create an instance of the converter object
    fn lower(&self) -> String {
        format!("{}.lower", self.ffi_converter_name())
    }

    fn write(&self) -> String {
        format!("{}.write", self.ffi_converter_name())
    }

    fn lift(&self) -> String {
        format!("{}.lift", self.ffi_converter_name())
    }

    fn read(&self) -> String {
        format!("{}.read", self.ffi_converter_name())
    }
}
