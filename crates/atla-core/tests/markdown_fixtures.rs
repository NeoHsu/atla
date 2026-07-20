use atla_core::markdown::{adf_to_markdown, markdown_to_adf};
use serde_json::Value;

#[test]
fn markdown_to_adf_matches_golden_fixture() {
    let markdown = include_str!("fixtures/markdown/common.md");
    let expected: Value = serde_json::from_str(include_str!("fixtures/markdown/common.adf.json"))
        .expect("valid common ADF fixture");

    assert_eq!(markdown_to_adf(markdown), expected);
}

#[test]
fn adf_to_markdown_matches_golden_fixture() {
    let adf: Value = serde_json::from_str(include_str!("fixtures/markdown/rich.adf.json"))
        .expect("valid rich ADF fixture");
    let expected = include_str!("fixtures/markdown/rich.md").trim_end_matches('\n');

    assert_eq!(adf_to_markdown(&adf), expected);
}
