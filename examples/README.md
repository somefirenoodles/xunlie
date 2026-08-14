# CLI example

From the repository root:

```console
cargo run -p xunlie-cli -- compile examples/minimal-source.json --out contract.json
cargo run -p xunlie-cli -- validate contract.json --format json
```

The first command writes canonical `xunlie.contract/v1` JSON. The second emits
one machine-readable result to stdout and returns exit code `0` when the
contract digest and invariants validate.
