#![no_main]
#![forbid(unsafe_code)]

use libfuzzer_sys::fuzz_target;
use xunlie_engine::compile;

fuzz_target!(|bytes: &[u8]| {
    // The public compiler accepts UTF-8 text. Lossy conversion keeps every
    // libFuzzer byte sequence useful without adding a second parser surface.
    let source = String::from_utf8_lossy(bytes);

    if let Ok(contract) = compile(&source) {
        contract
            .validate()
            .expect("a successfully compiled contract must validate");

        let first = contract
            .canonical_json()
            .expect("a successfully compiled contract must be serializable");
        let decoded: serde_json::Value =
            serde_json::from_str(&first).expect("canonical contract output must be valid JSON");
        let round_trip = serde_json::to_string(&decoded)
            .expect("a canonical contract JSON value must be serializable");

        assert_eq!(first, round_trip, "contract serialization must be canonical");
    }
});
