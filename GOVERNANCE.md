# Gobernanza

Xunlie usa maintainer governance con decisiones trazables.

- Product decide intención y prioridad.
- Architecture decide límites y compatibilidad.
- Quality posee el proceso de gates y evidencia.
- Security posee threat model y aceptación técnica de findings.
- Maintainers integran cambios; Release Managers publican artefactos verificados.

Decisiones reversibles se toman en PR. Decisiones estructurales usan ADR. Cambios al gobierno y SQ Plan requieren aprobación de Product, Architecture, Quality y Security. Los desacuerdos se resuelven con evidencia y riesgo explícito; hasta entonces prevalece el estado más seguro (`BLOCKED`).

`somefirenoodles` es titular del repositorio y OpenAI Codex ejecuta inicialmente todos los roles operativos. Esta concentración se registra como modo `solo-agent-assisted`; no se presentará como revisión independiente hasta incorporar un segundo maintainer o auditor externo.
