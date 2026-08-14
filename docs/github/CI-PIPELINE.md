# Diseño del pipeline de calidad

## Lane rápida de PR

Objetivo operativo: feedback principal en menos de 10 minutos sin omitir controles críticos.

| Job | Comando previsto | Bloquea |
|---|---|---|
| governance | `python scripts/validate_quality_system.py` | siempre |
| fmt | `cargo fmt --all -- --check` | G3 |
| clippy | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | G3 |
| tests | `cargo nextest run --workspace --all-features` + doctests | G3 |
| architecture | `cargo xtask architecture` | G3 |
| requirements | `cargo xtask traceability` | G3 |
| coverage | `cargo llvm-cov nextest --workspace --all-features` | G3 |
| dependencies | `cargo deny check` + dependency review | G3 |
| security | CodeQL, secrets, `zizmor`/`actionlint` | G3 |

Los jobs fallan cerrado ante output ausente. El reporte de un job conserva versión/digest de herramienta y se sube con retención definida.

## Lane profunda

Nightly y por PR crítica:

- propiedades con mayor número de casos;
- mutation testing de módulos afectados;
- fuzzing con corpus persistente;
- replay del corpus dorado;
- matriz Linux/macOS/Windows;
- caos de procesos, disco, timeout y red;
- benchmark contra baseline y presupuesto;
- OpenSSF Scorecard y auditoría de workflows.

Una falla nightly abre issue automáticamente, asigna dueño y bloquea G4/G5. No necesariamente revierte una PR ya integrada salvo regresión crítica; `main` continúa visible como degradada hasta reparar.

## Lane de release

1. Checkout de tag protegido en runner efímero.
2. Repetición de G4 sobre el commit exacto.
3. Build por matriz y generación de checksums/SBOM.
4. Artifact attestation/provenance.
5. Job separado descarga y verifica digest, firma, SBOM e instalación.
6. Aprobación del environment `release`.
7. Publicación/promoción del mismo digest.
8. Registro G5 y smoke test posterior.

## Pin y permisos

Acciones de terceros se fijan a SHA completo y el comentario conserva versión legible. Dependabot propone actualizaciones; estas pasan shadow/review cuando cambian la semántica del control. Cada job declara permisos mínimos y publicación usa OIDC.

## Escalado

Al crecer el repositorio se usa change detection solo para añadir controles especializados, nunca para saltar governance, trazabilidad o secret scanning. Tests pesados pueden distribuirse por shards reproducibles; el GateRecord no se completa hasta reunir todos los shards.

