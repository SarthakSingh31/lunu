fn main() {
    tonic_build::configure()
        .compile(&["proto/auth.proto"], &["proto"])
        .unwrap();
    tonic_build::configure()
        .type_attribute(
            "CustomerData",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "RetailerData",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "PartnerDesc",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "PartnerData",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
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
        .type_attribute("Fee", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(
            "PartnerRetailerFeeEntry",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "PaymentMethod",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "RetailerFeeEntry",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "RetailerFees",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "PutRetailerFeeEntry",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "PutRetailerFees",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "PartnerFeeEntry",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "PartnerFees",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "PutPartnerFeeEntry",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "PutPartnerFees",
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
