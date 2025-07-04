use std::sync::Arc;

// Import external types from the provider crate
use external_types_provider::{ExternalRecord, ExternalEnum, ExternalObject};

pub fn create_and_modify_external_record(name: String, value: i32) -> ExternalRecord {
    ExternalRecord {
        name: format!("Modified: {}", name),
        value: value + 100,
        is_active: true,
    }
}

pub fn cycle_external_enum(input: ExternalEnum) -> ExternalEnum {
    match input {
        ExternalEnum::First => ExternalEnum::Second,
        ExternalEnum::Second => ExternalEnum::Third,
        ExternalEnum::Third => ExternalEnum::First,
    }
}

pub fn wrap_external_object(obj: Arc<ExternalObject>, prefix: String) -> Arc<ExternalObject> {
    let current_name = obj.get_name();
    obj.set_name(format!("{}: {}", prefix, current_name));
    obj
}

pub fn process_external_record(record: ExternalRecord) -> String {
    format!("Processing record '{}' with value {} (active: {})", 
            record.name, record.value, record.is_active)
}

pub fn process_external_enum(enum_val: ExternalEnum) -> i32 {
    match enum_val {
        ExternalEnum::First => 10,
        ExternalEnum::Second => 20,
        ExternalEnum::Third => 30,
    }
}

pub fn process_external_object(obj: Arc<ExternalObject>, multiplier: i32) -> String {
    let name = obj.get_name();
    let calculated = obj.calculate(multiplier);
    format!("Object '{}' calculated {} with input {}", name, calculated, multiplier)
}

pub fn create_external_record_list(count: i32) -> Vec<ExternalRecord> {
    (0..count)
        .map(|i| ExternalRecord {
            name: format!("Record {}", i),
            value: i,
            is_active: i % 2 == 0,
        })
        .collect()
}

pub fn find_external_record_by_value(records: Vec<ExternalRecord>, target_value: i32) -> Option<ExternalRecord> {
    records.into_iter().find(|record| record.value == target_value)
}

uniffi::include_scaffolding!("lib"); 