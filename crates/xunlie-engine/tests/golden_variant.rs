#![forbid(unsafe_code)]

use xunlie_engine::{SourceDocument, VariantGeneration, generate_builtin_variant};

const SOURCE: &str = include_str!("../../xunlie-testkit/fixtures/independent-adds-source.json");
const GOLDEN: &str = include_str!("fixtures/certified_variant_v1.json");

#[test]
fn certified_variant_v1_matches_golden_vector() {
    let result = generate_builtin_variant(
        vec![SourceDocument::new(
            "crates/xunlie-testkit/fixtures/independent-adds-source.json",
            0,
            SOURCE,
        )],
        "history.independent-adds.reverse",
    )
    .unwrap();
    let VariantGeneration::Certified(variant) = result else {
        panic!("golden input must produce a certified variant")
    };

    assert_eq!(variant.canonical_json().unwrap(), GOLDEN.trim_end());
}
