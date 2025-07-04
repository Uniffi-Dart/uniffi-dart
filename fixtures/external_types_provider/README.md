# External Types Test Fixtures

Clean test structure for external types functionality in uniffi-dart.

## Structure

```
fixtures/external_types_provider/
├── src/lib.udl        # Provider type definitions  
├── src/lib.rs         # Provider Rust implementation
├── tests/             # Provider tests
├── consumer/          # Consumer crate that uses external types
│   ├── src/lib.udl   # External type declarations
│   ├── src/lib.rs    # Consumer implementation  
│   └── tests/        # Consumer tests
└── README.md         # This file
```

## Current Status

- **✅ Provider**: Builds and generates working Dart bindings
- **❌ Consumer**: Fails with expected external types error (perfect!)

## Testing

```bash
# Test provider (works)
cargo test -p external_types_provider

# Test consumer (fails until implementation)
cd fixtures/external_types_provider/consumer && cargo test
```

## Expected Error

The consumer correctly fails with:
```
not yet implemented: Renderable for Type::External { 
    module_path: "external_types_provider", 
    name: "ExternalRecord", 
    kind: DataClass 
}
```

Perfect target for external types implementation! 🎯

## Architecture

- **Provider crate**: Acts as a library that exports types for external use
- **Consumer crate**: Nested within provider, references external types via `[External="external_types_provider"]` syntax
- Clean dependency: consumer depends on provider via `path = ".."` 