#![no_main]
#![forbid(unsafe_code)]

use libfuzzer_sys::fuzz_target;
use xunlie_engine::CertifiedVariant;

fuzz_target!(|bytes: &[u8]| {
    if let Ok(variant) = serde_json::from_slice::<CertifiedVariant>(bytes) {
        let canonical = variant
            .canonical_json()
            .expect("a typed certified variant must be serializable");
        let decoded: CertifiedVariant = serde_json::from_str(&canonical)
            .expect("canonical certified variant output must be valid JSON");
        assert_eq!(variant, decoded, "typed round-trip must preserve the bundle");

        if variant.validate().is_ok() {
            decoded
                .validate()
                .expect("canonical round-trip must preserve integrity validation");
        }
    }
});
