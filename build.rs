fn main() {
    tonic_build::configure()
        .compile(&["proto/auth.proto"], &["proto"])
        .unwrap();
    tonic_build::configure()
        .type_attribute("Approval", "#[derive(serde::Deserialize)]")
        .compile(&["proto/account.proto"], &["proto"])
        .unwrap();
    tonic_build::configure()
        .compile(&["proto/storage.proto"], &["proto"])
        .unwrap();
    tonic_build::configure()
        .compile(&["proto/email.proto"], &["proto"])
        .unwrap();
}
