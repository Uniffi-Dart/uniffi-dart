macro_rules! impl_code_type_for_primitive {
    ($T:ty, $class_name:literal, $canonical_name:literal) => {
        paste! {
            #[derive(Debug)]
            pub struct $T;

            impl crate::gen::CodeType for $T  {
                fn type_label(&self,) -> String {
                    $class_name.into()
                }

                fn literal(&self, literal: &uniffi_bindgen::interface::Literal) -> String {
                    $crate::gen::primitives::render_literal(&literal)
                }

                fn canonical_name(&self,) -> String {
                    $canonical_name.into()
                }

                fn ffi_converter_name(&self) -> String {
                    format!("FfiConverter{}", self.canonical_name())
                }
            }
        }
    };
}

macro_rules! impl_renderable_for_primitive {
    (BytesCodeType, $class_name:literal, $canonical_name:literal) => {
        impl Renderable for BytesCodeType {
            fn render_type_helper(&self, _type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
                let cl_name = &self.ffi_converter_name();
                let type_signature = &self.type_label();

                quote! {
                    class $cl_name {
                        static $type_signature lift(RustBuffer value) {
                            return $cl_name.read(value.asUint8List()).value;
                        }

                        static LiftRetVal<$type_signature> read(Uint8List buf) {
                            final length = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0);
                            final bytes = Uint8List.view(buf.buffer, buf.offsetInBytes + 4, length);
                            return LiftRetVal(bytes, length + 4);
                        }

                        static RustBuffer lower($type_signature value) {
                            final buf = Uint8List(allocationSize(value));
                            write(value, buf);
                            return toRustBuffer(buf);
                        }

                        static int allocationSize([$type_signature? value]) {
                          if (value == null) {
                              return 4;
                          }
                          return 4 + value.length;
                        }

                        static int write($type_signature value, Uint8List buf) {
                            buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, value.length);
                            buf.setRange(4, 4 + value.length, value);
                            return 4 + value.length;
                        }
                    }
                }
            }
        }
    };
    ($T:ty, $class_name:literal, $canonical_name:literal, $allocation_size:literal) => {
        impl Renderable for $T {
            fn render_type_helper(&self, _type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
                use crate::gen::code_type::CodeType;
                let endian = (if $canonical_name.contains("Float") {
                    ", Endian.little"
                } else {
                    ""
                });

                let cl_name = &self.ffi_converter_name();
                let type_signature = &self.type_label();
                let conversion_name = &$canonical_name
                                    .replace("UInt", "Uint")
                                    .replace("Double", "Float");

                quote! {
                    class $cl_name {
                        // According to generated funtion signatures, we won't need to convert number types
                        static $type_signature lift($type_signature value) => value;


                        static LiftRetVal<$type_signature> read(Uint8List buf) {
                            return LiftRetVal(buf.buffer.asByteData(buf.offsetInBytes).get$conversion_name(0), $allocation_size);
                        }

                        static $type_signature lower($type_signature value) => value;


                        static int allocationSize([$type_signature value = 0]) {
                          return $allocation_size;
                        }

                        static int write($type_signature value, Uint8List buf) {
                            buf.buffer.asByteData(buf.offsetInBytes).set$conversion_name(0, value$endian);
                            return $cl_name.allocationSize();
                        }

                    }
                }
            }
        }
    };
    ($T:ty, $class_name:literal, $canonical_name:literal, $allocation_size:literal, $min_value:literal, $max_value:literal, $type_name:literal) => {
        impl Renderable for $T {
            fn render_type_helper(&self, _type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
                let cl_name = &self.ffi_converter_name();
                let type_signature = &self.type_label();
                let conversion_name = &$canonical_name
                    .replace("UInt", "Uint")
                    .replace("Double", "Float");

                let error_message =
                    format!("\"Value out of range for {}: \" + value.toString()", $type_name);

                quote! {
                    class $cl_name {
                        static $type_signature lift($type_signature value) => value;

                        static LiftRetVal<$type_signature> read(Uint8List buf) {
                            return LiftRetVal(buf.buffer.asByteData(buf.offsetInBytes).get$conversion_name(0), $allocation_size);
                        }

                        static $type_signature lower($type_signature value) {
                            if (value < $min_value || value > $max_value) {
                                throw ArgumentError($error_message);
                            }
                            return value;
                        }

                        static int allocationSize([$type_signature value = 0]) {
                            return $allocation_size;
                        }

                        static int write($type_signature value, Uint8List buf) {
                            buf.buffer.asByteData(buf.offsetInBytes).set$conversion_name(0, lower(value));
                            return $allocation_size;
                        }
                    }
                }
            }
        }
    };
}
