# TODO Overview

## Introduction

This document provides an overview of the ongoing refactor of this codebase and the remaining features to be implemented. The refactor aims to improve code structure, enhance maintainability, and ensure efficiency.

In this codebase, rendering Dart language bindings directly from Rust should involve using Rust traits `DartCodeOracle`, `Renderer`, `CodeType`, and `Renderable` to abstractly define the conversion logic for various Rust types and structures into their Dart equivalents. Each Rust component (functions, enums, structs) implements `CodeType` which implements `Renderable` traits to generate Dart syntax, ensuring type-safe, consistent representation across languages without relying on external templates. This method leverages Rust's paradigms, efficiently mapping complex Rust structures to Dart, while maintaining type integrity and language idiomatics through the use of `Renderer` and `DartCodeOracle`.

## ✅ Completed Features

- [x] Core Architecture (DartCodeOracle, Renderer, CodeType, Renderable traits)
- [x] Primitive Types (int8-64, uint8-64, float32/64, bool, string)
- [x] Duration Type
- [x] Complex Types (Records/Structs, Enums, Objects)
- [x] Function Generation and Calling
- [x] Error Handling Infrastructure (basic toString() using class names, trait interfaces skipped)  
- [x] FFI Infrastructure and Bindings
- [x] Basic Memory Management (RustBuffer, etc.)
- [x] Type Converters and Lifting/Lowering
- [x] **Comprehensive Test Suite** - 30 fixtures covering all major UniFFI functionality
  - Core types, async patterns, error handling, object-oriented patterns
  - Time handling, performance benchmarking, documentation generation
  - External type integration and cross-crate dependencies

## ⚠️ Partially Implemented (Needs Fixing)

- [ ] **Callbacks** (HIGH PRIORITY) - Code exists with basic funtionality but some tests are excluded
- [ ] **Futures and Async Dart** (HIGH PRIORITY) - Basic infrastructure exists, needs completion to support error types
- [ ] **Resource Disposal** (MEDIUM PRIORITY) - Some code exists but fixture excluded
- [ ] **Collections Types** (HIGH PRIORITY) - Maps and Sequences partially implemented

## ❌ Remaining Tasks Overview

*Based on comprehensive fixture analysis identifying 5 critical blocking features*

### 🔑 Critical Priority

- [ ] **HashMap/Map Support** - Core collection type missing
  - **Impact**: Blocks `simple-fns/` and multiple other fixtures
  - **Error**: `not yet implemented: Renderable for Type::Map`
  - **Status**: Critical blocker for core functionality

- [ ] **Proc-Macro Support** - Modern UniFFI development pattern  
  - **Impact**: Blocks `enum-types/`, `large-error/`, `proc-macro/` fixtures
  - **Error**: `#[derive(uniffi::Enum)]` not supported by UDL processing
  - **Status**: High impact for modern development patterns

### 📈 High Priority

- [ ] **Callbacks** - Re-enable and fix callback interface implementation
- [ ] **Futures and Async Dart** - Complete async/await support
- [ ] **Collections Types** - Complete Sequences implementation (Maps covered above)
- [ ] **Trait Interfaces** - Implement Display trait support for error objects

### 🔧 Medium Priority  

- [ ] **Dictionary Default Values** - Named parameters with defaults
  - **Impact**: Blocks `struct-default-values/` fixture
  - **Gap**: Generates positional constructors instead of named parameters
  - **Status**: Developer experience improvement

- [ ] **Trait Method Support** - Advanced trait functionality
  - **Impact**: Blocks `trait-methods/` fixture  
  - **Gap**: `[Traits=(Display, Debug, Eq, Hash)]` attribute not implemented
  - **Status**: Advanced feature completion

- [ ] **External crates** - Support for external Rust crates
- [ ] **Command Line Interface** - Create CLI tool for binding generation
- [ ] **Memory Optimizations** - Improve memory usage and cleanup

### 🛠️ Low Priority

- [ ] **BigInt Support** - Large integer boundary handling
  - **Impact**: Affects `type-limits/` for u64 boundary testing
  - **Gap**: Dart FFI layer needs `BigInt` conversion for large u64 values
  - **Status**: Edge case handling

- [ ] **Other Types**:
  - [ ] Bytes/Binary Data
  - [ ] Timestamp
  - [ ] Custom Types
- [ ] **Better Internal documentation** - Document the codebase architecture
- [ ] **Old Code Removal** - Clean up deprecated/unused code

## Architecture Notes

The codebase successfully implements the trait-based architecture described in the introduction. The `src/gen/` module contains well-organized code generation logic with separate modules for different type categories (primitives, enums, records, objects, etc.). The FFI layer is properly abstracted and the type system mapping is largely complete for basic types.

## Additional Notes

- Members are encouraged to communicate openly about challenges and suggestions.
- Documentation is crucial. Please document all changes and their impact.

## Current Status & Next Steps

### ✅ **Comprehensive Foundation Complete**

The project has a **solid foundation** with most core features implemented and a comprehensive 30-fixture test suite providing complete coverage of UniFFI functionality. Our systematic testing has identified exactly what needs to be implemented.

### 🎯 **Clear Development Priority**

Focus should be on the **5 critical blocking features** identified by our fixtures:

1. **HashMap/Map Support** (Critical) - Unblocks multiple fixtures immediately
2. **Proc-Macro Support** (Critical) - Enables modern UniFFI patterns
3. **Dictionary Default Values** (Medium) - Improves developer experience
4. **Trait Method Support** (Medium) - Completes advanced functionality  
5. **BigInt Support** (Low) - Handles edge cases

### 📈 **Implementation Recommendation**

Implementing **HashMap/Map support** alone would unlock significant functionality and demonstrate immediate progress. The comprehensive fixture suite provides excellent regression protection during development.

## Conclusion

This document serves as a guideline for the ongoing development process. The project has achieved **comprehensive test coverage** and identified **clear priorities** - the next phase focuses on implementing the 5 blocking features to unlock full UniFFI parity with other language bindings.
