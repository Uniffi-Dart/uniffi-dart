// Regression fixture: a data-carrying enum variant's Dart class is named
// `{Variant}{Enum}`. When that equals a real top-level type name, the generator
// used to emit two same-named Dart classes — a Dart compile error. The generator
// now disambiguates the variant class with a `Variant` suffix, looping past any
// further collision, against the names of every type that emits a top-level class
// (records, enums, objects, callback interfaces).

#[derive(uniffi::Record, Clone, PartialEq, Debug)]
pub struct FieldCondition {
    key: String,
}

// Forces a SECOND-ORDER collision: the `Field` variant's base name
// `FieldCondition` clashes with the record above, so it is suffixed to
// `FieldConditionVariant` — which clashes with THIS record, so the generator must
// keep going (`FieldConditionVariant2`). A one-shot suffix would emit a duplicate.
#[derive(uniffi::Record, Clone, PartialEq, Debug)]
pub struct FieldConditionVariant {
    note: String,
}

// A flat enum whose name equals the `Kind` variant's `{Kind}{Condition}` class,
// exercising the enum branch of the reserved-name set.
#[derive(uniffi::Enum, PartialEq, Debug)]
pub enum KindCondition {
    A,
    B,
}

#[derive(uniffi::Enum, PartialEq, Debug)]
pub enum Condition {
    Field { condition: FieldCondition },
    Kind { kind: KindCondition },
    Plain { value: i32 },
}

#[uniffi::export]
pub fn roundtrip_condition(c: Condition) -> Condition {
    c
}

uniffi::include_scaffolding!("api");
