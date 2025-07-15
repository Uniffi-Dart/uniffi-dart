use camino::Utf8Path;

fn main() {
    uniffi_dart::generate_scaffolding(Utf8Path::new("src/api.udl")).unwrap();
} 