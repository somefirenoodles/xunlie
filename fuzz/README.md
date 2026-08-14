# Fuzzing

`source_parser` exercises the public `xunlie_engine::compile` boundary with arbitrary bytes. It
covers JSON decoding, source-schema validation, history resolution and ContractIR construction.
Successful compilations must also satisfy three invariants:

1. the emitted contract validates;
2. canonical output is valid JSON;
3. decoding and compactly encoding that output is byte-for-byte stable.

The fuzz package is an independent workspace because `cargo-fuzz`/libFuzzer require nightly,
while product builds remain on the stable toolchain pinned in `rust-toolchain.toml`. Dependencies
are exact-versioned and `fuzz/Cargo.lock` is committed for reproducibility.

## Local run

Install the same dated nightly and runner used by CI:

```text
rustup toolchain install nightly-2026-08-01 --profile minimal
cargo +nightly-2026-08-01 install cargo-fuzz --version 0.13.2 --locked
cargo +nightly-2026-08-01 fuzz run source_parser -- -max_total_time=60
```

For a bounded smoke run equivalent to CI, replace `60` with `30`. An unbounded local campaign can
omit `-max_total_time` and should be stopped with Ctrl+C.

libFuzzer writes minimized crash inputs under `fuzz/artifacts/source_parser/`. When a crash reveals
a defect, first turn the minimized input into a deterministic regression test in the owning crate;
then add it to `fuzz/corpus/source_parser/` only when it contributes a distinct parser shape. Crash
artifacts and generated targets are intentionally ignored.

The seed corpus is deliberately small: one valid add, one complete add/replace/revoke history, and
one unsupported schema. libFuzzer derives malformed JSON and boundary variants from these seeds.
