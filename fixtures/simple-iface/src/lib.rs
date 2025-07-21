use std::sync::Arc;

#[derive(Debug)]
pub struct Object {
    inner: i32,
}

impl Object {
    pub fn new(inner: i32) -> Self {
        Self { inner }
    }

    pub fn get_inner(&self) -> i32 {
        self.inner
    }

    pub fn some_method(self: Arc<Self>) -> Option<Arc<Self>> {
        None
    }
}

pub fn make_object(inner: i32) -> Arc<Object> {
    Arc::new(Object::new(inner))
}

uniffi::include_scaffolding!("api"); 