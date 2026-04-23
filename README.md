# uniffi-dart

Dart frontend for UniFFI bindings

![License: MIT](https://img.shields.io/github/license/acterglobal/uniffi-dart?style=flat-square) ![Status: experimental](https://img.shields.io/badge/status-experimental-red?style=flat-square)

## Installation

Add uniffi-dart as a dependency in your `Cargo.toml`:

```toml
[dependencies]
uniffi-dart = "0.1.1+v0.30.0"
```

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

## Versioning

uniffi-dart is versioned independently from uniffi-rs. We follow the
[SemVer rules from the Cargo Book](https://doc.rust-lang.org/cargo/reference/semver.html)
where versions are compatible when their left-most non-zero component
matches. A breaking change is any modification to the generated Dart
bindings that requires consumers to update their code.

Because the project is still young, the major version is 0 and most
updates bump the minor version.

To keep binding generators in sync, uniffi-dart targets a specific
uniffi-rs release. If you use multiple external binding generators, pick
versions that target the same uniffi-rs version.

Tags follow the format `vX.Y.Z+vA.B.C`, where `X.Y.Z` is the
uniffi-dart version and `A.B.C` is the targeted uniffi-rs version.

| uniffi-rs target | Latest uniffi-dart release |
|------------------|----------------------------|
| v0.30.0          | v0.1.1+v0.30.0             |

## License & Credits

The code is released under MIT License. See the LICENSE file in the repository root for details.

The project is building on top of the great work of Mozillas UniFFI, with inspirations from other external frontends (like Kotlin and Go) and with the help of the [ffi-gen](https://github.com/acterglobal/ffi-gen) lib. Thanks folks!
