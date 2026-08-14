# Roles y RACI

## Roles

- **PO — Product Owner:** intención, prioridad y aceptación de producto.
- **CA — Chief Architect:** baseline, límites, contratos y ADR.
- **QL — Quality Lead:** SQ Plan, independencia, gates, auditoría y CAPA.
- **SL — Security Lead:** amenazas, vulnerabilidades, permisos e incidentes.
- **DEV — Developer/Maintainer:** diseño detallado, implementación, tests y mantenimiento.
- **REV — Reviewer/CODEOWNER:** revisión independiente especializada.
- **RM — Release Manager:** ensamblado, verificación y publicación de release.
- **AUD — Auditor:** muestreo y evaluación independiente del sistema.

Una persona puede cubrir varios roles en fase temprana, salvo autor/aprobador del mismo cambio y solicitante/aprobador de una excepción.

## Asignación operativa inicial

- Titular del repositorio: `somefirenoodles`.
- Ejecutor de todos los roles operativos: OpenAI Codex.
- Modo: `solo-agent-assisted`.
- Independencia: no satisfecha; DEC-011 bloquea una declaración G0/G5 independiente.

Los controles automáticos pueden aportar evidencia independiente del código que evalúan, pero no constituyen una segunda identidad responsable ni pueden aceptar riesgo legal/residual por sí solos.

## Matriz

`A` accountable, `R` responsible, `C` consulted, `I` informed.

| Actividad | PO | CA | QL | SL | DEV | REV | RM | AUD |
|---|---|---|---|---|---|---|---|---|
| Aprobar SQ Plan | A | C | R | C | I | I | I | C |
| Aprobar requisitos | A/R | C | C | C | I | I | I | I |
| Aprobar arquitectura/ADR | C | A/R | C | C | C | C | I | I |
| Implementar incremento | I | C | C | C | A/R | C | I | I |
| Aceptar PR crítica | I | C | A | C | R | R | I | I |
| Threat model/finding | I | C | C | A/R | C | C | I | I |
| Gate de etapa | C | C | A/R | C | I | C | C | I |
| Aceptar waiver de calidad | C | C | A/R | C | I | I | I | I |
| Aceptar riesgo de seguridad | A | C | C | R | I | I | I | I |
| Crear artefacto de release | I | I | C | C | C | I | A/R | I |
| Verificar/promover release | I | I | A | C | I | R | R | I |
| Auditar una release | I | I | C | C | I | I | I | A/R |
| Cerrar CAPA | C | C | A | C | R | C | I | C |

## Autoridad de bloqueo

QL bloquea por calidad/evidencia; SL por riesgo de seguridad; CA por violación arquitectónica; PO por incumplimiento de intención. Resolver el bloqueo requiere corregir la causa o seguir el proceso formal de excepción cuando sea admisible.
