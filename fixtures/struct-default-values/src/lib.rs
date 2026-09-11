pub struct Bookmark {
    pub guid: Option<String>,
    pub position: i32,
    pub last_modified: Option<i32>,
    pub url: String,
    pub title: Option<String>,
}

pub struct Contact {
    pub name: String,
    pub email: Option<String>,
    pub age: i32,
    pub nickname: Option<String>,
}

pub struct UdlStringDefaults {
    pub identifier: String,
    pub braced: String,
}

#[derive(uniffi::Record)]
pub struct StringDefaults {
    #[uniffi(default = "$amount")]
    pub identifier: String,
    #[uniffi(default = "${amount}")]
    pub braced: String,
    #[uniffi(default = "'\\$amount")]
    pub escaped: String,
}

#[uniffi::export]
pub fn echo_string_defaults(value: StringDefaults) -> StringDefaults {
    value
}

uniffi::include_scaffolding!("api");
