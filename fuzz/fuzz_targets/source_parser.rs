#![no_main]
#![forbid(unsafe_code)]

use libfuzzer_sys::fuzz_target;
use xunlie_domain::ContractIr;
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
        let decoded: ContractIr =
            serde_json::from_str(&first).expect("canonical contract output must be valid JSON");
        decoded
            .validate()
            .expect("a decoded canonical contract must validate");
        let round_trip = decoded
            .canonical_json()
            .expect("a decoded canonical contract must be serializable");

        assert_eq!(first, round_trip, "typed round-trip must remain canonical");
    }
});
