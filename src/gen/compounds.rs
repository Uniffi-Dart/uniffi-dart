use crate::gen::CodeType;
use genco::lang::dart;
use genco::prelude::*;
use paste::paste;
use uniffi_bindgen::interface::Type;

use super::oracle::{AsCodeType, DartCodeOracle};
use crate::gen::render::{AsRenderable, Renderable, TypeHelperRenderer};

macro_rules! impl_code_type_for_compound {
     ($T:ty, $type_label_pattern:literal, $canonical_name_pattern: literal) => {
        paste! {
            #[derive(Debug)]
            pub struct $T {
                self_type: Type,
                inner: Type,
            }

            impl $T {
                pub fn new(self_type: Type, inner: Type) -> Self {
                    Self { self_type, inner }
                }
                fn inner(&self) -> &Type {
                    &self.inner
                }
            }

            impl CodeType for $T  {
                fn type_label(&self) -> String {
                    format!($type_label_pattern, DartCodeOracle::find(self.inner()).type_label())
                }

                fn canonical_name(&self) -> String {
                    format!($canonical_name_pattern, DartCodeOracle::find(self.inner()).canonical_name())
                }

                fn add_additional_type_helpers(&self, type_helper: &dyn TypeHelperRenderer) {
                    type_helper.include_once_check(&self.inner().as_codetype().canonical_name(), self.inner());
                }
            }
        }
    }
 }

macro_rules! impl_renderable_for_compound {
    ($T:ty, $type_label_pattern:literal, $canonical_name_pattern: literal) => {
       paste! {
            impl Renderable for $T {
                fn render_type_helper(&self, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
                    type_helper.include_once_check(&self.ffi_converter_name(), &self.self_type);
                    let inner_codetype = DartCodeOracle::find(self.inner());

                    let original_canonical = inner_codetype.canonical_name();
                    let canonical_with_exception =
                        DartCodeOracle::exception_safe_name(&original_canonical);
                    let inner_already_registered =
                        type_helper.include_once_check(&original_canonical, &self.inner())
                            || (canonical_with_exception != original_canonical
                                && type_helper.include_once_check(
                                    &canonical_with_exception,
                                    &self.inner(),
                                ));

                    let raw_type_label = inner_codetype.type_label();
                    let inner_type_label =
                        DartCodeOracle::exception_safe_name(&raw_type_label);

                    let cl_name_buf =
                        format!($canonical_name_pattern, canonical_with_exception.as_str());
                    let cl_name = &cl_name_buf;
                    let type_label_buf =
                        format!($type_label_pattern, inner_type_label.as_str());
                    let type_label = &type_label_buf;

                    let raw_converter_name = inner_codetype.ffi_converter_name();
                    let inner_cl_converter_name_buf =
                        DartCodeOracle::exception_safe_name(&raw_converter_name);
                    let inner_cl_converter_name = &inner_cl_converter_name_buf;
                    let inner_data_type_buf = canonical_with_exception
                        .as_str()
                        .replace("UInt", "Uint")
                        .replace("Double", "Float");
                    let inner_data_type = &inner_data_type_buf;
                    let _inner_type_signature =
                        if inner_data_type.contains("Float") { "double" } else { "int" };

                    // Render inner helper for Sequences and primitives that haven't been rendered yet
                    let inner_helper = if !inner_already_registered {
                        match self.inner() {
                            Type::Sequence { .. }
                            | Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64
                            | Type::UInt8 | Type::UInt16 | Type::UInt32 | Type::UInt64
                            | Type::Float32 | Type::Float64 => {
                                self.inner().as_renderable().render_type_helper(type_helper)
                            }
                            _ => quote!()
                        }
                    } else {
                        quote!()
                    };

                    quote! {
                        class $cl_name {

                            static $type_label lift( RustBuffer buf) {
                                return $cl_name.read(buf.asUint8List()).value;
                            }

                            static LiftRetVal<$type_label> read( Uint8List buf) {
                                if (ByteData.view(buf.buffer, buf.offsetInBytes).getInt8(0) == 0){
                                    return LiftRetVal(null, 1);
                                }
                                final result = $inner_cl_converter_name.read(Uint8List.view(buf.buffer, buf.offsetInBytes + 1));
                                return LiftRetVal<$type_label>(result.value, result.bytesRead + 1);
                            }


                            static int allocationSize([$type_label value]) {
                                if (value == null) {
                                    return 1;
                                }
                                return $inner_cl_converter_name.allocationSize(value) + 1;
                            }

                            static RustBuffer lower( $type_label value) {
                                if (value == null) {
                                    return toRustBuffer(Uint8List.fromList([0]));
                                }

                                final length = $cl_name.allocationSize(value);

                                final Pointer<Uint8> frameData = calloc<Uint8>(length); // Allocate a pointer large enough.
                                final buf = frameData.asTypedList(length); // Create a list that uses our pointer to copy in the data.

                                $cl_name.write(value, buf);

                                final bytes = calloc<ForeignBytes>();
                                bytes.ref.len = length;
                                bytes.ref.data = frameData;
                                return RustBuffer.fromBytes(bytes.ref);
                            }

                            static int write( $type_label value, Uint8List buf) {
                                if (value == null) {
                                    buf[0] = 0;
                                    return 1;
                                }
                                // we have a value
                                buf[0] = 1;

                                return $inner_cl_converter_name.write(value, Uint8List.view(buf.buffer, buf.offsetInBytes + 1)) + 1;
                            }
                        }
                        $inner_helper
                    }
                }
            }
       }
   };

   (SequenceCodeType, $canonical_name_pattern: literal) => {
        paste! {
            impl Renderable for SequenceCodeType {
                fn render_type_helper(&self, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {

                    let converter_name = self.ffi_converter_name();
                    if type_helper.include_once_check(&converter_name, &self.self_type) {
                        return quote!();
                    }
                    let inner_codetype = self.inner().as_codetype();

                    let original_canonical = inner_codetype.canonical_name();
                    let canonical_with_exception =
                        DartCodeOracle::exception_safe_name(&original_canonical);
                    type_helper.include_once_check(&original_canonical, &self.inner());
                    if canonical_with_exception != original_canonical {
                        type_helper.include_once_check(&canonical_with_exception, &self.inner());
                    }

                    let raw_type_label = inner_codetype.type_label();
                    let inner_type_label =
                        DartCodeOracle::exception_safe_name(&raw_type_label);

                    let cl_name_buf =
                        format!($canonical_name_pattern, canonical_with_exception.as_str());
                    let cl_name = &cl_name_buf;
                    let type_label_buf = format!("List<{}>", inner_type_label.as_str());
                    let type_label = &type_label_buf;

                    let raw_converter_name = inner_codetype.ffi_converter_name();
                    let inner_cl_converter_name_buf =
                        DartCodeOracle::exception_safe_name(&raw_converter_name);
                    let inner_cl_converter_name = &inner_cl_converter_name_buf;
                    let inner_data_type = canonical_with_exception
                        .as_str()
                        .replace("UInt", "Uint")
                        .replace("Double", "Float");
                    let _inner_type_signature = if inner_data_type.contains("Float") { "double" } else { "int" };


                    quote! {
                        class $cl_name {

                            static $type_label lift( RustBuffer buf) {
                                return $cl_name.read(buf.asUint8List()).value;
                            }

                            static LiftRetVal<$type_label> read( Uint8List buf) {
                                $type_label res = [];
                                final length = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0);
                                int offset = buf.offsetInBytes + 4;
                                for (var i = 0; i < length; i++) {
                                    final ret = $inner_cl_converter_name.read(Uint8List.view(buf.buffer, offset));
                                    offset += ret.bytesRead;
                                    res.add(ret.value);
                                }
                                return LiftRetVal(res, offset - buf.offsetInBytes);
                            }

                            static int write( $type_label value, Uint8List buf) {
                                buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, value.length);
                                int offset = buf.offsetInBytes + 4;
                                for (var i = 0; i < value.length; i++) {
                                    offset += $inner_cl_converter_name.write(value[i], Uint8List.view(buf.buffer, offset));
                                }
                                return offset - buf.offsetInBytes;
                            }
                            static int allocationSize($type_label value) {
                                return value.map((l) => $inner_cl_converter_name.allocationSize(l)).fold(0, (a, b) => a + b) + 4;
                            }

                            static RustBuffer lower( $type_label value) {
                                final buf = Uint8List(allocationSize(value));
                                write(value, buf);
                                return toRustBuffer(buf);
                            }
                        }
                    }
                }
            }
        }
   }
}

impl_code_type_for_compound!(OptionalCodeType, "{}?", "Optional{}");
impl_code_type_for_compound!(SequenceCodeType, "List<{}>", "Sequence{}");

impl_renderable_for_compound!(OptionalCodeType, "{}?", "FfiConverterOptional{}");
impl_renderable_for_compound!(SequenceCodeType, "FfiConverterSequence{}");

// Map<K, V>
#[derive(Debug)]
pub struct MapCodeType {
    self_type: Type,
    key: Type,
    value: Type,
}

impl MapCodeType {
    pub fn new(self_type: Type, key: Type, value: Type) -> Self {
        Self {
            self_type,
            key,
            value,
        }
    }

    fn key(&self) -> &Type {
        &self.key
    }

    fn value(&self) -> &Type {
        &self.value
    }
}

impl CodeType for MapCodeType {
    fn type_label(&self) -> String {
        format!(
            "Map<{}, {}>",
            DartCodeOracle::find(self.key()).type_label(),
            DartCodeOracle::find(self.value()).type_label()
        )
    }

    fn canonical_name(&self) -> String {
        let key = DartCodeOracle::find(self.key()).canonical_name();
        let val = DartCodeOracle::find(self.value()).canonical_name();
        format!("Map{}To{}", key, val)
    }
}

impl Renderable for MapCodeType {
    fn render_type_helper(&self, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
        type_helper.include_once_check(&self.ffi_converter_name(), &self.self_type);

        let key_codetype = DartCodeOracle::find(self.key());
        let val_codetype = DartCodeOracle::find(self.value());

        type_helper.include_once_check(&key_codetype.canonical_name(), self.key());
        type_helper.include_once_check(&val_codetype.canonical_name(), self.value());

        let cl_name = &self.ffi_converter_name();
        let key_type_label_owned = key_codetype.type_label();
        let val_type_label_owned = val_codetype.type_label();
        let key_type_label = &key_type_label_owned;
        let val_type_label = &val_type_label_owned;

        let key_conv_owned = key_codetype.ffi_converter_name();
        let val_conv_owned = val_codetype.ffi_converter_name();
        let key_conv = &key_conv_owned;
        let val_conv = &val_conv_owned;

        quote! {
            class $cl_name {
                static Map<$key_type_label, $val_type_label> lift(RustBuffer buf) {
                    return $cl_name.read(buf.asUint8List()).value;
                }

                static LiftRetVal<Map<$key_type_label, $val_type_label>> read(Uint8List buf) {
                    final map = <$key_type_label, $val_type_label>{};
                    final length = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0);
                    int offset = buf.offsetInBytes + 4;
                    for (var i = 0; i < length; i++) {
                        final k = $key_conv.read(Uint8List.view(buf.buffer, offset));
                        offset += k.bytesRead;
                        final v = $val_conv.read(Uint8List.view(buf.buffer, offset));
                        offset += v.bytesRead;
                        map[k.value] = v.value;
                    }
                    return LiftRetVal(map, offset - buf.offsetInBytes);
                }

                static int write(Map<$key_type_label, $val_type_label> value, Uint8List buf) {
                    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, value.length);
                    int offset = buf.offsetInBytes + 4;
                    for (final entry in value.entries) {
                        offset += $key_conv.write(entry.key, Uint8List.view(buf.buffer, offset));
                        offset += $val_conv.write(entry.value, Uint8List.view(buf.buffer, offset));
                    }
                    return offset - buf.offsetInBytes;
                }

                static int allocationSize(Map<$key_type_label, $val_type_label> value) {
                    return value.entries
                        .map((e) => $key_conv.allocationSize(e.key) + $val_conv.allocationSize(e.value))
                        .fold(4, (a, b) => a + b);
                }

                static RustBuffer lower(Map<$key_type_label, $val_type_label> value) {
                    final buf = Uint8List(allocationSize(value));
                    write(value, buf);
                    return toRustBuffer(buf);
                }
            }
        }
    }
}
