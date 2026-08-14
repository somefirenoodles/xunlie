# Desarrollo local

Esta guía reproduce los controles obligatorios de CI para el workspace Rust de Xunlie. El
archivo `rust-toolchain.toml` es la única fuente de verdad para la versión de Rust y sus
componentes; no se debe seleccionar `stable` de forma implícita en scripts o workflows.

## Requisitos

- Git.
- `rustup` con el toolchain indicado por `rust-toolchain.toml`.
- Python 3 para validar los contratos de gobernanza.
- `cargo-deny 0.20.2` para la auditoría local de dependencias.

Instalación inicial:

```text
rustup show active-toolchain
rustup component add rustfmt clippy
cargo install cargo-deny --version 0.20.2 --locked
cargo fetch --locked
```

`rustup show active-toolchain` debe mostrar el canal fijado por el repositorio. Si el archivo
`rust-toolchain.toml` no existe o no puede resolverse, el incremento M1 está incompleto: no se
debe sustituir silenciosamente por otro toolchain.

## Ciclo rápido

Durante la implementación:

```text
cargo check --workspace --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
```

Para iterar sobre un crate se puede usar `cargo test -p <crate>`, pero antes de abrir un PR se
ejecuta la suite completa.

## Comprobación previa al PR

Los siguientes comandos corresponden a los jobs bloqueantes de `.github/workflows/ci.yml`:

```text
python scripts/validate_quality_system.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo test --doc --workspace --all-features --locked
cargo deny --workspace --all-features --locked check advisories bans licenses sources
```

No se admite que un comando modifique `Cargo.lock` durante esta comprobación. Si una dependencia
cambia de forma intencional, se actualiza el lockfile por separado y se revisa su diff.

## Matriz de CI

| Job | Plataforma | Propósito |
|---|---|---|
| `governance` | Ubuntu 24.04 | consistencia de requisitos, trazabilidad y gates |
| `rustfmt-and-clippy` | Ubuntu 24.04 | formato, compilación lint-clean y doctests |
| `tests-ubuntu-24.04` | Ubuntu 24.04 | comportamiento nativo Linux |
| `tests-windows-2025` | Windows 2025 | portabilidad y rutas/procesos Windows |
| `cargo-deny` | Ubuntu 24.04 | advisories, licencias, versiones y fuentes |
| `codeql-rust` | Ubuntu 24.04 | análisis estático de seguridad con consultas extendidas |

Las Actions externas están fijadas a commits completos. Dependabot puede proponer su actualización,
pero el comentario de versión y el SHA deben cambiar juntos. CodeQL usa `build-mode: none`, soportado
para Rust, para evitar una segunda compilación del workspace; su resultado no sustituye Clippy,
tests ni `cargo-deny`.

## Fallos frecuentes

- **`--locked` rechaza el build:** `Cargo.toml` y `Cargo.lock` no coinciden. Regenera y revisa el
  lockfile conscientemente; no retires `--locked`.
- **Diferencia de formato en Windows:** `rustfmt.toml` normaliza saltos de línea a LF. Ejecuta
  `cargo fmt --all` y conserva la configuración de Git del repositorio.
- **Licencia no permitida:** confirma la licencia efectiva y su impacto. Una excepción en
  `deny.toml` necesita una justificación concreta; no se permiten comodines.
- **Múltiples versiones:** comienzan como advertencia porque el ecosistema puede necesitarlas.
  Se corrigen cuando afectan superficie, tamaño o seguridad; no se silencian con `skip-tree` sin
  una causa documentada.
- **CodeQL no puede publicar:** el repositorio público debe tener code scanning habilitado y el job
  necesita `security-events: write`. Los PR de fuentes no confiables nunca reciben permisos extra.
