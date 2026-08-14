# Auditoría del bootstrap GitHub

**Repositorio:** [somefirenoodles/xunlie](https://github.com/somefirenoodles/xunlie)  
**Visibilidad/licencia:** público / MIT  
**Commit bootstrap:** `304162bec22f3a032b4c442b655dbfa50286f75e`  
**Workflow:** [governance run 31761453729](https://github.com/somefirenoodles/xunlie/actions/runs/31761453729)

## Resultado

- Repositorio creado bajo la identidad autenticada `somefirenoodles`.
- `main` establecida como rama por defecto.
- Workflow `validate-quality-system`: PASS.
- Ruleset `main-protection` activo, sin bypass y con check estricto.
- Solo squash merge, historia lineal y borrado/force-push bloqueados.
- Secret scanning, push protection, Dependabot security updates y private vulnerability reporting activos.
- Issues, Discussions, templates, CODEOWNERS y labels de gobierno disponibles.

## Diferencia contra la baseline G0

La protección actual permite operación con un solo actor y exige cero approvals. No están activos aún firma obligatoria, tag ruleset, segundo CODEOWNER ni aprobación independiente. Por tanto, `CTRL-REPO-001` permanece `BLOCKED` para G0 aunque el repositorio ya esté correctamente publicado y protegido contra integración sin CI.

## Próxima prueba

El intento controlado `git push origin HEAD:main` fue rechazado con `GH013`: GitHub exigió pull request y el check `validate-quality-system`. Esta actualización se publica mediante branch y PR; la integración solo puede ocurrir por squash después del check obligatorio.
