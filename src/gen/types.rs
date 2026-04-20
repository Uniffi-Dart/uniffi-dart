use std::collections::{BTreeSet, HashSet};
use std::{cell::RefCell, collections::HashMap};

use genco::prelude::*;
use uniffi_bindgen::interface::AsType;
use uniffi_bindgen::{interface::Type, ComponentInterface};

use super::render::{AsRenderable, Renderer, TypeHelperRenderer};
use super::{enums, functions, objects, oracle::AsCodeType, records};
use crate::gen::oracle::DartCodeOracle;

type FunctionDefinition = dart::Tokens;

pub struct TypeHelpersRenderer<'a> {
    ci: &'a ComponentInterface,
    include_once_names: RefCell<HashMap<String, Type>>,
    // Tracks ad-hoc "include once" names that don't map to a concrete `Type`
    include_once_custom: RefCell<HashSet<String>>,
    import_prefix: String,
}

impl<'a> TypeHelpersRenderer<'a> {
    pub fn with_import_prefix(ci: &'a ComponentInterface, import_prefix: String) -> Self {
        Self {
            ci,
            include_once_names: RefCell::new(HashMap::new()),
            include_once_custom: RefCell::new(HashSet::new()),
            import_prefix,
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

    fn include_once_by_name(&self, name: &str) -> bool {
        let mut set = self.include_once_custom.borrow_mut();
        !set.insert(name.to_string())
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
        let modules_to_import = self
            .ci
            .iter_external_types()
            .map(|ty| {
                self.ci
                    .namespace_for_type(ty)
                    .expect("external type should have module_path")
            })
            .collect::<BTreeSet<_>>();
        let import_prefix = &self.import_prefix;
        let imports: dart::Tokens = quote!(
            $( for imp in modules_to_import {
                $(format!("import \"{}{}.dart\"", import_prefix, imp));
                $(format!("import \"{}{}.dart\"", import_prefix, imp)) as $imp;
            })
        );

        // let function_definitions = quote!($( for fun in self.ci.function_definitions() => $(functions::generate_function("this", fun, self))));

        let function_definitions = quote!(
            $(for fun in self.ci.function_definitions() =>
                $(functions::generate_function(fun, self))

            )
        );

        // Ensure callback interfaces and their method parameter/return types
        // are registered for helper generation. We must do this before taking
        // the snapshot in get_include_names(), so that compound types like
        // Sequence<Record> or Optional<Record> that only appear in callback
        // interface signatures get their FfiConverter classes generated.
        for callback in self.ci.callback_interface_definitions() {
            self.include_once_check(
                &callback.as_type().as_codetype().canonical_name(),
                &callback.as_type(),
            );
            // Walk callback method signatures to register compound types
            for method in callback.methods() {
                for arg in method.arguments() {
                    arg.as_renderable().render_type(&arg.as_type(), self);
                }
                if let Some(ret) = method.return_type() {
                    ret.as_renderable().render_type(ret, self);
                }
            }
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

                const UniffiInternalError(this.errorCode, this.panicMessage);

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

            // New version that separates FFI call from lifting to avoid deserializing garbage on error
            T rustCallWithLifter<T, F>(F Function(Pointer<RustCallStatus>) ffiCall, T Function(F) lifter, [UniffiRustCallStatusErrorHandler? errorHandler]) {
                final status = calloc<RustCallStatus>();
                try {
                    final rawResult = ffiCall(status);
                    checkCallStatus(errorHandler ?? NullRustCallStatusErrorHandler(), status);
                    return lifter(rawResult);
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
                    return rustCall((status) => $(self.ci.ffi_rustbuffer_alloc().name())(size, status));
                }

                static RustBuffer fromBytes(ForeignBytes bytes) {
                    return rustCall((status) => $(self.ci.ffi_rustbuffer_from_bytes().name())(bytes, status));
                }

                // static RustBuffer from(Pointer<Uint8> bytes, int len) {
                //   final foreignBytes = ForeignBytes(len: len, data: bytes);
                //   return rustCall((status) => _UniffiLib.instance.ffi_uniffi_futures_rustbuffer_from_bytes(foreignBytes));
                // }

                void free() {
                    rustCall((status) => $(self.ci.ffi_rustbuffer_free().name())(this, status));
                }

                RustBuffer reserve(int additionalCapacity) {
                return rustCall((status) => $(self.ci.ffi_rustbuffer_reserve().name())(this, additionalCapacity, status));
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

            final _uniffiRustFutureContinuationHandles = UniffiHandleMap<Completer<int>>();

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
                final handle = _uniffiRustFutureContinuationHandles.insert(completer);
                final callbackData = Pointer<Void>.fromAddress(handle);

                late final NativeCallable<UniffiRustFutureContinuationCallback> callback;

                void repoll() {
                    pollFunc(
                        rustFuture,
                        callback.nativeFunction,
                        callbackData,
                    );
                }

                void onResponse(int data, int pollResult) {
                    if (pollResult == UNIFFI_RUST_FUTURE_POLL_READY) {
                        final readyCompleter =
                            _uniffiRustFutureContinuationHandles.maybeRemove(data);
                        if (readyCompleter != null && !readyCompleter.isCompleted) {
                            readyCompleter.complete(pollResult);
                        }
                    } else if (pollResult == UNIFFI_RUST_FUTURE_POLL_MAYBE_READY) {
                        repoll();
                    } else {
                        final errorCompleter =
                            _uniffiRustFutureContinuationHandles.maybeRemove(data);
                        if (errorCompleter != null && !errorCompleter.isCompleted) {
                            errorCompleter.completeError(
                                UniffiInternalError.panicked(
                                    "Unexpected poll result from Rust future: $pollResult",
                                ),
                            );
                        }
                    }
                }

                callback = NativeCallable<UniffiRustFutureContinuationCallback>.listener(
                  onResponse,
                );

                try {
                    repoll();
                    await completer.future;

                    final status = calloc<RustCallStatus>();
                    try {
                        final result = completeFunc(rustFuture, status);
                        checkCallStatus(
                            errorHandler ?? NullRustCallStatusErrorHandler(),
                            status,
                        );
                        return liftFunc(result);
                    } finally {
                        calloc.free(status);
                    }
                } finally {
                    callback.close();
                    _uniffiRustFutureContinuationHandles.maybeRemove(handle);
                    freeFunc(rustFuture);
                }
            }

            typedef UniffiForeignFutureFree = Void Function(Uint64);
            typedef UniffiForeignFutureFreeDart = void Function(int);

            class _UniffiForeignFutureState {
                bool cancelled = false;
            }

            final _uniffiForeignFutureHandleMap = UniffiHandleMap<_UniffiForeignFutureState>();

            void _uniffiForeignFutureFree(int handle) {
                final state = _uniffiForeignFutureHandleMap.maybeRemove(handle);
                if (state != null) {
                    state.cancelled = true;
                }
            }

            final Pointer<NativeFunction<UniffiForeignFutureFree>>
                _uniffiForeignFutureFreePointer =
                    Pointer.fromFunction<UniffiForeignFutureFree>(_uniffiForeignFutureFree);

            final class UniffiForeignFuture extends Struct {
                @Uint64()
                external int handle;

                external Pointer<NativeFunction<UniffiForeignFutureFree>> free;
            }

            // As of uniffi 0.30, foreign handles must always have the lowest bit set
            // This is achieved here with an odd number sequence.
            class UniffiHandleMap<T> {
                final Map<int, T> _map = {};
                int _counter = 1;

                int insert(T obj) {
                final handle = _counter;
                _counter += 2;
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
                if (maybeRemove(handle) == null) {
                    throw UniffiInternalError(
                        UniffiInternalError.unexpectedStaleHandle, "Handle not found");
                }
                }

                T? maybeRemove(int handle) {
                return _map.remove(handle);
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
        Type::Map {
            key_type,
            value_type,
        } => quote!(Map<$(generate_type(key_type)), $(generate_type(value_type))>),
        Type::Enum { name, .. } => quote!($(DartCodeOracle::class_name(name))),
        Type::Timestamp => quote!(DateTime),
        Type::Duration => quote!(Duration),
        Type::Record { name, .. } => quote!($name),
        Type::Custom { name, .. } => quote!($name),
        Type::CallbackInterface { name, .. } => quote!($name),
    }
}
