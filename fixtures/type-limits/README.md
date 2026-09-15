The u64 cases exercise generated bindings against Rust, including the upper half
of the unsigned range. They run as part of the normal workspace test suite:

```sh
cargo test -p type_limits
```

To keep the generated package and run the same cases under JIT and AOT:

```sh
UNIFFI_DART_TEST_DIR="$PWD/target/u64-test-output" cargo test -p type_limits -- --nocapture
```

The test prints its generated package directory. In that directory:

```sh
dart run test/u64_cases.dart
dart build cli --target=test/u64_cases.dart --output=build/u64-probe
./build/u64-probe/bundle/bin/u64_cases
```

`dart build cli` bundles the native library through the package's build hook.
Keep the bundle together when running it on another machine with the same OS and
architecture. `dart compile exe` alone does not bundle the native asset.

The cases include scalar boundaries, 1,000 deterministic values, byte offsets,
range/type rejection before a Rust function is entered, checked int conversion,
unchanged smaller integer types, defaults, nullable values, nested lists/maps,
record and enum construction, custom aliases, objects, synchronous and async
callbacks, and invalid callback results. The standalone runner uses explicit
checks, so AOT does not depend on assertions being enabled.
