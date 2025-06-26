use genco::lang::dart;
use genco::prelude::*;
use paste::paste;
use crate::gen::CodeType;
use uniffi_bindgen::interface::Type;

use super::oracle::{AsCodeType, DartCodeOracle};
use crate::gen::render::{Renderable, TypeHelperRenderer};

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
            }
        }
    }
 }

macro_rules! impl_renderable_for_compound {
    ($T:ty, $type_label_pattern:literal, $canonical_name_pattern: literal) => {
       paste! {
            impl Renderable for $T {
                fn render_type_helper(&self, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
                    type_helper.include_once_check($canonical_name_pattern, &self.self_type);
                    let inner_codetype = DartCodeOracle::find(self.inner());
                    let inner_type_label = inner_codetype.type_label();

                    type_helper.include_once_check(&inner_codetype.canonical_name(), &self.inner()); // Add the Inner FFI Converter

                    let cl_name = &format!($canonical_name_pattern, inner_codetype.canonical_name());
                    let type_label = &format!($type_label_pattern, &inner_type_label);

                    let inner_cl_converter_name = &inner_codetype.ffi_converter_name();
                    let inner_data_type = &inner_codetype.canonical_name().replace("UInt", "Uint").replace("Double", "Float");
                    let _inner_type_signature = if inner_data_type.contains("Float") { "double" } else { "int" };


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
                    }
                }
            }
       }
   };

   (SequenceCodeType, $canonical_name_pattern: literal) => {
        paste! {
            impl Renderable for SequenceCodeType {
                fn render_type_helper(&self, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {

                    type_helper.include_once_check($canonical_name_pattern, &self.self_type);
                    let inner_codetype = self.inner().as_codetype();
                    let inner_type_label = inner_codetype.type_label();

                    type_helper.include_once_check(&inner_codetype.canonical_name(), &self.inner()); // Add the Inner FFI Converter

                    let cl_name = &format!($canonical_name_pattern, inner_codetype.canonical_name());
                    let type_label = &format!("List<{}>", &inner_type_label);

                    let inner_cl_converter_name = &inner_codetype.ffi_converter_name();
                    let inner_data_type = &inner_codetype.canonical_name().replace("UInt", "Uint").replace("Double", "Float");
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
                                return value.map((l) => $inner_cl_converter_name.allocationSize(l)).reduce((a, b) => a + b) + 4;
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
