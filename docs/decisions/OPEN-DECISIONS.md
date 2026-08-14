# Registro de decisiones abiertas

Estas decisiones requieren autoridad del propietario. Las recomendaciones permiten avanzar sin ocultar supuestos.

| ID | Decisión | Recomendación | Bloquea | Estado |
|---|---|---|---|---|
| DEC-001 | Owner GitHub | `somefirenoodles` | — | aceptada |
| DEC-002 | Visibilidad | público | — | aceptada |
| DEC-003 | Licencia | MIT | — | aceptada |
| DEC-004 | Operación | OpenAI Codex orquesta instancias autoras y revisoras separadas; `somefirenoodles` conserva la titularidad | — | aceptada |
| DEC-005 | Plataformas v1 | Linux x86_64 primario; macOS arm64 y Windows x86_64 para CLI/replay | G1/G2 | abierta |
| DEC-006 | Primer adapter real | protocolo genérico + adapter del agente usado por el equipo piloto | G2 | abierta |
| DEC-007 | Política de telemetría | local/off por defecto; export OpenTelemetry opt-in y redactado | G2 | abierta |
| DEC-008 | Retención/coste | evidencia de release por vida +24 meses; corridas PR 90 días salvo regulación | G2 | abierta |
| DEC-009 | Compatibilidad pública | garantizar lectura de bundle `N-1` desde la primera beta | G2 | abierta |
| DEC-010 | Distribución | binarios GitHub Releases + imagen OCI; package managers después de GA | G2/G5 | abierta |
| DEC-011 | Aprobación independiente | protocolo de revisión por agentes separados, candidato congelado, evidencia reproducible y quorum unánime | — | aceptada mediante ADR-0004 |

## Preguntas de aprobación

1. ¿Qué agente y qué plataformas constituyen el piloto real?

## Modelo operativo resuelto

Codex separa autoría, revisión especializada y agregación mediante instancias con tareas y
permisos distintos. La independencia se demuestra en el GateRecord, no por la marca del modelo: el
revisor no modifica el candidato, reproduce evidencia sobre el SHA exacto y emite un veredicto
individual. El orquestador no cuenta como voto y cualquier desacuerdo falla cerrado. La titularidad
y la aceptación de riesgo legal o de negocio permanecen en `somefirenoodles`.

Resolver una fila exige ADR o registro firmado; no basta eliminarla de esta tabla.
