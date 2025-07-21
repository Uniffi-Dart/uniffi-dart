# uniffi-dart

Dart frontend for UniFFI bindings

![License: MIT](https://img.shields.io/github/license/acterglobal/uniffi-dart?style=flat-square) ![Status: experimental](https://img.shields.io/badge/status-experimental-red?style=flat-square)

## Work status

Reference: [TODOs](./TODO.md)

## MSRV: 1.75

This project must always work on latest stable rust + version before. We are also testing it against 1.1.70.0 , which we consider the Minimum Support Rust Version (MSRV) at this point. Rust lower than that will probably not compile the project.

## Testing & Fixtures

uniffi-dart includes a **comprehensive test suite** with 30 fixtures covering all major UniFFI functionality:

### **Fixture Coverage**

- **Core Types**: Primitives, collections, optionals, type limits
- **Async Patterns**: Comprehensive async/Future support with object-oriented patterns  
- **Error Handling**: Error types, large errors, exception scenarios
- **Object-Oriented**: Interfaces, constructors, methods, traits
- **Time Handling**: Timestamps, durations, ISO 8601 formatting
- **Performance**: FFI call overhead benchmarking
- **Documentation**: UDL and proc-macro documentation generation
- **External Types**: Cross-crate type sharing and custom type wrapping

See [`TODO.md`](./TODO.md) for detailed development priorities and blocking feature analysis.

### **Running Tests**

Run all fixture tests:

```bash
cargo nextest run --all --nocapture
```

Run specific fixture tests:

```bash
cargo nextest run -p simple_fns --nocapture
cargo nextest run -p dart_async --nocapture
cargo nextest run -p time_types --nocapture
```

For nightly compiler features (`genco` whitespace detection):

```bash
cargo +nightly nextest run -p hello_world --nocapture
```

### **Identified Blockers**

Our comprehensive fixture suite has identified 5 critical blocking features:

1. **HashMap/Map support** - Core collection type missing
2. **Proc-macro support** - Modern UniFFI development pattern
3. **Dictionary default values** - Named parameters with defaults  
4. **Trait method support** - Advanced trait functionality
5. **BigInt support** - Large integer boundary handling

## License & Credits

The code is released under MIT License. See the LICENSE file in the repository root for details.

The project is building on top of the great work of Mozillas UniFFI, with inspirations from other external frontends (like Kotlin and Go) and with the help of the [ffi-gen](https://github.com/acterglobal/ffi-gen) lib. Thanks folks!
