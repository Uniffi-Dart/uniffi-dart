use std::sync::Arc;

pub enum Animal {
    Dog,
    Cat,
}

// Though it has the proc-macro, we drop the variant
// literals if there is not a repr type defined
#[derive(uniffi::Enum)]
pub enum AnimalNoReprInt {
    Dog = 3,
    Cat = 4,
}

#[repr(u8)]
#[derive(uniffi::Enum)]
pub enum AnimalUInt {
    Dog = 3,
    Cat = 4,
}

#[repr(u64)]
#[derive(uniffi::Enum)]
pub enum AnimalLargeUInt {
    Dog = 4294967298, // u32::MAX as u64 + 3
    Cat = 4294967299, // u32::MAX as u64 + 4
}

#[repr(i8)]
#[derive(Debug, uniffi::Enum)]
pub enum AnimalSignedInt {
    Dog = -3,
    Cat = -2,
    Koala,   // -1
    Wallaby, // 0
    Wombat,  // 1
}

#[derive(Default, PartialEq, Eq, Clone, uniffi::Record)]
pub struct AnimalRecord {
    name: String,
}

impl AnimalRecord {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[derive(Default, PartialEq, Eq, uniffi::Object)]
#[uniffi::export(Eq)]
pub struct AnimalObject {
    pub value: AnimalRecord,
}

impl AnimalObject {
    pub fn new(name: &str) -> Self {
        Self {
            value: AnimalRecord::new(name),
        }
    }
}

#[uniffi::export]
impl AnimalObject {
    fn get_record(&self) -> AnimalRecord {
        self.value.clone()
    }
}

// An enum with non-primitive types
#[derive(uniffi::Enum)]
pub enum AnimalEnum {
    None,
    Dog(Arc<AnimalObject>),
    Cat(AnimalRecord),
}

#[uniffi::export]
pub fn get_animal_enum(animal: Animal) -> AnimalEnum {
    match animal {
        Animal::Dog => AnimalEnum::Dog(Arc::new(AnimalObject::new("dog"))),
        Animal::Cat => AnimalEnum::Cat(AnimalRecord::new("cat")),
    }
}

#[derive(uniffi::Enum)]
pub enum AnimalNamedEnum {
    None,
    Dog { object: Arc<AnimalObject> },
    Cat { record: AnimalRecord },
}

#[uniffi::export]
pub fn get_animal(a: Option<Animal>) -> Animal {
    a.unwrap_or(Animal::Dog)
}

uniffi::include_scaffolding!("api"); 