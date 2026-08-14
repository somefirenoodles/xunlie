# Configuración del repositorio GitHub

Este documento separa la configuración remota activa de la configuración objetivo para aprobación independiente. La evidencia exportada está en `quality/evidence/github-bootstrap.json`.

## Baseline

- Repositorio: [somefirenoodles/xunlie](https://github.com/somefirenoodles/xunlie), público y MIT.
- Rama por defecto: `main`.
- Merge: squash habilitado; merge commits y rebase merge deshabilitados.
- Auto-delete de branches integrado.
- Issues, security advisories y vulnerability alerts habilitados.
- Actions permitidas por allowlist; `GITHUB_TOKEN` con `contents: read` por defecto.
- Labels iniciales: `type:feature`, `type:defect`, `type:architecture`, `type:security`, `type:process`, `needs:triage`, `needs:adr`, `risk:critical` y `risk:high`.

## Ruleset activo `main-protection`

Ruleset ID `20826197`, enforcement `active`, creado el 2026-08-13:

- bloquear borrado y force-push;
- requerir pull request;
- cero bypass, incluso para el owner;
- cero approvals mientras solo exista una identidad operativa;
- descartar approvals obsoletos cuando existan;
- resolver conversaciones antes de merge;
- historia lineal y solo squash merge;
- branch estrictamente actualizada antes de integrar;
- check obligatorio `validate-quality-system`.

Esta protección es operativa, no independiente. DEC-011 bloquea G0/G5 hasta añadir un segundo maintainer/auditor y elevar el ruleset.

## Elevación requerida para G0 independiente

- 1 aprobación general; 2 para dominio, protocolo, seguridad, workflows, arquitectura y release;
- requerir CODEOWNER y aprobación del último push por persona distinta;
- commits/tags firmados y merge queue cuando haya capacidad suficiente;
- equipo break-glass separado, restringido y auditado;
- probar rechazo de autoaprobación y bypass.

Check activo de bootstrap:

- `validate-quality-system`

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

La API confirma ruleset activo, ningún bypass, check obligatorio, secret scanning/push protection, Dependabot security updates, squash-only y rama `main`. Falta comprobar mediante intento controlado el rechazo de push directo y registrar el ciclo completo de una PR; esta PR realiza esa prueba.

Referencias oficiales: [reglas disponibles en rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), [artifact attestations](https://docs.github.com/en/actions/concepts/security/artifact-attestations) y [dependency review](https://docs.github.com/en/code-security/how-tos/secure-your-supply-chain/manage-your-dependency-security/configure-dependency-review-action).
