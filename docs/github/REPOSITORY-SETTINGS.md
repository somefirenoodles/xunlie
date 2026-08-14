# Configuración requerida del repositorio GitHub

Este documento es una receta de administración. No constituye evidencia de que los controles remotos estén activos. G0 exige export/captura verificable de la configuración aplicada.

## Baseline

- Repositorio: nombre `xunlie`; owner y visibilidad pendientes.
- Rama por defecto: `main`.
- Merge: squash habilitado; merge commits y rebase merge deshabilitados.
- Auto-delete de branches integrado.
- Issues, security advisories y vulnerability alerts habilitados.
- Actions permitidas por allowlist; `GITHUB_TOKEN` con `contents: read` por defecto.
- Labels iniciales: `type:feature`, `type:defect`, `type:architecture`, `type:security`, `type:process`, `needs:triage`, `needs:adr`, `risk:critical` y `risk:high`.

## Ruleset `main-protection`

Crear primero en modo **Evaluate**, probar PR válida y PR adversarial y entonces activar.

- bloquear borrado y force-push;
- requerir pull request;
- 1 aprobación general; 2 para dominio, protocolo, seguridad, workflows, arquitectura y release;
- requerir CODEOWNER y aprobación del último push por persona distinta;
- descartar approvals obsoletos;
- resolver conversaciones antes de merge;
- historia lineal y commits firmados;
- branch actualizada o merge queue antes de integrar;
- sin bypass ordinario; equipo break-glass restringido y auditado;
- impedir creación/actualización si fallan status checks.

Checks de G0:

- `governance / validate-quality-system`

Checks que se hacen obligatorios al comenzar G3, después de existir y pasar en `main`:

- `ci / fmt`
- `ci / clippy`
- `ci / test-linux`
- `ci / test-windows`
- `quality / requirements`
- `quality / architecture`
- `quality / coverage`
- `security / codeql`
- `security / dependency-review`
- `security / cargo-deny`
- `security / workflow-security`

No se configura como required un nombre de check inexistente: se introduce workflow, se prueba, se registra evidencia y luego se activa el ruleset en la misma ventana de cambio controlada.

## Ruleset de tags `v*`

- restringir creación y actualización a Release Managers;
- impedir borrado y actualización no fast-forward;
- requerir tag firmado;
- nombre SemVer `vMAJOR.MINOR.PATCH` o prerelease permitido.

## Environments

`release` requiere dos revisores, impide auto-review, limita branches/tags protegidos y usa OIDC/credenciales efímeras. Publicación y verificación son jobs separados. El artefacto se promueve por digest.

## Seguridad

- Dependabot alerts/updates y dependency review;
- secret scanning y push protection;
- CodeQL default/advanced setup para Rust y GitHub Actions cuando exista código;
- artifact attestations para binarios y SBOM;
- private vulnerability reporting si el repositorio es público.

## Evidencia de activación

Exportar o capturar vía API: rulesets, bypass actors, required checks, Actions policy, environments, CODEOWNERS efectivo y resultado de dos pruebas: push directo rechazado y PR sin aprobación/check rechazado. Guardar digests en el GateRecord G0.

Referencias oficiales: [reglas disponibles en rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), [artifact attestations](https://docs.github.com/en/actions/concepts/security/artifact-attestations) y [dependency review](https://docs.github.com/en/code-security/how-tos/secure-your-supply-chain/manage-your-dependency-security/configure-dependency-review-action).
