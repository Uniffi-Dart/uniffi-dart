use std::fmt;
use std::sync::Arc;

#[uniffi::export]
pub trait Greeter: Send + Sync {
    fn greet(&self, name: String) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, uniffi::Object)]
#[uniffi::export(Debug, Display, Eq, Hash)]
pub struct FriendlyGreeter {
    phrase: String,
}

#[uniffi::export]
impl FriendlyGreeter {
    #[uniffi::constructor]
    pub fn new(phrase: String) -> Arc<Self> {
        Arc::new(Self { phrase })
    }

    pub fn to_trait(self: Arc<Self>) -> Arc<dyn Greeter> {
        self as Arc<dyn Greeter>
    }

    pub fn greet(&self, name: String) -> String {
        <Self as Greeter>::greet(self, name)
    }
}

impl Greeter for FriendlyGreeter {
    fn greet(&self, name: String) -> String {
        format!("{} {name}", self.phrase)
    }
}

impl fmt::Display for FriendlyGreeter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FriendlyGreeter({})", self.phrase)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, uniffi::Object)]
#[uniffi::export(Debug, Display, Eq, Hash)]
pub struct ProcFriendlyGreeter {
    phrase: String,
}

#[uniffi::export]
impl ProcFriendlyGreeter {
    #[uniffi::constructor]
    pub fn new(phrase: String) -> Arc<Self> {
        Arc::new(Self { phrase })
    }

    pub fn to_trait(self: Arc<Self>) -> Arc<dyn Greeter> {
        self as Arc<dyn Greeter>
    }

    pub fn greet(&self, name: String) -> String {
        <Self as Greeter>::greet(self, name)
    }
}

impl Greeter for ProcFriendlyGreeter {
    fn greet(&self, name: String) -> String {
        format!("{} {}", self.phrase.to_uppercase(), name.to_uppercase())
    }
}

impl fmt::Display for ProcFriendlyGreeter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProcFriendlyGreeter({})", self.phrase)
    }
}

#[derive(Default, uniffi::Object)]
pub struct Registry;

#[uniffi::export]
impl Registry {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn make_friendly(&self, phrase: String) -> Arc<dyn Greeter> {
        FriendlyGreeter::new(phrase).to_trait()
    }

    pub fn make_proc(&self, phrase: String) -> Arc<dyn Greeter> {
        ProcFriendlyGreeter::new(phrase).to_trait()
    }
}

// Keep lifecycle counts separate from the other fixture objects/finalizers.
static DISPOSAL_DROPS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

#[derive(uniffi::Object)]
pub struct DisposalProbe;

#[uniffi::export]
impl DisposalProbe {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }

    pub fn greet(&self, name: String) -> String {
        format!("Hello {name}")
    }

    pub fn as_greeter(self: Arc<Self>) -> Arc<dyn Greeter> {
        self
    }
}

impl Greeter for DisposalProbe {
    fn greet(&self, name: String) -> String {
        self.greet(name)
    }
}

impl Drop for DisposalProbe {
    fn drop(&mut self) {
        DISPOSAL_DROPS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

#[uniffi::export]
pub fn disposal_drop_count() -> u32 {
    DISPOSAL_DROPS.load(std::sync::atomic::Ordering::SeqCst)
}

uniffi::include_scaffolding!("api");
