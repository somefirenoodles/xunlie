use xunlie_engine::compile;

const SOURCE: &str = r#"{"schemaVersion":"xunlie.source/v1","operations":[{"op":"add","requirement":{"id":"REQ-F-002","kind":"functional","priority":"must","statement":"Compile a canonical ContractIR."}}]}"#;

#[test]
fn contract_ir_v1_semantic_payload_matches_golden_vector() {
    let contract = compile(SOURCE).expect("golden source must compile");
    let actual = contract
        .semantic_canonical_json()
        .expect("typed semantic payload must serialize");
    let expected = include_str!("fixtures/contract_ir_v1.semantic.json").trim();

    assert_eq!(actual, expected);
    assert_eq!(
        contract.content_digest().as_str(),
        "sha256:a874b1034165ad316d49d21d4992d8532777b3f8b243436ed8882351ae471386"
    );
}
