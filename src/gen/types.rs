use std::collections::BTreeSet;
use std::{cell::RefCell, collections::HashMap};

use genco::prelude::*;
use uniffi_bindgen::interface::AsType;
use uniffi_bindgen::{interface::Type, ComponentInterface};


use super::render::{AsRenderable, Renderable, Renderer, TypeHelperRenderer};
use super::{enums, functions, objects, oracle::AsCodeType, records};
use crate::gen::oracle::DartCodeOracle;

type FunctionDefinition = dart::Tokens;

pub struct TypeHelpersRenderer<'a> {
    ci: &'a ComponentInterface,
    include_once_names: RefCell<HashMap<String, Type>>,
}

impl<'a> TypeHelpersRenderer<'a> {
    pub fn new(ci: &'a ComponentInterface) -> Self {
        Self {
            ci,
            include_once_names: RefCell::new(HashMap::new()),
        }
    }

    pub fn get_include_names(&self) -> HashMap<String, Type> {
        self.include_once_names.clone().into_inner()
    }
}

impl TypeHelperRenderer for TypeHelpersRenderer<'_> {
    // Checks if the type imports for each type have already been added
    fn include_once_check(&self, name: &str, ty: &Type) -> bool {
        let mut map = self.include_once_names.borrow_mut();
        let found = map.insert(name.to_string(), ty.clone()).is_some();
        drop(map);
        found
    }

    fn check(&self, name: &str) -> bool {
        let map = self.include_once_names.borrow();
        let contains = map.contains_key(&name.to_string());
        drop(map);
        contains
    }

    fn get_object(&self, name: &str) -> Option<&uniffi_bindgen::interface::Object> {
        self.ci.get_object_definition(name)
    }

    fn get_enum(&self, name: &str) -> Option<&uniffi_bindgen::interface::Enum> {
        self.ci.get_enum_definition(name)
    }

    fn get_ci(&self) -> &ComponentInterface {
        self.ci
    }

    fn get_record(&self, name: &str) -> Option<&uniffi_bindgen::interface::Record> {
        self.ci.get_record_definition(name)
    }
}

impl Renderer<(FunctionDefinition, dart::Tokens)> for TypeHelpersRenderer<'_> {
    // TODO: Implimient a two pass system where the first pass will render the main code, and the second pass will render the helper code
    // this is so the generator knows what helper code to include.

    fn render(&self) -> (dart::Tokens, dart::Tokens) {
        // Render all the types and their helpers
        let types_definitions = quote! {
            $( for rec in self.ci.record_definitions() => $(records::generate_record(rec, self)))

            $( for enm in self.ci.enum_definitions() => $(enums::generate_enum(enm, self)))
            $( for obj in self.ci.object_definitions() => $(objects::generate_object(obj, self)))
        };

        // Render all unique imports, sorted alphabetically
        let modules_to_import = self.ci.iter_external_types()
            .map(|ty| self.ci.namespace_for_type(ty).expect("external type should have module_path"))
            .collect::<BTreeSet<_>>();
        // The second import statement uses a library prefix, to distinguish conflicting identifiers e.g. RustBuffer vs. ext.RustBuffer
        let imports: dart::Tokens = quote!(
            $( for imp in modules_to_import {
                $(format!("import \"{}.dart\"", imp));
                $(format!("import \"{}.dart\"", imp)) as $imp;
            })
        );

        // let function_definitions = quote!($( for fun in self.ci.function_definitions() => $(functions::generate_function("this", fun, self))));

        let function_definitions = quote!(
            $(for fun in self.ci.function_definitions() => 
                $(functions::generate_function(fun, self))
                
            )
        );

        let mut callback_code = quote!();
        // Process all callback interfaces to ensure they're included
        for callback in self.ci.callback_interface_definitions() {
            let callback_name = callback.name().to_string();
            let callback_codetype = super::callback_interface::CallbackInterfaceCodeType::new(callback_name, callback.as_type());
            // Force the callback interface to be processed, due to the way the code is generated we need to ensure it's processed, when a callback is used in a function signature
            // TODO: This is a hack to ensure the callback interface is processed, we need to test to ensure there's no chance of duplicates
            callback_code.append(callback_codetype.render_type_helper(self));
        }

        // Let's include the string converter
        self.include_once_check(&Type::String.as_codetype().canonical_name(), &Type::String);
        let helpers_definitions = quote! {
            $(for (_, ty) in self.get_include_names().iter() => $(ty.as_renderable().render_type_helper(self)) )
        };

        let types_helper_code = quote! {
            import "dart:async";
            import "dart:convert";
            import "dart:ffi";
            import "dart:io" show Platform, File, Directory;
            import "dart:isolate";
            import "dart:typed_data";
            import "package:ffi/ffi.dart";
            $(imports)

            $(types_definitions)


            class UniffiInternalError implements Exception {
                static const int bufferOverflow = 0;
                static const int incompleteData = 1;
                static const int unexpectedOptionalTag = 2;
                static const int unexpectedEnumCase = 3;
                static const int unexpectedNullPointer = 4;
                static const int unexpectedRustCallStatusCode = 5;
                static const int unexpectedRustCallError = 6;
                static const int unexpectedStaleHandle = 7;
                static const int rustPanic = 8;

                final int errorCode;
                final String? panicMessage;

                const UniffiInternalError(int this.errorCode, String this.panicMessage);

                static UniffiInternalError panicked(String message) {
                return UniffiInternalError(rustPanic, message);
                }

                @override
                String toString() {
                switch (errorCode) {
                    case bufferOverflow:
                    return "UniFfi::BufferOverflow";
                    case incompleteData:
                    return "UniFfi::IncompleteData";
                    case unexpectedOptionalTag:
                    return "UniFfi::UnexpectedOptionalTag";
                    case unexpectedEnumCase:
                    return "UniFfi::UnexpectedEnumCase";
                    case unexpectedNullPointer:
                    return "UniFfi::UnexpectedNullPointer";
                    case unexpectedRustCallStatusCode:
                    return "UniFfi::UnexpectedRustCallStatusCode";
                    case unexpectedRustCallError:
                    return "UniFfi::UnexpectedRustCallError";
                    case unexpectedStaleHandle:
                    return "UniFfi::UnexpectedStaleHandle";
                    case rustPanic:
                    return $[str](UniFfi::rustPanic: $panicMessage);
                    default:
                    return $[str](UniFfi::UnknownError: $errorCode);
                }
                }
            }

            const int CALL_SUCCESS = 0;
            const int CALL_ERROR = 1;
            const int CALL_UNEXPECTED_ERROR = 2;

            final class RustCallStatus extends Struct {
                @Int8()
                external int code;

                external RustBuffer errorBuf;

                //Pointer<RustCallStatus> asPointer() => Pointer<RustCallStatus>.fromAddress(address);
            }

            void checkCallStatus(UniffiRustCallStatusErrorHandler errorHandler, Pointer<RustCallStatus> status) {

                if (status.ref.code == CALL_SUCCESS) {
                return;
                } else if (status.ref.code == CALL_ERROR) {
                throw errorHandler.lift(status.ref.errorBuf);
                } else if (status.ref.code == CALL_UNEXPECTED_ERROR) {
                if (status.ref.errorBuf.len > 0) {
                    throw UniffiInternalError.panicked(FfiConverterString.lift(status.ref.errorBuf));
                } else {
                    throw UniffiInternalError.panicked("Rust panic");
                }
                } else {
                throw UniffiInternalError.panicked("Unexpected RustCallStatus code: ${status.ref.code}");
                }
            }

            T rustCall<T>(T Function(Pointer<RustCallStatus>) callback, [UniffiRustCallStatusErrorHandler? errorHandler]) {
                final status = calloc<RustCallStatus>();
                try {
                    final result = callback(status);
                    checkCallStatus(errorHandler ?? NullRustCallStatusErrorHandler(), status);
                    return result;
                } finally {
                calloc.free(status);
                }
            }

            class NullRustCallStatusErrorHandler extends UniffiRustCallStatusErrorHandler {
                @override
                Exception lift(RustBuffer errorBuf) {
                errorBuf.free();
                return UniffiInternalError.panicked("Unexpected CALL_ERROR");
                }
            }

            abstract class UniffiRustCallStatusErrorHandler {
                Exception lift(RustBuffer errorBuf);
            }

            final class RustBuffer extends Struct {
                @Uint64()
                external int capacity;

                @Uint64()
                external int len;

                external Pointer<Uint8> data;

                static RustBuffer alloc(int size) {
                    return rustCall((status) => $(DartCodeOracle::find_lib_instance()).$(self.ci.ffi_rustbuffer_alloc().name())(size, status));
                }

                static RustBuffer fromBytes(ForeignBytes bytes) {
                    return rustCall((status) => $(DartCodeOracle::find_lib_instance()).$(self.ci.ffi_rustbuffer_from_bytes().name())(bytes, status));
                }

                // static RustBuffer from(Pointer<Uint8> bytes, int len) {
                //   final foreignBytes = ForeignBytes(len: len, data: bytes);
                //   return rustCall((status) => _UniffiLib.instance.ffi_uniffi_futures_rustbuffer_from_bytes(foreignBytes));
                // }

                void free() {
                    rustCall((status) => $(DartCodeOracle::find_lib_instance()).$(self.ci.ffi_rustbuffer_free().name())(this, status));
                }

                RustBuffer reserve(int additionalCapacity) {
                return rustCall((status) => $(DartCodeOracle::find_lib_instance()).$(self.ci.ffi_rustbuffer_reserve().name())(this, additionalCapacity, status));
                }

                Uint8List asUint8List() {
                final dataList = data.asTypedList(len);
                final byteData = ByteData.sublistView(dataList);
                return Uint8List.view(byteData.buffer);
                }

                @override
                String toString() {
                return "RustBuffer{capacity: $capacity, len: $len, data: $data}";
                }
            }

            RustBuffer toRustBuffer(Uint8List data) {
                final length = data.length;

                final Pointer<Uint8> frameData = calloc<Uint8>(length); // Allocate a pointer large enough.
                final pointerList = frameData.asTypedList(length); // Create a list that uses our pointer and copy in the data.
                pointerList.setAll(0, data); // FIXME: can we remove this memcopy somehow?

                final bytes = calloc<ForeignBytes>();
                bytes.ref.len = length;
                bytes.ref.data = frameData;
                return RustBuffer.fromBytes(bytes.ref);
            }

            final class ForeignBytes extends Struct {
                @Int32()
                external int len;
                external Pointer<Uint8> data;

                //ForeignBytes({required this.len, required this.data});

                // factory ForeignBytes.fromTypedData(Uint8List typedData) {
                //   final data = calloc<Uint8>(typedData.length);
                //   final dataList = data.asTypedList(typedData.length);
                //   dataList.setAll(0, typedData);
                //   return ForeignBytes(len: typedData.length, data: data);
                // }

                void free() {
                calloc.free(data);
                }
            }

            class LiftRetVal<T> {
                final T value;
                final int bytesRead;
                const LiftRetVal(this.value, this.bytesRead);

                LiftRetVal<T> copyWithOffset(int offset) {
                    return LiftRetVal(value, bytesRead + offset);
                }
            }

            abstract class FfiConverter<D, F> {
                const FfiConverter();

                D lift(F value);
                F lower(D value);
                D read(ByteData buffer, int offset);
                void write(D value, ByteData buffer, int offset);
                int size(D value);
            }

            mixin FfiConverterPrimitive<T> on FfiConverter<T, T> {
                @override
                T lift(T value) => value;

                @override
                T lower(T value) => value;
            }

            Uint8List createUint8ListFromInt(int value) {
                int length = value.bitLength ~/ 8 + 1;

                // Ensure the length is either 4 or 8
                if (length != 4 && length != 8) {
                length = (value < 0x100000000) ? 4 : 8;
                }

                Uint8List uint8List = Uint8List(length);

                for (int i = length - 1; i >= 0; i--) {
                uint8List[i] = value & 0xFF;
                value >>= 8;
                }

                return uint8List;
            }

            $(helpers_definitions)

            const int UNIFFI_RUST_FUTURE_POLL_READY = 0;
            const int UNIFFI_RUST_FUTURE_POLL_MAYBE_READY = 1;

            typedef UniffiRustFutureContinuationCallback = Void Function(Uint64, Int8);

            Future<T> uniffiRustCallAsync<T, F>(
                Pointer<Void> Function() rustFutureFunc,
                void Function(Pointer<Void>, Pointer<NativeFunction<UniffiRustFutureContinuationCallback>>, Pointer<Void>) pollFunc,
                F Function(Pointer<Void>, Pointer<RustCallStatus>) completeFunc,
                void Function(Pointer<Void>) freeFunc,
                T Function(F) liftFunc, [
                UniffiRustCallStatusErrorHandler? errorHandler,
            ]) async {
                final rustFuture = rustFutureFunc();
                final completer = Completer<int>();

                late final NativeCallable<UniffiRustFutureContinuationCallback> callback;

                void poll() {
                    pollFunc(
                        rustFuture,
                        callback.nativeFunction,
                        Pointer<Void>.fromAddress(0),
                    );
                }
                void onResponse(int _idx, int pollResult) {
                    if (pollResult == UNIFFI_RUST_FUTURE_POLL_READY) {
                        completer.complete(pollResult);
                    } else {
                        poll();
                    }
                }
                callback = NativeCallable<UniffiRustFutureContinuationCallback>.listener(onResponse);

                try {
                    poll();
                    await completer.future;
                    callback.close();


                    final status = calloc<RustCallStatus>();
                    try {

                        final result = completeFunc(rustFuture, status);

                        return liftFunc(result);
                    } finally {
                        calloc.free(status);
                    }
                } finally {
                    freeFunc(rustFuture);
                }
            }

            class UniffiHandleMap<T> {
                final Map<int, T> _map = {};
                int _counter = 0;
            
                int insert(T obj) {
                final handle = _counter++;
                _map[handle] = obj;
                return handle;
                }
            
                T get(int handle) {
                final obj = _map[handle];
                if (obj == null) {
                    throw UniffiInternalError(
                        UniffiInternalError.unexpectedStaleHandle, "Handle not found");
                }
                return obj;
                }
            
                void remove(int handle) {
                if (_map.remove(handle) == null) {
                    throw UniffiInternalError(
                        UniffiInternalError.unexpectedStaleHandle, "Handle not found");
                }
                }
            }

        };

        (types_helper_code, function_definitions)
    }
}

pub fn generate_type(ty: &Type) -> dart::Tokens {
    match ty {
        Type::UInt8
        | Type::UInt32
        | Type::Int8
        | Type::Int16
        | Type::Int64
        | Type::UInt16
        | Type::Int32
        | Type::UInt64 => quote!(int),
        Type::Float32 | Type::Float64 => quote!(double),
        Type::String => quote!(String),
        Type::Bytes => quote!(Uint8List),
        Type::Object { name, .. } => quote!($name),
        Type::Boolean => quote!(bool),
        Type::Optional { inner_type } => quote!($(generate_type(inner_type))?),
        Type::Sequence { inner_type } => quote!(List<$(generate_type(inner_type))>),
        Type::Enum { name, .. } => quote!($(DartCodeOracle::class_name(name))),
        Type::Duration => quote!(Duration),
        Type::Record { name, .. } => quote!($name),
        Type::Custom { name, .. } => quote!($name),
        _ => todo!("Type::{:?}", ty),
    }
}
