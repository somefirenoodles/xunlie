# Xunlie

[![governance](https://github.com/somefirenoodles/xunlie/actions/workflows/governance.yml/badge.svg)](https://github.com/somefirenoodles/xunlie/actions/workflows/governance.yml)
[![CI](https://github.com/somefirenoodles/xunlie/actions/workflows/ci.yml/badge.svg)](https://github.com/somefirenoodles/xunlie/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Xunlie es un compilador de contratos para ingeniería de software agéntica. Convierte una
historia ordenada de requisitos en un `ContractIR` canónico, versionado y verificable. Si una
fuente es inválida o una historia contiene un conflicto no resuelto, falla de forma cerrada y
no produce un contrato parcial.

Este es el primer incremento ejecutable. Incluye la ingesta JSON `xunlie.source/v1`, resolución
determinista de operaciones `add`, `replace` y `revoke`, validación del contrato, dos digests
SHA-256 y una CLI apta para uso humano o automatización. `contentDigest` identifica el significado
del contrato; `artifactDigest` protege también su procedencia y metadatos.

## Probarlo

Requiere Rust mediante `rustup`; el toolchain exacto se instala automáticamente desde
`rust-toolchain.toml`.

```console
cargo run -p xunlie-cli -- compile examples/minimal-source.json --out contract.json
cargo run -p xunlie-cli -- validate contract.json --format json
```

El primer comando escribe JSON canónico `xunlie.contract/v1`. El segundo valida tanto el esquema
como sus invariantes y comprueba ambos digests. Los resultados correctos van a `stdout`, los
errores a `stderr`, y los códigos de salida son estables.

Ejemplo mínimo de entrada:

```json
{
  "schemaVersion": "xunlie.source/v1",
  "operations": [
    {
      "op": "add",
      "requirement": {
        "id": "REQ-F-001",
        "kind": "functional",
        "priority": "must",
        "statement": "Preserve exact source provenance."
      }
    }
  ]
}
```

## Qué existe hoy

| Componente | Responsabilidad |
|---|---|
| `xunlie-domain` | ContractIR, digests, diagnósticos y resolución pura de historias |
| `xunlie-engine` | Ingesta de fuentes y compilación sin IR parcial |
| `xunlie-cli` | Comandos `compile` y `validate`, salida humana/JSON |
| `xunlie-testkit` | Builders y fixtures reutilizables |
| `xtask` | Puerta local agregada de arquitectura, formato, lint y pruebas |

El alcance siguiente es certificar transformaciones equivalentes de historias, ejecutar agentes
en workspaces aislados y comparar los resultados pareados. Esas capacidades todavía no forman
parte de este incremento.

## Calidad local

```console
cargo xtask quality
```

El comando ejecuta la validación de gobernanza, las restricciones arquitectónicas, `rustfmt`,
Clippy con warnings como errores y toda la suite. La CI repite los controles en Ubuntu 24.04 y
Windows 2025, además de `cargo-deny` y CodeQL.

Para instalar herramientas o resolver problemas del entorno, consulta la
[guía de desarrollo local](docs/development/LOCAL-DEVELOPMENT.md). La interfaz y los códigos de
salida están documentados en [`xunlie-cli`](crates/xunlie-cli/README.md).

## Mapa del repositorio

| Ruta | Propósito |
|---|---|
| `crates/` | Código Rust del producto |
| `examples/` | Fuentes ejecutables de ejemplo |
| `docs/architecture/` | Baseline, límites, dependencias y ADR |
| `docs/quality/` | Plan de calidad, gates, métricas y auditoría |
| `docs/requirements/` | Requisitos funcionales y de calidad |
| `quality/` | Contratos ejecutables del sistema de calidad |
| `scripts/` | Validadores auxiliares sin dependencias externas |
| `.github/` | Gobernanza y automatización del repositorio |

Xunlie es software público bajo licencia [MIT](LICENSE).
