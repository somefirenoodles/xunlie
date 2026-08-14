# xunlie-cli

The `xunlie` binary is the stable automation boundary for the Xunlie core.

```text
xunlie compile <input> --out <contract.json> [--format human|json]
xunlie validate <contract.json> [--format human|json]
```

`compile` currently accepts `xunlie.source/v1` JSON. Source path separators are
normalized before they become provenance identities, so persisted contracts are
portable between Windows and Unix. See
`examples/minimal-source.json` for an executable source document.

Successful command results go to stdout. Errors go to stderr. In JSON mode,
the selected stream contains one `xunlie.cli.result/v1` object and no log text.
Every successful result exposes `contentDigest` for semantic equivalence and
`artifactDigest` for integrity of the complete persisted contract, including provenance.

## Stable exit codes

| Code | Meaning |
|---:|---|
| `0` | command succeeded, including `--help` and `--version` |
| `2` | command-line usage error |
| `10` | input could not be read |
| `11` | source compilation failed |
| `12` | ContractIR JSON or invariants are invalid |
| `13` | compiled output could not be written |
| `70` | CLI infrastructure failed, including writing its own result |

New failure classes receive new codes; existing meanings are not reassigned
within CLI major version 1.
