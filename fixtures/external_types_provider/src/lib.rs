use std::sync::{Arc, Mutex};

// Record type implementation  
pub struct ExternalRecord {
    pub name: String,
    pub value: i32,
    pub is_active: bool,
}

// Enum type implementation
pub enum ExternalEnum {
    First,
    Second,
    Third,
}

// Object type implementation
pub struct ExternalObject {
    name: Mutex<String>,
}

impl ExternalObject {
    pub fn new(name: String) -> Arc<Self> {
        Arc::new(Self {
            name: Mutex::new(name),
        })
    }

    pub fn get_name(&self) -> String {
        self.name.lock().unwrap().clone()
    }

    pub fn set_name(&self, name: String) {
        *self.name.lock().unwrap() = name;
    }

    pub fn calculate(&self, input: i32) -> i32 {
        input * 2 + 1
    }
}

// Module functions
pub fn create_external_record(name: String, value: i32) -> ExternalRecord {
    ExternalRecord {
        name,
        value,
        is_active: true,
    }
}

pub fn get_external_enum(variant: i32) -> ExternalEnum {
    match variant {
        0 => ExternalEnum::First,
        1 => ExternalEnum::Second,
        _ => ExternalEnum::Third,
    }
}

pub fn create_external_object(name: String) -> Arc<ExternalObject> {
    ExternalObject::new(name)
}

pub fn external_record_to_string(record: ExternalRecord) -> String {
    format!("ExternalRecord{{ name: {}, value: {}, is_active: {} }}", 
            record.name, record.value, record.is_active)
}

pub fn external_enum_to_int(enum_val: ExternalEnum) -> i32 {
    match enum_val {
        ExternalEnum::First => 0,
        ExternalEnum::Second => 1,
        ExternalEnum::Third => 2,
    }
}

pub fn external_object_get_name(obj: Arc<ExternalObject>) -> String {
    obj.get_name()
}

uniffi::include_scaffolding!("lib"); 