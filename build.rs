fn main() {
    tonic_build::configure()
        .compile(&["proto/auth.proto"], &["proto"])
        .unwrap();
    tonic_build::configure()
        .type_attribute(
            "Approval",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "LimitLevel",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "LimitPeriod",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute("Money", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute("Source", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(
            "RoutingEntry",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute("Routing", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(
            "SetRouting",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .compile(&["proto/account.proto"], &["proto"])
        .unwrap();
    tonic_build::configure()
        .compile(&["proto/storage.proto"], &["proto"])
        .unwrap();
    tonic_build::configure()
        .compile(&["proto/email.proto"], &["proto"])
        .unwrap();
}
