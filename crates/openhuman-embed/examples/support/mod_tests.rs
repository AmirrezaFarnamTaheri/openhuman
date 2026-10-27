use super::*;

#[test]
fn pointed_inference_supplies_a_default_model_without_a_hosted_provider() {
    PointedInference::install("http://127.0.0.1:9");
    assert_eq!(
        offline_config().default_model.as_deref(),
        Some("embed-test-model")
    );
}
