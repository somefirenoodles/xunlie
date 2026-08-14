# Xunlie

[![governance](https://github.com/somefirenoodles/xunlie/actions/workflows/governance.yml/badge.svg)](https://github.com/somefirenoodles/xunlie/actions/workflows/governance.yml)
[![CI](https://github.com/somefirenoodles/xunlie/actions/workflows/ci.yml/badge.svg)](https://github.com/somefirenoodles/xunlie/actions/workflows/ci.yml)
[![Deep quality](https://github.com/somefirenoodles/xunlie/actions/workflows/deep-quality.yml/badge.svg)](https://github.com/somefirenoodles/xunlie/actions/workflows/deep-quality.yml)
[![Release](https://img.shields.io/github/v/release/somefirenoodles/xunlie)](https://github.com/somefirenoodles/xunlie/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust 1.85+](https://img.shields.io/badge/MSRV-1.85-orange.svg)](rust-toolchain.toml)

Xunlie es un compilador de contratos para ingeniería de software agéntica. Convierte una
historia ordenada de requisitos en un `ContractIR` canónico, versionado y verificable. Si una
fuente es inválida o una historia contiene un conflicto no resuelto, falla de forma cerrada y
no produce un contrato parcial.

El incremento ejecutable incluye ingesta JSON `xunlie.source/v1`, resolución determinista de
operaciones `add`, `replace` y `revoke`, validación del contrato y variantes certificadas de
historias. `contentDigest` identifica el significado del contrato; `artifactDigest` protege
también su procedencia y metadatos, y `certificateDigest` enlaza la prueba de equivalencia M2.

> **Estado:** versión temprana anterior a `1.0`. El formato `xunlie.contract/v1` es ejecutable y está
> probado, pero todavía puede evolucionar antes de `1.0.0`. Consulte el
> [changelog](CHANGELOG.md) antes de actualizar.

## Probarlo

Requiere Rust mediante `rustup`; el toolchain exacto se instala automáticamente desde
`rust-toolchain.toml`.

```console
cargo run -p xunlie-cli -- compile examples/minimal-source.json --out contract.json
cargo run -p xunlie-cli -- validate contract.json --format json
cargo run -p xunlie-cli -- variant examples/independent-adds-source.json --operator reverse-independent-adds --out certified-variant.json --format json
cargo run -p xunlie-cli -- verify-variant examples/independent-adds-source.json certified-variant.json --format json
```

El primer comando escribe JSON canónico `xunlie.contract/v1`. El segundo valida tanto el esquema
como sus invariantes y comprueba ambos digests. Los resultados correctos van a `stdout`, los
errores a `stderr`, y los códigos de salida son estables.

`variant` solo escribe un artefacto cuando sus precondiciones pasan y la historia transformada
compila al mismo `contentDigest`. `verify-variant` vuelve a ejecutar el operador y compara fuentes,
digests, evidencia y certificado. Consulta el diseño de
[variantes certificadas](docs/architecture/CERTIFIED-VARIANTS.md).

Para instalar la CLI desde un checkout local:

```console
cargo install --path crates/xunlie-cli --locked
xunlie --help
```

También hay binarios para Linux x86_64 y Windows x86_64 en
[GitHub Releases](https://github.com/somefirenoodles/xunlie/releases/latest). Cada release incluye
`SHA256SUMS` y provenance verificable; consulta la
[guía de releases](docs/development/RELEASING.md) antes de confiar en un artefacto descargado.
Las funciones M2 descritas aquí pertenecen a `Unreleased`: hasta el siguiente tag deben ejecutarse
desde este checkout y no se presuponen presentes en los binarios de `v0.1.0`.

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
| `xunlie-domain` | ContractIR, certificados, digests, diagnósticos y resolución pura |
| `xunlie-engine` | Ingesta, compilación, operadores de variante y replay determinista |
| `xunlie-cli` | Comandos `compile`, `validate`, `variant` y `verify-variant` |
| `xunlie-testkit` | Builders y fixtures reutilizables |
| `xtask` | Puerta local agregada de arquitectura, formato, lint y pruebas |

El alcance siguiente es ejecutar agentes en workspaces aislados y comparar los resultados
pareados. Esas capacidades todavía no forman parte de este incremento.

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

## Participar

- Lea [CONTRIBUTING.md](CONTRIBUTING.md) antes de proponer un cambio.
- Use [Issues](https://github.com/somefirenoodles/xunlie/issues) para trabajo reproducible y
  [Discussions](https://github.com/somefirenoodles/xunlie/discussions) para preguntas o ideas.
- Reporte vulnerabilidades mediante el canal privado de [SECURITY.md](SECURITY.md).
- Toda interacción está sujeta al [Código de conducta](CODE_OF_CONDUCT.md).
- Los cambios relevantes y notas de compatibilidad viven en [CHANGELOG.md](CHANGELOG.md).

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

Xunlie es software público bajo licencia [MIT](LICENSE). Su gobierno y política de compatibilidad
se describen en [GOVERNANCE.md](GOVERNANCE.md) y [VERSIONING.md](VERSIONING.md).
