use genco::prelude::*;

pub fn generate_web_runtime(module_name: &str) -> dart::Tokens {
    quote! {
        @JS($(quoted(format!("{module_name}.init"))))
        external JSPromise<JSAny?> _uniffiWasmInit(JSString wasmUrl);

        Future<void>? _initFuture;
        bool _initialized = false;
        String? _initializedWasmPath;

        Future<void> ensureInitialized({String? wasmPath}) {
            if (!_initialized && _initFuture == null && wasmPath == null) {
                throw ArgumentError("wasmPath is required for the first successful web initialization");
            }

            if (_initialized) {
                if (_initializedWasmPath != null && wasmPath != null && wasmPath != _initializedWasmPath) {
                    throw StateError("UniFFI web module already initialized with a different wasmPath");
                }
                return Future.value();
            }

            final existing = _initFuture;
            if (existing != null) {
                if (_initializedWasmPath != null && wasmPath != null && wasmPath != _initializedWasmPath) {
                    throw StateError("UniFFI web module initialization already in flight with a different wasmPath");
                }
                return existing;
            }

            _initializedWasmPath = wasmPath;
            final future = _doInitialize(wasmPath: wasmPath!).then((_) {
                _initialized = true;
            }).catchError((error, stack) {
                _initFuture = null;
                _initializedWasmPath = null;
                throw error;
            });

            _initFuture = future;
            return future;
        }

        Future<void> _doInitialize({required String wasmPath}) async {
            await _uniffiWasmInit(wasmPath.toJS).toDart;
            _checkApiVersion();
            _checkApiChecksums();
        }

        extension type _UniffiWebError(JSObject _) implements JSObject {
            external String? get kind;
            external String? get typeName;
            external JSUint8Array? get payload;
            external String? get message;
        }

        class UniffiInternalError implements Exception {
            static const int rustPanic = 8;

            final int errorCode;
            final String? panicMessage;

            const UniffiInternalError(this.errorCode, this.panicMessage);

            static UniffiInternalError panicked(String message) {
                return UniffiInternalError(rustPanic, message);
            }

            @override
            String toString() {
                return "UniFfi::rustPanic: " + (panicMessage ?? "");
            }
        }

        Never _throwWebError(Object error) {
            throw _decodeWebError(error);
        }

        Exception _decodeWebError(Object error) {
            try {
                final envelope = _UniffiWebError(error as JSObject);
                switch (envelope.kind) {
                    case "uniffi_error":
                        return UniffiInternalError.panicked("UniFFI error " + (envelope.typeName ?? "unknown"));
                    case "uniffi_internal":
                    case "uniffi_panic":
                        return UniffiInternalError.panicked(envelope.message ?? "Rust panic");
                }
            } catch (_) {
                // Fall through to the generic conversion below.
            }

            return UniffiInternalError.panicked(error.toString());
        }
    }
}
