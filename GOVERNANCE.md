# Gobernanza

Xunlie usa maintainer governance con decisiones trazables.

- Product decide intención y prioridad.
- Architecture decide límites y compatibilidad.
- Quality posee el proceso de gates y evidencia.
- Security posee threat model y aceptación técnica de findings.
- Maintainers integran cambios; Release Managers publican artefactos verificados.

Decisiones reversibles se toman en PR. Decisiones estructurales usan ADR. Cambios al gobierno y SQ Plan requieren aprobación de Product, Architecture, Quality y Security. Los desacuerdos se resuelven con evidencia y riesgo explícito; hasta entonces prevalece el estado más seguro (`BLOCKED`).

`somefirenoodles` es titular del repositorio y conserva la autoridad sobre intención, riesgo legal y
decisiones de negocio. OpenAI Codex opera en modo `orchestrated-multi-agent`: una instancia autora
no puede aprobar su propio candidato y revisores-agente separados, sin permiso de modificarlo,
reproducen los controles y emiten veredictos registrados. Un hallazgo crítico/alto o la falta de
quorum bloquea el gate. El protocolo normativo está en `quality/roles.json` y ADR-0004.
