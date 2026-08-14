# Pipeline de integración y release

Xunlie divide sus controles en workflows pequeños, con permisos mínimos y nombres de job
estables para que el ruleset de `main` pueda exigirlos.

## `governance.yml`

Ejecuta `scripts/validate_quality_system.py` en cada PR y push a `main`. Comprueba requisitos,
trazabilidad, riesgos, herramientas, gates y la existencia de la documentación normativa y
comunitaria.

Job obligatorio: `validate-quality-system`.

## `ci.yml`

Es la lane funcional y de seguridad cotidiana:

| Job | Evidencia |
|---|---|
| `governance` | segunda comprobación del contrato de calidad dentro de la CI principal |
| `rustfmt-and-clippy` | formato, Clippy con warnings como errores y doctests |
| `tests-ubuntu-24.04` | suite completa mediante cargo-nextest en Linux |
| `tests-windows-2025` | la misma suite mediante cargo-nextest en Windows |
| `cargo-deny` | advisories, licencias, fuentes y dependencias prohibidas |
| `codeql-rust` | consultas CodeQL `security-extended` para Rust |

El toolchain de producto proviene de `rust-toolchain.toml`; nextest y las Actions externas están
fijados a versiones y commits completos.

## `deep-quality.yml`

Ejecuta en cada PR, push a `main` y bajo demanda:

| Job | Evidencia |
|---|---|
| `msrv-1.85.0` | el workspace compila con la versión mínima declarada |
| `coverage` | cobertura de líneas mínima del 75 % y reporte LCOV descargable |
| `fuzz-source-parser` | campaña libFuzzer acotada contra el límite público del compilador |
| `mutation-certified-variants` | `cargo-mutants 27.1.0` exige que la suite detecte mutaciones en certificados y generación |

El fuzzing usa `nightly-2026-08-01`, `cargo-fuzz 0.13.2`, lockfile independiente y corpus
versionado. El umbral de cobertura es un piso de regresión, no una sustitución de pruebas
dirigidas.

Mutation testing usa `.cargo/mutants.toml` para limitar la campaña a la frontera crítica de M2,
ejecuta sin orden aleatorio y publica `mutants.out` incluso ante fallo. Un mutante no detectado hace
fallar el job; ampliar o reducir el alcance requiere revisión explícita de riesgo.

## `release.yml`

Solo se activa al publicar un tag SemVer `v*`. Antes de crear un release:

1. verifica que el tag coincida con `Cargo.toml` y pertenezca a `main`;
2. construye y prueba los binarios Linux x86_64 y Windows x86_64;
3. empaqueta binario, licencia y README;
4. genera y verifica `SHA256SUMS`;
5. emite attestations de build provenance;
6. publica los mismos bytes en GitHub Releases.

El job de publicación es el único con `contents: write`, `id-token: write` y
`attestations: write`. El procedimiento de operación y recuperación está en
[`RELEASING.md`](../development/RELEASING.md).

## Pinning y permisos

- Todo `uses:` referencia un commit SHA completo con su versión legible en comentario.
- El repositorio exige SHA pinning para Actions.
- `GITHUB_TOKEN` recibe `contents: read` por defecto; cada job eleva solo lo necesario.
- Checkout nunca persiste credenciales.
- Las dependencias Rust y del workspace de fuzzing usan lockfiles versionados.

## Controles posteriores

Replay de variantes y mutation testing dirigido están activos desde M2. Campañas globales
prolongadas, caos, benchmarks y comparación de builds independientes pertenecen a milestones
posteriores y no se presentan como controles activos antes de contar con workflows reproducibles.
