# CLI example

From the repository root:

```console
cargo run -p xunlie-cli -- compile examples/minimal-source.json --out contract.json
cargo run -p xunlie-cli -- validate contract.json --format json
cargo run -p xunlie-cli -- variant examples/independent-adds-source.json --operator reverse-independent-adds --out certified-variant.json --format json
cargo run -p xunlie-cli -- verify-variant examples/independent-adds-source.json certified-variant.json --format json
```

The first command writes canonical `xunlie.contract/v1` JSON. The second emits
one machine-readable result to stdout and returns exit code `0` when the
contract digest and invariants validate.

The variant commands create and independently replay a
`xunlie.certified-variant/v1` artifact. Failed preconditions produce no partial output.
