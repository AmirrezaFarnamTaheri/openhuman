use super::*;
use std::collections::BTreeSet;

#[test]
fn compiled_feature_report_covers_every_manifest_feature() {
    let manifest: toml::Value = toml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/Cargo.toml"
    )))
    .expect("core Cargo manifest must parse");
    let declared: BTreeSet<String> = manifest
        .get("features")
        .and_then(toml::Value::as_table)
        .expect("core Cargo manifest must declare its features")
        .keys()
        .cloned()
        .collect();
    let advertised: BTreeSet<String> = compiled_features().into_keys().collect();

    assert_eq!(
        advertised, declared,
        "compiled_features must report every manifest feature, including disabled gates"
    );
}
