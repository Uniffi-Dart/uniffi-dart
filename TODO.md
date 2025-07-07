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

## ⚠️ Partially Implemented (Needs Fixing)
- [ ] **Callbacks** (HIGH PRIORITY) - Code exists with basic funtionality but some tests are excluded
- [ ] **Futures and Async Dart** (HIGH PRIORITY) - Basic infrastructure exists, needs completion to support error types
- [ ] **Resource Disposal** (MEDIUM PRIORITY) - Some code exists but fixture excluded
- [ ] **Collections Types** (HIGH PRIORITY) - Maps and Sequences partially implemented

## ❌ Remaining Tasks Overview

### High Priority
- [ ] **Collections Types** - Complete Maps and Sequences implementation
- [ ] **Callbacks** - Re-enable and fix callback interface implementation
- [ ] **Futures and Async Dart** - Complete async/await support
- [ ] **Trait Interfaces** - Implement Display trait support for error objects with proper toString() methods using uniffi_trait_display FFI methods
- [ ] **Other Types**: 
  - [ ] Bytes/Binary Data
  - [ ] Timestamp 
  - [ ] Custom Types

### Medium Priority  
- [ ] **Command Line Interface** - Create CLI tool for binding generation
- [ ] **External crates** - Support for external Rust crates
- [ ] **Memory Optimizations** - Improve memory usage and cleanup
- [ ] **Better Internal documentation** - Document the codebase architecture

### Low Priority
- [ ] **Fixtures/Tests Enhancement** - Expand test coverage
- [ ] **Old Code Removal** - Clean up deprecated/unused code  
- [ ] **Refine Old Tests** - Update tests to cover all types


## Architecture Notes
The codebase successfully implements the trait-based architecture described in the introduction. The `src/gen/` module contains well-organized code generation logic with separate modules for different type categories (primitives, enums, records, objects, etc.). The FFI layer is properly abstracted and the type system mapping is largely complete for basic types.

## Additional Notes
- Members are encouraged to communicate openly about challenges and suggestions.
- Documentation is crucial. Please document all changes and their impact.

## Conclusion
This document serves as a guideline for the ongoing refactor process. It is a dynamic document and should be updated as the refactor progresses. The project has a solid foundation with most core features implemented - focus should be on fixing the excluded components and completing the remaining type support.
