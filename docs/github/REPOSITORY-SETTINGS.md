# Configuración del repositorio GitHub

Este documento distingue la configuración remota activa de las mejoras que requieren más de una
identidad operativa. La evidencia histórica del bootstrap se conserva en
`quality/evidence/github-bootstrap.json`.

## Baseline activo

- Repositorio público: [somefirenoodles/xunlie](https://github.com/somefirenoodles/xunlie), bajo
  licencia MIT y con `main` como rama predeterminada.
- Solo se permite squash merge; merge commits y rebase merge están deshabilitados.
- Las ramas integradas se eliminan automáticamente.
- Issues, Discussions y private vulnerability reporting están habilitados.
- Dependabot security updates, secret scanning y push protection están habilitados.
- Actions está habilitado con `GITHUB_TOKEN` de solo lectura por defecto y exige que todo `uses:`
  esté fijado a un commit SHA completo.
- Las Actions pueden provenir de cualquier publicador, pero el pin SHA obligatorio evita
  referencias mutables. Cada alta de una Action sigue requiriendo revisión en código.

## Ruleset activo `main-protection`

El ruleset de rama con ID `20826197` está en enforcement `active`, sin bypass:

- impide borrar `main` o hacer force-push;
- requiere pull request, historia lineal y squash merge;
- descarta aprobaciones obsoletas tras nuevos commits;
- exige resolver todas las conversaciones;
- exige que la rama esté actualizada con `main` antes de integrar;
- requiere los checks bloqueantes enumerados abajo.

La lista se amplía solo después de que un nombre nuevo haya aparecido y pasado en una PR. Los checks
obligatorios actuales son:

- `validate-quality-system`;
- `rustfmt-and-clippy`;
- `tests-ubuntu-24.04`;
- `tests-windows-2025`;
- `cargo-deny`;
- `codeql-rust`;
- `msrv-1.85.0`;
- `coverage`;
- `fuzz-source-parser`;
- `mutation-certified-variants`.

No se configura como obligatorio un nombre inexistente: primero se ejecuta el workflow, después se
verifica la evidencia y finalmente se actualiza el ruleset.

## Tags de release

Los tags `v*` se protegen contra borrado y actualización no fast-forward mediante un ruleset sin
bypass. La creación permanece permitida para que el mantenedor pueda publicar releases. El
workflow valida además SemVer, coincidencia con la versión del workspace y pertenencia del commit a
`main` antes de construir o publicar.

Los binarios se publican con checksums SHA-256 y build provenance. No se declara firma nativa de
binarios ni reproducibilidad bit a bit hasta disponer de un segundo constructor independiente.

## Límite de independencia

El repositorio tiene una sola identidad operativa. Por ello el ruleset exige cero aprobaciones: una
autoaprobación no aportaría independencia real. `DEC-011` mantiene bloqueada la afirmación de un
gate independiente hasta incorporar un segundo maintainer o auditor.

Cuando exista esa capacidad, la elevación prevista es:

- una aprobación general y dos para dominio, seguridad, workflows, arquitectura y release;
- CODEOWNERS y aprobación del último push por una persona distinta;
- equipo break-glass separado, restringido y auditado;
- prueba periódica del rechazo de autoaprobación y bypass.

## Evidencia operativa

La API de GitHub confirma el ruleset activo, ausencia de bypass, permisos de workflow de solo
lectura, SHA pinning, secret scanning, push protection, Dependabot security updates y squash-only.
Cada cambio de controles se valida mediante una PR real; la guía de release exige además verificar
assets, checksums y attestations una vez publicada la versión.

Referencias oficiales: [reglas disponibles en rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets),
[artifact attestations](https://docs.github.com/en/actions/concepts/security/artifact-attestations) y
[dependency review](https://docs.github.com/en/code-security/how-tos/secure-your-supply-chain/manage-your-dependency-security/configure-dependency-review-action).
