use camino::Utf8Path;

fn main() {
    uniffi_dart::generate_scaffolding(Utf8Path::new("src/lib.udl")).unwrap();
}