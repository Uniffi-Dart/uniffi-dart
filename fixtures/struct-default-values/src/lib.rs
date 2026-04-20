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

uniffi::include_scaffolding!("api");
