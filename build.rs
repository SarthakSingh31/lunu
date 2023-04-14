fn main() {
    tonic_build::configure()
        .compile(&["proto/auth.proto"], &["proto"])
        .unwrap();
    tonic_build::configure()
        .compile(&["proto/account.proto"], &["proto"])
        .unwrap();
    tonic_build::configure()
        .compile(&["proto/storage.proto"], &["proto"])
        .unwrap();
}
