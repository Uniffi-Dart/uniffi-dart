pub struct Bookmark {
    pub guid: Option<String>,
    pub position: i32,
    pub last_modified: Option<i32>,
    pub url: String,
    pub title: Option<String>,
}

uniffi::include_scaffolding!("api"); 